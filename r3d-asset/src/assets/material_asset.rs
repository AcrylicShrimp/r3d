use super::{SemanticShaderBindingKey, Shader};
use crate::{
    Asset, AssetDepsProvider, AssetLoadError, AssetSource, AssetType, GfxBridge, GfxBuffer,
    GfxSampler, GfxTextureView, TypedAsset,
};
use half::f16;
use serde::{Deserialize, Serialize};
use std::{io::Write, sync::Arc};
use uuid::Uuid;
use wgpu::{BufferAddress, BufferSize, BufferUsages};
use zerocopy::AsBytes;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum MaterialBindingKey {
    Semantic(SemanticShaderBindingKey),
    Named(String),
}

#[derive(Debug, Clone)]
pub enum MaterialBindingValue {
    Buffer {
        buffer: GfxBuffer,
        offset: BufferAddress,
        size: Option<BufferSize>,
    },
    TextureView {
        view: GfxTextureView,
    },
    TextureViewArray {
        views: Vec<GfxTextureView>,
    },
    Sampler {
        sampler: GfxSampler,
    },
}

#[derive(Debug, Clone)]
pub struct MaterialBindingProp {
    pub key: MaterialBindingKey,
    pub value: MaterialBindingValue,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum MaterialInstancePropKey {
    Semantic(SemanticShaderBindingKey),
    Named(String),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum MaterialInstancePropValue {
    Uint8x2([u8; 2]),
    Uint8x4([u8; 4]),
    Sint8x2([i8; 2]),
    Sint8x4([i8; 4]),
    Unorm8x2([u8; 2]),
    Unorm8x4([u8; 4]),
    Snorm8x2([i8; 2]),
    Snorm8x4([i8; 4]),
    Uint16x2([u16; 2]),
    Uint16x4([u16; 4]),
    Sint16x2([i16; 2]),
    Sint16x4([i16; 4]),
    Unorm16x2([u16; 2]),
    Unorm16x4([u16; 4]),
    Snorm16x2([i16; 2]),
    Snorm16x4([i16; 4]),
    Float16x2([f16; 2]),
    Float16x4([f16; 4]),
    Float32([f32; 1]),
    Float32x2([f32; 2]),
    Float32x3([f32; 3]),
    Float32x4([f32; 4]),
    Uint32([u32; 1]),
    Uint32x2([u32; 2]),
    Uint32x3([u32; 3]),
    Uint32x4([u32; 4]),
    Sint32([i32; 1]),
    Sint32x2([i32; 2]),
    Sint32x3([i32; 3]),
    Sint32x4([i32; 4]),
    Float64([f64; 1]),
    Float64x2([f64; 2]),
    Float64x3([f64; 3]),
    Float64x4([f64; 4]),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct MaterialInstanceProp {
    pub key: MaterialInstancePropKey,
    pub value: MaterialInstancePropValue,
}

#[derive(Clone)]
pub struct MaterialPreset {
    pub shader: Shader,
    pub binding_props: Vec<MaterialBindingProp>,
    pub instance_props: Vec<MaterialInstanceProp>,
}

// TODO: I think we should provide shared default material instance for each material preset.
/// A material asset. Note that this asset does not provide material instance directly;
/// instead it provides shader instance and preconfigured fields to create a material instance.
/// This is because to allow mutation of material instance, e.g. changing the value of a binding.
pub trait MaterialAsset: Asset {
    fn preset(&self) -> &MaterialPreset;
}

pub type MaterialBindingKeySource = MaterialBindingKey;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum MaterialBindingValueSource {
    Uint8x2([u8; 2]),
    Uint8x4([u8; 4]),
    Sint8x2([i8; 2]),
    Sint8x4([i8; 4]),
    Unorm8x2([u8; 2]),
    Unorm8x4([u8; 4]),
    Snorm8x2([i8; 2]),
    Snorm8x4([i8; 4]),
    Uint16x2([u16; 2]),
    Uint16x4([u16; 4]),
    Sint16x2([i16; 2]),
    Sint16x4([i16; 4]),
    Unorm16x2([u16; 2]),
    Unorm16x4([u16; 4]),
    Snorm16x2([i16; 2]),
    Snorm16x4([i16; 4]),
    Float16x2([f16; 2]),
    Float16x4([f16; 4]),
    Float32([f32; 1]),
    Float32x2([f32; 2]),
    Float32x3([f32; 3]),
    Float32x4([f32; 4]),
    Uint32([u32; 1]),
    Uint32x2([u32; 2]),
    Uint32x3([u32; 3]),
    Uint32x4([u32; 4]),
    Sint32([i32; 1]),
    Sint32x2([i32; 2]),
    Sint32x3([i32; 3]),
    Sint32x4([i32; 4]),
    Float64([f64; 1]),
    Float64x2([f64; 2]),
    Float64x3([f64; 3]),
    Float64x4([f64; 4]),
    TextureView { texture: Uuid },
    TextureViewArray { textures: Vec<Uuid> },
    SamplerTexture { texture: Uuid },
    SamplerSprite { texture: Uuid, sprite: String },
    SamplerNinePatch { texture: Uuid, nine_patch: String },
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct MaterialBindingPropSource {
    pub key: MaterialBindingKeySource,
    pub value: MaterialBindingValueSource,
}

pub type MaterialInstancePropSource = MaterialInstanceProp;

#[derive(Serialize, Deserialize)]
pub struct MaterialSource {
    pub shader: Uuid,
    pub binding_props: Vec<MaterialBindingPropSource>,
    pub instance_props: Vec<MaterialInstancePropSource>,
}

impl MaterialSource {
    pub fn deserialize(src: impl AsRef<[u8]>) -> bincode::Result<Self> {
        bincode::deserialize(src.as_ref())
    }

    pub fn serialize_into(&self, dst: impl Write) -> bincode::Result<()> {
        bincode::serialize_into(dst, self)
    }
}

impl AssetSource for MaterialSource {
    type Asset = dyn MaterialAsset;

    fn dependencies(&self) -> Vec<Uuid> {
        let mut deps = Vec::with_capacity(1 + self.binding_props.len());
        deps.push(self.shader);

        for prop in &self.binding_props {
            match &prop.value {
                MaterialBindingValueSource::TextureView { texture } => {
                    deps.push(*texture);
                }
                MaterialBindingValueSource::TextureViewArray { textures } => {
                    deps.extend(textures.iter().cloned());
                }
                MaterialBindingValueSource::SamplerTexture { texture } => {
                    deps.push(*texture);
                }
                MaterialBindingValueSource::SamplerSprite { texture, .. } => {
                    deps.push(*texture);
                }
                MaterialBindingValueSource::SamplerNinePatch { texture, .. } => {
                    deps.push(*texture);
                }
                _ => {}
            }
        }

        deps
    }

    fn load(
        self,
        id: Uuid,
        deps_provider: &dyn AssetDepsProvider,
        gfx_bridge: &dyn GfxBridge,
    ) -> Result<Arc<Self::Asset>, AssetLoadError> {
        let shader = deps_provider.find_dependency(self.shader).ok_or_else(|| {
            AssetLoadError::MissingDependency {
                expected_id: self.shader,
                expected_ty: AssetType::Shader,
            }
        })?;
        let shader = shader
            .as_shader()
            .ok_or_else(|| AssetLoadError::DependencyTypeMismatch {
                expected_id: self.shader,
                expected_ty: AssetType::Shader,
                actual_ty: shader.ty(),
            })?;

        let mut binding_data = Vec::new();
        let mut binding_offsets = Vec::new();
        let mut binding_sizes = Vec::new();

        for prop in &self.binding_props {
            let bytes = match &prop.value {
                MaterialBindingValueSource::Uint8x2(value) => value.as_bytes(),
                MaterialBindingValueSource::Uint8x4(value) => value.as_bytes(),
                MaterialBindingValueSource::Sint8x2(value) => value.as_bytes(),
                MaterialBindingValueSource::Sint8x4(value) => value.as_bytes(),
                MaterialBindingValueSource::Unorm8x2(value) => value.as_bytes(),
                MaterialBindingValueSource::Unorm8x4(value) => value.as_bytes(),
                MaterialBindingValueSource::Snorm8x2(value) => value.as_bytes(),
                MaterialBindingValueSource::Snorm8x4(value) => value.as_bytes(),
                MaterialBindingValueSource::Uint16x2(value) => value.as_bytes(),
                MaterialBindingValueSource::Uint16x4(value) => value.as_bytes(),
                MaterialBindingValueSource::Sint16x2(value) => value.as_bytes(),
                MaterialBindingValueSource::Sint16x4(value) => value.as_bytes(),
                MaterialBindingValueSource::Unorm16x2(value) => value.as_bytes(),
                MaterialBindingValueSource::Unorm16x4(value) => value.as_bytes(),
                MaterialBindingValueSource::Snorm16x2(value) => value.as_bytes(),
                MaterialBindingValueSource::Snorm16x4(value) => value.as_bytes(),
                MaterialBindingValueSource::Float16x2(value) => value.as_bytes(),
                MaterialBindingValueSource::Float16x4(value) => value.as_bytes(),
                MaterialBindingValueSource::Float32(value) => value.as_bytes(),
                MaterialBindingValueSource::Float32x2(value) => value.as_bytes(),
                MaterialBindingValueSource::Float32x3(value) => value.as_bytes(),
                MaterialBindingValueSource::Float32x4(value) => value.as_bytes(),
                MaterialBindingValueSource::Uint32(value) => value.as_bytes(),
                MaterialBindingValueSource::Uint32x2(value) => value.as_bytes(),
                MaterialBindingValueSource::Uint32x3(value) => value.as_bytes(),
                MaterialBindingValueSource::Uint32x4(value) => value.as_bytes(),
                MaterialBindingValueSource::Sint32(value) => value.as_bytes(),
                MaterialBindingValueSource::Sint32x2(value) => value.as_bytes(),
                MaterialBindingValueSource::Sint32x3(value) => value.as_bytes(),
                MaterialBindingValueSource::Sint32x4(value) => value.as_bytes(),
                MaterialBindingValueSource::Float64(value) => value.as_bytes(),
                MaterialBindingValueSource::Float64x2(value) => value.as_bytes(),
                MaterialBindingValueSource::Float64x3(value) => value.as_bytes(),
                MaterialBindingValueSource::Float64x4(value) => value.as_bytes(),
                MaterialBindingValueSource::TextureView { .. } => {
                    continue;
                }
                MaterialBindingValueSource::TextureViewArray { .. } => {
                    continue;
                }
                MaterialBindingValueSource::SamplerTexture { .. } => {
                    continue;
                }
                MaterialBindingValueSource::SamplerSprite { .. } => {
                    continue;
                }
                MaterialBindingValueSource::SamplerNinePatch { .. } => {
                    continue;
                }
            };

            debug_assert_eq!(bytes.is_empty(), false);

            binding_data.extend_from_slice(bytes);
            binding_offsets.push(binding_data.len() as BufferAddress);
            binding_sizes.push(BufferSize::new(bytes.len() as u64).unwrap());
        }

        let binding_buffer = if binding_data.is_empty() {
            None
        } else {
            Some(gfx_bridge.upload_vertex_buffer(BufferUsages::UNIFORM, &binding_data))
        };
        let mut binding_index = 0;

        let binding_props = self
            .binding_props
            .iter()
            .map(|prop| {
                let value = match &prop.value {
                    MaterialBindingValueSource::TextureView { texture: uuid } => {
                        let texture = deps_provider.find_dependency(*uuid).ok_or_else(|| {
                            AssetLoadError::MissingDependency {
                                expected_id: *uuid,
                                expected_ty: AssetType::Texture,
                            }
                        })?;
                        let texture = texture.as_texture().ok_or_else(|| {
                            AssetLoadError::DependencyTypeMismatch {
                                expected_id: *uuid,
                                expected_ty: AssetType::Texture,
                                actual_ty: texture.ty(),
                            }
                        })?;
                        MaterialBindingValue::TextureView {
                            view: texture.view_handle().clone(),
                        }
                    }
                    MaterialBindingValueSource::TextureViewArray { textures } => {
                        let views = textures
                            .iter()
                            .map(|uuid| {
                                let texture =
                                    deps_provider.find_dependency(*uuid).ok_or_else(|| {
                                        AssetLoadError::MissingDependency {
                                            expected_id: *uuid,
                                            expected_ty: AssetType::Texture,
                                        }
                                    })?;
                                let texture = texture.as_texture().ok_or_else(|| {
                                    AssetLoadError::DependencyTypeMismatch {
                                        expected_id: *uuid,
                                        expected_ty: AssetType::Texture,
                                        actual_ty: texture.ty(),
                                    }
                                })?;
                                Ok(texture.view_handle().clone())
                            })
                            .collect::<Result<Vec<_>, AssetLoadError>>()?;
                        MaterialBindingValue::TextureViewArray { views }
                    }
                    MaterialBindingValueSource::SamplerTexture { texture: uuid } => {
                        let texture = deps_provider.find_dependency(*uuid).ok_or_else(|| {
                            AssetLoadError::MissingDependency {
                                expected_id: *uuid,
                                expected_ty: AssetType::Texture,
                            }
                        })?;
                        let texture = texture.as_texture().ok_or_else(|| {
                            AssetLoadError::DependencyTypeMismatch {
                                expected_id: *uuid,
                                expected_ty: AssetType::Texture,
                                actual_ty: texture.ty(),
                            }
                        })?;
                        MaterialBindingValue::Sampler {
                            sampler: texture.sampler_handle().clone(),
                        }
                    }
                    MaterialBindingValueSource::SamplerSprite {
                        texture: uuid,
                        sprite,
                    } => {
                        let texture = deps_provider.find_dependency(*uuid).ok_or_else(|| {
                            AssetLoadError::MissingDependency {
                                expected_id: *uuid,
                                expected_ty: AssetType::Texture,
                            }
                        })?;
                        let texture = texture.as_texture().ok_or_else(|| {
                            AssetLoadError::DependencyTypeMismatch {
                                expected_id: *uuid,
                                expected_ty: AssetType::Texture,
                                actual_ty: texture.ty(),
                            }
                        })?;
                        let sprites = texture.sprites();
                        let sprite_index = sprites
                            .iter()
                            .position(|item| item.name.as_str() == sprite.as_str())
                            .ok_or_else(|| AssetLoadError::InvalidSpriteName {
                                texture_id: *uuid,
                                sprite_name: sprite.clone(),
                            })?;
                        MaterialBindingValue::Sampler {
                            sampler: sprites[sprite_index].sampler_handle.clone(),
                        }
                    }
                    MaterialBindingValueSource::SamplerNinePatch {
                        texture: uuid,
                        nine_patch,
                    } => {
                        let texture = deps_provider.find_dependency(*uuid).ok_or_else(|| {
                            AssetLoadError::MissingDependency {
                                expected_id: *uuid,
                                expected_ty: AssetType::Texture,
                            }
                        })?;
                        let texture = texture.as_texture().ok_or_else(|| {
                            AssetLoadError::DependencyTypeMismatch {
                                expected_id: *uuid,
                                expected_ty: AssetType::Texture,
                                actual_ty: texture.ty(),
                            }
                        })?;
                        let nine_patches = texture.nine_patches();
                        let nine_patch_index = nine_patches
                            .iter()
                            .position(|item| item.name.as_str() == nine_patch.as_str())
                            .ok_or_else(|| AssetLoadError::InvalidNinePatchName {
                                texture_id: *uuid,
                                nine_patch_name: nine_patch.clone(),
                            })?;
                        MaterialBindingValue::Sampler {
                            sampler: nine_patches[nine_patch_index].sampler_handle.clone(),
                        }
                    }
                    _ => {
                        let value = MaterialBindingValue::Buffer {
                            buffer: binding_buffer.clone().unwrap(),
                            offset: binding_offsets[binding_index],
                            size: Some(binding_sizes[binding_index]),
                        };
                        binding_index += 1;
                        value
                    }
                };

                Ok(MaterialBindingProp {
                    key: prop.key.clone(),
                    value,
                })
            })
            .collect::<Result<Vec<_>, AssetLoadError>>()?;
        let instance_props = self.instance_props.clone();

        Ok(Arc::new(Material {
            id,
            preset: MaterialPreset {
                shader: shader.clone(),
                binding_props,
                instance_props,
            },
        }))
    }
}

struct Material {
    id: Uuid,
    preset: MaterialPreset,
}

impl Asset for Material {
    fn id(&self) -> Uuid {
        self.id
    }

    fn as_typed(self: Arc<Self>) -> TypedAsset {
        TypedAsset::Material(self)
    }
}

impl MaterialAsset for Material {
    fn preset(&self) -> &MaterialPreset {
        &self.preset
    }
}
