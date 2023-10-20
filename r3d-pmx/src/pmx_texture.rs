use crate::{
    cursor::Cursor,
    parse::{Parse, ParseError},
    pmx_header::PmxConfig,
};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum PmxTextureParseError {
    #[error("unexpected EOF detected")]
    UnexpectedEof,
    #[error("failed to parse a Rust primitive: {0}")]
    RustPrimitiveParseError(#[from] crate::primitives::RustPrimitiveParseError),
    #[error("failed to parse a PMX primitive: {0}")]
    PmxPrimitiveParseError(#[from] crate::pmx_primitives::PmxPrimitiveParseError),
}

impl ParseError for PmxTextureParseError {
    fn error_unexpected_eof() -> Self {
        Self::UnexpectedEof
    }
}

#[derive(Debug, Clone)]
pub struct PmxTexture {
    pub path: String,
}

impl Parse for PmxTexture {
    type Error = PmxTextureParseError;

    fn parse(config: &PmxConfig, cursor: &mut Cursor) -> Result<Self, Self::Error> {
        // texture path (dynamic size)
        let path = String::parse(config, cursor)?;

        Ok(Self { path })
    }
}

impl Parse for Vec<PmxTexture> {
    type Error = PmxTextureParseError;

    fn parse(config: &PmxConfig, cursor: &mut Cursor) -> Result<Self, Self::Error> {
        // texture count (4 bytes)
        let size = 4;
        cursor.ensure_bytes::<Self::Error>(size)?;

        let count = u32::parse(config, cursor)? as usize;
        let mut textures = Vec::with_capacity(count);

        for _ in 0..count {
            textures.push(PmxTexture::parse(config, cursor)?);
        }

        Ok(textures)
    }
}
