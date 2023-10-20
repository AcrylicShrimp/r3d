use crate::{
    cursor::Cursor,
    parse::{Parse, ParseError},
    pmx_header::{PmxConfig, PmxIndexSize},
};
use std::ops::Deref;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum PmxPrimitiveParseError {
    #[error("unexpected EOF detected")]
    UnexpectedEof,
    #[error("failed to parse a Rust primitive: {0}")]
    RustPrimitiveParseError(#[from] crate::primitives::RustPrimitiveParseError),
}

impl ParseError for PmxPrimitiveParseError {
    fn error_unexpected_eof() -> Self {
        Self::UnexpectedEof
    }
}

macro_rules! define_index {
    ($name:ident($ty:ty)) => {
        #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
        pub struct $name($ty);

        impl $name {
            pub fn new(index: $ty) -> Self {
                Self(index)
            }

            pub fn get(self) -> $ty {
                self.0
            }
        }

        impl From<$name> for $ty {
            fn from(index: $name) -> Self {
                index.0
            }
        }

        impl From<$ty> for $name {
            fn from(index: $ty) -> Self {
                Self::new(index)
            }
        }

        impl Deref for $name {
            type Target = $ty;

            fn deref(&self) -> &Self::Target {
                &self.0
            }
        }
    };
}

define_index!(PmxVertexIndex(u32));
define_index!(PmxTextureIndex(i32));
define_index!(PmxMaterialIndex(i32));
define_index!(PmxBoneIndex(i32));
define_index!(PmxMorphIndex(i32));
define_index!(PmxRigidbodyIndex(i32));

impl Parse for PmxVertexIndex {
    type Error = PmxPrimitiveParseError;

    fn parse(config: &PmxConfig, cursor: &mut Cursor) -> Result<Self, Self::Error> {
        let index = match config.vertex_index_size {
            PmxIndexSize::U8 => u8::parse(config, cursor)? as u32,
            PmxIndexSize::U16 => u16::parse(config, cursor)? as u32,
            PmxIndexSize::U32 => u32::parse(config, cursor)?,
        };

        Ok(Self::new(index))
    }
}

impl Parse for PmxTextureIndex {
    type Error = PmxPrimitiveParseError;

    fn parse(config: &PmxConfig, cursor: &mut Cursor) -> Result<Self, Self::Error> {
        let index = match config.texture_index_size {
            PmxIndexSize::U8 => i8::parse(config, cursor)? as i32,
            PmxIndexSize::U16 => i16::parse(config, cursor)? as i32,
            PmxIndexSize::U32 => i32::parse(config, cursor)?,
        };

        Ok(Self::new(index))
    }
}

impl Parse for PmxMaterialIndex {
    type Error = PmxPrimitiveParseError;

    fn parse(config: &PmxConfig, cursor: &mut Cursor) -> Result<Self, Self::Error> {
        let index = match config.material_index_size {
            PmxIndexSize::U8 => i8::parse(config, cursor)? as i32,
            PmxIndexSize::U16 => i16::parse(config, cursor)? as i32,
            PmxIndexSize::U32 => i32::parse(config, cursor)?,
        };

        Ok(Self::new(index))
    }
}

impl Parse for PmxBoneIndex {
    type Error = PmxPrimitiveParseError;

    fn parse(config: &PmxConfig, cursor: &mut Cursor) -> Result<Self, Self::Error> {
        let index = match config.bone_index_size {
            PmxIndexSize::U8 => i8::parse(config, cursor)? as i32,
            PmxIndexSize::U16 => i16::parse(config, cursor)? as i32,
            PmxIndexSize::U32 => i32::parse(config, cursor)?,
        };

        Ok(Self::new(index))
    }
}

impl Parse for PmxMorphIndex {
    type Error = PmxPrimitiveParseError;

    fn parse(config: &PmxConfig, cursor: &mut Cursor) -> Result<Self, Self::Error> {
        let index = match config.morph_index_size {
            PmxIndexSize::U8 => i8::parse(config, cursor)? as i32,
            PmxIndexSize::U16 => i16::parse(config, cursor)? as i32,
            PmxIndexSize::U32 => i32::parse(config, cursor)?,
        };

        Ok(Self::new(index))
    }
}

impl Parse for PmxRigidbodyIndex {
    type Error = PmxPrimitiveParseError;

    fn parse(config: &PmxConfig, cursor: &mut Cursor) -> Result<Self, Self::Error> {
        let index = match config.rigidbody_index_size {
            PmxIndexSize::U8 => i8::parse(config, cursor)? as i32,
            PmxIndexSize::U16 => i16::parse(config, cursor)? as i32,
            PmxIndexSize::U32 => i32::parse(config, cursor)?,
        };

        Ok(Self::new(index))
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PmxVec2 {
    pub x: f32,
    pub y: f32,
}

impl Parse for PmxVec2 {
    type Error = PmxPrimitiveParseError;

    fn parse(config: &PmxConfig, cursor: &mut Cursor) -> Result<Self, Self::Error> {
        let x = f32::parse(config, cursor)?;
        let y = f32::parse(config, cursor)?;

        Ok(Self { x, y })
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PmxVec3 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

impl Parse for PmxVec3 {
    type Error = PmxPrimitiveParseError;

    fn parse(config: &PmxConfig, cursor: &mut Cursor) -> Result<Self, Self::Error> {
        let x = f32::parse(config, cursor)?;
        let y = f32::parse(config, cursor)?;
        let z = f32::parse(config, cursor)?;

        Ok(Self { x, y, z })
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PmxVec4 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub w: f32,
}

impl Parse for PmxVec4 {
    type Error = PmxPrimitiveParseError;

    fn parse(config: &PmxConfig, cursor: &mut Cursor) -> Result<Self, Self::Error> {
        let x = f32::parse(config, cursor)?;
        let y = f32::parse(config, cursor)?;
        let z = f32::parse(config, cursor)?;
        let w = f32::parse(config, cursor)?;

        Ok(Self { x, y, z, w })
    }
}
