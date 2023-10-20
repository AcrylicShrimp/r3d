use crate::{
    cursor::Cursor,
    parse::{Parse, ParseError},
    pmx_header::PmxConfig,
    pmx_primitives::{PmxRigidbodyIndex, PmxVec3},
};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum PmxJointParseError {
    #[error("unexpected EOF detected")]
    UnexpectedEof,
    #[error("failed to parse a Rust primitive: {0}")]
    RustPrimitiveParseError(#[from] crate::primitives::RustPrimitiveParseError),
    #[error("failed to parse a PMX primitive: {0}")]
    PmxPrimitiveParseError(#[from] crate::pmx_primitives::PmxPrimitiveParseError),
    #[error("joint kind `{kind}` is invalid; must be zero")]
    InvalidJointKind { kind: u8 },
}

impl ParseError for PmxJointParseError {
    fn error_unexpected_eof() -> Self {
        Self::UnexpectedEof
    }
}

#[derive(Debug, Clone)]
pub struct PmxJoint {
    pub name_local: String,
    pub name_universal: String,
    pub kind: PmxJointKind,
    pub rigidbody_index_pair: (PmxRigidbodyIndex, PmxRigidbodyIndex),
    pub position: PmxVec3,
    /// in radians
    pub rotation: PmxVec3,
    pub position_limit_min: PmxVec3,
    pub position_limit_max: PmxVec3,
    pub rotation_limit_min: PmxVec3,
    pub rotation_limit_max: PmxVec3,
    pub spring_position: PmxVec3,
    /// in radians
    pub spring_rotation: PmxVec3,
}

impl Parse for PmxJoint {
    type Error = PmxJointParseError;

    fn parse(config: &PmxConfig, cursor: &mut impl Cursor) -> Result<Self, Self::Error> {
        // dynamic size
        let name_local = String::parse(config, cursor)?;
        let name_universal = String::parse(config, cursor)?;

        // kind (1 byte)
        // rigidbody_index_pair (N bytes)
        // position (12 bytes)
        // rotation (12 bytes)
        // position_limit_min (12 bytes)
        // position_limit_max (12 bytes)
        // rotation_limit_min (12 bytes)
        // rotation_limit_max (12 bytes)
        // spring_position (12 bytes)
        // spring_rotation (12 bytes)
        let size = 1 + config.rigidbody_index_size.size() * 2 + 12 * 8;
        cursor.checked().ensure_bytes::<Self::Error>(size)?;

        let kind = PmxJointKind::parse(config, cursor)?;
        let rigidbody_index_1 = PmxRigidbodyIndex::parse(config, cursor)?;
        let rigidbody_index_2 = PmxRigidbodyIndex::parse(config, cursor)?;
        let position = PmxVec3::parse(config, cursor)?;
        let rotation = PmxVec3::parse(config, cursor)?;
        let position_limit_min = PmxVec3::parse(config, cursor)?;
        let position_limit_max = PmxVec3::parse(config, cursor)?;
        let rotation_limit_min = PmxVec3::parse(config, cursor)?;
        let rotation_limit_max = PmxVec3::parse(config, cursor)?;
        let spring_position = PmxVec3::parse(config, cursor)?;
        let spring_rotation = PmxVec3::parse(config, cursor)?;

        Ok(Self {
            name_local,
            name_universal,
            kind,
            rigidbody_index_pair: (rigidbody_index_1, rigidbody_index_2),
            position,
            rotation,
            position_limit_min,
            position_limit_max,
            rotation_limit_min,
            rotation_limit_max,
            spring_position,
            spring_rotation,
        })
    }
}

impl Parse for Vec<PmxJoint> {
    type Error = PmxJointParseError;

    fn parse(config: &PmxConfig, cursor: &mut impl Cursor) -> Result<Self, Self::Error> {
        // count (4 bytes)
        let size = 4;
        cursor.checked().ensure_bytes::<Self::Error>(size)?;

        let count = u32::parse(config, cursor)? as usize;
        let mut joints = Vec::with_capacity(count);

        for _ in 0..count {
            joints.push(PmxJoint::parse(config, cursor)?);
        }

        Ok(joints)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PmxJointKind {
    Spring6Dof,
}

impl Parse for PmxJointKind {
    type Error = PmxJointParseError;

    fn parse(config: &PmxConfig, cursor: &mut impl Cursor) -> Result<Self, Self::Error> {
        // since joint kind has a fixed size, we don't need to check the size here
        let kind = u8::parse(config, cursor)?;

        match kind {
            0 => Ok(Self::Spring6Dof),
            kind => Err(PmxJointParseError::InvalidJointKind { kind }),
        }
    }
}
