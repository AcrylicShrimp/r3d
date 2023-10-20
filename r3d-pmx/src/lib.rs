mod cursor;
mod parse;
mod pmx_bone;
mod pmx_display;
mod pmx_header;
mod pmx_joint;
mod pmx_material;
mod pmx_morph;
mod pmx_primitives;
mod pmx_rigidbody;
mod pmx_surface;
mod pmx_texture;
mod pmx_vertex;
mod primitives;

use byteorder::{ByteOrder, LittleEndian};
use std::fmt::Display;
use thiserror::Error;

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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct PmxConfig {
    pub text_encoding: PmxTextEncoding,
    pub additional_vec4_count: u8,
    pub vertex_index_size: PmxIndexSize,
    pub texture_index_size: PmxIndexSize,
    pub material_index_size: PmxIndexSize,
    pub bone_index_size: PmxIndexSize,
    pub morph_index_size: PmxIndexSize,
    pub rigidbody_index_size: PmxIndexSize,
}

#[derive(Debug, Clone)]
pub struct PmxHeader {
    pub version: f32,
    pub config: PmxConfig,
    pub model_name_local: String,
    pub model_name_universal: String,
    pub model_comment_local: String,
    pub model_comment_universal: String,
}

pub type PmxVertexIndex = u32;
pub type PmxBoneIndex = i32;
pub type PmxTextureIndex = i32;
pub type PmxMaterialIndex = i32;
pub type PmxMorphIndex = i32;
pub type PmxRigidbodyIndex = i32;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PmxVec2 {
    pub x: f32,
    pub y: f32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PmxVec3 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PmxVec4 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub w: f32,
}

