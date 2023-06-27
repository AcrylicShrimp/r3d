use crate::engine::gfx::{GfxContextHandle, ReflectedShader};
use codegen::Handle;
use std::{
    collections::{hash_map::Entry, HashMap},
    num::NonZeroU32,
};
use wgpu::{
    BindGroupLayout, BindGroupLayoutDescriptor, BindGroupLayoutEntry, BindingType,
    ColorTargetState, FragmentState, FrontFace, MultisampleState, PipelineLayoutDescriptor,
    PolygonMode, PrimitiveState, PrimitiveTopology, RenderPipeline, RenderPipelineDescriptor,
    ShaderModule, VertexBufferLayout, VertexFormat, VertexState, VertexStepMode,
};

mod semantic_bindings {
    use super::SemanticShaderBinding;
    use std::{mem::size_of, num::NonZeroU64};
    use wgpu::{
        BindingType, BufferBindingType, SamplerBindingType, TextureSampleType, TextureViewDimension,
    };

    pub const CAMERA_TRANSFORM: SemanticShaderBinding = SemanticShaderBinding {
        key: super::SemanticShaderBindingKey::new(1),
        name: "camera_transform",
        ty: BindingType::Buffer {
            ty: BufferBindingType::Uniform,
            has_dynamic_offset: false,
            min_binding_size: Some(unsafe {
                NonZeroU64::new_unchecked(size_of::<[f32; (3 + 1) * 3]>() as u64)
            }),
        },
        count: None,
    };

    pub const SPRITE_TEXTURE: SemanticShaderBinding = SemanticShaderBinding {
        key: super::SemanticShaderBindingKey::new(101),
        name: "sprite_texture",
        ty: BindingType::Texture {
            sample_type: TextureSampleType::Float { filterable: true },
            view_dimension: TextureViewDimension::D2,
            multisampled: false,
        },
        count: None,
    };
    pub const SPRITE_SAMPLER: SemanticShaderBinding = SemanticShaderBinding {
        key: super::SemanticShaderBindingKey::new(102),
        name: "sprite_sampler",
        ty: BindingType::Sampler(SamplerBindingType::Filtering),
        count: None,
    };
}

mod semantic_inputs {
    use super::{SemanticShaderInput, SemanticShaderInputKey};
    use wgpu::{VertexFormat, VertexStepMode};

    pub const POSITION: SemanticShaderInput = SemanticShaderInput {
        key: SemanticShaderInputKey::new(1),
        name: "position",
        format: VertexFormat::Float32x2,
        step_mode: VertexStepMode::Vertex,
    };
    pub const UV: SemanticShaderInput = SemanticShaderInput {
        key: SemanticShaderInputKey::new(2),
        name: "uv",
        format: VertexFormat::Float32x2,
        step_mode: VertexStepMode::Vertex,
    };

    pub const TRANSFORM_ROW_0: SemanticShaderInput = SemanticShaderInput {
        key: SemanticShaderInputKey::new(101),
        name: "transform_row_0",
        format: VertexFormat::Float32x3,
        step_mode: VertexStepMode::Instance,
    };
    pub const TRANSFORM_ROW_1: SemanticShaderInput = SemanticShaderInput {
        key: SemanticShaderInputKey::new(102),
        name: "transform_row_1",
        format: VertexFormat::Float32x3,
        step_mode: VertexStepMode::Instance,
    };
    pub const TRANSFORM_ROW_2: SemanticShaderInput = SemanticShaderInput {
        key: SemanticShaderInputKey::new(103),
        name: "transform_row_2",
        format: VertexFormat::Float32x3,
        step_mode: VertexStepMode::Instance,
    };

    pub const SPRITE_RECT: SemanticShaderInput = SemanticShaderInput {
        key: SemanticShaderInputKey::new(201),
        name: "sprite_rect",
        format: VertexFormat::Float32x4,
        step_mode: VertexStepMode::Instance,
    };
    pub const SPRITE_SIZE: SemanticShaderInput = SemanticShaderInput {
        key: SemanticShaderInputKey::new(202),
        name: "sprite_size",
        format: VertexFormat::Float32x2,
        step_mode: VertexStepMode::Instance,
    };
    pub const SPRITE_COLOR: SemanticShaderInput = SemanticShaderInput {
        key: SemanticShaderInputKey::new(203),
        name: "sprite_color",
        format: VertexFormat::Float32x4,
        step_mode: VertexStepMode::Instance,
    };
}

mod semantic_outputs {
    use super::{SemanticShaderOutput, SemanticShaderOutputKey};
    use wgpu::{BlendState, ColorTargetState, ColorWrites, TextureFormat};

