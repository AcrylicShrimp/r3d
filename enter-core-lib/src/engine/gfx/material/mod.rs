use codegen::Handle;
use half::f16;
use std::{collections::HashMap, num::NonZeroU32, sync::Arc};
use wgpu::{
    BindGroup, BindGroupDescriptor, BindGroupEntry, BindingResource, BindingType, Buffer,
    BufferAddress, BufferBinding, BufferSize, Device, Sampler, TextureView, VertexFormat,
    VertexStepMode,
};
use zerocopy::AsBytes;

mod bind_group_layout_cache;
mod pipeline_cache;
mod pipeline_layout_cache;
mod shader;
mod shader_reflection;

pub use bind_group_layout_cache::*;
pub use pipeline_cache::*;
pub use pipeline_layout_cache::*;
pub use shader::*;
pub use shader_reflection::*;

#[derive(Handle)]
pub struct Material {
    pub shader: ShaderHandle,
    pub pipeline_layout: CachedPipelineLayout,
    pub semantic_inputs: HashMap<SemanticShaderInputKey, SemanticInputData>,
    pub bind_properties: HashMap<BindingPropKey, BindGroupIndex>,
    pub bind_group_holders: Vec<BindGroupHolder>,
    pub per_instance_properties: HashMap<String, PerInstanceProperty>,
}

impl Material {
    pub fn new(shader: ShaderHandle, pipeline_layout_cache: &mut PipelineLayoutCache) -> Self {
        let semantic_inputs = HashMap::from_iter(
            shader
                .reflected_shader
                .per_instance_input
                .elements
                .iter()
                .enumerate()
                .filter_map(|(index, input)| {
                    input.semantic_input.map(|key| {
                        (
                            key,
                            SemanticInputData {
                                step_mode: VertexStepMode::Instance,
                                offset: input.attribute.offset,
                                shader_location: input.attribute.shader_location,
                                index,
                            },
                        )
                    })
                })
                .chain(
                    shader
                        .reflected_shader
                        .per_vertex_input
                        .elements
                        .iter()
                        .enumerate()
                        .filter_map(|(index, input)| {
                            input.semantic_input.map(|key| {
                                (
                                    key,
                                    SemanticInputData {
                                        step_mode: VertexStepMode::Vertex,
                                        offset: input.attribute.offset,
                                        shader_location: input.attribute.shader_location,
                                        index,
                                    },
                                )
                            })
                        }),
                ),
        );
        let bind_properties = HashMap::from_iter(
            shader
                .bind_group_layouts
                .iter()
                .enumerate()
                .flat_map(|(group_index, (group, layout))| {
                    layout
                        .key()
                        .entries
                        .iter()
                        .enumerate()
                        .map(move |(entry_index, entry)| {
                            ((*group, entry.binding), group_index, entry_index)
                        })
                })
                .filter_map(|((group, binding), group_index, entry_index)| {
                    shader
                        .reflected_shader
                        .bindings
                        .iter()
                        .find(|element| element.group == group && element.binding == binding)
                        .map(|element| {
                            (
                                match element.semantic_binding {
                                    Some(semantic_key) => BindingPropKey::SemanticKey(semantic_key),
                                    None => BindingPropKey::StringKey(element.name.clone()),
                                },
                                BindGroupIndex {
                                    group_index,
                                    entry_index,
                                },
                            )
                        })
                }),
        );
        let bind_group_holders =
            Vec::from_iter(shader.bind_group_layouts.iter().map(|(group, layout)| {
                BindGroupHolder {
                    is_dirty: false,
                    group: *group,
                    bind_group: None,
                    entries: Vec::from_iter(layout.key().entries.iter().map(|entry| {
                        BindGroupEntryHolder {
                            binding: entry.binding,
                            binding_ty: entry.ty,
                            count: entry.count,
                            resource: None,
                        }
                    })),
                }
            }));
        let per_instance_properties = HashMap::from_iter(
            shader
                .reflected_shader
                .per_instance_input
                .elements
                .iter()
                .map(|input| {
                    (
                        input.name.clone(),
                        PerInstanceProperty {
                            format: input.attribute.format,
                            offset: input.attribute.offset,
                            value: None,
                        },
                    )
                }),
        );

        let mut bind_group_layouts = Vec::from_iter(
            shader
                .bind_group_layouts
                .iter()
                .map(|(group, layout)| (*group, layout.clone())),
        );
        bind_group_layouts.sort_unstable_by_key(|(group, _)| *group);

        let bind_group_layouts =
            Vec::from_iter(bind_group_layouts.into_iter().map(|(_, layout)| layout));
        let pipeline_layout = pipeline_layout_cache.create_layout(bind_group_layouts);

        Self {
            shader,
            pipeline_layout,
            semantic_inputs,
            bind_properties,
            bind_group_holders,
            per_instance_properties,
        }
    }

