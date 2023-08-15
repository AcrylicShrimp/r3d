use super::{
    shader::{SemanticShaderInputKey, ShaderManager},
    SemanticShaderBindingKey, SemanticShaderOutputKey,
};
use naga::{
    front::wgsl::{parse_str, ParseError},
    AddressSpace, ArraySize, Binding, ConstantInner, Function, ImageClass, ImageDimension, Module,
    ScalarKind, ScalarValue, ShaderStage, StructMember, Type, TypeInner, VectorSize,
};
use std::num::{NonZeroU32, NonZeroU64};
use thiserror::Error;
use wgpu::{
    BindGroupLayoutEntry, BindingType, BufferAddress, BufferBindingType, SamplerBindingType,
    ShaderStages, TextureSampleType, TextureViewDimension, VertexAttribute, VertexBufferLayout,
    VertexFormat, VertexStepMode,
};

#[derive(Error, Debug)]
pub enum ShaderInspectionError {
    #[error("failed to parse shader source: {0}")]
    ParseError(#[from] ParseError),
    #[error("no vertex entry point found")]
    NoVertexEntryPoint,
    #[error("no fragment entry point found")]
    NoFragmentEntryPoint,
}

#[derive(Debug, Clone)]
pub struct ReflectedShader {
    pub vertex_entry_point_name: String,
    pub fragment_entry_point_name: String,
    pub bindings: Vec<ReflectedShaderBindingElement>,
    pub per_instance_input: ReflectedShaderInput,
    pub per_vertex_input: ReflectedShaderInput,
    pub outputs: Vec<ReflectedShaderOutputElement>,
}

#[derive(Debug, Clone)]
pub struct ReflectedShaderBindingElement {
    pub semantic_binding: Option<SemanticShaderBindingKey>,
    pub name: String,
    pub group: u32,
    pub binding: u32,
    pub kind: ReflectedShaderBindingElementKind,
}

impl From<&ReflectedShaderBindingElement> for BindGroupLayoutEntry {
    fn from(value: &ReflectedShaderBindingElement) -> Self {
        Self {
            binding: value.binding,
            visibility: ShaderStages::VERTEX_FRAGMENT,
            ty: match &value.kind {
                ReflectedShaderBindingElementKind::Buffer { size } => BindingType::Buffer {
                    ty: BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: Some(*size),
                },
                ReflectedShaderBindingElementKind::Texture {
                    sample_type,
                    view_dimension,
                    multisampled,
                    ..
                } => BindingType::Texture {
                    sample_type: *sample_type,
                    view_dimension: *view_dimension,
                    multisampled: *multisampled,
                },
                ReflectedShaderBindingElementKind::Sampler { binding_type } => {
                    BindingType::Sampler(*binding_type)
                }
            },
            count: if let ReflectedShaderBindingElementKind::Texture { array_size, .. } =
                &value.kind
            {
                *array_size
            } else {
                None
            },
        }
    }
}

#[derive(Debug, Clone)]
pub enum ReflectedShaderBindingElementKind {
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

#[derive(Debug, Clone)]
pub struct ReflectedShaderInput {
    pub step_mode: VertexStepMode,
    pub stride: BufferAddress,
    pub elements: Vec<ReflectedShaderInputElement>,
}

impl ReflectedShaderInput {
    pub fn empty(step_mode: VertexStepMode) -> Self {
        Self {
            step_mode,
            stride: 0,
            elements: Vec::new(),
        }
    }

