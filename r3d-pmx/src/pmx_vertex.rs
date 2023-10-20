use crate::{
    cursor::Cursor,
    parse::{Parse, ParseError},
    pmx_header::PmxConfig,
    pmx_primitives::{PmxBoneIndex, PmxVec2, PmxVec3, PmxVec4},
};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum PmxVertexParseError {
    #[error("unexpected EOF detected")]
    UnexpectedEof,
    #[error("failed to parse a Rust primitive: {0}")]
    RustPrimitiveParseError(#[from] crate::primitives::RustPrimitiveParseError),
    #[error("failed to parse a PMX primitive: {0}")]
    PmxPrimitiveParseError(#[from] crate::pmx_primitives::PmxPrimitiveParseError),
    #[error("deform kind `{kind}` is invalid; it must be in the range of [0, 3] in PMX 2.0")]
    InvalidDeformKind { kind: u8 },
}

impl ParseError for PmxVertexParseError {
    fn error_unexpected_eof() -> Self {
        Self::UnexpectedEof
    }
}

#[derive(Debug, Clone)]
pub struct PmxVertex {
    pub position: PmxVec3,
    pub normal: PmxVec3,
    pub uv: PmxVec2,
    /// up to 4 additional vec4s
    pub additional_vec4s: [PmxVec4; 4],
    pub deform_kind: PmxVertexDeformKind,
    pub edge_size: f32,
}

impl Parse for PmxVertex {
    type Error = PmxVertexParseError;

    fn parse(config: &PmxConfig, cursor: &mut impl Cursor) -> Result<Self, Self::Error> {
        // position (12 bytes)
        // normal (12 bytes)
        // uv (8 bytes)
        // additional vec4s (16 bytes) * 4
        let size = 12 + 12 + 8 + 16 * 4;
        cursor.checked().ensure_bytes::<Self::Error>(size)?;

        let position = PmxVec3::parse(config, cursor)?;
        let normal = PmxVec3::parse(config, cursor)?;
        let uv = PmxVec2::parse(config, cursor)?;
        let mut additional_vec4s = [PmxVec4 {
            x: 0.0,
            y: 0.0,
            z: 0.0,
            w: 0.0,
        }; 4];

        for i in 0..config.additional_vec4_count {
            additional_vec4s[i] = PmxVec4::parse(config, cursor)?;
        }

        // dynamic size
        let deform_kind = PmxVertexDeformKind::parse(config, cursor)?;

        // edge size (4 bytes)
        let size = 4;
        cursor.checked().ensure_bytes::<Self::Error>(size)?;

        let edge_size = f32::parse(config, cursor)?;

        Ok(Self {
            position,
            normal,
            uv,
            additional_vec4s,
            deform_kind,
            edge_size,
        })
    }
}

impl Parse for Vec<PmxVertex> {
    type Error = PmxVertexParseError;

    fn parse(config: &PmxConfig, cursor: &mut impl Cursor) -> Result<Self, Self::Error> {
        // vertex count (4 bytes)
        let size = 4;
        cursor.checked().ensure_bytes::<Self::Error>(size)?;

        let count = u32::parse(config, cursor)? as usize;
        let mut vertices = Vec::with_capacity(count);

        for _ in 0..count {
            vertices.push(PmxVertex::parse(config, cursor)?);
        }

        Ok(vertices)
    }
}

#[derive(Debug, Clone)]
pub enum PmxVertexDeformKind {
    Bdef1 {
        bone_index: PmxBoneIndex,
    },
    Bdef2 {
        bone_index_1: PmxBoneIndex,
        bone_index_2: PmxBoneIndex,
        bone_weight: f32,
    },
    Bdef4 {
        bone_index_1: PmxBoneIndex,
        bone_index_2: PmxBoneIndex,
        bone_index_3: PmxBoneIndex,
        bone_index_4: PmxBoneIndex,
        bone_weight_1: f32,
        bone_weight_2: f32,
        bone_weight_3: f32,
        bone_weight_4: f32,
    },
    Sdef {
        bone_index_1: PmxBoneIndex,
        bone_index_2: PmxBoneIndex,
        bone_weight: f32,
        c: PmxVec3,
        r0: PmxVec3,
        r1: PmxVec3,
    },
}

impl Parse for PmxVertexDeformKind {
    type Error = PmxVertexParseError;

    fn parse(config: &PmxConfig, cursor: &mut impl Cursor) -> Result<Self, Self::Error> {
        let kind = u8::parse(config, &mut cursor.checked())?;

        Ok(match kind {
            0 => {
                // bone index (N byte) * 1
                let size = config.bone_index_size.size() * 1;
                cursor.checked().ensure_bytes::<Self::Error>(size)?;

                let bone_index = PmxBoneIndex::parse(config, cursor)?;

                PmxVertexDeformKind::Bdef1 { bone_index }
            }
            1 => {
                // bone index (N bytes) * 2
                // bone weight (4 bytes)
                let size = config.bone_index_size.size() * 2 + 4;
                cursor.checked().ensure_bytes::<Self::Error>(size)?;

                let bone_index_1 = PmxBoneIndex::parse(config, cursor)?;
                let bone_index_2 = PmxBoneIndex::parse(config, cursor)?;
                let bone_weight = f32::parse(config, cursor)?;

                PmxVertexDeformKind::Bdef2 {
                    bone_index_1,
                    bone_index_2,
                    bone_weight,
                }
            }
            2 => {
                // bone index (N bytes) * 4
                // bone weight (4 bytes) * 4
                let size = config.bone_index_size.size() * 4 + 4 * 4;
                cursor.checked().ensure_bytes::<Self::Error>(size)?;

                let bone_index_1 = PmxBoneIndex::parse(config, cursor)?;
                let bone_index_2 = PmxBoneIndex::parse(config, cursor)?;
                let bone_index_3 = PmxBoneIndex::parse(config, cursor)?;
                let bone_index_4 = PmxBoneIndex::parse(config, cursor)?;
                let bone_weight_1 = f32::parse(config, cursor)?;
                let bone_weight_2 = f32::parse(config, cursor)?;
                let bone_weight_3 = f32::parse(config, cursor)?;
                let bone_weight_4 = f32::parse(config, cursor)?;

                PmxVertexDeformKind::Bdef4 {
                    bone_index_1,
                    bone_index_2,
                    bone_index_3,
                    bone_index_4,
                    bone_weight_1,
                    bone_weight_2,
                    bone_weight_3,
                    bone_weight_4,
                }
            }
            3 => {
                // bone index (N bytes) * 2
                // bone weight (4 bytes)
                // c (12 bytes)
                // r0 (12 bytes)
                // r1 (12 bytes)
                let size = config.bone_index_size.size() * 2 + 4 + 12 * 3;
                cursor.checked().ensure_bytes::<Self::Error>(size)?;

                let bone_index_1 = PmxBoneIndex::parse(config, cursor)?;
                let bone_index_2 = PmxBoneIndex::parse(config, cursor)?;
                let bone_weight = f32::parse(config, cursor)?;
                let c = PmxVec3::parse(config, cursor)?;
                let r0 = PmxVec3::parse(config, cursor)?;
                let r1 = PmxVec3::parse(config, cursor)?;

                PmxVertexDeformKind::Sdef {
                    bone_index_1,
                    bone_index_2,
                    bone_weight,
                    c,
                    r0,
                    r1,
                }
            }
            kind => return Err(PmxVertexParseError::InvalidDeformKind { kind }),
        })
    }
}
