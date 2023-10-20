use crate::{
    cursor::Cursor,
    parse::{Parse, ParseError},
    pmx_header::PmxConfig,
    pmx_primitives::{PmxBoneIndex, PmxVec3},
};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum PmxRigidbodyParseError {
    #[error("unexpected EOF detected")]
    UnexpectedEof,
    #[error("failed to parse a Rust primitive: {0}")]
    RustPrimitiveParseError(#[from] crate::primitives::RustPrimitiveParseError),
    #[error("failed to parse a PMX primitive: {0}")]
    PmxPrimitiveParseError(#[from] crate::pmx_primitives::PmxPrimitiveParseError),
    #[error("rigidbody shape kind `{kind}` is invalid: must be in the range of [0, 2]")]
    InvalidRigidbodyShapeKind { kind: u8 },
    #[error("rigidbody physics mode `{mode}` is invalid: must be in the range of [0, 2]")]
    InvalidRigidbodyPhysicsMode { mode: u8 },
}

impl ParseError for PmxRigidbodyParseError {
    fn error_unexpected_eof() -> Self {
        Self::UnexpectedEof
    }
}

#[derive(Debug, Clone)]
pub struct PmxRigidbody {
    pub name_local: String,
    pub name_universal: String,
    pub bone_index: PmxBoneIndex,
    pub group_id: i8,
    pub non_collision_group: i16,
    pub shape: PmxRigidbodyShape,
    pub mass: f32,
    pub linear_damping: f32,
    pub angular_damping: f32,
    pub restitution_coefficient: f32,
    pub friction_coefficient: f32,
    pub physics_mode: PmxRigidbodyPhysicsMode,
}

impl Parse for PmxRigidbody {
    type Error = PmxRigidbodyParseError;

    fn parse(config: &PmxConfig, cursor: &mut Cursor) -> Result<Self, Self::Error> {
        // dynamic size
        let name_local = String::parse(config, cursor)?;
        let name_universal = String::parse(config, cursor)?;

        // bone_index (4 bytes)
        // group_id (1 byte)
        // non_collision_group (2 bytes)
        // shape (37 byte)
        // mass (4 bytes)
        // linear_damping (4 bytes)
        // angular_damping (4 bytes)
        // restitution_coefficient (4 bytes)
        // friction_coefficient (4 bytes)
        // physics_mode (1 byte)
        let size = config.bone_index_size.size() + 1 + 2 + 37 + 4 + 4 + 4 + 4 + 4 + 1;
        cursor.ensure_bytes::<Self::Error>(size)?;

        let bone_index = PmxBoneIndex::parse(config, cursor)?;
        let group_id = i8::parse(config, cursor)?;
        let non_collision_group = i16::parse(config, cursor)?;
        let shape = PmxRigidbodyShape::parse(config, cursor)?;
        let mass = f32::parse(config, cursor)?;
        let linear_damping = f32::parse(config, cursor)?;
        let angular_damping = f32::parse(config, cursor)?;
        let restitution_coefficient = f32::parse(config, cursor)?;
        let friction_coefficient = f32::parse(config, cursor)?;
        let physics_mode = PmxRigidbodyPhysicsMode::parse(config, cursor)?;

        Ok(Self {
            name_local,
            name_universal,
            bone_index,
            group_id,
            non_collision_group,
            shape,
            mass,
            linear_damping,
            angular_damping,
            restitution_coefficient,
            friction_coefficient,
            physics_mode,
        })
    }
}

impl Parse for Vec<PmxRigidbody> {
    type Error = PmxRigidbodyParseError;

    fn parse(config: &PmxConfig, cursor: &mut Cursor) -> Result<Self, Self::Error> {
        // count (4 bytes)
        let size = 4;
        cursor.ensure_bytes::<Self::Error>(size)?;

        let count = u32::parse(config, cursor)? as usize;
        let mut rigidbodies = Vec::with_capacity(count);

        for _ in 0..count {
            rigidbodies.push(PmxRigidbody::parse(config, cursor)?);
        }

        Ok(rigidbodies)
    }
}

#[derive(Debug, Clone)]
pub struct PmxRigidbodyShape {
    pub kind: PmxRigidbodyShapeKind,
    pub size: PmxVec3,
    pub position: PmxVec3,
    /// in radians
    pub rotation: PmxVec3,
}

impl Parse for PmxRigidbodyShape {
    type Error = PmxRigidbodyParseError;

    fn parse(config: &PmxConfig, cursor: &mut Cursor) -> Result<Self, Self::Error> {
        // since shape has a fixed size, we don't need to check the size here
        let kind = PmxRigidbodyShapeKind::parse(config, cursor)?;
        let size = PmxVec3::parse(config, cursor)?;
        let position = PmxVec3::parse(config, cursor)?;
        let rotation = PmxVec3::parse(config, cursor)?;

        Ok(Self {
            kind,
            size,
            position,
            rotation,
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PmxRigidbodyShapeKind {
    Sphere,
    Box,
    Capsule,
}

impl Parse for PmxRigidbodyShapeKind {
    type Error = PmxRigidbodyParseError;

    fn parse(config: &PmxConfig, cursor: &mut Cursor) -> Result<Self, Self::Error> {
        // since shape kind has a fixed size, we don't need to check the size here
        let kind = u8::parse(config, cursor)?;

        match kind {
            0 => Ok(Self::Sphere),
            1 => Ok(Self::Box),
            2 => Ok(Self::Capsule),
            kind => Err(PmxRigidbodyParseError::InvalidRigidbodyShapeKind { kind }),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PmxRigidbodyPhysicsMode {
    Static,
    Dynamic,
    DynamicWithBone,
}

impl Parse for PmxRigidbodyPhysicsMode {
    type Error = PmxRigidbodyParseError;

    fn parse(config: &PmxConfig, cursor: &mut Cursor) -> Result<Self, Self::Error> {
        // since physics mode has a fixed size, we don't need to check the size here
        let mode = u8::parse(config, cursor)?;

        match mode {
            0 => Ok(Self::Static),
            1 => Ok(Self::Dynamic),
            2 => Ok(Self::DynamicWithBone),
            mode => Err(PmxRigidbodyParseError::InvalidRigidbodyPhysicsMode { mode }),
        }
    }
}
