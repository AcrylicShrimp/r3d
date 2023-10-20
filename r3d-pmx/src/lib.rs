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

use cursor::UncheckedCursor;
use parse::Parse;
use pmx_bone::PmxBone;
use pmx_display::PmxDisplay;
use pmx_header::PmxHeader;
use pmx_joint::PmxJoint;
use pmx_material::PmxMaterial;
use pmx_morph::PmxMorph;
use pmx_rigidbody::PmxRigidbody;
use pmx_surface::PmxSurface;
use pmx_texture::PmxTexture;
use pmx_vertex::PmxVertex;
use std::fmt::Display;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum PmxParseError {
    #[error("failed to parse PMX header: {0}")]
    PmxHeaderParseError(#[from] pmx_header::PmxHeaderParseError),
    #[error("failed to parse PMX vertex: {0}")]
    PmxVertexParseError(#[from] pmx_vertex::PmxVertexParseError),
    #[error("failed to parse PMX surface: {0}")]
    PmxSurfaceParseError(#[from] pmx_surface::PmxSurfaceParseError),
    #[error("failed to parse PMX texture: {0}")]
    PmxTextureParseError(#[from] pmx_texture::PmxTextureParseError),
    #[error("failed to parse PMX material: {0}")]
    PmxMaterialParseError(#[from] pmx_material::PmxMaterialParseError),
    #[error("failed to parse PMX bone: {0}")]
    PmxBoneParseError(#[from] pmx_bone::PmxBoneParseError),
    #[error("failed to parse PMX morph: {0}")]
    PmxMorphParseError(#[from] pmx_morph::PmxMorphParseError),
    #[error("failed to parse PMX display: {0}")]
    PmxDisplayParseError(#[from] pmx_display::PmxDisplayParseError),
    #[error("failed to parse PMX rigidbody: {0}")]
    PmxRigidbodyParseError(#[from] pmx_rigidbody::PmxRigidbodyParseError),
    #[error("failed to parse PMX joint: {0}")]
    PmxJointParseError(#[from] pmx_joint::PmxJointParseError),
}

#[derive(Debug, Clone)]
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

impl Pmx {
    pub fn parse(buf: Vec<u8>) -> Result<Self, PmxParseError> {
        let mut cursor = UncheckedCursor::new(buf);

        let header = PmxHeader::parse(&mut cursor)?;
        let vertices = Vec::parse(&header.config, &mut cursor)?;
        let surfaces = Vec::parse(&header.config, &mut cursor)?;
        let textures = Vec::parse(&header.config, &mut cursor)?;
        let materials = Vec::parse(&header.config, &mut cursor)?;
        let bones = Vec::parse(&header.config, &mut cursor)?;
        let morphs = Vec::parse(&header.config, &mut cursor)?;
        let displays = Vec::parse(&header.config, &mut cursor)?;
        let rigidbodies = Vec::parse(&header.config, &mut cursor)?;
        let joints = Vec::parse(&header.config, &mut cursor)?;

        Ok(Self {
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
