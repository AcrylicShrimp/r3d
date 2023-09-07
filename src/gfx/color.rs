use std::{
    fmt::Display,
    ops::{Mul, MulAssign},
};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ColorParseHexError {
    #[error("the hex string is too short")]
    TooShortError,
    #[error("the red component of the hex string is invalid")]
    RedComponentError,
    #[error("the green component of the hex string is invalid")]
    GreenComponentError,
    #[error("the blue component of the hex string is invalid")]
    BlueComponentError,
    #[error("the alpha component of the hex string is invalid")]
    AlphaComponentError,
    #[error(
        "a color part of the hex string has incorrect length; only 3, 6, or 8 characters allowed"
    )]
    IncorrectLengthError,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Color {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}

impl Color {
    pub fn from_rgb(r: f32, g: f32, b: f32) -> Self {
        Self { r, g, b, a: 1f32 }
    }

    pub fn from_rgba(r: f32, g: f32, b: f32, a: f32) -> Self {
        Self { r, g, b, a }
    }

    pub fn parse_hex(hex: impl AsRef<str>) -> Result<Self, ColorParseHexError> {
        let hex = hex.as_ref();

        if hex.len() < 2 {
            return Err(ColorParseHexError::TooShortError);
        }

        let hex = if hex.starts_with("#") {
            &hex[1..]
        } else if hex.starts_with("0x") {
            &hex[2..]
        } else if hex.starts_with("0X") {
            &hex[2..]
        } else {
            &hex[0..]
        };

        if hex.len() == 3 {
            let r = u8::from_str_radix(&hex[0..1], 16)
                .map_err(|_| ColorParseHexError::RedComponentError)? as u32;
            let g = u8::from_str_radix(&hex[1..2], 16)
                .map_err(|_| ColorParseHexError::GreenComponentError)? as u32;
            let b = u8::from_str_radix(&hex[2..3], 16)
                .map_err(|_| ColorParseHexError::BlueComponentError)? as u32;

            Ok(Self {
                r: (r << 4 & r) as f32 / 255f32,
                g: (g << 4 & g) as f32 / 255f32,
                b: (b << 4 & b) as f32 / 255f32,
                a: 1f32,
            })
        } else if hex.len() == 6 {
            let r = u8::from_str_radix(&hex[0..2], 16)
                .map_err(|_| ColorParseHexError::RedComponentError)?;
            let g = u8::from_str_radix(&hex[2..4], 16)
                .map_err(|_| ColorParseHexError::GreenComponentError)?;
            let b = u8::from_str_radix(&hex[4..6], 16)
                .map_err(|_| ColorParseHexError::BlueComponentError)?;

            Ok(Self {
                r: r as f32 / 255f32,
                g: g as f32 / 255f32,
                b: b as f32 / 255f32,
                a: 1f32,
            })
        } else if hex.len() == 8 {
            let r = u8::from_str_radix(&hex[0..2], 16)
                .map_err(|_| ColorParseHexError::RedComponentError)?;
            let g = u8::from_str_radix(&hex[2..4], 16)
                .map_err(|_| ColorParseHexError::GreenComponentError)?;
            let b = u8::from_str_radix(&hex[4..6], 16)
                .map_err(|_| ColorParseHexError::BlueComponentError)?;
            let a = u8::from_str_radix(&hex[6..8], 16)
                .map_err(|_| ColorParseHexError::AlphaComponentError)?;

            Ok(Self {
                r: r as f32 / 255f32,
                g: g as f32 / 255f32,
                b: b as f32 / 255f32,
                a: a as f32 / 255f32,
            })
        } else {
            Err(ColorParseHexError::IncorrectLengthError)
        }
    }

    pub fn transparent() -> Self {
        Self {
            r: 0f32,
            g: 0f32,
            b: 0f32,
            a: 0f32,
        }
    }

    pub fn black() -> Self {
        Self {
            r: 0f32,
            g: 0f32,
            b: 0f32,
            a: 1f32,
        }
    }

    pub fn red() -> Self {
        Self {
            r: 1f32,
            g: 0f32,
            b: 0f32,
            a: 1f32,
        }
    }

    pub fn green() -> Self {
        Self {
            r: 0f32,
            g: 1f32,
            b: 0f32,
            a: 1f32,
        }
    }

    pub fn blue() -> Self {
        Self {
            r: 0f32,
            g: 0f32,
            b: 1f32,
            a: 1f32,
        }
    }

    pub fn yellow() -> Self {
        Self {
            r: 1f32,
            g: 1f32,
            b: 0f32,
            a: 1f32,
        }
    }

    pub fn magenta() -> Self {
        Self {
            r: 1f32,
            g: 0f32,
            b: 1f32,
            a: 1f32,
        }
    }

    pub fn cyan() -> Self {
        Self {
            r: 0f32,
            g: 1f32,
            b: 1f32,
            a: 1f32,
        }
    }

    pub fn white() -> Self {
        Self {
            r: 1f32,
            g: 1f32,
            b: 1f32,
            a: 1f32,
        }
    }
}

impl Mul for Color {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self::Output {
        Self {
            r: self.r * rhs.r,
            g: self.g * rhs.g,
            b: self.b * rhs.b,
            a: self.a * rhs.a,
        }
    }
}

impl MulAssign for Color {
    fn mul_assign(&mut self, rhs: Self) {
        self.r *= rhs.r;
        self.g *= rhs.g;
        self.b *= rhs.b;
        self.a *= rhs.a;
    }
}

impl Display for Color {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Color(r={}, g={}, b={}, a={})",
            self.r, self.g, self.b, self.a
        )
    }
}
