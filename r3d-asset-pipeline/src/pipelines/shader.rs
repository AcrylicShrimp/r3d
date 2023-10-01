use crate::{AssetPipeline, PipelineGfxBridge};
use anyhow::Context;
use asset::assets::{
    ShaderGlobalItem, ShaderGlobalItemKind, ShaderInput, ShaderInputField, ShaderOutputItem,
    ShaderReflection, ShaderSource,
};
use naga::{
    AddressSpace, ArraySize, Binding, Function, FunctionArgument, GlobalVariable, ImageClass,
    ImageDimension, Module, ResourceBinding, ScalarKind, ShaderStage, StructMember, Type,
    TypeInner, VectorSize,
};
use serde::{Deserialize, Serialize};
use std::num::{NonZeroU32, NonZeroU64};
use thiserror::Error;
use wgpu::{
    BufferAddress, SamplerBindingType, TextureSampleType, TextureViewDimension, VertexAttribute,
    VertexFormat, VertexStepMode,
};

#[derive(Error, Debug)]
pub enum ShaderReflectionError {
    #[error("failed to parse shader source: {0}")]
    ParseError(#[from] naga::front::wgsl::ParseError),
    #[error("no vertex entry point found")]
    NoVertexEntryPoint,
    #[error("no fragment entry point found")]
    NoFragmentEntryPoint,
}

#[derive(Serialize, Deserialize)]
pub struct ShaderMetadata;

impl AssetPipeline for ShaderSource {
    type Metadata = ShaderMetadata;

