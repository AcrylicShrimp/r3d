use crate::{
    cursor::Cursor,
    parse::{Parse, ParseError},
    pmx_header::PmxConfig,
    pmx_primitives::{
        PmxBoneIndex, PmxMaterialIndex, PmxMorphIndex, PmxRigidbodyIndex, PmxVec3, PmxVec4,
        PmxVertexIndex,
    },
};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum PmxMorphParseError {
    #[error("unexpected EOF detected")]
    UnexpectedEof,
    #[error("failed to parse a Rust primitive: {0}")]
    RustPrimitiveParseError(#[from] crate::primitives::RustPrimitiveParseError),
    #[error("failed to parse a PMX primitive: {0}")]
    PmxPrimitiveParseError(#[from] crate::pmx_primitives::PmxPrimitiveParseError),
    #[error("morph panel kind `{kind}` is invalid; it must be in the range of [0, 4]")]
    InvalidMorphPanelKind { kind: u8 },
    #[error("morph offset kind `{kind}` is invalid; it must be in the range of [0, 10]")]
    InvalidMorphOffsetKind { kind: u8 },
}

impl ParseError for PmxMorphParseError {
    fn error_unexpected_eof() -> Self {
        Self::UnexpectedEof
    }
}

#[derive(Debug, Clone)]
pub struct PmxMorph {
    pub name_local: String,
    pub name_universal: String,
    pub panel_kind: PmxMorphPanelKind,
    pub offset: PmxMorphOffset,
}

impl Parse for PmxMorph {
    type Error = PmxMorphParseError;

    fn parse(config: &PmxConfig, cursor: &mut Cursor) -> Result<Self, Self::Error> {
        // dynamic size
        let name_local = String::parse(config, cursor)?;
        let name_universal = String::parse(config, cursor)?;

        // panel kind (1 byte)
        let size = 1;
        cursor.ensure_bytes::<Self::Error>(size)?;

        let panel_kind = PmxMorphPanelKind::parse(config, cursor)?;

        // dynamic size
        let offset = PmxMorphOffset::parse(config, cursor)?;

        Ok(Self {
            name_local,
            name_universal,
            panel_kind,
            offset,
        })
    }
}

impl Parse for Vec<PmxMorph> {
    type Error = PmxMorphParseError;

    fn parse(config: &PmxConfig, cursor: &mut Cursor) -> Result<Self, Self::Error> {
        // morph count (4 bytes)
        let size = 4;
        cursor.ensure_bytes::<Self::Error>(size)?;

        let count = u32::parse(config, cursor)? as usize;
        let mut morphs = Vec::with_capacity(count);

        for _ in 0..count {
            morphs.push(PmxMorph::parse(config, cursor)?);
        }

        Ok(morphs)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PmxMorphPanelKind {
    Hidden,
    /// bottom-left in MMD
    Eyebrows,
    /// top-left in MMD
    Eyes,
    /// top-right in MMD
    Mouth,
    /// bottom-right in MMD
    Other,
}

impl Parse for PmxMorphPanelKind {
    type Error = PmxMorphParseError;

    fn parse(config: &PmxConfig, cursor: &mut Cursor) -> Result<Self, Self::Error> {
        let kind = u8::parse(config, cursor)?;

        match kind {
            0 => Ok(Self::Hidden),
            1 => Ok(Self::Eyebrows),
            2 => Ok(Self::Eyes),
            3 => Ok(Self::Mouth),
            4 => Ok(Self::Other),
            kind => Err(PmxMorphParseError::InvalidMorphPanelKind { kind }),
        }
    }
}

#[derive(Debug, Clone)]
pub enum PmxMorphOffset {
    Group(Vec<PmxMorphOffsetGroup>),
    Vertex(Vec<PmxMorphOffsetVertex>),
    Bone(Vec<PmxMorphOffsetBone>),
    Uv {
        offsets: Vec<PmxMorphOffsetUv>,
        /// extra UV index [0, 4]
        uv_index: u8,
    },
    Material(Vec<PmxMorphOffsetMaterial>),
    Flip(Vec<PmxMorphOffsetFlip>),
    Impulse(Vec<PmxMorphOffsetImpulse>),
}

impl Parse for PmxMorphOffset {
    type Error = PmxMorphParseError;

    fn parse(config: &PmxConfig, cursor: &mut Cursor) -> Result<Self, Self::Error> {
        // offset kind (1 byte)
        let size = 1;
        cursor.ensure_bytes::<Self::Error>(size)?;

        let kind = u8::parse(config, cursor)?;

        match kind {
            0 => Ok(Self::Group(Vec::parse(config, cursor)?)),
            1 => Ok(Self::Vertex(Vec::parse(config, cursor)?)),
            2 => Ok(Self::Bone(Vec::parse(config, cursor)?)),
            uv_index @ 3..=7 => {
                let uv_index = uv_index - 3;
                let offsets = Vec::parse(config, cursor)?;

                Ok(Self::Uv { offsets, uv_index })
            }
            8 => Ok(Self::Material(Vec::parse(config, cursor)?)),
            9 => Ok(Self::Flip(Vec::parse(config, cursor)?)),
            10 => Ok(Self::Impulse(Vec::parse(config, cursor)?)),
            kind => Err(PmxMorphParseError::InvalidMorphOffsetKind { kind }),
        }
    }
}

pub trait PmxMorphOffsetSizeHint {
    fn size_hint(config: &PmxConfig) -> usize;
}

impl<T: Parse<Error = PmxMorphParseError> + PmxMorphOffsetSizeHint> Parse for Vec<T> {
    type Error = PmxMorphParseError;

    fn parse(config: &PmxConfig, cursor: &mut Cursor) -> Result<Self, Self::Error> {
        // offset count (4 bytes)
        let size = 4;
        cursor.ensure_bytes::<Self::Error>(size)?;

        let count = u32::parse(config, cursor)? as usize;

        // offset data (count * size_hint bytes)
        let size = count * T::size_hint(config);
        cursor.ensure_bytes::<Self::Error>(size)?;

        let mut offsets = Vec::with_capacity(count);

        for _ in 0..count {
            offsets.push(T::parse(config, cursor)?);
        }

        Ok(offsets)
    }
}

#[derive(Debug, Clone)]
pub struct PmxMorphOffsetGroup {
    pub index: PmxMorphIndex,
    pub coefficient: f32,
}

impl PmxMorphOffsetSizeHint for PmxMorphOffsetGroup {
    fn size_hint(config: &PmxConfig) -> usize {
        config.morph_index_size.size() + 4
    }
}

impl Parse for PmxMorphOffsetGroup {
    type Error = PmxMorphParseError;

    fn parse(config: &PmxConfig, cursor: &mut Cursor) -> Result<Self, Self::Error> {
        // since group morph offset has a fixed size, we don't need to check the size here
        let index = PmxMorphIndex::parse(config, cursor)?;
        let coefficient = f32::parse(config, cursor)?;

        Ok(Self { index, coefficient })
    }
}

#[derive(Debug, Clone)]
pub struct PmxMorphOffsetVertex {
    pub index: PmxVertexIndex,
    pub translation: PmxVec3,
}

impl PmxMorphOffsetSizeHint for PmxMorphOffsetVertex {
    fn size_hint(config: &PmxConfig) -> usize {
        config.vertex_index_size.size() + 12
    }
}

impl Parse for PmxMorphOffsetVertex {
    type Error = PmxMorphParseError;

    fn parse(config: &PmxConfig, cursor: &mut Cursor) -> Result<Self, Self::Error> {
        // since vertex morph offset has a fixed size, we don't need to check the size here
        let index = PmxVertexIndex::parse(config, cursor)?;
        let translation = PmxVec3::parse(config, cursor)?;

        Ok(Self { index, translation })
    }
}

#[derive(Debug, Clone)]
pub struct PmxMorphOffsetBone {
    pub index: PmxBoneIndex,
    pub translation: PmxVec3,
    pub rotation: PmxVec4,
}

impl PmxMorphOffsetSizeHint for PmxMorphOffsetBone {
    fn size_hint(config: &PmxConfig) -> usize {
        config.bone_index_size.size() + 12 + 16
    }
}

impl Parse for PmxMorphOffsetBone {
    type Error = PmxMorphParseError;

    fn parse(config: &PmxConfig, cursor: &mut Cursor) -> Result<Self, Self::Error> {
        // since bone morph offset has a fixed size, we don't need to check the size here
        let index = PmxBoneIndex::parse(config, cursor)?;
        let translation = PmxVec3::parse(config, cursor)?;
        let rotation = PmxVec4::parse(config, cursor)?;

        Ok(Self {
            index,
            translation,
            rotation,
        })
    }
}

#[derive(Debug, Clone)]
pub struct PmxMorphOffsetUv {
    pub index: PmxVertexIndex,
    pub vec4: PmxVec4,
}

impl PmxMorphOffsetSizeHint for PmxMorphOffsetUv {
    fn size_hint(config: &PmxConfig) -> usize {
        config.vertex_index_size.size() + 16
    }
}

impl Parse for PmxMorphOffsetUv {
    type Error = PmxMorphParseError;

    fn parse(config: &PmxConfig, cursor: &mut Cursor) -> Result<Self, Self::Error> {
        // since UV morph offset has a fixed size, we don't need to check the size here
        let index = PmxVertexIndex::parse(config, cursor)?;
        let vec4 = PmxVec4::parse(config, cursor)?;

        Ok(Self { index, vec4 })
    }
}

#[derive(Debug, Clone)]
pub struct PmxMorphOffsetMaterial {
    /// -1 for all materials
    pub index: PmxMaterialIndex,
    pub diffuse_color: PmxVec4,
    pub specular_color: PmxVec3,
    pub specular_strength: f32,
    pub ambient_color: PmxVec3,
    pub edge_color: PmxVec4,
    pub edge_size: f32,
    pub texture_tint_color: PmxVec4,
    pub environment_tint_color: PmxVec4,
    pub toon_tint_color: PmxVec4,
}

impl PmxMorphOffsetSizeHint for PmxMorphOffsetMaterial {
    fn size_hint(config: &PmxConfig) -> usize {
        config.material_index_size.size() + 1 + 16 + 12 + 4 + 12 + 16 + 4 + 16 + 16 + 16
    }
}

impl Parse for PmxMorphOffsetMaterial {
    type Error = PmxMorphParseError;

    fn parse(config: &PmxConfig, cursor: &mut Cursor) -> Result<Self, Self::Error> {
        // since material morph offset has a fixed size, we don't need to check the size here
        let index = PmxMaterialIndex::parse(config, cursor)?;
        let _unused = u8::parse(config, cursor)?;
        let diffuse_color = PmxVec4::parse(config, cursor)?;
        let specular_color = PmxVec3::parse(config, cursor)?;
        let specular_strength = f32::parse(config, cursor)?;
        let ambient_color = PmxVec3::parse(config, cursor)?;
        let edge_color = PmxVec4::parse(config, cursor)?;
        let edge_size = f32::parse(config, cursor)?;
        let texture_tint_color = PmxVec4::parse(config, cursor)?;
        let environment_tint_color = PmxVec4::parse(config, cursor)?;
        let toon_tint_color = PmxVec4::parse(config, cursor)?;

        Ok(Self {
            index,
            diffuse_color,
            specular_color,
            specular_strength,
            ambient_color,
            edge_color,
            edge_size,
            texture_tint_color,
            environment_tint_color,
            toon_tint_color,
        })
    }
}

#[derive(Debug, Clone)]
pub struct PmxMorphOffsetFlip {
    pub index: PmxMorphIndex,
    pub coefficient: f32,
}

impl PmxMorphOffsetSizeHint for PmxMorphOffsetFlip {
    fn size_hint(config: &PmxConfig) -> usize {
        config.morph_index_size.size() + 4
    }
}

impl Parse for PmxMorphOffsetFlip {
    type Error = PmxMorphParseError;

    fn parse(config: &PmxConfig, cursor: &mut Cursor) -> Result<Self, Self::Error> {
        // since flip morph offset has a fixed size, we don't need to check the size here
        let index = PmxMorphIndex::parse(config, cursor)?;
        let coefficient = f32::parse(config, cursor)?;

        Ok(Self { index, coefficient })
    }
}

#[derive(Debug, Clone)]
pub struct PmxMorphOffsetImpulse {
    pub index: PmxRigidbodyIndex,
    /// `true` if `velocity` and `torque` is in local coordinate otherwise `false`.
    pub is_local: bool,
    pub velocity: PmxVec3,
    pub torque: PmxVec3,
}

impl PmxMorphOffsetSizeHint for PmxMorphOffsetImpulse {
    fn size_hint(config: &PmxConfig) -> usize {
        config.rigidbody_index_size.size() + 1 + 12 + 12
    }
}

impl Parse for PmxMorphOffsetImpulse {
    type Error = PmxMorphParseError;

    fn parse(config: &PmxConfig, cursor: &mut Cursor) -> Result<Self, Self::Error> {
        // since impulse morph offset has a fixed size, we don't need to check the size here
        let index = PmxRigidbodyIndex::parse(config, cursor)?;
        let is_local = bool::parse(config, cursor)?;
        let velocity = PmxVec3::parse(config, cursor)?;
        let torque = PmxVec3::parse(config, cursor)?;

        Ok(Self {
            index,
            is_local,
            velocity,
            torque,
        })
    }
}
