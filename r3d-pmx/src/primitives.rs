use crate::{
    cursor::Cursor,
    parse::{Parse, ParseError},
    pmx_header::{PmxConfig, PmxTextEncoding},
};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum RustPrimitiveParseError {
    #[error("unexpected EOF detected")]
    UnexpectedEof,
    #[error("invalid utf8: {0}")]
    FromUtf8Error(#[from] std::str::Utf8Error),
    #[error("invalid utf16: {0}")]
    FromUtf16Error(#[from] std::string::FromUtf16Error),
    #[error("`{len}` is not a valid utf16 length; it must be even")]
    OddUtf16Length { len: usize },
}

impl ParseError for RustPrimitiveParseError {
    fn error_unexpected_eof() -> Self {
        Self::UnexpectedEof
    }
}

impl Parse for bool {
    type Error = RustPrimitiveParseError;

    fn parse(config: &PmxConfig, cursor: &mut Cursor) -> Result<Self, Self::Error> {
        Ok(u8::parse(config, cursor)? != 0)
    }
}

impl Parse for i8 {
    type Error = RustPrimitiveParseError;

    fn parse(_config: &PmxConfig, cursor: &mut Cursor) -> Result<Self, Self::Error> {
        Ok(i8::from_le_bytes(
            *cursor.read::<RustPrimitiveParseError, 1>()?,
        ))
    }
}

impl Parse for i16 {
    type Error = RustPrimitiveParseError;

    fn parse(_config: &PmxConfig, cursor: &mut Cursor) -> Result<Self, Self::Error> {
        Ok(i16::from_le_bytes(
            *cursor.read::<RustPrimitiveParseError, 2>()?,
        ))
    }
}

impl Parse for i32 {
    type Error = RustPrimitiveParseError;

    fn parse(_config: &PmxConfig, cursor: &mut Cursor) -> Result<Self, Self::Error> {
        Ok(i32::from_le_bytes(
            *cursor.read::<RustPrimitiveParseError, 4>()?,
        ))
    }
}

impl Parse for u8 {
    type Error = RustPrimitiveParseError;

    fn parse(_config: &PmxConfig, cursor: &mut Cursor) -> Result<Self, Self::Error> {
        Ok(u8::from_le_bytes(
            *cursor.read::<RustPrimitiveParseError, 1>()?,
        ))
    }
}

impl Parse for u16 {
    type Error = RustPrimitiveParseError;

    fn parse(_config: &PmxConfig, cursor: &mut Cursor) -> Result<Self, Self::Error> {
        Ok(u16::from_le_bytes(
            *cursor.read::<RustPrimitiveParseError, 2>()?,
        ))
    }
}

impl Parse for u32 {
    type Error = RustPrimitiveParseError;

    fn parse(_config: &PmxConfig, cursor: &mut Cursor) -> Result<Self, Self::Error> {
        Ok(u32::from_le_bytes(
            *cursor.read::<RustPrimitiveParseError, 4>()?,
        ))
    }
}

impl Parse for f32 {
    type Error = RustPrimitiveParseError;

    fn parse(_config: &PmxConfig, cursor: &mut Cursor) -> Result<Self, Self::Error> {
        Ok(f32::from_le_bytes(
            *cursor.read::<RustPrimitiveParseError, 4>()?,
        ))
    }
}

impl Parse for String {
    type Error = RustPrimitiveParseError;

    fn parse(config: &PmxConfig, cursor: &mut Cursor) -> Result<Self, Self::Error> {
        // string length (4 bytes)
        let size = 4;
        cursor.ensure_bytes::<Self::Error>(size)?;

        let len = u32::parse(config, cursor)? as usize;

        // string data (len bytes)
        let size = len;
        cursor.ensure_bytes::<Self::Error>(size)?;

        match config.text_encoding {
            PmxTextEncoding::Utf16le => {
                if len & 1 != 0 {
                    return Err(RustPrimitiveParseError::OddUtf16Length { len });
                }

                let bytes = cursor.read_dynamic::<RustPrimitiveParseError>(len)?;
                let chars = Vec::from_iter(
                    bytes
                        .chunks_exact(2)
                        .map(|chunk| u16::from_le_bytes([chunk[0], chunk[1]])),
                );
                let string = String::from_utf16(&chars)?;
                return Ok(string);
            }
            PmxTextEncoding::Utf8 => {
                let bytes = cursor.read_dynamic::<RustPrimitiveParseError>(len)?;
                let string = std::str::from_utf8(bytes)?;
                return Ok(string.to_owned());
            }
        }
    }
}
