use crate::{
    cursor::Cursor,
    parse::{Parse, ParseError},
    pmx_header::PmxConfig,
    pmx_primitives::{PmxBoneIndex, PmxMorphIndex},
};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum PmxDisplayParseError {
    #[error("unexpected EOF detected")]
    UnexpectedEof,
    #[error("failed to parse a Rust primitive: {0}")]
    RustPrimitiveParseError(#[from] crate::primitives::RustPrimitiveParseError),
    #[error("failed to parse a PMX primitive: {0}")]
    PmxPrimitiveParseError(#[from] crate::pmx_primitives::PmxPrimitiveParseError),
    #[error("display frame kind `{kind}` is invalid: must be in the range of [0, 1]")]
    InvalidDisplayFrameKind { kind: u8 },
}

impl ParseError for PmxDisplayParseError {
    fn error_unexpected_eof() -> Self {
        Self::UnexpectedEof
    }
}

#[derive(Debug, Clone)]
pub struct PmxDisplay {
    pub name_local: String,
    pub name_universal: String,
    pub is_special: bool,
    pub frames: Vec<PmxDisplayFrame>,
}

impl Parse for PmxDisplay {
    type Error = PmxDisplayParseError;

    fn parse(config: &PmxConfig, cursor: &mut impl Cursor) -> Result<Self, Self::Error> {
        // dynamic size
        let name_local = String::parse(config, cursor)?;
        let name_universal = String::parse(config, cursor)?;

        // is_special (1 byte)
        let size = 1;
        cursor.checked().ensure_bytes::<Self::Error>(size)?;

        let is_special = bool::parse(config, cursor)?;

        // dynamic size
        let frames = Vec::parse(config, cursor)?;

        Ok(Self {
            name_local,
            name_universal,
            is_special,
            frames,
        })
    }
}

impl Parse for Vec<PmxDisplay> {
    type Error = PmxDisplayParseError;

    fn parse(config: &PmxConfig, cursor: &mut impl Cursor) -> Result<Self, Self::Error> {
        // count (4 bytes)
        let size = 4;
        cursor.checked().ensure_bytes::<Self::Error>(size)?;

        let count = u32::parse(config, cursor)? as usize;
        let mut displays = Vec::with_capacity(count);

        for _ in 0..count {
            displays.push(PmxDisplay::parse(config, cursor)?);
        }

        Ok(displays)
    }
}

#[derive(Debug, Clone)]
pub enum PmxDisplayFrame {
    Bone { index: PmxBoneIndex },
    Morph { index: PmxMorphIndex },
}

impl Parse for PmxDisplayFrame {
    type Error = PmxDisplayParseError;

    fn parse(config: &PmxConfig, cursor: &mut impl Cursor) -> Result<Self, Self::Error> {
        // kind (1 byte)
        let size = 1;
        cursor.checked().ensure_bytes::<Self::Error>(size)?;

        let kind = u8::parse(config, cursor)?;

        match kind {
            0 => {
                // index (N bytes)
                let size = config.bone_index_size.size();
                cursor.checked().ensure_bytes::<Self::Error>(size)?;

                let index = PmxBoneIndex::parse(config, cursor)?;

                Ok(Self::Bone { index })
            }
            1 => {
                // index (N bytes)
                let size = config.morph_index_size.size();
                cursor.checked().ensure_bytes::<Self::Error>(size)?;

                let index = PmxMorphIndex::parse(config, cursor)?;

                Ok(Self::Morph { index })
            }
            kind => Err(PmxDisplayParseError::InvalidDisplayFrameKind { kind }),
        }
    }
}

impl Parse for Vec<PmxDisplayFrame> {
    type Error = PmxDisplayParseError;

    fn parse(config: &PmxConfig, cursor: &mut impl Cursor) -> Result<Self, Self::Error> {
        // count (4 bytes)
        let size = 4;
        cursor.checked().ensure_bytes::<Self::Error>(size)?;

        let count = u32::parse(config, cursor)? as usize;
        let mut frames = Vec::with_capacity(count);

        for _ in 0..count {
            frames.push(PmxDisplayFrame::parse(config, cursor)?);
        }

        Ok(frames)
    }
}