    pub fn vertex_buffer_layout_builder(&self) -> ReflectedShaderInputVertexBufferLayoutBuilder {
        ReflectedShaderInputVertexBufferLayoutBuilder {
            step_mode: self.step_mode,
            stride: self.stride,
            attributes: self.elements.iter().map(|e| e.attribute).collect(),
        }
    }
}

pub struct ReflectedShaderInputVertexBufferLayoutBuilder {
    step_mode: VertexStepMode,
    stride: BufferAddress,
    attributes: Vec<VertexAttribute>,
}

impl<'a> From<&'a ReflectedShaderInputVertexBufferLayoutBuilder> for VertexBufferLayout<'a> {
    fn from(value: &'a ReflectedShaderInputVertexBufferLayoutBuilder) -> Self {
        Self {
            array_stride: value.stride,
            step_mode: value.step_mode,
            attributes: &value.attributes,
        }
    }
}

#[derive(Debug, Clone)]
pub struct ReflectedShaderInputElement {
    pub semantic_input: Option<SemanticShaderInputKey>,
    pub name: String,
    pub attribute: VertexAttribute,
}

#[derive(Debug, Clone)]
pub struct ReflectedShaderOutputElement {
    pub semantic_output: Option<SemanticShaderOutputKey>,
    pub name: String,
    pub location: u32,
}

pub fn inspect_shader(
    shader_mgr: &ShaderManager,
    source: impl AsRef<str>,
) -> Result<ReflectedShader, ShaderInspectionError> {
    let module = parse_str(source.as_ref())?;
    let bindings = reflect_globals(shader_mgr, &module);

    let mut vertex_entry_point_name = None;
    let mut fragment_entry_point_name = None;
    let mut per_instance_input = None;
    let mut per_vertex_input = None;
    let mut outputs = None;

    for entry_point in &module.entry_points {
        match entry_point.stage {
            ShaderStage::Vertex => {
                vertex_entry_point_name = Some(entry_point.name.clone());

                for vertex_input in
                    reflect_vertex_entry_point(shader_mgr, &module, &entry_point.function)
                {
                    match vertex_input.step_mode {
                        VertexStepMode::Vertex => {
                            per_vertex_input = Some(vertex_input);
                        }
                        VertexStepMode::Instance => {
                            per_instance_input = Some(vertex_input);
                        }
                    }
                }
            }
            ShaderStage::Fragment => {
                fragment_entry_point_name = Some(entry_point.name.clone());

                if let Some(fragment_outputs) =
                    reflect_fragment_entry_point(shader_mgr, &module, &entry_point.function)
                {
                    outputs = Some(fragment_outputs);
                }
            }
            ShaderStage::Compute => continue,
        }
    }

    Ok(ReflectedShader {
        vertex_entry_point_name: vertex_entry_point_name
            .ok_or(ShaderInspectionError::NoVertexEntryPoint)?,
        fragment_entry_point_name: fragment_entry_point_name
            .ok_or(ShaderInspectionError::NoFragmentEntryPoint)?,
        bindings,
        per_instance_input: per_instance_input
            .unwrap_or_else(|| ReflectedShaderInput::empty(VertexStepMode::Instance)),
        per_vertex_input: per_vertex_input
            .unwrap_or_else(|| ReflectedShaderInput::empty(VertexStepMode::Vertex)),
        outputs: outputs.unwrap_or_else(|| Vec::new()),
    })
}

fn reflect_globals(
    shader_mgr: &ShaderManager,
    module: &Module,
) -> Vec<ReflectedShaderBindingElement> {
    let mut bindings = Vec::new();

    for (_, global) in module.global_variables.iter() {
        let name = if let Some(name) = &global.name {
            name
        } else {
            continue;
        };
        let (group, binding) = if let Some(binding) = &global.binding {
            (binding.group, binding)
        } else {
            continue;
        };
        let element_kind = match global.space {
            AddressSpace::Uniform | AddressSpace::Handle => {
                shader_ty_to_binding_element_kind(&module, &module.types[global.ty])
            }
            _ => continue,
        };
        let element_kind = if let Some(element_kind) = element_kind {
            element_kind
        } else {
            continue;
        };
        let semantic_binding = shader_mgr.find_semantic_binding(name).and_then(|key| {
            let semantic_binding = shader_mgr.get_semantic_binding(key).unwrap();

            match (&semantic_binding.ty, &element_kind) {
                (
                    BindingType::Buffer {
                        min_binding_size, ..
                    },
                    ReflectedShaderBindingElementKind::Buffer { size },
                ) if *min_binding_size == Some(*size) => {}
                (
                    BindingType::Texture {
                        sample_type,
                        view_dimension,
                        multisampled,
                    },
                    ReflectedShaderBindingElementKind::Texture {
                        sample_type: element_sample_type,
                        view_dimension: element_view_dimension,
                        multisampled: element_multisampled,
                        ..
                    },
                ) if *sample_type == *element_sample_type
                    && *view_dimension == *element_view_dimension
                    && *multisampled == *element_multisampled => {}
                (
                    BindingType::Sampler(binding_type),
                    ReflectedShaderBindingElementKind::Sampler {
                        binding_type: element_binding_type,
                    },
                ) if *binding_type == *element_binding_type => {}
                _ => return None,
            }

            Some(key)
        });

        bindings.push(ReflectedShaderBindingElement {
            semantic_binding,
            name: name.clone(),
            group,
            binding: binding.binding,
            kind: element_kind,
        });
    }

    bindings
}

fn reflect_vertex_entry_point(
    shader_mgr: &ShaderManager,
    module: &Module,
    function: &Function,
) -> Vec<ReflectedShaderInput> {
    let mut inputs = vec![];

    for argument in &function.arguments {
        let ty = &module.types[argument.ty];
        let name = if let Some(name) = ty.name.as_ref() {
            name
        } else {
            continue;
        };
        let step_mode = match name.as_str() {
            "InstanceIn" | "InstanceInput" => VertexStepMode::Instance,
            "VertexIn" | "VertexInput" => VertexStepMode::Vertex,
            _ => continue,
        };
        let (members, span) = if let TypeInner::Struct { members, span } = &ty.inner {
            (members, *span)
        } else {
            continue;
        };

        inputs.push(reflect_shader_input(
            shader_mgr, module, step_mode, span, members,
        ));
    }

    inputs
}

fn reflect_shader_input(
    shader_mgr: &ShaderManager,
    module: &Module,
    step_mode: VertexStepMode,
    span: u32,
    members: &[StructMember],
) -> ReflectedShaderInput {
    let mut elements = Vec::with_capacity(members.len());

    for member in members {
        let name = if let Some(name) = member.name.as_ref() {
            name
        } else {
            continue;
        };
        let location = if let Some(binding) = member.binding.as_ref() {
            match binding {
                Binding::BuiltIn(_) => todo!(),
                Binding::Location { location, .. } => *location,
            }
        } else {
            continue;
        };
        let format = if let Some(format) = shader_ty_to_vertex_format(&module.types[member.ty]) {
            format
        } else {
            continue;
        };
        let semantic_input = shader_mgr.find_semantic_input(name).and_then(|key| {
            let semantic_input = shader_mgr.get_semantic_input(key).unwrap();

            if semantic_input.step_mode != step_mode {
                return None;
            }

            if semantic_input.format != format {
                return None;
            }

            Some(key)
        });

        elements.push(ReflectedShaderInputElement {
            semantic_input,
            name: name.clone(),
            attribute: VertexAttribute {
                format,
                offset: member.offset as BufferAddress,
                shader_location: location,
            },
        });
    }

    ReflectedShaderInput {
        step_mode,
        stride: span as BufferAddress,
        elements,
    }
}

fn reflect_fragment_entry_point(
    shader_mgr: &ShaderManager,
    module: &Module,
    function: &Function,
) -> Option<Vec<ReflectedShaderOutputElement>> {
    let result = if let Some(result) = &function.result {
        result
    } else {
        return None;
    };

    let ty = &module.types[result.ty];
    let name = if let Some(name) = ty.name.as_ref() {
        name
    } else {
        return None;
    };

    match name.as_str() {
        "FragmentOut" | "FragmentOutput" => {}
        _ => return None,
    };

    let members = if let TypeInner::Struct { members, .. } = &ty.inner {
        members
    } else {
        return None;
    };

    Some(reflect_shader_output_elements(shader_mgr, members))
}

fn reflect_shader_output_elements(
    shader_mgr: &ShaderManager,
    members: &[StructMember],
) -> Vec<ReflectedShaderOutputElement> {
    let mut elements = Vec::with_capacity(members.len());

    for member in members {
        let name = if let Some(name) = member.name.as_ref() {
            name
        } else {
            continue;
        };
        let location = if let Some(binding) = member.binding.as_ref() {
            match binding {
                Binding::BuiltIn(_) => todo!(),
                Binding::Location { location, .. } => *location,
            }
        } else {
            continue;
        };
        let semantic_output = shader_mgr.find_semantic_output(name).and_then(|key| {
            let semantic_output = shader_mgr.get_semantic_output(key).unwrap();

            if semantic_output.location != location {
                return None;
            }

            Some(key)
        });

        elements.push(ReflectedShaderOutputElement {
            semantic_output,
            name: name.clone(),
            location,
        });
    }

    elements
}

fn shader_ty_to_binding_element_kind(
    module: &Module,
    ty: &Type,
) -> Option<ReflectedShaderBindingElementKind> {
    fn aligned_size(size: u64, alignment: u64) -> u64 {
        (size + alignment - 1) / alignment * alignment
    }

    fn parse_array_size(module: &Module, size: ArraySize) -> Option<u32> {
        let size = match size {
            ArraySize::Constant(constant) => constant,
            _ => return None,
        };
        let size = match &module.constants[size].inner {
            ConstantInner::Scalar { value, .. } => *value,
            _ => return None,
        };
        let size = match size {
            ScalarValue::Sint(i) => {
                if i <= 0 {
                    return None;
                } else {
                    i as u32
                }
            }
            ScalarValue::Uint(i) => {
                if i == 0 {
                    return None;
                } else {
                    i as u32
                }
            }
            _ => return None,
        };
        Some(size)
    }

    match &ty.inner {
        TypeInner::Scalar { width, .. } => Some(ReflectedShaderBindingElementKind::Buffer {
            size: unsafe { NonZeroU64::new_unchecked(aligned_size(*width as u64, 16)) },
        }),
        TypeInner::Vector { size, width, .. } => Some(ReflectedShaderBindingElementKind::Buffer {
            size: unsafe {
                NonZeroU64::new_unchecked(aligned_size(*size as u64 * *width as u64, 16))
            },
        }),
        TypeInner::Matrix {
            columns,
            rows,
            width,
        } => Some(ReflectedShaderBindingElementKind::Buffer {
            size: unsafe {
                NonZeroU64::new_unchecked(
                    aligned_size(*columns as u64 * *width as u64, 16) * *rows as u64,
                )
            },
        }),
        TypeInner::Array { size, stride, .. } => {
            let size = if let Some(size) = parse_array_size(module, *size) {
                size
            } else {
                return None;
            };

            Some(ReflectedShaderBindingElementKind::Buffer {
                size: unsafe {
                    NonZeroU64::new_unchecked(
                        if *stride < 16 {
                            aligned_size(*stride as u64, 16)
                        } else {
                            *stride as u64
                        } * size as u64,
                    )
                },
            })
        }
        TypeInner::Struct { span, .. } => Some(ReflectedShaderBindingElementKind::Buffer {
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

            Some(ReflectedShaderBindingElementKind::Texture {
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
        TypeInner::Sampler { comparison } => Some(ReflectedShaderBindingElementKind::Sampler {
            binding_type: if *comparison {
                SamplerBindingType::Comparison
            } else {
                SamplerBindingType::Filtering
            },
        }),
        TypeInner::BindingArray { base, size } => {
            let inner = if let Some(inner) =
                shader_ty_to_binding_element_kind(module, &module.types[*base])
            {
                inner
            } else {
                return None;
            };

            match inner {
                ReflectedShaderBindingElementKind::Texture {
                    sample_type,
                    view_dimension: dimension,
                    multisampled,
                    array_size,
                } => {
                    if array_size.is_some() {
                        return None;
                    }

                    let size = if let Some(size) = parse_array_size(module, *size) {
                        size
                    } else {
                        return None;
                    };

                    Some(ReflectedShaderBindingElementKind::Texture {
                        sample_type,
                        view_dimension: dimension,
                        multisampled,
                        array_size: Some(unsafe { NonZeroU32::new_unchecked(size) }),
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