    pub const COLOR: SemanticShaderOutput = SemanticShaderOutput {
        key: SemanticShaderOutputKey::new(1),
        name: "color",
        target: ColorTargetState {
            format: TextureFormat::Bgra8UnormSrgb,
            blend: Some(BlendState::ALPHA_BLENDING),
            write_mask: ColorWrites::ALL,
        },
        location: 0,
    };
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SemanticShaderBindingKey(NonZeroU32);

impl SemanticShaderBindingKey {
    pub const fn new(key: u32) -> Self {
        Self(unsafe { NonZeroU32::new_unchecked(key) })
    }
}

#[derive(Debug, Clone, Hash)]
pub struct SemanticShaderBinding {
    pub key: SemanticShaderBindingKey,
    pub name: &'static str,
    pub ty: BindingType,
    pub count: Option<NonZeroU32>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SemanticShaderInputKey(NonZeroU32);

impl SemanticShaderInputKey {
    pub const fn new(key: u32) -> Self {
        Self(unsafe { NonZeroU32::new_unchecked(key) })
    }
}

#[derive(Debug, Clone, Hash)]
pub struct SemanticShaderInput {
    pub key: SemanticShaderInputKey,
    pub name: &'static str,
    pub format: VertexFormat,
    pub step_mode: VertexStepMode,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SemanticShaderOutputKey(NonZeroU32);

impl SemanticShaderOutputKey {
    pub const fn new(key: u32) -> Self {
        Self(unsafe { NonZeroU32::new_unchecked(key) })
    }
}

#[derive(Debug, Clone, Hash)]
pub struct SemanticShaderOutput {
    pub key: SemanticShaderOutputKey,
    pub name: &'static str,
    pub target: ColorTargetState,
    pub location: u32,
}

#[derive(Debug)]
pub struct ShaderBindGroupLayout {
    pub layout: BindGroupLayout,
    pub entries: Vec<BindGroupLayoutEntry>,
}

#[derive(Handle)]
pub struct Shader {
    pub shader_module: ShaderModule,
    pub render_pipeline: RenderPipeline,
    pub bind_group_layouts: HashMap<u32, ShaderBindGroupLayout>,
    pub reflected_shader: ReflectedShader,
}

pub struct ShaderLayoutManager {
    gfx_ctx: GfxContextHandle,
    binding_names: HashMap<&'static str, SemanticShaderBindingKey>,
    input_names: HashMap<&'static str, SemanticShaderInputKey>,
    output_names: HashMap<&'static str, SemanticShaderOutputKey>,
    bindings: HashMap<SemanticShaderBindingKey, SemanticShaderBinding>,
    inputs: HashMap<SemanticShaderInputKey, SemanticShaderInput>,
    outputs: HashMap<SemanticShaderOutputKey, SemanticShaderOutput>,
}

impl ShaderLayoutManager {
    pub fn new(gfx_ctx: GfxContextHandle) -> Self {
        let mut this = Self {
            gfx_ctx,
            binding_names: HashMap::new(),
            input_names: HashMap::new(),
            output_names: HashMap::new(),
            bindings: HashMap::new(),
            inputs: HashMap::new(),
            outputs: HashMap::new(),
        };

        this.register_binding(semantic_bindings::CAMERA_TRANSFORM);
        this.register_binding(semantic_bindings::SPRITE_TEXTURE);
        this.register_binding(semantic_bindings::SPRITE_SAMPLER);

        this.register_input(semantic_inputs::POSITION);
        this.register_input(semantic_inputs::UV);
        this.register_input(semantic_inputs::TRANSFORM_ROW_0);
        this.register_input(semantic_inputs::TRANSFORM_ROW_1);
        this.register_input(semantic_inputs::TRANSFORM_ROW_2);
        this.register_input(semantic_inputs::SPRITE_RECT);
        this.register_input(semantic_inputs::SPRITE_SIZE);
        this.register_input(semantic_inputs::SPRITE_COLOR);

        this.register_output(semantic_outputs::COLOR);

        this
    }

    fn register_binding(&mut self, binding: SemanticShaderBinding) {
        self.binding_names.insert(binding.name, binding.key);
        self.bindings.insert(binding.key, binding);
    }

    fn register_input(&mut self, input: SemanticShaderInput) {
        self.input_names.insert(input.name, input.key);
        self.inputs.insert(input.key, input);
    }

    fn register_output(&mut self, output: SemanticShaderOutput) {
        self.output_names.insert(output.name, output.key);
        self.outputs.insert(output.key, output);
    }

    pub fn find_semantic_binding(&self, name: &str) -> Option<SemanticShaderBindingKey> {
        self.binding_names.get(name).copied()
    }

