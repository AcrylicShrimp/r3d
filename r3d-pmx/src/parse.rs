use crate::{cursor::Cursor, pmx_header::PmxConfig};

pub trait Parse: Sized {
    type Error: ParseError;

    fn parse(config: &PmxConfig, cursor: &mut impl Cursor) -> Result<Self, Self::Error>;
}

pub trait ParseError {
    fn error_unexpected_eof() -> Self;
}
