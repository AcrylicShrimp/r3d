use crate::{Asset, AssetDepsProvider, AssetLoadError, AssetSource, GfxBridge, TypedAsset};
use serde::{Deserialize, Serialize};
use std::{
    num::{NonZeroU32, NonZeroU64},
    sync::Arc,
};
use uuid::Uuid;
use wgpu::{
    BufferAddress, SamplerBindingType, TextureSampleType, TextureViewDimension, VertexAttribute,
    VertexStepMode,
};

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SemanticShaderBindingKey(NonZeroU32);

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SemanticShaderInputKey(NonZeroU32);

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SemanticShaderOutputKey(NonZeroU32);

/// A fully reflected shader.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ShaderReflection {
    pub vertex_entry_point: String,
    pub fragment_entry_point: String,
    pub globals: Vec<ShaderGlobalItem>,
    pub vertex_input: ShaderInput,
    pub instance_input: ShaderInput,
    pub outputs: Vec<ShaderOutputItem>,
}

/// Represents a shader global item, a.k.a. uniform.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ShaderGlobalItem {
    pub sematic_key: Option<SemanticShaderBindingKey>,
    pub name: String,
    pub group: u32,
    pub binding: u32,
    pub kind: ShaderGlobalItemKind,
}

/// A shader global item kind, e.g. a buffer, a texture, or a sampler.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum ShaderGlobalItemKind {
    Buffer {
        size: NonZeroU64,
    },
    Texture {
        sample_type: TextureSampleType,
        view_dimension: TextureViewDimension,
        multisampled: bool,
        array_size: Option<NonZeroU32>,
    },
    Sampler {
        binding_type: SamplerBindingType,
    },
}

/// Represents a struct of shader input. This single struct contains all the
/// fields of the input of corresponding step mode.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ShaderInput {
    pub step_mode: VertexStepMode,
    pub stride: BufferAddress,
    pub fields: Vec<ShaderInputField>,
}

/// Each field of a shader input struct.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ShaderInputField {
    pub semantic_key: Option<SemanticShaderInputKey>,
    pub name: String,
    pub attribute: VertexAttribute,
}

/// Each output item of a shader, e.g. a color output.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ShaderOutputItem {
    pub semantic_key: Option<SemanticShaderOutputKey>,
    pub name: String,
    pub location: u32,
}

/// Represents a shader asset. It does not provide compiled shader module.
/// To get shader module, you have to compile it manually.
pub trait ShaderAsset: Asset {
    fn source(&self) -> &str;
    fn reflection(&self) -> &ShaderReflection;
}

#[derive(Serialize, Deserialize)]
pub struct ShaderSource {
    pub source: String,
    pub reflection: ShaderReflection,
}

impl AssetSource for ShaderSource {
    type Asset = dyn ShaderAsset;

    fn dependencies(&self) -> Vec<Uuid> {
        vec![]
    }

    fn load(
        self,
        id: Uuid,
        _deps_provider: &dyn AssetDepsProvider,
        _gfx_bridge: &dyn GfxBridge,
    ) -> Result<Arc<Self::Asset>, AssetLoadError> {
        Ok(Arc::new(Shader {
            id,
            source: self.source,
            reflection: self.reflection,
        }))
    }
}

struct Shader {
    id: Uuid,
    source: String,
    reflection: ShaderReflection,
}

impl Asset for Shader {
    fn id(&self) -> Uuid {
        self.id
    }

    fn as_typed(self: Arc<Self>) -> TypedAsset {
        TypedAsset::Shader(self)
    }
}

impl ShaderAsset for Shader {
    fn source(&self) -> &str {
        &self.source
    }

    fn reflection(&self) -> &ShaderReflection {
        &self.reflection
    }
}
