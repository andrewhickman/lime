mod lex;
#[cfg(test)]
mod tests;

use std::borrow::Cow;
use std::error::Error;
use std::{fmt, mem};

use cassowary::{Constraint, Expression, Term, WeightedRelation};
use specs::prelude::*;

use de::{DeserializeError, Seed};
use layout::de::parse::lex::{Lexer, Prod, Rel, Sum, Token};
use layout::Position;

pub(in layout::de) fn parse_constraint<'de, 'src>(
    seed: Seed<'de, 'src>,
    src: Cow<'de, str>,
) -> Result<Constraint, impl Error> {
    Parser::new(seed, &src)?.parse_constraint()
}

pub(in layout::de) fn parse_expression<'de, 'src>(
    seed: Seed<'de, 'src>,
    src: Cow<'de, str>,
) -> Result<Expression, impl Error> {
    Parser::new(seed, &src)?.parse_expression()
}

struct Parser<'de: 'src, 'src> {
    lexer: Lexer<'de, 'src>,
    peek: Token<'de, 'src>,
    seed: Seed<'de, 'src>,
}

impl<'de, 'src> Parser<'de, 'src> {
    fn new(seed: Seed<'de, 'src>, src: &'src Cow<'de, str>) -> Result<Self, ParseError> {
        let mut parser = Parser {
            lexer: Lexer::new(src),
            peek: Token::Eos,
            seed,
        };
        parser.bump()?;
        Ok(parser)
    }

    fn bump(&mut self) -> Result<Token<'de, 'src>, ParseError> {
        Ok(mem::replace(&mut self.peek, self.lexer.next()?))
    }

    fn parse_constraint(&mut self) -> Result<Constraint, ParseError> {
        let strength = self.parse_strength()?;
        let lhs = self.parse_expression()?;
        let rel = self.parse_relation(strength)?;
        let rhs = self.parse_expression()?;
        Ok(lhs | rel | rhs)
    }

    fn parse_strength(&mut self) -> Result<f64, ParseError> {
        match self.bump()? {
            Token::Strength(strength) => Ok(strength),
            tok => Err(ParseError::UnexpectedToken {
                expected: "one of 'weak', 'medium', 'strong' or 'required'",
                found: tok.to_string(),
            }),
        }
    }

    fn parse_expression(&mut self) -> Result<Expression, ParseError> {
        let mut expr = Expression::from_constant(0.0);
        let mut op = Sum::Add;
        loop {
            let toc = self.parse_term_or_const()?;

            match (op, toc) {
                (Sum::Add, TermOrConst::Term(term)) => expr += term,
                (Sum::Add, TermOrConst::Const(c)) => expr += c,
                (Sum::Sub, TermOrConst::Term(term)) => expr -= term,
                (Sum::Sub, TermOrConst::Const(c)) => expr -= c,
            }

            if let Some(sum) = self.try_parse_sum()? {
                op = sum;
            } else {
                return Ok(expr);
            }
        }
    }

    fn parse_term_or_const(&mut self) -> Result<TermOrConst, ParseError> {
        let mut lhs = TermOrConst::Const(1.0);
        let mut op = Prod::Mul;
        loop {
            let rhs = match self.bump()? {
                Token::Coefficient(f) => TermOrConst::Const(f),
                Token::Variable(ent, var) => TermOrConst::Term(self.resolve_variable(ent, var)?),
                tok => {
                    return Err(ParseError::UnexpectedToken {
                        expected: "coefficient or variable",
                        found: tok.to_string(),
                    })
                }
            };
            lhs = lhs.prod(op, rhs)?;
            if let Some(prod) = self.try_parse_prod()? {
                op = prod;
            } else {
                return Ok(lhs);
            }
        }
    }

    fn try_parse_prod(&mut self) -> Result<Option<Prod>, ParseError> {
        match self.peek {
            Token::Prod(prod) => {
                self.bump()?;
                Ok(Some(prod))
            }
            _ => Ok(None),
        }
    }

