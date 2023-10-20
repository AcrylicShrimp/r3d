use crate::{
    cursor::Cursor,
    parse::{Parse, ParseError},
};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum PmxHeaderParseError {
    #[error("unexpected EOF detected")]
    UnexpectedEof,
    #[error("failed to parse a Rust primitive: {0}")]
    RustPrimitiveParseError(#[from] crate::primitives::RustPrimitiveParseError),
    #[error("`{signature:?}` is not a valid PMX signature")]
    InvalidSignature { signature: [u8; 4] },
    #[error("PMX version `{version}` is not supported")]
    UnsupportedVersion { version: f32 },
    #[error("global count `{global_count}` is invalid; it must be 8 in PMX 2.0")]
    InvalidGlobalCount { global_count: u8 },
    #[error("text encoding `{encoding}` is invalid")]
    InvalidTextEncoding { encoding: u8 },
    #[error(
        "additional vec4 count `{count}` is invalid; it must be in the range of [0, 4] in PMX 2.0"
    )]
    InvalidAdditionalVec4Count { count: u8 },
    #[error("index size `{size}` is invalid at global index `{index}`; it must be 1, 2, or 4")]
    InvalidIndexSize { size: u8, index: usize },
}

impl ParseError for PmxHeaderParseError {
    fn error_unexpected_eof() -> Self {
        Self::UnexpectedEof
    }
}

#[derive(Debug, Clone)]
pub struct PmxHeader {
    pub signature: [u8; 4],
    pub version: f32,
    pub config: PmxConfig,
    pub model_name_local: String,
    pub model_name_universal: String,
    pub model_comment_local: String,
    pub model_comment_universal: String,
}

impl PmxHeader {
    pub fn parse(cursor: &mut impl Cursor) -> Result<Self, PmxHeaderParseError> {
        /// Minimum size of PMX 2.0 header.
        /// - 4 bytes: signature
        /// - 4 bytes: version
        /// - 1 byte: global count
        /// - 8 bytes: globals (fixed 8 bytes in PMX 2.0)
        const HEADER_SIZE: usize = 4 + 4 + 1 + 8;
        cursor
            .checked()
            .ensure_bytes::<PmxHeaderParseError>(HEADER_SIZE)?;

        // typically the signature is `PMX ` as 4 bytes, but some files do not have a space at the end
        let signature = *cursor.read::<PmxHeaderParseError, 4>()?;
        if &signature[0..3] != b"PMX" {
            return Err(PmxHeaderParseError::InvalidSignature { signature });
        }

        // version should be 2.0, with some tolerance
        let version = cursor.read::<PmxHeaderParseError, 4>()?;
        let version = f32::from_le_bytes(*version);
        if version < 1.95 || 2.05 < version {
            return Err(PmxHeaderParseError::UnsupportedVersion { version });
        }

        let config = PmxConfig::parse(cursor)?;

        let mut cursor = cursor.checked();
        let model_name_local = String::parse(&config, &mut cursor)?;
        let model_name_universal = String::parse(&config, &mut cursor)?;
        let model_comment_local = String::parse(&config, &mut cursor)?;
        let model_comment_universal = String::parse(&config, &mut cursor)?;

        Ok(Self {
            signature,
            version,
            config,
            model_name_local,
            model_name_universal,
            model_comment_local,
            model_comment_universal,
        })
    }
}

#[derive(Debug, Clone)]
pub struct PmxConfig {
    pub text_encoding: PmxTextEncoding,
    pub additional_vec4_count: usize,
    pub vertex_index_size: PmxIndexSize,
    pub texture_index_size: PmxIndexSize,
    pub material_index_size: PmxIndexSize,
    pub bone_index_size: PmxIndexSize,
    pub morph_index_size: PmxIndexSize,
    pub rigidbody_index_size: PmxIndexSize,
}

impl PmxConfig {
    pub fn parse(cursor: &mut impl Cursor) -> Result<Self, PmxHeaderParseError> {
        // global count is fixed to 8 in PMX 2.0
        let global_count = cursor.read::<PmxHeaderParseError, 1>()?[0];
        if global_count != 8 {
            return Err(PmxHeaderParseError::InvalidGlobalCount { global_count });
        }

        let globals = cursor.read::<PmxHeaderParseError, 8>()?;
        let text_encoding = match globals[0] {
            0 => PmxTextEncoding::Utf16le,
            1 => PmxTextEncoding::Utf8,
            encoding => {
                return Err(PmxHeaderParseError::InvalidTextEncoding { encoding });
            }
        };
        let additional_vec4_count = match globals[1] {
            count @ 0..=4 => count as usize,
            count => {
                return Err(PmxHeaderParseError::InvalidAdditionalVec4Count { count });
            }
        };

        let vertex_index_size = PmxIndexSize::parse(&globals, 2)?;
        let texture_index_size = PmxIndexSize::parse(&globals, 3)?;
        let material_index_size = PmxIndexSize::parse(&globals, 4)?;
        let bone_index_size = PmxIndexSize::parse(&globals, 5)?;
        let morph_index_size = PmxIndexSize::parse(&globals, 6)?;
        let rigidbody_index_size = PmxIndexSize::parse(&globals, 7)?;

        Ok(Self {
            text_encoding,
            additional_vec4_count,
            vertex_index_size,
            texture_index_size,
            material_index_size,
            bone_index_size,
            morph_index_size,
            rigidbody_index_size,
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PmxTextEncoding {
    Utf16le,
    Utf8,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PmxIndexSize {
    U8,
    U16,
    U32,
}

impl PmxIndexSize {
    pub fn size(self) -> usize {
        match self {
            Self::U8 => 1,
            Self::U16 => 2,
            Self::U32 => 4,
        }
    }

    pub fn parse(globals: &[u8; 8], index: usize) -> Result<Self, PmxHeaderParseError> {
        match globals[index] {
            1 => Ok(Self::U8),
            2 => Ok(Self::U16),
            4 => Ok(Self::U32),
            size => Err(PmxHeaderParseError::InvalidIndexSize { size, index }),
        }
    }
}