    fn process(
        file_content: Vec<u8>,
        _metadata: &crate::Metadata<Self::Metadata>,
        gfx_bridge: &dyn PipelineGfxBridge,
    ) -> anyhow::Result<Self> {
        let source = std::str::from_utf8(&file_content)
            .with_context(|| "failed to decode shader source into utf8 string")?;
        let module = naga::front::wgsl::parse_str(&source)
            .with_context(|| "failed to parse wgsl shader source")?;

        let globals = reflect_globals(gfx_bridge, &module);

        let mut vertex_entry_point = None;
        let mut fragment_entry_point = None;
        let mut vertex_input = None;
        let mut instance_input = None;
        let mut outputs = None;

        for entry_point in &module.entry_points {
            match entry_point.stage {
                ShaderStage::Vertex => {
                    vertex_entry_point = Some(entry_point.name.clone());

                    for input in
                        reflect_vertex_entry_point(gfx_bridge, &module, &entry_point.function)
                    {
                        match input.step_mode {
                            VertexStepMode::Vertex => {
                                vertex_input = Some(input);
                            }
                            VertexStepMode::Instance => {
                                instance_input = Some(input);
                            }
                        }
                    }
                }
                ShaderStage::Fragment => {
                    fragment_entry_point = Some(entry_point.name.clone());

                    if let Some(fragment_outputs) =
                        reflect_fragment_entry_point(gfx_bridge, &module, &entry_point.function)
                    {
                        outputs = Some(fragment_outputs);
                    }
                }
                ShaderStage::Compute => continue,
            }
        }

        Ok(ShaderSource {
            source: source.to_owned(),
            reflection: ShaderReflection {
                vertex_entry_point: vertex_entry_point
                    .ok_or(ShaderReflectionError::NoVertexEntryPoint)?,
                fragment_entry_point: fragment_entry_point
                    .ok_or(ShaderReflectionError::NoFragmentEntryPoint)?,
                globals,
                vertex_input: vertex_input.unwrap_or_else(|| ShaderInput {
                    step_mode: VertexStepMode::Vertex,
                    stride: 0,
                    fields: vec![],
                }),
                instance_input: instance_input.unwrap_or_else(|| ShaderInput {
                    step_mode: VertexStepMode::Instance,
                    stride: 0,
                    fields: vec![],
                }),
                outputs: outputs.unwrap_or_else(|| vec![]),
            },
        })
    }
}

fn reflect_globals(gfx_bridge: &dyn PipelineGfxBridge, module: &Module) -> Vec<ShaderGlobalItem> {
    module
        .global_variables
        .iter()
        .filter_map(|(_, item)| reflect_global_item(gfx_bridge, module, item))
        .collect()
}

fn reflect_global_item(
    gfx_bridge: &dyn PipelineGfxBridge,
    module: &Module,
    item: &GlobalVariable,
) -> Option<ShaderGlobalItem> {
    let name = item.name.as_ref()?;
    let ResourceBinding { group, binding } = item.binding.clone()?;
    let kind = match item.space {
        AddressSpace::Uniform | AddressSpace::Handle => {
            shader_ty_to_global_item_kind(module, &module.types[item.ty])?
        }
        _ => {
            return None;
        }
    };

    Some(ShaderGlobalItem {
        sematic_key: gfx_bridge.get_semantic_binding_key(module, name),
        name: name.clone(),
        group,
        binding,
        kind,
    })
}

fn reflect_vertex_entry_point(
    gfx_bridge: &dyn PipelineGfxBridge,
    module: &Module,
    function: &Function,
) -> Vec<ShaderInput> {
    function
        .arguments
        .iter()
        .filter_map(|argument| reflect_vertex_entry_point_argument(gfx_bridge, module, argument))
        .collect()
}

fn reflect_vertex_entry_point_argument(
    gfx_bridge: &dyn PipelineGfxBridge,
    module: &Module,
    argument: &FunctionArgument,
) -> Option<ShaderInput> {
    let ty = &module.types[argument.ty];
    let ty_name = ty.name.as_ref()?;
    let step_mode = match ty_name.as_str() {
        "InstanceIn" | "InstanceInput" => VertexStepMode::Instance,
        "VertexIn" | "VertexInput" => VertexStepMode::Vertex,
        _ => return None,
    };
    let (members, span) = if let TypeInner::Struct { members, span } = &ty.inner {
        (members, *span)
    } else {
        return None;
    };
    Some(reflect_shader_input(
        gfx_bridge, module, step_mode, span, members,
    ))
}

fn reflect_shader_input(
    gfx_bridge: &dyn PipelineGfxBridge,
    module: &Module,
    step_mode: VertexStepMode,
    stride: u32,
    members: &[StructMember],
) -> ShaderInput {
    let fields = members
        .iter()
        .filter_map(|member| reflect_shader_input_field(gfx_bridge, module, step_mode, member))
        .collect();
    ShaderInput {
        step_mode,
        stride: stride as BufferAddress,
        fields,
    }
}

fn reflect_shader_input_field(
    gfx_bridge: &dyn PipelineGfxBridge,
    module: &Module,
    step_mode: VertexStepMode,
    member: &StructMember,
) -> Option<ShaderInputField> {
    let name = member.name.as_ref()?;
    let location = match member.binding.as_ref()? {
        Binding::BuiltIn(_) => return None,
        Binding::Location { location, .. } => *location,
    };
    let format = shader_ty_to_vertex_format(&module.types[member.ty])?;
    Some(ShaderInputField {
        semantic_key: gfx_bridge.get_semantic_input_key(step_mode, format, name),
        name: name.clone(),
        attribute: VertexAttribute {
            format,
            offset: member.offset as BufferAddress,
            shader_location: location,
        },
    })
}

fn reflect_fragment_entry_point(
    gfx_bridge: &dyn PipelineGfxBridge,
    module: &Module,
    function: &Function,
) -> Option<Vec<ShaderOutputItem>> {
    let result = function.result.as_ref()?;
    let ty = &module.types[result.ty];
    let ty_name = ty.name.as_ref()?;

    match ty_name.as_str() {
        "FragmentOut" | "FragmentOutput" => {}
        _ => return None,
    };

    let members = if let TypeInner::Struct { members, .. } = &ty.inner {
        members
    } else {
        return None;
    };

    Some(
        members
            .iter()
            .filter_map(|member| reflect_shader_output_item(gfx_bridge, member))
            .collect(),
    )
}

fn reflect_shader_output_item(
    gfx_bridge: &dyn PipelineGfxBridge,
    member: &StructMember,
) -> Option<ShaderOutputItem> {
    let name = member.name.as_ref()?;
    let location = match member.binding.as_ref()? {
        Binding::BuiltIn(_) => return None,
        Binding::Location { location, .. } => *location,
    };
    Some(ShaderOutputItem {
        semantic_key: gfx_bridge.get_semantic_output_key(location, name),
        name: name.clone(),
        location,
    })
}

fn shader_ty_to_global_item_kind(module: &Module, ty: &Type) -> Option<ShaderGlobalItemKind> {
    fn aligned_size(size: u64, alignment: u64) -> u64 {
        (size + alignment - 1) / alignment * alignment
    }

    fn parse_array_size(size: ArraySize) -> Option<u32> {
        match size {
            ArraySize::Constant(constant) => Some(constant.get()),
            _ => None,
        }
    }

    match &ty.inner {
        TypeInner::Scalar { width, .. } => Some(ShaderGlobalItemKind::Buffer {
            size: unsafe { NonZeroU64::new_unchecked(aligned_size(*width as u64, 16)) },
        }),
        TypeInner::Vector { size, width, .. } => Some(ShaderGlobalItemKind::Buffer {
            size: unsafe {
                NonZeroU64::new_unchecked(aligned_size(*size as u64 * *width as u64, 16))
            },
        }),
        TypeInner::Matrix {
            columns,
            rows,
            width,
        } => Some(ShaderGlobalItemKind::Buffer {
            size: unsafe {
                NonZeroU64::new_unchecked(
                    aligned_size(*columns as u64 * *width as u64, 16) * *rows as u64,
                )
            },
        }),
        TypeInner::Array { size, stride, .. } => {
            let size = parse_array_size(*size)? as u64;
            let stride = if *stride < 16 {
                aligned_size(*stride as u64, 16)
            } else {
                *stride as u64
            };

            Some(ShaderGlobalItemKind::Buffer {
                size: unsafe { NonZeroU64::new_unchecked(stride * size) },
            })
        }
        TypeInner::Struct { span, .. } => Some(ShaderGlobalItemKind::Buffer {
            size: unsafe { NonZeroU64::new_unchecked(*span as u64) },
        }),
        TypeInner::Image { dim, class, .. } => {
            let (sample_type, multisampled) = match *class {
                ImageClass::Sampled { kind, multi } => {
                    let sample_type = match kind {
                        ScalarKind::Sint => TextureSampleType::Sint,
                        ScalarKind::Uint => TextureSampleType::Uint,
                        ScalarKind::Float => TextureSampleType::Float { filterable: true },
                        ScalarKind::Bool => {
                            return None;
                        }
                    };
                    (sample_type, multi)
                }
                ImageClass::Depth { multi } => (TextureSampleType::Depth, multi),
                ImageClass::Storage { .. } => {
                    return None;
                }
            };

            Some(ShaderGlobalItemKind::Texture {
                sample_type,
                view_dimension: match *dim {
                    ImageDimension::D1 => TextureViewDimension::D1,
                    ImageDimension::D2 => TextureViewDimension::D2,
                    ImageDimension::D3 => TextureViewDimension::D3,
                    ImageDimension::Cube => TextureViewDimension::Cube,
                },
                multisampled,
                array_size: None,
            })
        }
        TypeInner::Sampler { comparison } => Some(ShaderGlobalItemKind::Sampler {
            binding_type: if *comparison {
                SamplerBindingType::Comparison
            } else {
                SamplerBindingType::Filtering
            },
        }),
        TypeInner::BindingArray { base, size } => {
            match shader_ty_to_global_item_kind(module, &module.types[*base])? {
                ShaderGlobalItemKind::Texture {
                    sample_type,
                    view_dimension: dimension,
                    multisampled,
                    array_size,
                } => {
                    if array_size.is_some() {
                        return None;
                    }

                    Some(ShaderGlobalItemKind::Texture {
                        sample_type,
                        view_dimension: dimension,
                        multisampled,
                        array_size: Some(unsafe {
                            NonZeroU32::new_unchecked(parse_array_size(*size)?)
                        }),
                    })
                }
                _ => None,
            }
        }
        _ => None,
    }
}

fn shader_ty_to_vertex_format(ty: &Type) -> Option<VertexFormat> {
    match &ty.inner {
        TypeInner::Scalar { kind, width } => match (*kind, *width) {
            (ScalarKind::Sint, 4) => Some(VertexFormat::Sint32),
            (ScalarKind::Uint, 4) => Some(VertexFormat::Uint32),
            (ScalarKind::Float, 4) => Some(VertexFormat::Float32),
            (ScalarKind::Float, 8) => Some(VertexFormat::Float64),
            _ => None,
        },
        TypeInner::Vector { size, kind, width } => match (*size, *kind, *width) {
            (VectorSize::Bi, ScalarKind::Sint, 4) => Some(VertexFormat::Sint32x2),
            (VectorSize::Bi, ScalarKind::Uint, 4) => Some(VertexFormat::Uint32x2),
            (VectorSize::Bi, ScalarKind::Float, 4) => Some(VertexFormat::Float32x2),
            (VectorSize::Bi, ScalarKind::Float, 8) => Some(VertexFormat::Float64x2),
            (VectorSize::Tri, ScalarKind::Sint, 4) => Some(VertexFormat::Sint32x3),
            (VectorSize::Tri, ScalarKind::Uint, 4) => Some(VertexFormat::Uint32x3),
            (VectorSize::Tri, ScalarKind::Float, 4) => Some(VertexFormat::Float32x3),
            (VectorSize::Tri, ScalarKind::Float, 8) => Some(VertexFormat::Float64x3),
            (VectorSize::Quad, ScalarKind::Sint, 4) => Some(VertexFormat::Sint32x4),
            (VectorSize::Quad, ScalarKind::Uint, 4) => Some(VertexFormat::Uint32x4),
            (VectorSize::Quad, ScalarKind::Float, 4) => Some(VertexFormat::Float32x4),
            (VectorSize::Quad, ScalarKind::Float, 8) => Some(VertexFormat::Float64x4),
            _ => None,
        },
        _ => None,
    }
}