    fn try_parse_sum(&mut self) -> Result<Option<Sum>, ParseError> {
        match self.peek {
            Token::Sum(sum) => {
                self.bump()?;
                Ok(Some(sum))
            }
            _ => Ok(None),
        }
    }

    fn resolve_variable(
        &mut self,
        name: Cow<'de, str>,
        var: &'src str,
    ) -> Result<Term, ParseError> {
        let entity = self.seed.get_entity(name)?;
        let mut poss = WriteStorage::<Position>::fetch(self.seed.res);
        let pos = poss.entry(entity).unwrap().or_insert_with(Default::default);
        let variable = match var {
            "left" => pos.left_var(),
            "top" => pos.top_var(),
            "right" => pos.right_var(),
            "bottom" => pos.bottom_var(),
            // TODO: width and height
            var => {
                return Err(ParseError::UnexpectedToken {
                    expected: "one of 'left', 'top', 'right' or 'bottom'",
                    found: var.to_string(),
                })
            }
        };
        Ok(Term {
            variable,
            coefficient: 1.0,
        })
    }

    fn parse_relation(&mut self, strength: f64) -> Result<WeightedRelation, ParseError> {
        match self.bump()? {
            Token::Rel(Rel::Eq) => Ok(WeightedRelation::EQ(strength)),
            Token::Rel(Rel::Le) => Ok(WeightedRelation::LE(strength)),
            Token::Rel(Rel::Ge) => Ok(WeightedRelation::GE(strength)),
            tok => Err(ParseError::UnexpectedToken {
                expected: "one of '==', '<=' or '>='",
                found: tok.to_string(),
            }),
        }
    }
}

#[derive(Copy, Clone, Debug)]
enum TermOrConst {
    Term(Term),
    Const(f64),
}

impl TermOrConst {
    fn prod(self, op: Prod, other: Self) -> Result<Self, ParseError> {
        match (self, op, other) {
            (TermOrConst::Term(_), Prod::Mul, TermOrConst::Term(_)) => {
                Err(ParseError::NonAffineExpression)
            }
            (TermOrConst::Term(t), Prod::Mul, TermOrConst::Const(c)) => {
                Ok(TermOrConst::Term(t * c))
            }
            (TermOrConst::Term(_), Prod::Div, TermOrConst::Term(_)) => {
                Err(ParseError::NonAffineExpression)
            }
            (TermOrConst::Term(t), Prod::Div, TermOrConst::Const(c)) => {
                Ok(TermOrConst::Term(t / c))
            }
            (TermOrConst::Const(c), Prod::Mul, TermOrConst::Term(t)) => {
                Ok(TermOrConst::Term(c * t))
            }
            (TermOrConst::Const(l), Prod::Mul, TermOrConst::Const(r)) => {
                Ok(TermOrConst::Const(l * r))
            }
            (TermOrConst::Const(_), Prod::Div, TermOrConst::Term(_)) => {
                Err(ParseError::NonAffineExpression)
            }
            (TermOrConst::Const(l), Prod::Div, TermOrConst::Const(r)) => {
                Ok(TermOrConst::Const(l / r))
            }
        }
    }
}

#[derive(Debug)]
enum ParseError {
    De(DeserializeError),
    NonAffineExpression,
    UnexpectedToken {
        expected: &'static str,
        found: String,
    },
    InvalidToken(String),
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ParseError::De(err) => write!(f, "{}", err),
            ParseError::NonAffineExpression => write!(f, "non-affine constraint expression"),
            ParseError::UnexpectedToken { expected, found } => {
                write!(f, "expected {}, found '{}'", expected, found)
            }
            ParseError::InvalidToken(tok) => write!(f, "invalid token '{}'", tok),
        }
    }
}

impl Error for ParseError {}

impl From<DeserializeError> for ParseError {
    fn from(err: DeserializeError) -> Self {
        ParseError::De(err)
    }
}