#[derive(Debug, Clone)]
pub enum PmxDeformKind {
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

#[derive(Debug, Clone)]
pub struct PmxVertex {
    pub position: PmxVec3,
    pub normal: PmxVec3,
    pub uv: PmxVec2,
    pub additional_vec4s: [PmxVec4; 4], // Up to 4 additional vec4s
    pub deform_kind: PmxDeformKind,
    pub edge_scale: f32,
}

#[derive(Debug, Clone)]
pub struct PmxSurface {
    /// CW winding order
    pub vertex_indices: [PmxVertexIndex; 3],
}

#[derive(Debug, Clone)]
pub struct PmxTexture {
    pub path: String,
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PmxMaterialEnvironmentBlendMode {
    Disabled,
    Multiplicative,
    Additive,
    /// Uses `additional_vec4s[0]` as `uv`.
    AdditionalVec4UV,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PmxMaterialToonMode {
    /// Refers to `textures[texture_index]`.
    Texture { index: PmxTextureIndex },
    /// Refers to the pre-defined internal toon texture with `index`.
    /// The pre-defined textures are usually named as `toon01.bmp` to `toon10.bmp` in the textures.
    InternalTexture { index: u8 },
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
    pub edge_scale: f32,
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PmxBoneInheritanceMode {
    Both,
    RotationOnly,
    TranslationOnly,
}

#[derive(Debug, Clone)]
pub struct PmxBoneInheritance {
    pub index: PmxBoneIndex,
    pub coefficient: f32,
    pub inheritance_mode: PmxBoneInheritanceMode,
}

#[derive(Debug, Clone)]
pub struct PmxBoneFixedAxis {
    pub direction: PmxVec3,
}

#[derive(Debug, Clone)]
pub struct PmxBoneLocalCoordinate {
    pub x_axis: PmxVec3,
    pub z_axis: PmxVec3,
}

#[derive(Debug, Clone)]
pub struct PmxBoneExternalParent {
    pub index: i32, // 4 bytes signed integer, not bone index
}

#[derive(Debug, Clone)]
pub struct PmxBoneIKAngleLimit {
    /// in radians
    pub min: PmxVec3,
    /// in radians
    pub max: PmxVec3,
}

#[derive(Debug, Clone)]
pub struct PmxBoneIKLink {
    pub index: PmxBoneIndex,
    pub angle_limit: Option<PmxBoneIKAngleLimit>,
}

#[derive(Debug, Clone)]
pub struct PmxBoneIK {
    pub index: PmxBoneIndex,
    pub loop_count: i32,
    /// in radians
    pub limit_angle: f32,
    pub links: Vec<PmxBoneIKLink>,
}

#[derive(Debug, Clone)]
pub enum PmxBoneTailPosition {
    Vec3 { position: PmxVec3 },
    BoneIndex { index: PmxBoneIndex },
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

#[derive(Debug, Clone)]
pub enum PmxMorphOffset {
    Group {
        index: PmxMorphIndex,
        coefficient: f32,
    },
    Vertex {
        index: PmxVertexIndex,
        translation: PmxVec3,
    },
    Bone {
        index: PmxBoneIndex,
        translation: PmxVec3,
        rotation: PmxVec4,
    },
    Uv {
        index: PmxVertexIndex,
        vec4: PmxVec4,
        uv_index: u8,
    },
    Material {
        /// -1 for all materials
        index: PmxMaterialIndex,
        diffuse_color: PmxVec4,
        specular_color: PmxVec3,
        specular_strength: f32,
        ambient_color: PmxVec3,
        edge_color: PmxVec4,
        edge_scale: f32,
        texture_tint_color: PmxVec4,
        environment_tint_color: PmxVec4,
        toon_tint_color: PmxVec4,
    },
    Flip {
        index: PmxMorphIndex,
        coefficient: f32,
    },
    Impulse {
        index: PmxRigidbodyIndex,
        /// `true` if `velocity` and `torque` is in local coordinate otherwise `false`.
        is_local: bool,
        velocity: PmxVec3,
        torque: PmxVec3,
    },
}

#[derive(Debug, Clone)]
pub struct PmxMorph {
    pub name_local: String,
    pub name_universal: String,
    pub panel_kind: PmxMorphPanelKind,
    pub offsets: Vec<PmxMorphOffset>,
}

#[derive(Debug, Clone)]
pub enum PmxDisplayFrame {
    Bone { index: PmxBoneIndex },
    Morph { index: PmxMorphIndex },
}

#[derive(Debug, Clone)]
pub struct PmxDisplay {
    pub name_local: String,
    pub name_universal: String,
    pub is_special: bool,
    pub frames: Vec<PmxDisplayFrame>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PmxRigidbodyShapeKind {
    Sphere,
    Box,
    Capsule,
}

#[derive(Debug, Clone)]
pub struct PmxRigidbodyShape {
    pub kind: PmxRigidbodyShapeKind,
    pub size: PmxVec3,
    pub position: PmxVec3,
    /// in radians
    pub rotation: PmxVec3,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PmxRigidbodyPhysicsMode {
    Static,
    Dynamic,
    DynamicWithBone,
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PmxJointKind {
    Spring6Dof,
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

#[derive(Clone)]
pub struct Pmx {
    pub header: PmxHeader,
    pub vertices: Vec<PmxVertex>,
    pub surfaces: Vec<PmxSurface>,
    pub textures: Vec<PmxTexture>,
    pub materials: Vec<PmxMaterial>,
    pub bones: Vec<PmxBone>,
    pub morphs: Vec<PmxMorph>,
    pub displays: Vec<PmxDisplay>,
    pub rigidbodies: Vec<PmxRigidbody>,
    pub joints: Vec<PmxJoint>,
}

impl Display for Pmx {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "PMX v{}", self.header.version)?;
        writeln!(f, "  model name (local): {}", self.header.model_name_local)?;
        writeln!(
            f,
            "  model name (universal): {}",
            self.header.model_name_universal
        )?;
        writeln!(
            f,
            "  model comment (local): {}",
            self.header.model_comment_local
        )?;
        writeln!(
            f,
            "  model comment (universal): {}",
            self.header.model_comment_universal
        )?;
        writeln!(f, "  vertices: {}", self.vertices.len())?;
        writeln!(f, "  surfaces: {}", self.surfaces.len())?;
        writeln!(f, "  textures: {}", self.textures.len())?;
        writeln!(f, "  materials: {}", self.materials.len())?;
        writeln!(f, "  bones: {}", self.bones.len())?;
        writeln!(f, "  morphs: {}", self.morphs.len())?;
        writeln!(f, "  displays: {}", self.displays.len())?;
        writeln!(f, "  rigidbodies: {}", self.rigidbodies.len())?;
        writeln!(f, "  joints: {}", self.joints.len())?;
        Ok(())
    }
}

pub trait PmxCommonError {
    fn buffer_too_short() -> Self;
}

#[derive(Error, Debug)]
pub enum PmxParseError {
    #[error("buffer too short")]
    BufferTooShort,
    #[error("wrong signature")]
    WrongSignature,
    #[error("version `{version}` is not supported")]
    UnsupportedVersion { version: f32 },
    #[error("`{expected}` globals expected, but `{actual}` found")]
    WrongGlobalCount { expected: u8, actual: u8 },
    #[error("wrong global value `{value}` at index `{index}`")]
    WrongGlobalValue { index: usize, value: u8 },
    #[error("wrong vertex weight deform kind `{value}`")]
    WrongWeightDeformKind { value: u8 },
    #[error("surface count `{count}` is not divisible by 3")]
    SurfaceCountNotDivisibleBy3 { count: u32 },
    #[error("wrong material environment blend mode `{value}`")]
    WrongEnvironmentBlendMode { value: u8 },
    #[error("wrong toon mode `{value}`")]
    WrongToonMode { value: u8 },
    #[error("wrong morph panel kind `{value}`")]
    WrongMorphPanelKind { value: u8 },
    #[error("wrong morph offset kind `{value}`")]
    WrongMorphOffsetKind { value: u8 },
    #[error("wrong display frame kind `{value}`")]
    WrongDisplayFrameKind { value: u8 },
    #[error("wrong rigidbody shape kind `{value}`")]
    WrongRigidbodyShapeKind { value: u8 },
    #[error("wrong rigidbody physics mode `{value}`")]
    WrongRigidbodyPhysicsMode { value: u8 },
    #[error("wrong joint kind `{value}`")]
    WrongJointKind { value: u8 },
    #[error("`{context}` has invalid text encoding: {inner}")]
    InvalidTextEncoding {
        context: &'static str,
        inner: PmxTextParseError,
    },
    #[error("invalid pmx format: {0}")]
    TryFromSliceError(#[from] std::array::TryFromSliceError),
}

impl PmxCommonError for PmxParseError {
    fn buffer_too_short() -> Self {
        PmxParseError::BufferTooShort
    }
}

#[derive(Error, Debug)]
pub enum PmxTextParseError {
    #[error("buffer too short")]
    BufferTooShort,
    #[error("invalid utf8: {0}")]
    FromUtf8Error(#[from] std::string::FromUtf8Error),
    #[error("invalid utf16: {0}")]
    FromUtf16Error(#[from] std::string::FromUtf16Error),
    #[error("{len} is too short for utf16 string")]
    Utf16BytesTooShort { len: usize },
}

impl PmxCommonError for PmxTextParseError {
    fn buffer_too_short() -> Self {
        PmxTextParseError::BufferTooShort
    }
}

pub fn parse_pmx(buffer: &[u8]) -> Result<Pmx, PmxParseError> {
    /// Minimum size of PMX 2.0 header.
    /// - 4 bytes: signature
    /// - 4 bytes: version
    /// - 1 byte: global count
    /// - 8 bytes: globals (fixed 8 bytes in PMX 2.0)
    /// - 4 * 4 bytes: model name (local, universal), model comment (local, universal)
    const HEADER_SIZE: usize = 4 + 4 + 1 + 8 + 4 * 4;

    /// Minimum size of PMX 2.0 body.
    /// - 4 bytes: vertex count
    /// - 4 bytes: surface count
    /// - 4 bytes: texture count
    /// - 4 bytes: material count
    /// - 4 bytes: bone count
    /// - 4 bytes: morph count
    /// - 4 bytes: displayframe count
    /// - 4 bytes: rigidbody count
    /// - 4 bytes: joint count
    const BODY_SIZE: usize = 4 * 9;

    if buffer.len() < HEADER_SIZE + BODY_SIZE {
        return Err(PmxParseError::BufferTooShort);
    }

    let mut buffer = buffer;
    let header = parse_pmx_header(&mut buffer)?;
    let vertices = parse_pmx_vertices(
        header.config.bone_index_size,
        header.config.additional_vec4_count,
        &mut buffer,
    )?;
    let surfaces = parse_pmx_surfaces(header.config.vertex_index_size, &mut buffer)?;
    let textures = parse_pmx_textures(header.config.text_encoding, &mut buffer)?;
    let materials = parse_pmx_materials(
        header.config.text_encoding,
        header.config.texture_index_size,
        &mut buffer,
    )?;
    let bones = parse_pmx_bones(
        header.config.text_encoding,
        header.config.bone_index_size,
        &mut buffer,
    )?;
    let morphs = parse_pmx_morphs(&header.config, &mut buffer)?;
    let displays = parse_pmx_displays(&header.config, &mut buffer)?;
    let rigidbodies = parse_pmx_rigidbodies(&header.config, &mut buffer)?;
    let joints = parse_pmx_joints(&header.config, &mut buffer)?;

    Ok(Pmx {
        header,
        vertices,
        surfaces,
        textures,
        materials,
        bones,
        morphs,
        displays,
        rigidbodies,
        joints,
    })
}

fn parse_pmx_header(buffer: &mut &[u8]) -> Result<PmxHeader, PmxParseError> {
    // typically the signature is `PMX ` as 4 bytes, but some files do not have a space at the end
    if &buffer[0..3] != b"PMX" {
        return Err(PmxParseError::WrongSignature);
    }

    let version = f32::from_le_bytes(buffer[4..8].try_into()?);

    // version should be 2.0, with some tolerance
    if version < 1.95 || 2.05 < version {
        return Err(PmxParseError::UnsupportedVersion { version });
    }

    let global_count = buffer[8];

    // global count is fixed to 8 in PMX 2.0
    if global_count != 8 {
        return Err(PmxParseError::WrongGlobalCount {
            expected: 8,
            actual: global_count,
        });
    }

    let globals = &buffer[9..17];
    let text_encoding = match globals[0] {
        0 => PmxTextEncoding::Utf16le,
        1 => PmxTextEncoding::Utf8,
        value => return Err(PmxParseError::WrongGlobalValue { index: 0, value }),
    };
    let additional_vec4_count = match globals[1] {
        value if value <= 4 => value,
        value => return Err(PmxParseError::WrongGlobalValue { index: 1, value }),
    };
    let vertex_index_size = parse_pmx_index_size(globals, 2)?;
    let texture_index_size = parse_pmx_index_size(globals, 3)?;
    let material_index_size = parse_pmx_index_size(globals, 4)?;
    let bone_index_size = parse_pmx_index_size(globals, 5)?;
    let morph_index_size = parse_pmx_index_size(globals, 6)?;
    let rigidbody_index_size = parse_pmx_index_size(globals, 7)?;

    let config = PmxConfig {
        text_encoding,
        additional_vec4_count,
        vertex_index_size,
        texture_index_size,
        material_index_size,
        bone_index_size,
        morph_index_size,
        rigidbody_index_size,
    };

    // advance buffer
    *buffer = &buffer[17..];

    let model_name_local = parse_pmx_text(config.text_encoding, buffer).map_err(|inner| {
        PmxParseError::InvalidTextEncoding {
            context: "model name (local)",
            inner,
        }
    })?;
    let model_name_universal = parse_pmx_text(config.text_encoding, buffer).map_err(|inner| {
        PmxParseError::InvalidTextEncoding {
            context: "model name (universal)",
            inner,
        }
    })?;
    let model_comment_local = parse_pmx_text(config.text_encoding, buffer).map_err(|inner| {
        PmxParseError::InvalidTextEncoding {
            context: "model comment (local)",
            inner,
        }
    })?;
    let model_comment_universal =
        parse_pmx_text(config.text_encoding, buffer).map_err(|inner| {
            PmxParseError::InvalidTextEncoding {
                context: "model comment (universal)",
                inner,
            }
        })?;

    Ok(PmxHeader {
        version,
        config,
        model_name_local,
        model_name_universal,
        model_comment_local,
        model_comment_universal,
    })
}

fn parse_pmx_index_size(globals: &[u8], index: usize) -> Result<PmxIndexSize, PmxParseError> {
    match globals[index] {
        1 => Ok(PmxIndexSize::U8),
        2 => Ok(PmxIndexSize::U16),
        4 => Ok(PmxIndexSize::U32),
        value => Err(PmxParseError::WrongGlobalValue { index, value }),
    }
}

fn parse_pmx_vertices(
    bone_index_size: PmxIndexSize,
    additional_vec4_count: u8,
    buffer: &mut &[u8],
) -> Result<Vec<PmxVertex>, PmxParseError> {
    let count = parse_u32::<PmxParseError>(buffer)?;
    let mut vertices = Vec::with_capacity(count as usize);

    for _ in 0..count {
        let position = parse_pmx_vec3(buffer)?;
        let normal = parse_pmx_vec3(buffer)?;
        let uv = parse_pmx_vec2(buffer)?;
        let mut additional_vec4s = [PmxVec4 {
            x: 0.0,
            y: 0.0,
            z: 0.0,
            w: 0.0,
        }; 4];

        for index in 0..additional_vec4_count {
            additional_vec4s[index as usize] = parse_pmx_vec4(buffer)?;
        }

        let deform_kind = parse_u8::<PmxParseError>(buffer)?;
        let deform_kind = match deform_kind {
            0 => parse_pmx_deform_kind_bdef1(bone_index_size, buffer)?,
            1 => parse_pmx_deform_kind_bdef2(bone_index_size, buffer)?,
            2 => parse_pmx_deform_kind_bdef4(bone_index_size, buffer)?,
            3 => parse_pmx_deform_kind_sdef(bone_index_size, buffer)?,
            value => return Err(PmxParseError::WrongWeightDeformKind { value }),
        };
        let edge_scale = parse_float::<PmxParseError>(buffer)?;

        vertices.push(PmxVertex {
            position,
            normal,
            uv,
            additional_vec4s,
            deform_kind,
            edge_scale,
        });
    }

    Ok(vertices)
}

fn parse_pmx_deform_kind_bdef1(
    bone_index_size: PmxIndexSize,
    buffer: &mut &[u8],
) -> Result<PmxDeformKind, PmxParseError> {
    let bone_index = parse_pmx_non_vertex_index(bone_index_size, buffer)?;

    Ok(PmxDeformKind::Bdef1 { bone_index })
}

fn parse_pmx_deform_kind_bdef2(
    bone_index_size: PmxIndexSize,
    buffer: &mut &[u8],
) -> Result<PmxDeformKind, PmxParseError> {
    let bone_index_1 = parse_pmx_non_vertex_index(bone_index_size, buffer)?;
    let bone_index_2 = parse_pmx_non_vertex_index(bone_index_size, buffer)?;
    let bone_weight = parse_float::<PmxParseError>(buffer)?;

    Ok(PmxDeformKind::Bdef2 {
        bone_index_1,
        bone_index_2,
        bone_weight,
    })
}

fn parse_pmx_deform_kind_bdef4(
    bone_index_size: PmxIndexSize,
    buffer: &mut &[u8],
) -> Result<PmxDeformKind, PmxParseError> {
    let bone_index_1 = parse_pmx_non_vertex_index(bone_index_size, buffer)?;
    let bone_index_2 = parse_pmx_non_vertex_index(bone_index_size, buffer)?;
    let bone_index_3 = parse_pmx_non_vertex_index(bone_index_size, buffer)?;
    let bone_index_4 = parse_pmx_non_vertex_index(bone_index_size, buffer)?;

    if buffer.len() < 16 {
        return Err(PmxParseError::BufferTooShort);
    }

    let mut bone_weight_1 = f32::from_le_bytes(buffer[0..4].try_into().unwrap());
    let mut bone_weight_2 = f32::from_le_bytes(buffer[4..8].try_into().unwrap());
    let mut bone_weight_3 = f32::from_le_bytes(buffer[8..12].try_into().unwrap());
    let mut bone_weight_4 = f32::from_le_bytes(buffer[12..16].try_into().unwrap());

    // advance buffer
    *buffer = &buffer[16..];

    // normalize weights
    let sum = bone_weight_1 + bone_weight_2 + bone_weight_3 + bone_weight_4;

    if 1e-3 <= f32::abs(sum) {
        let scale = 1.0 / sum;
        bone_weight_1 *= scale;
        bone_weight_2 *= scale;
        bone_weight_3 *= scale;
        bone_weight_4 *= scale;
    }

    Ok(PmxDeformKind::Bdef4 {
        bone_index_1,
        bone_index_2,
        bone_index_3,
        bone_index_4,
        bone_weight_1,
        bone_weight_2,
        bone_weight_3,
        bone_weight_4,
    })
}

fn parse_pmx_deform_kind_sdef(
    bone_index_size: PmxIndexSize,
    buffer: &mut &[u8],
) -> Result<PmxDeformKind, PmxParseError> {
    let bone_index_1 = parse_pmx_non_vertex_index(bone_index_size, buffer)?;
    let bone_index_2 = parse_pmx_non_vertex_index(bone_index_size, buffer)?;
    let bone_weight = parse_float::<PmxParseError>(buffer)?;

    let c = parse_pmx_vec3(buffer)?;
    let r0 = parse_pmx_vec3(buffer)?;
    let r1 = parse_pmx_vec3(buffer)?;

    Ok(PmxDeformKind::Sdef {
        bone_index_1,
        bone_index_2,
        bone_weight,
        c,
        r0,
        r1,
    })
}

fn parse_pmx_surfaces(
    vertex_index_size: PmxIndexSize,
    buffer: &mut &[u8],
) -> Result<Vec<PmxSurface>, PmxParseError> {
    let mut count = parse_u32::<PmxParseError>(buffer)?;

    if count % 3 != 0 {
        return Err(PmxParseError::SurfaceCountNotDivisibleBy3 { count });
    }

    // 3 vertex indices per surface
    count /= 3;

    let mut surfaces = Vec::with_capacity(count as usize);

    for _ in 0..count {
        let vertex_index_1 = parse_pmx_vertex_index(vertex_index_size, buffer)?;
        let vertex_index_2 = parse_pmx_vertex_index(vertex_index_size, buffer)?;
        let vertex_index_3 = parse_pmx_vertex_index(vertex_index_size, buffer)?;

        surfaces.push(PmxSurface {
            vertex_indices: [vertex_index_1, vertex_index_2, vertex_index_3],
        });
    }

    Ok(surfaces)
}

fn parse_pmx_textures(
    text_encoding: PmxTextEncoding,
    buffer: &mut &[u8],
) -> Result<Vec<PmxTexture>, PmxParseError> {
    let count = parse_u32::<PmxParseError>(buffer)?;
    let mut textures = Vec::with_capacity(count as usize);

    for _ in 0..count {
        let path = parse_pmx_text(text_encoding, buffer).map_err(|inner| {
            PmxParseError::InvalidTextEncoding {
                context: "texture path",
                inner,
            }
        })?;

        // replace `\` with `/`
        let path = path.replace('\\', "/");

        textures.push(PmxTexture { path });
    }

    Ok(textures)
}

fn parse_pmx_materials(
    text_encoding: PmxTextEncoding,
    texture_index_size: PmxIndexSize,
    buffer: &mut &[u8],
) -> Result<Vec<PmxMaterial>, PmxParseError> {
    let count = parse_u32::<PmxParseError>(buffer)?;
    let mut materials = Vec::with_capacity(count as usize);

    for _ in 0..count {
        let name_local = parse_pmx_text(text_encoding, buffer).map_err(|inner| {
            PmxParseError::InvalidTextEncoding {
                context: "material name (local)",
                inner,
            }
        })?;
        let name_universal = parse_pmx_text(text_encoding, buffer).map_err(|inner| {
            PmxParseError::InvalidTextEncoding {
                context: "material name (universal)",
                inner,
            }
        })?;
        let diffuse_color = parse_pmx_vec4(buffer)?;
        let specular_color = parse_pmx_vec3(buffer)?;
        let specular_strength = parse_float::<PmxParseError>(buffer)?;
        let ambient_color = parse_pmx_vec3(buffer)?;
        let flags = parse_pmx_material_flags(buffer)?;
        let edge_color = parse_pmx_vec4(buffer)?;
        let edge_scale = parse_float::<PmxParseError>(buffer)?;
        let texture_index = parse_pmx_non_vertex_index(texture_index_size, buffer)?;
        let environment_texture_index = parse_pmx_non_vertex_index(texture_index_size, buffer)?;
        let environment_blend_mode = parse_u8::<PmxParseError>(buffer)?;
        let environment_blend_mode = match environment_blend_mode {
            0 => PmxMaterialEnvironmentBlendMode::Disabled,
            1 => PmxMaterialEnvironmentBlendMode::Multiplicative,
            2 => PmxMaterialEnvironmentBlendMode::Additive,
            3 => PmxMaterialEnvironmentBlendMode::AdditionalVec4UV,
            value => return Err(PmxParseError::WrongEnvironmentBlendMode { value }),
        };
        let toon_mode = parse_u8::<PmxParseError>(buffer)?;
        let toon_mode = match toon_mode {
            0 => PmxMaterialToonMode::Texture {
                index: parse_pmx_non_vertex_index(texture_index_size, buffer)?,
            },
            1 => PmxMaterialToonMode::InternalTexture {
                index: parse_u8::<PmxParseError>(buffer)?,
            },
            value => return Err(PmxParseError::WrongToonMode { value }),
        };
        let metadata = parse_pmx_text(text_encoding, buffer).map_err(|inner| {
            PmxParseError::InvalidTextEncoding {
                context: "material metadata",
                inner,
            }
        })?;
        let surface_count = parse_u32::<PmxParseError>(buffer)?;

        materials.push(PmxMaterial {
            name_local,
            name_universal,
            diffuse_color,
            specular_color,
            specular_strength,
            ambient_color,
            flags,
            edge_color,
            edge_scale,
            texture_index,
            environment_texture_index,
            environment_blend_mode,
            toon_mode,
            metadata,
            surface_count,
        })
    }

    Ok(materials)
}

fn parse_pmx_material_flags(buffer: &mut &[u8]) -> Result<PmxMaterialFlags, PmxParseError> {
    let flags = parse_u8::<PmxParseError>(buffer)?;

    let cull_back_face = flags & 0b0000_0001 != 0;
    let cast_shadow_on_ground = flags & 0b0000_0010 != 0;
    let cast_shadow_on_object = flags & 0b0000_0100 != 0;
    let receive_shadow = flags & 0b0000_1000 != 0;
    let has_edge = flags & 0b0001_0000 != 0;

    return Ok(PmxMaterialFlags {
        cull_back_face,
        cast_shadow_on_ground,
        cast_shadow_on_object,
        receive_shadow,
        has_edge,
    });
}

fn parse_pmx_bones(
    text_encoding: PmxTextEncoding,
    bone_index_size: PmxIndexSize,
    buffer: &mut &[u8],
) -> Result<Vec<PmxBone>, PmxParseError> {
    let count = parse_u32::<PmxParseError>(buffer)?;
    let mut bones = Vec::with_capacity(count as usize);

    for _ in 0..count {
        let name_local = parse_pmx_text(text_encoding, buffer).map_err(|inner| {
            PmxParseError::InvalidTextEncoding {
                context: "bone name (local)",
                inner,
            }
        })?;
        let name_universal = parse_pmx_text(text_encoding, buffer).map_err(|inner| {
            PmxParseError::InvalidTextEncoding {
                context: "bone name (universal)",
                inner,
            }
        })?;
        let position = parse_pmx_vec3(buffer)?;
        let parent_index = parse_pmx_non_vertex_index(bone_index_size, buffer)?;
        let layer = parse_u32::<PmxParseError>(buffer)?;
        let flags = parse_pmx_bone_flags(buffer)?;
        let tail_position = match flags.indexed_tail_position {
            true => PmxBoneTailPosition::BoneIndex {
                index: parse_pmx_non_vertex_index(bone_index_size, buffer)?,
            },
            false => PmxBoneTailPosition::Vec3 {
                position: parse_pmx_vec3(buffer)?,
            },
        };
        let inheritance = match (flags.inherit_rotation, flags.inherit_translation) {
            (true, true) => {
                let index = parse_pmx_non_vertex_index(bone_index_size, buffer)?;
                let coefficient = parse_float::<PmxParseError>(buffer)?;
                let inheritance_mode = PmxBoneInheritanceMode::Both;

                Some(PmxBoneInheritance {
                    index,
                    coefficient,
                    inheritance_mode,
                })
            }
            (true, false) => {
                let index = parse_pmx_non_vertex_index(bone_index_size, buffer)?;
                let coefficient = parse_float::<PmxParseError>(buffer)?;
                let inheritance_mode = PmxBoneInheritanceMode::RotationOnly;

                Some(PmxBoneInheritance {
                    index,
                    coefficient,
                    inheritance_mode,
                })
            }
            (false, true) => {
                let index = parse_pmx_non_vertex_index(bone_index_size, buffer)?;
                let coefficient = parse_float::<PmxParseError>(buffer)?;
                let inheritance_mode = PmxBoneInheritanceMode::TranslationOnly;

                Some(PmxBoneInheritance {
                    index,
                    coefficient,
                    inheritance_mode,
                })
            }
            (false, false) => None,
        };
        let fixed_axis = match flags.fixed_axis {
            true => {
                let direction = parse_pmx_vec3(buffer)?;

                Some(PmxBoneFixedAxis { direction })
            }
            false => None,
        };
        let local_coordinate = match flags.local_coordinate {
            true => {
                let x_axis = parse_pmx_vec3(buffer)?;
                let z_axis = parse_pmx_vec3(buffer)?;

                Some(PmxBoneLocalCoordinate { x_axis, z_axis })
            }
            false => None,
        };
        let external_parent = match flags.external_parent_deform {
            true => {
                let index = parse_i32::<PmxParseError>(buffer)?;

                Some(PmxBoneExternalParent { index })
            }
            false => None,
        };
        let ik = match flags.supports_ik {
            true => {
                let index = parse_pmx_non_vertex_index(bone_index_size, buffer)?;
                let loop_count = parse_i32::<PmxParseError>(buffer)?;
                let limit_angle = parse_float::<PmxParseError>(buffer)?;
                let link_count = parse_u32::<PmxParseError>(buffer)?;
                let mut links = Vec::with_capacity(link_count as usize);

                for _ in 0..link_count {
                    let index = parse_pmx_non_vertex_index(bone_index_size, buffer)?;
                    let has_limit = parse_u8::<PmxParseError>(buffer)? != 0;
                    let angle_limit = if has_limit {
                        let min = parse_pmx_vec3(buffer)?;
                        let max = parse_pmx_vec3(buffer)?;

                        Some(PmxBoneIKAngleLimit { min, max })
                    } else {
                        None
                    };

                    links.push(PmxBoneIKLink { index, angle_limit });
                }

                Some(PmxBoneIK {
                    index,
                    loop_count,
                    limit_angle,
                    links,
                })
            }
            false => None,
        };

        bones.push(PmxBone {
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
        });
    }

    Ok(bones)
}

fn parse_pmx_bone_flags(buffer: &mut &[u8]) -> Result<PmxBoneFlags, PmxParseError> {
    if buffer.len() < 2 {
        return Err(PmxParseError::BufferTooShort);
    }

    let flags = [buffer[0], buffer[1]];

    // advance buffer
    *buffer = &buffer[2..];

    let indexed_tail_position = flags[0] & 0b0000_0001 != 0;
    let is_rotatable = flags[0] & 0b0000_0010 != 0;
    let is_translatable = flags[0] & 0b0000_0100 != 0;
    let is_visible = flags[0] & 0b0000_1000 != 0;
    let is_enabled = flags[0] & 0b0001_0000 != 0;
    let supports_ik = flags[0] & 0b0010_0000 != 0;
    let inherit_rotation = flags[1] & 0b0000_0001 != 0;
    let inherit_translation = flags[1] & 0b0000_0010 != 0;
    let fixed_axis = flags[1] & 0b0000_0100 != 0;
    let local_coordinate = flags[1] & 0b0000_1000 != 0;
    let physics_after_deform = flags[1] & 0b0001_0000 != 0;
    let external_parent_deform = flags[1] & 0b0010_0000 != 0;

    Ok(PmxBoneFlags {
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

fn parse_pmx_morphs(
    config: &PmxConfig,
    buffer: &mut &[u8],
) -> Result<Vec<PmxMorph>, PmxParseError> {
    let count = parse_u32::<PmxParseError>(buffer)?;
    let mut morphs = Vec::with_capacity(count as usize);

    for _ in 0..count {
        let name_local = parse_pmx_text(config.text_encoding, buffer).map_err(|inner| {
            PmxParseError::InvalidTextEncoding {
                context: "morph name (local)",
                inner,
            }
        })?;
        let name_universal = parse_pmx_text(config.text_encoding, buffer).map_err(|inner| {
            PmxParseError::InvalidTextEncoding {
                context: "morph name (universal)",
                inner,
            }
        })?;
        let panel_kind = parse_u8::<PmxParseError>(buffer)?;
        let panel_kind = match panel_kind {
            0 => PmxMorphPanelKind::Hidden,
            1 => PmxMorphPanelKind::Eyebrows,
            2 => PmxMorphPanelKind::Eyes,
            3 => PmxMorphPanelKind::Mouth,
            4 => PmxMorphPanelKind::Other,
            value => return Err(PmxParseError::WrongMorphPanelKind { value }),
        };
        let offset_kind = parse_u8::<PmxParseError>(buffer)?;
        let offset_count = parse_u32::<PmxParseError>(buffer)?;

        let mut offsets = Vec::with_capacity(offset_count as usize);

        for _ in 0..offset_count {
            match offset_kind {
                0 => {
                    let index = parse_pmx_non_vertex_index(config.morph_index_size, buffer)?;
                    let coefficient = parse_float::<PmxParseError>(buffer)?;

                    offsets.push(PmxMorphOffset::Group { index, coefficient });
                }
                1 => {
                    let index = parse_pmx_vertex_index(config.vertex_index_size, buffer)?;
                    let translation = parse_pmx_vec3(buffer)?;

                    offsets.push(PmxMorphOffset::Vertex { index, translation });
                }
                2 => {
                    let index = parse_pmx_non_vertex_index(config.bone_index_size, buffer)?;
                    let translation = parse_pmx_vec3(buffer)?;
                    let rotation = parse_pmx_vec4(buffer)?;

                    offsets.push(PmxMorphOffset::Bone {
                        index,
                        translation,
                        rotation,
                    });
                }
                uv_index @ 3..=7 => {
                    let index = parse_pmx_vertex_index(config.vertex_index_size, buffer)?;
                    let vec4 = parse_pmx_vec4(buffer)?;

                    offsets.push(PmxMorphOffset::Uv {
                        index,
                        vec4,
                        uv_index: uv_index - 3,
                    });
                }
                8 => {
                    let index = parse_pmx_non_vertex_index(config.material_index_size, buffer)?;
                    let _unused = parse_u8::<PmxParseError>(buffer)?;
                    let diffuse_color = parse_pmx_vec4(buffer)?;
                    let specular_color = parse_pmx_vec3(buffer)?;
                    let specular_strength = parse_float::<PmxParseError>(buffer)?;
                    let ambient_color = parse_pmx_vec3(buffer)?;
                    let edge_color = parse_pmx_vec4(buffer)?;
                    let edge_scale = parse_float::<PmxParseError>(buffer)?;
                    let texture_tint_color = parse_pmx_vec4(buffer)?;
                    let environment_tint_color = parse_pmx_vec4(buffer)?;
                    let toon_tint_color = parse_pmx_vec4(buffer)?;

                    offsets.push(PmxMorphOffset::Material {
                        index,
                        diffuse_color,
                        specular_color,
                        specular_strength,
                        ambient_color,
                        edge_color,
                        edge_scale,
                        texture_tint_color,
                        environment_tint_color,
                        toon_tint_color,
                    });
                }
                9 => {
                    let index = parse_pmx_non_vertex_index(config.morph_index_size, buffer)?;
                    let coefficient = parse_float::<PmxParseError>(buffer)?;

                    offsets.push(PmxMorphOffset::Flip { index, coefficient });
                }
                10 => {
                    let index = parse_pmx_non_vertex_index(config.rigidbody_index_size, buffer)?;
                    let is_local = parse_u8::<PmxParseError>(buffer)? != 0;
                    let velocity = parse_pmx_vec3(buffer)?;
                    let torque = parse_pmx_vec3(buffer)?;

                    offsets.push(PmxMorphOffset::Impulse {
                        index,
                        is_local,
                        velocity,
                        torque,
                    });
                }
                value => return Err(PmxParseError::WrongMorphOffsetKind { value }),
            }
        }

        morphs.push(PmxMorph {
            name_local,
            name_universal,
            panel_kind,
            offsets,
        });
    }

    Ok(morphs)
}

fn parse_pmx_displays(
    config: &PmxConfig,
    buffer: &mut &[u8],
) -> Result<Vec<PmxDisplay>, PmxParseError> {
    let count = parse_u32::<PmxParseError>(buffer)?;
    let mut displays = Vec::with_capacity(count as usize);

    for _ in 0..count {
        let name_local = parse_pmx_text(config.text_encoding, buffer).map_err(|inner| {
            PmxParseError::InvalidTextEncoding {
                context: "display name (local)",
                inner,
            }
        })?;
        let name_universal = parse_pmx_text(config.text_encoding, buffer).map_err(|inner| {
            PmxParseError::InvalidTextEncoding {
                context: "display name (universal)",
                inner,
            }
        })?;
        let is_special = parse_u8::<PmxParseError>(buffer)? != 0;
        let frame_count = parse_u32::<PmxParseError>(buffer)?;
        let mut frames = Vec::with_capacity(frame_count as usize);

        for _ in 0..frame_count {
            let frame_type = parse_u8::<PmxParseError>(buffer)?;

            match frame_type {
                0 => {
                    let index = parse_pmx_non_vertex_index(config.bone_index_size, buffer)?;

                    frames.push(PmxDisplayFrame::Bone { index });
                }
                1 => {
                    let index = parse_pmx_non_vertex_index(config.morph_index_size, buffer)?;

                    frames.push(PmxDisplayFrame::Morph { index });
                }
                value => return Err(PmxParseError::WrongDisplayFrameKind { value }),
            }
        }

        displays.push(PmxDisplay {
            name_local,
            name_universal,
            is_special,
            frames,
        });
    }

    Ok(displays)
}

fn parse_pmx_rigidbodies(
    config: &PmxConfig,
    buffer: &mut &[u8],
) -> Result<Vec<PmxRigidbody>, PmxParseError> {
    let count = parse_u32::<PmxParseError>(buffer)?;
    let mut rigidbodies = Vec::with_capacity(count as usize);

    for _ in 0..count {
        let name_local = parse_pmx_text(config.text_encoding, buffer).map_err(|inner| {
            PmxParseError::InvalidTextEncoding {
                context: "rigidbody name (local)",
                inner,
            }
        })?;
        let name_universal = parse_pmx_text(config.text_encoding, buffer).map_err(|inner| {
            PmxParseError::InvalidTextEncoding {
                context: "rigidbody name (universal)",
                inner,
            }
        })?;
        let bone_index = parse_pmx_non_vertex_index(config.bone_index_size, buffer)?;
        let group_id = parse_i8::<PmxParseError>(buffer)?;
        let non_collision_group = parse_i16::<PmxParseError>(buffer)?;
        let shape = {
            let kind = parse_u8::<PmxParseError>(buffer)?;
            let kind = match kind {
                0 => PmxRigidbodyShapeKind::Sphere,
                1 => PmxRigidbodyShapeKind::Box,
                2 => PmxRigidbodyShapeKind::Capsule,
                value => return Err(PmxParseError::WrongRigidbodyShapeKind { value }),
            };
            let size = parse_pmx_vec3(buffer)?;
            let position = parse_pmx_vec3(buffer)?;
            let rotation = parse_pmx_vec3(buffer)?;

            PmxRigidbodyShape {
                kind,
                size,
                position,
                rotation,
            }
        };
        let mass = parse_float::<PmxParseError>(buffer)?;
        let linear_damping = parse_float::<PmxParseError>(buffer)?;
        let angular_damping = parse_float::<PmxParseError>(buffer)?;
        let restitution_coefficient = parse_float::<PmxParseError>(buffer)?;
        let friction_coefficient = parse_float::<PmxParseError>(buffer)?;
        let physics_mode = parse_u8::<PmxParseError>(buffer)?;
        let physics_mode = match physics_mode {
            0 => PmxRigidbodyPhysicsMode::Static,
            1 => PmxRigidbodyPhysicsMode::Dynamic,
            2 => PmxRigidbodyPhysicsMode::DynamicWithBone,
            value => return Err(PmxParseError::WrongRigidbodyPhysicsMode { value }),
        };

        rigidbodies.push(PmxRigidbody {
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
        });
    }

    Ok(rigidbodies)
}

fn parse_pmx_joints(
    config: &PmxConfig,
    buffer: &mut &[u8],
) -> Result<Vec<PmxJoint>, PmxParseError> {
    let count = parse_u32::<PmxParseError>(buffer)?;
    let mut joints = Vec::with_capacity(count as usize);

    for _ in 0..count {
        let name_local = parse_pmx_text(config.text_encoding, buffer).map_err(|inner| {
            PmxParseError::InvalidTextEncoding {
                context: "joint name (local)",
                inner,
            }
        })?;
        let name_universal = parse_pmx_text(config.text_encoding, buffer).map_err(|inner| {
            PmxParseError::InvalidTextEncoding {
                context: "joint name (universal)",
                inner,
            }
        })?;
        let kind = parse_u8::<PmxParseError>(buffer)?;
        let kind = match kind {
            0 => PmxJointKind::Spring6Dof,
            value => return Err(PmxParseError::WrongJointKind { value }),
        };
        let rigid_body_index_1 = parse_pmx_non_vertex_index(config.rigidbody_index_size, buffer)?;
        let rigid_body_index_2 = parse_pmx_non_vertex_index(config.rigidbody_index_size, buffer)?;
        let position = parse_pmx_vec3(buffer)?;
        let rotation = parse_pmx_vec3(buffer)?;
        let position_limit_min = parse_pmx_vec3(buffer)?;
        let position_limit_max = parse_pmx_vec3(buffer)?;
        let rotation_limit_min = parse_pmx_vec3(buffer)?;
        let rotation_limit_max = parse_pmx_vec3(buffer)?;
        let spring_position = parse_pmx_vec3(buffer)?;
        let spring_rotation = parse_pmx_vec3(buffer)?;

        joints.push(PmxJoint {
            name_local,
            name_universal,
            kind,
            rigidbody_index_pair: (rigid_body_index_1, rigid_body_index_2),
            position,
            rotation,
            position_limit_min,
            position_limit_max,
            rotation_limit_min,
            rotation_limit_max,
            spring_position,
            spring_rotation,
        });
    }

    Ok(joints)
}

fn parse_pmx_vertex_index(
    index_size: PmxIndexSize,
    buffer: &mut &[u8],
) -> Result<u32, PmxParseError> {
    match index_size {
        PmxIndexSize::U8 => {
            if buffer.len() < 1 {
                return Err(PmxParseError::BufferTooShort);
            }

            let index = buffer[0];

            // advance buffer
            *buffer = &buffer[1..];

            Ok(index as u32)
        }
        PmxIndexSize::U16 => {
            if buffer.len() < 2 {
                return Err(PmxParseError::BufferTooShort);
            }

            let index = u16::from_le_bytes(buffer[0..2].try_into().unwrap());

            // advance buffer
            *buffer = &buffer[2..];

            Ok(index as u32)
        }
        PmxIndexSize::U32 => {
            if buffer.len() < 4 {
                return Err(PmxParseError::BufferTooShort);
            }

            let index = u32::from_le_bytes(buffer[0..4].try_into().unwrap());

            // advance buffer
            *buffer = &buffer[4..];

            Ok(index)
        }
    }
}

fn parse_pmx_non_vertex_index(
    index_size: PmxIndexSize,
    buffer: &mut &[u8],
) -> Result<i32, PmxParseError> {
    match index_size {
        PmxIndexSize::U8 => {
            if buffer.len() < 1 {
                return Err(PmxParseError::BufferTooShort);
            }

            let index = i8::from_le_bytes(buffer[0..1].try_into().unwrap());

            // advance buffer
            *buffer = &buffer[1..];

            Ok(index as i32)
        }
        PmxIndexSize::U16 => {
            if buffer.len() < 2 {
                return Err(PmxParseError::BufferTooShort);
            }

            let index = i16::from_le_bytes(buffer[0..2].try_into().unwrap());

            // advance buffer
            *buffer = &buffer[2..];

            Ok(index as i32)
        }
        PmxIndexSize::U32 => {
            if buffer.len() < 4 {
                return Err(PmxParseError::BufferTooShort);
            }

            let index = i32::from_le_bytes(buffer[0..4].try_into().unwrap());

            // advance buffer
            *buffer = &buffer[4..];

            Ok(index)
        }
    }
}

fn parse_pmx_text(
    text_encoding: PmxTextEncoding,
    buffer: &mut &[u8],
) -> Result<String, PmxTextParseError> {
    let len = parse_u32::<PmxTextParseError>(buffer)? as usize;

    if buffer.len() < len {
        return Err(PmxTextParseError::BufferTooShort);
    }

    let bytes = &buffer[0..len];

    // advance buffer
    *buffer = &buffer[len..];

    match text_encoding {
        PmxTextEncoding::Utf16le => {
            if bytes.len() & 1 != 0 {
                return Err(PmxTextParseError::Utf16BytesTooShort { len: bytes.len() });
            }

            let u16_slice: Vec<_> = bytes
                .chunks_exact(2)
                .map(|bytes| LittleEndian::read_u16(bytes))
                .collect();

            Ok(String::from_utf16(&u16_slice)?)
        }
        PmxTextEncoding::Utf8 => Ok(String::from_utf8(bytes.to_vec())?),
    }
}

fn parse_pmx_vec2(buffer: &mut &[u8]) -> Result<PmxVec2, PmxParseError> {
    if buffer.len() < 8 {
        return Err(PmxParseError::BufferTooShort);
    }

    let x = f32::from_le_bytes(buffer[0..4].try_into().unwrap());
    let y = f32::from_le_bytes(buffer[4..8].try_into().unwrap());

    // advance buffer
    *buffer = &buffer[8..];

    Ok(PmxVec2 { x, y })
}

fn parse_pmx_vec3(buffer: &mut &[u8]) -> Result<PmxVec3, PmxParseError> {
    if buffer.len() < 12 {
        return Err(PmxParseError::BufferTooShort);
    }

    let x = f32::from_le_bytes(buffer[0..4].try_into().unwrap());
    let y = f32::from_le_bytes(buffer[4..8].try_into().unwrap());
    let z = f32::from_le_bytes(buffer[8..12].try_into().unwrap());

    // advance buffer
    *buffer = &buffer[12..];

    Ok(PmxVec3 { x, y, z })
}

fn parse_pmx_vec4(buffer: &mut &[u8]) -> Result<PmxVec4, PmxParseError> {
    if buffer.len() < 16 {
        return Err(PmxParseError::BufferTooShort);
    }

    let x = f32::from_le_bytes(buffer[0..4].try_into().unwrap());
    let y = f32::from_le_bytes(buffer[4..8].try_into().unwrap());
    let z = f32::from_le_bytes(buffer[8..12].try_into().unwrap());
    let w = f32::from_le_bytes(buffer[12..16].try_into().unwrap());

    // advance buffer
    *buffer = &buffer[16..];

    Ok(PmxVec4 { x, y, z, w })
}

fn parse_u8<E: PmxCommonError>(buffer: &mut &[u8]) -> Result<u8, E> {
    if buffer.len() < 1 {
        return Err(E::buffer_too_short());
    }

    let value = buffer[0];

    // advance buffer
    *buffer = &buffer[1..];

    Ok(value)
}

fn parse_i8<E: PmxCommonError>(buffer: &mut &[u8]) -> Result<i8, E> {
    if buffer.len() < 1 {
        return Err(E::buffer_too_short());
    }

    let value = i8::from_le_bytes(buffer[0..1].try_into().unwrap());

    // advance buffer
    *buffer = &buffer[1..];

    Ok(value)
}

fn parse_u16<E: PmxCommonError>(buffer: &mut &[u8]) -> Result<u16, E> {
    if buffer.len() < 2 {
        return Err(E::buffer_too_short());
    }

    let value = u16::from_le_bytes(buffer[0..2].try_into().unwrap());

    // advance buffer
    *buffer = &buffer[2..];

    Ok(value)
}

fn parse_i16<E: PmxCommonError>(buffer: &mut &[u8]) -> Result<i16, E> {
    if buffer.len() < 2 {
        return Err(E::buffer_too_short());
    }

    let value = i16::from_le_bytes(buffer[0..2].try_into().unwrap());

    // advance buffer
    *buffer = &buffer[2..];

    Ok(value)
}

fn parse_u32<E: PmxCommonError>(buffer: &mut &[u8]) -> Result<u32, E> {
    if buffer.len() < 4 {
        return Err(E::buffer_too_short());
    }

    let value = u32::from_le_bytes(buffer[0..4].try_into().unwrap());

    // advance buffer
    *buffer = &buffer[4..];

    Ok(value)
}

fn parse_i32<E: PmxCommonError>(buffer: &mut &[u8]) -> Result<i32, E> {
    if buffer.len() < 4 {
        return Err(E::buffer_too_short());
    }

    let value = i32::from_le_bytes(buffer[0..4].try_into().unwrap());

    // advance buffer
    *buffer = &buffer[4..];

    Ok(value)
}

fn parse_float<E: PmxCommonError>(buffer: &mut &[u8]) -> Result<f32, E> {
    if buffer.len() < 4 {
        return Err(E::buffer_too_short());
    }

    let value = f32::from_le_bytes(buffer[0..4].try_into().unwrap());

    // advance buffer
    *buffer = &buffer[4..];

    Ok(value)
}
