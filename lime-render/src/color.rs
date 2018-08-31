use std::borrow::Cow;
use std::error::Error;
use std::fmt;
use std::num::ParseIntError;
use std::str::FromStr;

use serde::{de, Deserialize, Deserializer};
use vulkano::pipeline::vertex::{VertexMember, VertexMemberTy};

#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct Color {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}

impl Color {
    pub fn rgb(r: f32, g: f32, b: f32) -> Self {
        Color { r, g, b, a: 1.0 }
    }

    pub fn rgba(r: f32, g: f32, b: f32, a: f32) -> Self {
        Color { r, g, b, a }
    }

    pub const RED: Color = Color {
        r: 1.0,
        g: 0.0,
        b: 0.0,
        a: 1.0,
    };
    pub const GREEN: Color = Color {
        r: 0.0,
        g: 1.0,
        b: 0.0,
        a: 1.0,
    };
    pub const BLUE: Color = Color {
        r: 0.0,
        g: 0.0,
        b: 1.0,
        a: 1.0,
    };
}

impl PartialEq for Color {
    fn eq(&self, other: &Self) -> bool {
        ulps_eq!(self.r, other.r)
            && ulps_eq!(self.g, other.g)
            && ulps_eq!(self.b, other.b)
            && ulps_eq!(self.a, other.a)
    }
}

unsafe impl VertexMember for Color {
    fn format() -> (VertexMemberTy, usize) {
        (VertexMemberTy::F32, 4)
    }
}

impl From<[f32; 4]> for Color {
    fn from([r, g, b, a]: [f32; 4]) -> Self {
        Color { r, g, b, a }
    }
}

impl From<Color> for [f32; 4] {
    fn from(color: Color) -> Self {
        [color.r, color.g, color.b, color.a]
    }
}

impl FromStr for Color {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let alpha = match s.len() {
            7 => false,
            9 => true,
            _ => return Err(ParseError::InvalidLength),
        };

        if !s.starts_with('#') {
            return Err(ParseError::MissingHash);
        }

        let r = parse_byte(&s[1..])?;
        let g = parse_byte(&s[3..])?;
        let b = parse_byte(&s[5..])?;
        let a = if alpha { parse_byte(&s[7..])? } else { 1.0 };

        Ok(Color { r, g, b, a })
    }
}

fn parse_byte(s: &str) -> Result<f32, ParseIntError> {
    u8::from_str_radix(&s[..2], 16).map(|n| n as f32 / 255.0)
}

#[derive(Debug)]
pub enum ParseError {
    MissingHash,
    InvalidLength,
    InvalidDigit,
}

impl From<ParseIntError> for ParseError {
    fn from(_: ParseIntError) -> Self {
        ParseError::InvalidDigit
    }
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.description().fmt(f)
    }
}

impl Error for ParseError {
    fn description(&self) -> &str {
        match self {
            ParseError::MissingHash => "color string must start with the '#' symbol",
            ParseError::InvalidLength => "invalid length for color string",
            ParseError::InvalidDigit => "invalid digit in color string",
        }
    }
}

impl<'de> Deserialize<'de> for Color {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        Cow::<str>::deserialize(deserializer)?
            .parse()
            .map_err(de::Error::custom)
    }
}
