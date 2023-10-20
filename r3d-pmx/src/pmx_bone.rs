use crate::{
    cursor::Cursor,
    parse::{Parse, ParseError},
    pmx_header::PmxConfig,
    pmx_primitives::{PmxBoneIndex, PmxVec3},
};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum PmxBoneParseError {
    #[error("unexpected EOF detected")]
    UnexpectedEof,
    #[error("failed to parse a Rust primitive: {0}")]
    RustPrimitiveParseError(#[from] crate::primitives::RustPrimitiveParseError),
    #[error("failed to parse a PMX primitive: {0}")]
    PmxPrimitiveParseError(#[from] crate::pmx_primitives::PmxPrimitiveParseError),
}

impl ParseError for PmxBoneParseError {
    fn error_unexpected_eof() -> Self {
        Self::UnexpectedEof
    }
}

#[derive(Debug, Clone)]
pub struct PmxBone {
    pub name_local: String,
    pub name_universal: String,
    pub position: PmxVec3,
    pub parent_index: PmxBoneIndex,
    pub layer: u32,
    pub flags: PmxBoneFlags,
    pub tail_position: PmxBoneTailPosition,
    pub inheritance: Option<PmxBoneInheritance>,
    pub fixed_axis: Option<PmxBoneFixedAxis>,
    pub local_coordinate: Option<PmxBoneLocalCoordinate>,
    pub external_parent: Option<PmxBoneExternalParent>,
    pub ik: Option<PmxBoneIK>,
}

impl Parse for PmxBone {
    type Error = PmxBoneParseError;

    fn parse(config: &PmxConfig, cursor: &mut Cursor) -> Result<Self, Self::Error> {
        // dynamic size
        let name_local = String::parse(config, cursor)?;
        let name_universal = String::parse(config, cursor)?;

        // position (12 bytes)
        // parent index (N bytes)
        // layer (4 bytes)
        // flags (2 bytes)
        let size = 12 + config.bone_index_size.size() + 4 + 2;
        cursor.ensure_bytes::<Self::Error>(size)?;

        let position = PmxVec3::parse(config, cursor)?;
        let parent_index = PmxBoneIndex::parse(config, cursor)?;
        let layer = u32::parse(config, cursor)?;
        let flags = PmxBoneFlags::parse(config, cursor)?;

        // dynamic size
        let tail_position = match flags.indexed_tail_position {
            true => {
                // tail bone index (N bytes)
                let size = config.bone_index_size.size();
                cursor.ensure_bytes::<Self::Error>(size)?;

                PmxBoneTailPosition::BoneIndex {
                    index: PmxBoneIndex::parse(config, cursor)?,
                }
            }
            false => {
                // tail position (12 bytes)
                let size = 12;
                cursor.ensure_bytes::<Self::Error>(size)?;

                PmxBoneTailPosition::Vec3 {
                    position: PmxVec3::parse(config, cursor)?,
                }
            }
        };
        let inheritance = match flags.inherit_rotation || flags.inherit_translation {
            true => {
                // bone index (N bytes)
                // coefficient (4 bytes)
                let size = config.bone_index_size.size() + 4;
                cursor.ensure_bytes::<Self::Error>(size)?;

                let index = PmxBoneIndex::parse(config, cursor)?;
                let coefficient = f32::parse(config, cursor)?;
                let inheritance_mode = match (flags.inherit_rotation, flags.inherit_translation) {
                    (true, false) => PmxBoneInheritanceMode::RotationOnly,
                    (false, true) => PmxBoneInheritanceMode::TranslationOnly,
                    _ => PmxBoneInheritanceMode::Both,
                };

                Some(PmxBoneInheritance {
                    index,
                    coefficient,
                    inheritance_mode,
                })
            }
            false => None,
        };
        let fixed_axis = match flags.fixed_axis {
            true => Some(PmxBoneFixedAxis::parse(config, cursor)?),
            false => None,
        };
        let local_coordinate = match flags.local_coordinate {
            true => Some(PmxBoneLocalCoordinate::parse(config, cursor)?),
            false => None,
        };
        let external_parent = match flags.external_parent_deform {
            true => Some(PmxBoneExternalParent::parse(config, cursor)?),
            false => None,
        };
        let ik = match flags.supports_ik {
            true => Some(PmxBoneIK::parse(config, cursor)?),
            false => None,
        };

        Ok(Self {
            name_local,
            name_universal,
            position,
            parent_index,
            layer,
            flags,
            tail_position,
            inheritance,
            fixed_axis,
            local_coordinate,
            external_parent,
            ik,
        })
    }
}

impl Parse for Vec<PmxBone> {
    type Error = PmxBoneParseError;

    fn parse(config: &PmxConfig, cursor: &mut Cursor) -> Result<Self, Self::Error> {
        // bone count (4 bytes)
        let size = 4;
        cursor.ensure_bytes::<Self::Error>(size)?;

        let count = u32::parse(config, cursor)? as usize;
        let mut bones = Vec::with_capacity(count);

        for _ in 0..count {
            bones.push(PmxBone::parse(config, cursor)?);
        }

        Ok(bones)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct PmxBoneFlags {
    /// `true` if tail position is represented as bone index otherwise `false` (tail position is represented as vec3).
    pub indexed_tail_position: bool,
    /// `true` if rotation is allowed otherwise `false`.
    pub is_rotatable: bool,
    /// `true` if movement is allowed otherwise `false`.
    pub is_translatable: bool,
    /// `true` if visible otherwise `false`. (not used maybe?)
    pub is_visible: bool,
    /// `true` if enabled otherwise `false`. (not used maybe?)
    pub is_enabled: bool,
    /// `true` if IK is supported otherwise `false`.
    pub supports_ik: bool,
    /// `true` if local rotation is inherited otherwise `false`.
    pub inherit_rotation: bool,
    /// `true` if local position is inherited otherwise `false`.
    pub inherit_translation: bool,
    /// `true` if this bone's shaft is fixed in a direction otherwise `false`.
    pub fixed_axis: bool,
    /// Unknown flag.    
    pub local_coordinate: bool,
    /// Unknown flag.
    pub physics_after_deform: bool,
    /// Unknown flag.
    pub external_parent_deform: bool,
}

impl Parse for PmxBoneFlags {
    type Error = PmxBoneParseError;

    fn parse(config: &PmxConfig, cursor: &mut Cursor) -> Result<Self, Self::Error> {
        // since bone flags has a fixed size, we don't need to check the size here
        let flag_1 = u8::parse(config, cursor)?;
        let flag_2 = u8::parse(config, cursor)?;

        let indexed_tail_position = flag_1 & 0b0000_0001 != 0;
        let is_rotatable = flag_1 & 0b0000_0010 != 0;
        let is_translatable = flag_1 & 0b0000_0100 != 0;
        let is_visible = flag_1 & 0b0000_1000 != 0;
        let is_enabled = flag_1 & 0b0001_0000 != 0;
        let supports_ik = flag_1 & 0b0010_0000 != 0;
        let inherit_rotation = flag_2 & 0b0000_0001 != 0;
        let inherit_translation = flag_2 & 0b0000_0010 != 0;
        let fixed_axis = flag_2 & 0b0000_0100 != 0;
        let local_coordinate = flag_2 & 0b0000_1000 != 0;
        let physics_after_deform = flag_2 & 0b0001_0000 != 0;
        let external_parent_deform = flag_2 & 0b0010_0000 != 0;

        Ok(Self {
            indexed_tail_position,
            is_rotatable,
            is_translatable,
            is_visible,
            is_enabled,
            supports_ik,
            inherit_rotation,
            inherit_translation,
            fixed_axis,
            local_coordinate,
            physics_after_deform,
            external_parent_deform,
        })
    }
}

#[derive(Debug, Clone)]
pub enum PmxBoneTailPosition {
    Vec3 { position: PmxVec3 },
    BoneIndex { index: PmxBoneIndex },
}

#[derive(Debug, Clone)]
pub struct PmxBoneInheritance {
    pub index: PmxBoneIndex,
    pub coefficient: f32,
    pub inheritance_mode: PmxBoneInheritanceMode,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PmxBoneInheritanceMode {
    Both,
    RotationOnly,
    TranslationOnly,
}

#[derive(Debug, Clone)]
pub struct PmxBoneFixedAxis {
    pub direction: PmxVec3,
}

impl Parse for PmxBoneFixedAxis {
    type Error = PmxBoneParseError;

    fn parse(config: &PmxConfig, cursor: &mut Cursor) -> Result<Self, Self::Error> {
        // direction (12 bytes)
        let size = 12;
        cursor.ensure_bytes::<Self::Error>(size)?;

        let direction = PmxVec3::parse(config, cursor)?;

        Ok(Self { direction })
    }
}

#[derive(Debug, Clone)]
pub struct PmxBoneLocalCoordinate {
    pub x_axis: PmxVec3,
    pub z_axis: PmxVec3,
}

impl Parse for PmxBoneLocalCoordinate {
    type Error = PmxBoneParseError;

    fn parse(config: &PmxConfig, cursor: &mut Cursor) -> Result<Self, Self::Error> {
        // x axis (12 bytes)
        // z axis (12 bytes)
        let size = 12 + 12;
        cursor.ensure_bytes::<Self::Error>(size)?;

        let x_axis = PmxVec3::parse(config, cursor)?;
        let z_axis = PmxVec3::parse(config, cursor)?;

        Ok(Self { x_axis, z_axis })
    }
}

#[derive(Debug, Clone)]
pub struct PmxBoneExternalParent {
    /// 4 bytes signed integer, not bone index
    pub index: i32,
}

impl Parse for PmxBoneExternalParent {
    type Error = PmxBoneParseError;

    fn parse(config: &PmxConfig, cursor: &mut Cursor) -> Result<Self, Self::Error> {
        // external parent index (4 bytes)
        let size = 4;
        cursor.ensure_bytes::<Self::Error>(size)?;

        let index = i32::parse(config, cursor)?;

        Ok(Self { index })
    }
}

#[derive(Debug, Clone)]
pub struct PmxBoneIK {
    pub index: PmxBoneIndex,
    pub loop_count: i32,
    /// in radians
    pub limit_angle: f32,
    pub links: Vec<PmxBoneIKLink>,
}

impl Parse for PmxBoneIK {
    type Error = PmxBoneParseError;

    fn parse(config: &PmxConfig, cursor: &mut Cursor) -> Result<Self, Self::Error> {
        // bone index (N bytes)
        // loop count (4 bytes)
        // limit angle (4 bytes)
        let size = config.bone_index_size.size() + 4 + 4;
        cursor.ensure_bytes::<Self::Error>(size)?;

        let index = PmxBoneIndex::parse(config, cursor)?;
        let loop_count = i32::parse(config, cursor)?;
        let limit_angle = f32::parse(config, cursor)?;

        // dynamic size
        let links = Vec::<PmxBoneIKLink>::parse(config, cursor)?;

        Ok(Self {
            index,
            loop_count,
            limit_angle,
            links,
        })
    }
}

#[derive(Debug, Clone)]
pub struct PmxBoneIKLink {
    pub index: PmxBoneIndex,
    pub angle_limit: Option<PmxBoneIKAngleLimit>,
}

impl Parse for PmxBoneIKLink {
    type Error = PmxBoneParseError;

    fn parse(config: &PmxConfig, cursor: &mut Cursor) -> Result<Self, Self::Error> {
        // bone index (N bytes)
        let size = config.bone_index_size.size();
        cursor.ensure_bytes::<Self::Error>(size)?;

        let index = PmxBoneIndex::parse(config, cursor)?;

        // dynamic size
        let angle_limit = Option::<PmxBoneIKAngleLimit>::parse(config, cursor)?;

        Ok(Self { index, angle_limit })
    }
}

impl Parse for Vec<PmxBoneIKLink> {
    type Error = PmxBoneParseError;

    fn parse(config: &PmxConfig, cursor: &mut Cursor) -> Result<Self, Self::Error> {
        // link count (4 bytes)
        let size = 4;
        cursor.ensure_bytes::<Self::Error>(size)?;

        let count = u32::parse(config, cursor)? as usize;
        let mut links = Vec::with_capacity(count);

        for _ in 0..count {
            links.push(PmxBoneIKLink::parse(config, cursor)?);
        }

        Ok(links)
    }
}

#[derive(Debug, Clone)]
pub struct PmxBoneIKAngleLimit {
    /// in radians
    pub min: PmxVec3,
    /// in radians
    pub max: PmxVec3,
}

impl Parse for PmxBoneIKAngleLimit {
    type Error = PmxBoneParseError;

    fn parse(config: &PmxConfig, cursor: &mut Cursor) -> Result<Self, Self::Error> {
        // since angle limit has a fixed size, we don't need to check the size here
        let min = PmxVec3::parse(config, cursor)?;
        let max = PmxVec3::parse(config, cursor)?;

        Ok(Self { min, max })
    }
}

impl Parse for Option<PmxBoneIKAngleLimit> {
    type Error = PmxBoneParseError;

    fn parse(config: &PmxConfig, cursor: &mut Cursor) -> Result<Self, Self::Error> {
        // has angle limit (1 byte)
        let size = 1;
        cursor.ensure_bytes::<Self::Error>(size)?;

        let has_angle_limit = bool::parse(config, cursor)?;

        if !has_angle_limit {
            return Ok(None);
        }

        // angle limit (12 * 2 bytes)
        let size = 12 * 2;
        cursor.ensure_bytes::<Self::Error>(size)?;

        let angle_limit = PmxBoneIKAngleLimit::parse(config, cursor)?;

        Ok(Some(angle_limit))
    }
}
