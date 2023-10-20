use crate::{
    cursor::Cursor,
    parse::{Parse, ParseError},
    pmx_header::PmxConfig,
    pmx_primitives::{PmxTextureIndex, PmxVec3, PmxVec4},
};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum PmxMaterialParseError {
    #[error("unexpected EOF detected")]
    UnexpectedEof,
    #[error("failed to parse a Rust primitive: {0}")]
    RustPrimitiveParseError(#[from] crate::primitives::RustPrimitiveParseError),
    #[error("failed to parse a PMX primitive: {0}")]
    PmxPrimitiveParseError(#[from] crate::pmx_primitives::PmxPrimitiveParseError),
    #[error("environment blend mode `{mode}` is invalid; it must be in the range of [0, 3]")]
    InvalidEnvironmentBlendMode { mode: u8 },
    #[error("toon mode `{mode}` is invalid; it must be in the range of [0, 1]")]
    InvalidToonMode { mode: u8 },
}

impl ParseError for PmxMaterialParseError {
    fn error_unexpected_eof() -> Self {
        Self::UnexpectedEof
    }
}

#[derive(Debug, Clone)]
pub struct PmxMaterial {
    pub name_local: String,
    pub name_universal: String,
    pub diffuse_color: PmxVec4,
    pub specular_color: PmxVec3,
    pub specular_strength: f32,
    pub ambient_color: PmxVec3,
    pub flags: PmxMaterialFlags,
    pub edge_color: PmxVec4,
    pub edge_size: f32,
    pub texture_index: PmxTextureIndex,
    pub environment_texture_index: PmxTextureIndex,
    pub environment_blend_mode: PmxMaterialEnvironmentBlendMode,
    pub toon_mode: PmxMaterialToonMode,
    pub metadata: String,
    /// Number of surfaces that use this material. All surfaces use exactly one material.
    /// Surfaces are sorted by material index, so surfaces with the same material index are contiguous.
    /// A span of surfaces using material at index `N` can be computed as `sum(surface_counts[0..N])..sum(surface_counts[0..N+1])`.
    pub surface_count: u32,
}

impl Parse for PmxMaterial {
    type Error = PmxMaterialParseError;

    fn parse(config: &PmxConfig, cursor: &mut Cursor) -> Result<Self, Self::Error> {
        // dynamic size
        let name_local = String::parse(config, cursor)?;
        let name_universal = String::parse(config, cursor)?;

        // diffuse color (4 * 4 bytes)
        // specular color (3 * 4 bytes)
        // specular strength (4 bytes)
        // ambient color (3 * 4 bytes)
        // flags (1 byte)
        // edge color (4 * 4 bytes)
        // edge size (4 bytes)
        // texture index (N bytes)
        // environment texture index (N bytes)
        // environment blend mode (1 byte)
        let size = 4 * 4
            + 3 * 4
            + 4
            + 3 * 4
            + 1
            + 4 * 4
            + 4
            + config.texture_index_size.size()
            + config.texture_index_size.size()
            + 1;
        cursor.ensure_bytes::<Self::Error>(size)?;

        let diffuse_color = PmxVec4::parse(config, cursor)?;
        let specular_color = PmxVec3::parse(config, cursor)?;
        let specular_strength = f32::parse(config, cursor)?;
        let ambient_color = PmxVec3::parse(config, cursor)?;
        let flags = PmxMaterialFlags::parse(config, cursor)?;
        let edge_color = PmxVec4::parse(config, cursor)?;
        let edge_size = f32::parse(config, cursor)?;
        let texture_index = PmxTextureIndex::parse(config, cursor)?;
        let environment_texture_index = PmxTextureIndex::parse(config, cursor)?;
        let environment_blend_mode = PmxMaterialEnvironmentBlendMode::parse(config, cursor)?;

        // dynamic size
        let toon_mode = PmxMaterialToonMode::parse(config, cursor)?;
        let metadata = String::parse(config, cursor)?;

        // surface count (4 bytes)
        let size = 4;
        cursor.ensure_bytes::<Self::Error>(size)?;

        let surface_count = u32::parse(config, cursor)?;

        Ok(Self {
            name_local,
            name_universal,
            diffuse_color,
            specular_color,
            specular_strength,
            ambient_color,
            flags,
            edge_color,
            edge_size,
            texture_index,
            environment_texture_index,
            environment_blend_mode,
            toon_mode,
            metadata,
            surface_count,
        })
    }
}

impl Parse for Vec<PmxMaterial> {
    type Error = PmxMaterialParseError;

    fn parse(config: &PmxConfig, cursor: &mut Cursor) -> Result<Self, Self::Error> {
        // material count (4 bytes)
        let size = 4;
        cursor.ensure_bytes::<Self::Error>(size)?;

        let material_count = u32::parse(config, cursor)?;
        let mut materials = Vec::with_capacity(material_count as usize);

        for _ in 0..material_count {
            materials.push(PmxMaterial::parse(config, cursor)?);
        }

        Ok(materials)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct PmxMaterialFlags {
    /// `true` if back faces should be culled otherwise `false`.
    pub cull_back_face: bool,
    /// `true` if it should cast shadow on ground otherwise `false`.
    pub cast_shadow_on_ground: bool,
    /// `true` if it should cast shadow on object otherwise `false`.
    pub cast_shadow_on_object: bool,
    /// `true` if it should receive shadow otherwise `false`.
    pub receive_shadow: bool,
    /// `true` if it should be drawn with pencil-like outline otherwise `false`.
    pub has_edge: bool,
}

impl Parse for PmxMaterialFlags {
    type Error = PmxMaterialParseError;

    fn parse(config: &PmxConfig, cursor: &mut Cursor) -> Result<Self, Self::Error> {
        // since material flags has a fixed size, we don't need to check the size here
        let flags = u8::parse(config, cursor)?;

        let cull_back_face = flags & 0b0000_0001 != 0;
        let cast_shadow_on_ground = flags & 0b0000_0010 != 0;
        let cast_shadow_on_object = flags & 0b0000_0100 != 0;
        let receive_shadow = flags & 0b0000_1000 != 0;
        let has_edge = flags & 0b0001_0000 != 0;

        Ok(Self {
            cull_back_face,
            cast_shadow_on_ground,
            cast_shadow_on_object,
            receive_shadow,
            has_edge,
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PmxMaterialEnvironmentBlendMode {
    Disabled,
    Multiplicative,
    Additive,
    /// Uses `vertex.additional_vec4s[0].xy` as `uv`.
    AdditionalVec4UV,
}

impl Parse for PmxMaterialEnvironmentBlendMode {
    type Error = PmxMaterialParseError;

    fn parse(config: &PmxConfig, cursor: &mut Cursor) -> Result<Self, Self::Error> {
        // since environment blend mode has a fixed size, we don't need to check the size here
        let mode = u8::parse(config, cursor)?;

        Ok(match mode {
            0 => Self::Disabled,
            1 => Self::Multiplicative,
            2 => Self::Additive,
            3 => Self::AdditionalVec4UV,
            mode => return Err(PmxMaterialParseError::InvalidEnvironmentBlendMode { mode }),
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PmxMaterialToonMode {
    /// Refers to `textures[index]`.
    Texture { index: PmxTextureIndex },
    /// Refers to the pre-defined internal toon texture with `index`.
    /// The pre-defined textures are usually named as `toon01.bmp` to `toon10.bmp` in the textures.
    InternalTexture { index: u8 },
}

impl Parse for PmxMaterialToonMode {
    type Error = PmxMaterialParseError;

    fn parse(config: &PmxConfig, cursor: &mut Cursor) -> Result<Self, Self::Error> {
        // toon mode (1 byte)
        let size = 1;
        cursor.ensure_bytes::<Self::Error>(size)?;

        let toon_mode = u8::parse(config, cursor)?;

        Ok(match toon_mode {
            0 => {
                // texture index (N bytes)
                let size = config.texture_index_size.size();
                cursor.ensure_bytes::<Self::Error>(size)?;

                let index = PmxTextureIndex::parse(config, cursor)?;

                Self::Texture { index }
            }
            1 => {
                // internal texture index (1 byte)
                let size = 1;
                cursor.ensure_bytes::<Self::Error>(size)?;

                let index = u8::parse(config, cursor)?;

                Self::InternalTexture { index }
            }
            mode => return Err(PmxMaterialParseError::InvalidToonMode { mode }),
        })
    }
}
