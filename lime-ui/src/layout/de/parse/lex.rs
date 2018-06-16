use std::borrow::Cow;
use std::fmt;
use std::str::SplitWhitespace;

use cassowary::strength;

use layout::de::parse::ParseError;

pub(in layout::de::parse) enum Lexer<'de: 'src, 'src> {
    Borrowed(SplitWhitespace<'de>),
    Owned(SplitWhitespace<'src>),
}

impl<'de, 'src> Lexer<'de, 'src> {
    pub(in layout::de::parse) fn new(src: &'src Cow<'de, str>) -> Self {
        match src {
            Cow::Borrowed(s) => Lexer::Borrowed(s.split_whitespace()),
            Cow::Owned(s) => Lexer::Owned(s.split_whitespace()),
        }
    }

    pub(in layout::de::parse) fn next(&mut self) -> Result<Token<'de, 'src>, ParseError> {
        match self {
            Lexer::Borrowed(split) => Token::next(split),
            Lexer::Owned(split) => Token::next(split).map(Token::to_owned),
        }
    }
}

#[derive(Clone, Debug)]
pub(in layout::de::parse) enum Token<'de, 'src> {
    Coefficient(f64),
    Strength(f64),
    Variable(Cow<'de, str>, &'src str),
    Rel(Rel),
    Sum(Sum),
    Prod(Prod),
    Eos,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub(in layout::de::parse) enum Prod {
    Mul,
    Div,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub(in layout::de::parse) enum Sum {
    Add,
    Sub,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub(in layout::de::parse) enum Rel {
    Eq,
    Le,
    Ge,
}

impl<'src> Token<'src, 'src> {
    /// Parse token from a non-empty string.
    fn next(s: &mut SplitWhitespace<'src>) -> Result<Self, ParseError> {
        let s = match s.next() {
            None => return Ok(Token::Eos),
            Some(s) => s,
        };

        match s {
            "+" => return Ok(Token::Sum(Sum::Add)),
            "-" => return Ok(Token::Sum(Sum::Sub)),
            "*" => return Ok(Token::Prod(Prod::Mul)),
            "/" => return Ok(Token::Prod(Prod::Div)),
            "==" => return Ok(Token::Rel(Rel::Eq)),
            "<=" => return Ok(Token::Rel(Rel::Le)),
            ">=" => return Ok(Token::Rel(Rel::Ge)),
            "weak" => return Ok(Token::Strength(strength::WEAK)),
            "medium" => return Ok(Token::Strength(strength::MEDIUM)),
            "strong" => return Ok(Token::Strength(strength::STRONG)),
            "required" => return Ok(Token::Strength(strength::REQUIRED)),
            _ => (),
        }

        if is_numeric_first_byte(s.as_bytes()[0]) {
            if let Ok(f) = s.parse() {
                return Ok(Token::Coefficient(f));
            }
        }

        let mut split = s.split('.');
        if let (Some(first), Some(second), None) = (split.next(), split.next(), split.next()) {
            return Ok(Token::Variable(Cow::Borrowed(first), second));
        }

        Err(ParseError::InvalidToken(s.to_owned()))
    }

    fn to_owned<'de: 'src>(self) -> Token<'de, 'src> {
        match self {
            Token::Coefficient(f) => Token::Coefficient(f),
            Token::Strength(f) => Token::Strength(f),
            Token::Variable(ent, var) => Token::Variable(Cow::Owned(ent.into_owned()), var),
            Token::Rel(rel) => Token::Rel(rel),
            Token::Sum(sum) => Token::Sum(sum),
            Token::Prod(prod) => Token::Prod(prod),
            Token::Eos => Token::Eos,
        }
    }
}

fn is_numeric_first_byte(b: u8) -> bool {
    b == b'+' || b == b'-' || b.is_ascii_digit()
}

impl<'de, 'src> fmt::Display for Token<'de, 'src> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Token::Coefficient(c) => c.fmt(f),
            Token::Strength(s) => if s == &strength::WEAK {
                "weak"
            } else if s == &strength::MEDIUM {
                "medium"
            } else if s == &strength::STRONG {
                "strong"
            } else if s == &strength::REQUIRED {
                "required"
            } else {
                unreachable!()
            }.fmt(f),
            Token::Variable(ent, var) => write!(f, "{}.{}", ent, var),
            Token::Rel(rel) => rel.fmt(f),
            Token::Sum(sum) => sum.fmt(f),
            Token::Prod(prod) => prod.fmt(f),
            Token::Eos => "<end of string>".fmt(f),
        }
    }
}

impl fmt::Display for Rel {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Rel::Eq => "==",
            Rel::Le => "<=",
            Rel::Ge => ">=",
        }.fmt(f)
    }
}

impl fmt::Display for Prod {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Prod::Mul => "*",
            Prod::Div => "/",
        }.fmt(f)
    }
}

impl fmt::Display for Sum {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Sum::Add => "+",
            Sum::Sub => "-",
        }.fmt(f)
    }
}