    pub fn set_bind_property(
        &mut self,
        key: &BindingPropKey,
        resource: impl Into<BindGroupEntryResource>,
    ) -> bool {
        let index = if let Some(index) = self.bind_properties.get(key) {
            *index
        } else {
            return false;
        };
        let bind_group_holder = &mut self.bind_group_holders[index.group_index];
        let entry_holder = &mut bind_group_holder.entries[index.entry_index];
        let resource = resource.into();

        if !resource.is_match(entry_holder.binding_ty, entry_holder.count) {
            return false;
        }

        entry_holder.resource = Some(resource);
        bind_group_holder.is_dirty = true;
        true
    }

    pub fn set_per_instance_property(
        &mut self,
        name: impl AsRef<str>,
        value: impl Into<PerInstancePropertyValue>,
    ) -> bool {
        let property = if let Some(property) = self.per_instance_properties.get_mut(name.as_ref()) {
            property
        } else {
            return false;
        };
        let value = value.into();

        if value.to_vertex_format() != property.format {
            return false;
        }

        property.value = Some(value);
        true
    }

    pub fn update_bind_group(&mut self, device: &Device) {
        for bind_group_holder in &mut self.bind_group_holders {
            if !bind_group_holder.is_dirty {
                continue;
            }

            bind_group_holder.is_dirty = false;

            if bind_group_holder
                .entries
                .iter()
                .any(|entry| entry.resource.is_none())
            {
                bind_group_holder.bind_group = None;
                continue;
            }

            let layout = if let Some(layout) =
                self.shader.bind_group_layouts.get(&bind_group_holder.group)
            {
                layout
            } else {
                continue;
            };

            let entry_binding_resource_builders =
                Vec::from_iter(bind_group_holder.entries.iter().map(|entry| {
                    entry
                        .resource
                        .as_ref()
                        .unwrap()
                        .as_binding_resource_builder()
                }));
            let entries = Vec::from_iter(bind_group_holder.entries.iter().enumerate().map(
                |(index, entry)| BindGroupEntry {
                    binding: entry.binding,
                    resource: entry_binding_resource_builders[index].as_binding_resource(),
                },
            ));

            bind_group_holder.bind_group = Some(device.create_bind_group(&BindGroupDescriptor {
                label: None,
                layout: layout.as_ref(),
                entries: &entries,
            }));
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum BindingPropKey {
    SemanticKey(SemanticShaderBindingKey),
    StringKey(String),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SemanticInputData {
    pub step_mode: VertexStepMode,
    pub offset: BufferAddress,
    pub shader_location: u32,
    pub index: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct BindGroupIndex {
    pub group_index: usize,
    pub entry_index: usize,
}

#[derive(Debug)]
pub struct BindGroupHolder {
    pub is_dirty: bool,
    pub group: u32,
    pub bind_group: Option<BindGroup>,
    pub entries: Vec<BindGroupEntryHolder>,
}

#[derive(Debug, Clone)]
pub struct BindGroupEntryHolder {
    pub binding: u32,
    pub binding_ty: BindingType,
    pub count: Option<NonZeroU32>,
    pub resource: Option<BindGroupEntryResource>,
}

#[derive(Debug, Clone)]
pub enum BindGroupEntryResource {
    Buffer {
        buffer: Arc<Buffer>,
        offset: BufferAddress,
        size: Option<BufferSize>,
    },
    Sampler {
        sampler: Arc<Sampler>,
    },
    TextureView {
        texture_view: Arc<TextureView>,
    },
    TextureViewArray {
        texture_views: Vec<Arc<TextureView>>,
    },
}

impl BindGroupEntryResource {
    pub fn is_match(&self, binding_ty: BindingType, count: Option<NonZeroU32>) -> bool {
        match (self, binding_ty) {
            (BindGroupEntryResource::Buffer { .. }, BindingType::Buffer { .. })
                if count.is_none() =>
            {
                true
            }
            (BindGroupEntryResource::Sampler { .. }, BindingType::Sampler(_))
                if count.is_none() =>
            {
                true
            }
            (BindGroupEntryResource::TextureView { .. }, BindingType::Texture { .. })
                if count.is_none() =>
            {
                true
            }
            (
                BindGroupEntryResource::TextureViewArray { texture_views },
                BindingType::Texture { .. },
            ) if !texture_views.is_empty()
                && count.map(|count| count.get() as usize) == Some(texture_views.len()) =>
            {
                true
            }
            _ => false,
        }
    }

    pub fn as_binding_resource_builder(&self) -> BindGroupEntryResourceBindingResourceBuilder {
        match self {
            BindGroupEntryResource::Buffer {
                buffer,
                offset,
                size,
            } => BindGroupEntryResourceBindingResourceBuilder::Resource {
                resource: BindingResource::Buffer(BufferBinding {
                    buffer: &buffer,
                    offset: *offset,
                    size: *size,
                }),
            },
            BindGroupEntryResource::Sampler { sampler } => {
                BindGroupEntryResourceBindingResourceBuilder::Resource {
                    resource: BindingResource::Sampler(&sampler),
                }
            }
            BindGroupEntryResource::TextureView { texture_view } => {
                BindGroupEntryResourceBindingResourceBuilder::Resource {
                    resource: BindingResource::TextureView(&texture_view),
                }
            }
            BindGroupEntryResource::TextureViewArray { texture_views } => {
                BindGroupEntryResourceBindingResourceBuilder::TextureViewArray {
                    texture_views: texture_views
                        .iter()
                        .map(|texture_view| texture_view.as_ref())
                        .collect(),
                }
            }
        }
    }
}

pub enum BindGroupEntryResourceBindingResourceBuilder<'a> {
    Resource { resource: BindingResource<'a> },
    TextureViewArray { texture_views: Vec<&'a TextureView> },
}

impl<'a> BindGroupEntryResourceBindingResourceBuilder<'a> {
    pub fn as_binding_resource(&self) -> BindingResource {
        match self {
            BindGroupEntryResourceBindingResourceBuilder::Resource { resource } => resource.clone(),
            BindGroupEntryResourceBindingResourceBuilder::TextureViewArray { texture_views } => {
                BindingResource::TextureViewArray(&texture_views)
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct PerInstanceProperty {
    pub format: VertexFormat,
    pub offset: BufferAddress,
    pub value: Option<PerInstancePropertyValue>,
}

#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub enum PerInstancePropertyValue {
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

impl PerInstancePropertyValue {
    pub fn as_bytes(&self) -> &[u8] {
        match self {
            PerInstancePropertyValue::Uint8x2(inner) => inner.as_bytes(),
            PerInstancePropertyValue::Uint8x4(inner) => inner.as_bytes(),
            PerInstancePropertyValue::Sint8x2(inner) => inner.as_bytes(),
            PerInstancePropertyValue::Sint8x4(inner) => inner.as_bytes(),
            PerInstancePropertyValue::Unorm8x2(inner) => inner.as_bytes(),
            PerInstancePropertyValue::Unorm8x4(inner) => inner.as_bytes(),
            PerInstancePropertyValue::Snorm8x2(inner) => inner.as_bytes(),
            PerInstancePropertyValue::Snorm8x4(inner) => inner.as_bytes(),
            PerInstancePropertyValue::Uint16x2(inner) => inner.as_bytes(),
            PerInstancePropertyValue::Uint16x4(inner) => inner.as_bytes(),
            PerInstancePropertyValue::Sint16x2(inner) => inner.as_bytes(),
            PerInstancePropertyValue::Sint16x4(inner) => inner.as_bytes(),
            PerInstancePropertyValue::Unorm16x2(inner) => inner.as_bytes(),
            PerInstancePropertyValue::Unorm16x4(inner) => inner.as_bytes(),
            PerInstancePropertyValue::Snorm16x2(inner) => inner.as_bytes(),
            PerInstancePropertyValue::Snorm16x4(inner) => inner.as_bytes(),
            PerInstancePropertyValue::Float16x2(inner) => inner.as_bytes(),
            PerInstancePropertyValue::Float16x4(inner) => inner.as_bytes(),
            PerInstancePropertyValue::Float32(inner) => inner.as_bytes(),
            PerInstancePropertyValue::Float32x2(inner) => inner.as_bytes(),
            PerInstancePropertyValue::Float32x3(inner) => inner.as_bytes(),
            PerInstancePropertyValue::Float32x4(inner) => inner.as_bytes(),
            PerInstancePropertyValue::Uint32(inner) => inner.as_bytes(),
            PerInstancePropertyValue::Uint32x2(inner) => inner.as_bytes(),
            PerInstancePropertyValue::Uint32x3(inner) => inner.as_bytes(),
            PerInstancePropertyValue::Uint32x4(inner) => inner.as_bytes(),
            PerInstancePropertyValue::Sint32(inner) => inner.as_bytes(),
            PerInstancePropertyValue::Sint32x2(inner) => inner.as_bytes(),
            PerInstancePropertyValue::Sint32x3(inner) => inner.as_bytes(),
            PerInstancePropertyValue::Sint32x4(inner) => inner.as_bytes(),
            PerInstancePropertyValue::Float64(inner) => inner.as_bytes(),
            PerInstancePropertyValue::Float64x2(inner) => inner.as_bytes(),
            PerInstancePropertyValue::Float64x3(inner) => inner.as_bytes(),
            PerInstancePropertyValue::Float64x4(inner) => inner.as_bytes(),
        }
    }

    pub fn to_vertex_format(&self) -> VertexFormat {
        match self {
            Self::Uint8x2(_) => VertexFormat::Uint8x2,
            Self::Uint8x4(_) => VertexFormat::Uint8x4,
            Self::Sint8x2(_) => VertexFormat::Sint8x2,
            Self::Sint8x4(_) => VertexFormat::Sint8x4,
            Self::Unorm8x2(_) => VertexFormat::Unorm8x2,
            Self::Unorm8x4(_) => VertexFormat::Unorm8x4,
            Self::Snorm8x2(_) => VertexFormat::Snorm8x2,
            Self::Snorm8x4(_) => VertexFormat::Snorm8x4,
            Self::Uint16x2(_) => VertexFormat::Uint16x2,
            Self::Uint16x4(_) => VertexFormat::Uint16x4,
            Self::Sint16x2(_) => VertexFormat::Sint16x2,
            Self::Sint16x4(_) => VertexFormat::Sint16x4,
            Self::Unorm16x2(_) => VertexFormat::Unorm16x2,
            Self::Unorm16x4(_) => VertexFormat::Unorm16x4,
            Self::Snorm16x2(_) => VertexFormat::Snorm16x2,
            Self::Snorm16x4(_) => VertexFormat::Snorm16x4,
            Self::Float16x2(_) => VertexFormat::Float16x2,
            Self::Float16x4(_) => VertexFormat::Float16x4,
            Self::Float32(_) => VertexFormat::Float32,
            Self::Float32x2(_) => VertexFormat::Float32x2,
            Self::Float32x3(_) => VertexFormat::Float32x3,
            Self::Float32x4(_) => VertexFormat::Float32x4,
            Self::Uint32(_) => VertexFormat::Uint32,
            Self::Uint32x2(_) => VertexFormat::Uint32x2,
            Self::Uint32x3(_) => VertexFormat::Uint32x3,
            Self::Uint32x4(_) => VertexFormat::Uint32x4,
            Self::Sint32(_) => VertexFormat::Sint32,
            Self::Sint32x2(_) => VertexFormat::Sint32x2,
            Self::Sint32x3(_) => VertexFormat::Sint32x3,
            Self::Sint32x4(_) => VertexFormat::Sint32x4,
            Self::Float64(_) => VertexFormat::Float64,
            Self::Float64x2(_) => VertexFormat::Float64x2,
            Self::Float64x3(_) => VertexFormat::Float64x3,
            Self::Float64x4(_) => VertexFormat::Float64x4,
        }
    }
}
