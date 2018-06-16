use super::*;

use std::fmt::Write;

#[test]
fn lex() {
    let src = "weak medium strong required 12 -5.2 45 == <= >= entity.variable ";

    let mut split = src.split_whitespace();
    let mut out = String::new();
    loop {
        let tok = Token::next(&mut split).unwrap();
        if let Token::Eos = tok {
            break;
        }
        write!(&mut out, "{} ", tok).unwrap();
    }
    assert_eq!(src, out);
}