    pub fn find_semantic_input(&self, name: &str) -> Option<SemanticShaderInputKey> {
        self.input_names.get(name).copied()
    }

    pub fn find_semantic_output(&self, name: &str) -> Option<SemanticShaderOutputKey> {
        self.output_names.get(name).copied()
    }

    pub fn get_semantic_binding(
        &self,
        key: SemanticShaderBindingKey,
    ) -> Option<&SemanticShaderBinding> {
        self.bindings.get(&key)
    }

    pub fn get_semantic_input(&self, key: SemanticShaderInputKey) -> Option<&SemanticShaderInput> {
        self.inputs.get(&key)
    }

    pub fn get_semantic_output(
        &self,
        key: SemanticShaderOutputKey,
    ) -> Option<&SemanticShaderOutput> {
        self.outputs.get(&key)
    }

    pub fn create_render_pipeline(
        &self,
        shader_module: ShaderModule,
        reflected_shader: ReflectedShader,
    ) -> ShaderHandle {
        let mut bind_group_layout_entries = HashMap::<u32, Vec<_>>::new();

        for binding in &reflected_shader.bindings {
            let layout_entry = BindGroupLayoutEntry::from(binding);

            match bind_group_layout_entries.entry(binding.group) {
                Entry::Occupied(mut entry) => {
                    entry.get_mut().push(layout_entry);
                }
                Entry::Vacant(entry) => {
                    entry.insert(vec![layout_entry]);
                }
            }
        }

        let max_group = bind_group_layout_entries.keys().max().copied().unwrap_or(0);
        let bind_group_layouts = (0..=max_group)
            .map(|group| {
                let entries = bind_group_layout_entries
                    .get(&group)
                    .map(|bind_group| &bind_group[..])
                    .unwrap_or(&[]);

                self.gfx_ctx
                    .device
                    .create_bind_group_layout(&BindGroupLayoutDescriptor {
                        label: None,
                        entries,
                    })
            })
            .collect::<Vec<_>>();
        let bind_group_layouts_refs = bind_group_layouts.iter().collect::<Vec<_>>();
        let pipeline_layout =
            self.gfx_ctx
                .device
                .create_pipeline_layout(&PipelineLayoutDescriptor {
                    label: None,
                    bind_group_layouts: &bind_group_layouts_refs,
                    push_constant_ranges: &[],
                });
        let bind_group_layouts =
            HashMap::from_iter(bind_group_layouts.into_iter().enumerate().map(
                |(group, layout)| {
                    let entries = bind_group_layout_entries
                        .remove(&(group as u32))
                        .unwrap_or_else(|| Vec::new());
                    (group as u32, ShaderBindGroupLayout { layout, entries })
                },
            ));

        let per_vertex_input_buffer_layout_builder = reflected_shader
            .per_vertex_input
            .vertex_buffer_layout_builder();
        let per_instance_input_buffer_layout_builder = reflected_shader
            .per_instance_input
            .vertex_buffer_layout_builder();

        let max_target_location = reflected_shader
            .outputs
            .iter()
            .map(|output| output.location)
            .max()
            .unwrap_or(0);
        let mut targets = (0..=max_target_location).map(|_| None).collect::<Vec<_>>();

        for output in &reflected_shader.outputs {
            let target = output.semantic_output.and_then(|key| {
                self.get_semantic_output(key)
                    .map(|output| output.target.clone())
            });
            targets[output.location as usize] = target;
        }

        let render_pipeline =
            self.gfx_ctx
                .device
                .create_render_pipeline(&RenderPipelineDescriptor {
                    label: None,
                    layout: Some(&pipeline_layout),
                    vertex: VertexState {
                        module: &shader_module,
                        entry_point: &reflected_shader.vertex_entry_point_name,
                        buffers: &[
                            VertexBufferLayout::from(&per_vertex_input_buffer_layout_builder),
                            VertexBufferLayout::from(&per_instance_input_buffer_layout_builder),
                        ],
                    },
                    // TODO: Let materials specify the topology and other settings.
                    primitive: PrimitiveState {
                        topology: PrimitiveTopology::TriangleStrip,
                        strip_index_format: None,
                        front_face: FrontFace::Ccw,
                        cull_mode: None,
                        unclipped_depth: false,
                        polygon_mode: PolygonMode::Fill,
                        conservative: false,
                    },
                    depth_stencil: None,
                    multisample: MultisampleState::default(),
                    fragment: Some(FragmentState {
                        module: &shader_module,
                        entry_point: &reflected_shader.fragment_entry_point_name,
                        targets: &targets,
                    }),
                    multiview: None,
                });

        ShaderHandle::new(Shader {
            shader_module,
            render_pipeline,
            reflected_shader,
            bind_group_layouts,
        })
    }
}
