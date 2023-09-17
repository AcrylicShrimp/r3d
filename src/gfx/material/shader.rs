use super::{inspect_shader, BindGroupLayoutCache, CachedBindGroupLayout, ShaderInspectionError};
use crate::gfx::{GfxContextHandle, ReflectedShader};
use codegen::Handle;
use std::{
    borrow::Cow,
    collections::{hash_map::Entry, HashMap},
    num::NonZeroU32,
};
use wgpu::{
    BindGroupLayoutEntry, BindingType, ColorTargetState, ShaderModule, ShaderModuleDescriptor,
    ShaderSource, VertexFormat, VertexStepMode,
};

pub mod semantic_bindings {
    use super::{SemanticShaderBinding, SemanticShaderBindingKey};
    use std::{mem::size_of, num::NonZeroU64};
    use wgpu::{
        BindingType, BufferBindingType, SamplerBindingType, TextureSampleType, TextureViewDimension,
    };

    pub const KEY_CAMERA_TRANSFORM: SemanticShaderBindingKey = SemanticShaderBindingKey::new(1);
    pub const CAMERA_TRANSFORM: SemanticShaderBinding = SemanticShaderBinding {
        key: KEY_CAMERA_TRANSFORM,
        name: "camera_transform",
        ty: BindingType::Buffer {
            ty: BufferBindingType::Uniform,
            has_dynamic_offset: false,
            min_binding_size: Some(unsafe {
                NonZeroU64::new_unchecked(size_of::<[f32; 4 * 4]>() as u64)
            }),
        },
        count: None,
    };
    pub const KEY_SCREEN_SIZE: SemanticShaderBindingKey = SemanticShaderBindingKey::new(2);
    pub const SCREEN_SIZE: SemanticShaderBinding = SemanticShaderBinding {
        key: KEY_SCREEN_SIZE,
        name: "screen_size",
        ty: BindingType::Buffer {
            ty: BufferBindingType::Uniform,
            has_dynamic_offset: false,
            min_binding_size: Some(unsafe {
                NonZeroU64::new_unchecked(size_of::<[f32; 2 + 2]>() as u64)
            }),
        },
        count: None,
    };

    pub const KEY_SPRITE_TEXTURE: SemanticShaderBindingKey = SemanticShaderBindingKey::new(101);
    pub const SPRITE_TEXTURE: SemanticShaderBinding = SemanticShaderBinding {
        key: KEY_SPRITE_TEXTURE,
        name: "sprite_texture",
        ty: BindingType::Texture {
            sample_type: TextureSampleType::Float { filterable: true },
            view_dimension: TextureViewDimension::D2,
            multisampled: false,
        },
        count: None,
    };
    pub const KEY_SPRITE_SAMPLER: SemanticShaderBindingKey = SemanticShaderBindingKey::new(102);
    pub const SPRITE_SAMPLER: SemanticShaderBinding = SemanticShaderBinding {
        key: KEY_SPRITE_SAMPLER,
        name: "sprite_sampler",
        ty: BindingType::Sampler(SamplerBindingType::Filtering),
        count: None,
    };
}

pub mod semantic_inputs {
    use super::{SemanticShaderInput, SemanticShaderInputKey};
    use wgpu::{VertexFormat, VertexStepMode};

    pub const KEY_POSITION: SemanticShaderInputKey = SemanticShaderInputKey::new(1);
    pub const POSITION: SemanticShaderInput = SemanticShaderInput {
        key: KEY_POSITION,
        name: "position",
        format: VertexFormat::Float32x3,
        step_mode: VertexStepMode::Vertex,
    };
    pub const KEY_NORMAL: SemanticShaderInputKey = SemanticShaderInputKey::new(2);
    pub const NORMAL: SemanticShaderInput = SemanticShaderInput {
        key: KEY_NORMAL,
        name: "normal",
        format: VertexFormat::Float32x3,
        step_mode: VertexStepMode::Vertex,
    };
    pub const KEY_UV: SemanticShaderInputKey = SemanticShaderInputKey::new(3);
    pub const UV: SemanticShaderInput = SemanticShaderInput {
        key: KEY_UV,
        name: "uv",
        format: VertexFormat::Float32x2,
        step_mode: VertexStepMode::Vertex,
    };

    pub const KEY_TRANSFORM_ROW_0: SemanticShaderInputKey = SemanticShaderInputKey::new(101);
    pub const TRANSFORM_ROW_0: SemanticShaderInput = SemanticShaderInput {
        key: KEY_TRANSFORM_ROW_0,
        name: "transform_row_0",
        format: VertexFormat::Float32x4,
        step_mode: VertexStepMode::Instance,
    };
    pub const KEY_TRANSFORM_ROW_1: SemanticShaderInputKey = SemanticShaderInputKey::new(102);
    pub const TRANSFORM_ROW_1: SemanticShaderInput = SemanticShaderInput {
        key: KEY_TRANSFORM_ROW_1,
        name: "transform_row_1",
        format: VertexFormat::Float32x4,
        step_mode: VertexStepMode::Instance,
    };
    pub const KEY_TRANSFORM_ROW_2: SemanticShaderInputKey = SemanticShaderInputKey::new(103);
    pub const TRANSFORM_ROW_2: SemanticShaderInput = SemanticShaderInput {
        key: KEY_TRANSFORM_ROW_2,
        name: "transform_row_2",
        format: VertexFormat::Float32x4,
        step_mode: VertexStepMode::Instance,
    };
    pub const KEY_TRANSFORM_ROW_3: SemanticShaderInputKey = SemanticShaderInputKey::new(104);
    pub const TRANSFORM_ROW_3: SemanticShaderInput = SemanticShaderInput {
        key: KEY_TRANSFORM_ROW_3,
        name: "transform_row_3",
        format: VertexFormat::Float32x4,
        step_mode: VertexStepMode::Instance,
    };

    pub const KEY_SPRITE_SIZE: SemanticShaderInputKey = SemanticShaderInputKey::new(201);
    pub const SPRITE_SIZE: SemanticShaderInput = SemanticShaderInput {
        key: KEY_SPRITE_SIZE,
        name: "sprite_size",
        format: VertexFormat::Float32x2,
        step_mode: VertexStepMode::Instance,
    };
    pub const KEY_SPRITE_OFFSET: SemanticShaderInputKey = SemanticShaderInputKey::new(202);
    pub const SPRITE_OFFSET: SemanticShaderInput = SemanticShaderInput {
        key: KEY_SPRITE_OFFSET,
        name: "sprite_offset",
        format: VertexFormat::Float32x2,
        step_mode: VertexStepMode::Instance,
    };
    pub const KEY_SPRITE_UV_MIN: SemanticShaderInputKey = SemanticShaderInputKey::new(203);
    pub const SPRITE_UV_MIN: SemanticShaderInput = SemanticShaderInput {
        key: KEY_SPRITE_UV_MIN,
        name: "sprite_uv_min",
        format: VertexFormat::Float32x2,
        step_mode: VertexStepMode::Instance,
    };
    pub const KEY_SPRITE_UV_MAX: SemanticShaderInputKey = SemanticShaderInputKey::new(204);
    pub const SPRITE_UV_MAX: SemanticShaderInput = SemanticShaderInput {
        key: KEY_SPRITE_UV_MAX,
        name: "sprite_uv_max",
        format: VertexFormat::Float32x2,
        step_mode: VertexStepMode::Instance,
    };
    pub const KEY_SPRITE_COLOR: SemanticShaderInputKey = SemanticShaderInputKey::new(205);
    pub const SPRITE_COLOR: SemanticShaderInput = SemanticShaderInput {
        key: KEY_SPRITE_COLOR,
        name: "sprite_color",
        format: VertexFormat::Float32x4,
        step_mode: VertexStepMode::Instance,
    };

    pub const KEY_GLYPH_THICKNESS: SemanticShaderInputKey = SemanticShaderInputKey::new(301);
    pub const GLYPH_THICKNESS: SemanticShaderInput = SemanticShaderInput {
        key: KEY_GLYPH_THICKNESS,
        name: "glyph_thickness",
        format: VertexFormat::Float32,
        step_mode: VertexStepMode::Instance,
    };
    pub const KEY_GLYPH_SMOOTHNESS: SemanticShaderInputKey = SemanticShaderInputKey::new(302);
    pub const GLYPH_SMOOTHNESS: SemanticShaderInput = SemanticShaderInput {
        key: KEY_GLYPH_SMOOTHNESS,
        name: "glyph_smoothness",
        format: VertexFormat::Float32,
        step_mode: VertexStepMode::Instance,
    };
}

pub mod semantic_outputs {
    use super::{SemanticShaderOutput, SemanticShaderOutputKey};
    use wgpu::{BlendState, ColorTargetState, ColorWrites, TextureFormat};

    pub const KEY_COLOR: SemanticShaderOutputKey = SemanticShaderOutputKey::new(1);
    pub const COLOR: SemanticShaderOutput = SemanticShaderOutput {
        key: KEY_COLOR,
        name: "color",
        target: ColorTargetState {
            format: TextureFormat::Bgra8Unorm,
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

#[derive(Handle)]
pub struct Shader {
    pub shader_module: ShaderModule,
    pub bind_group_layouts: HashMap<u32, CachedBindGroupLayout>,
    pub reflected_shader: ReflectedShader,
}

pub struct ShaderManager {
    gfx_ctx: GfxContextHandle,
    binding_names: HashMap<&'static str, SemanticShaderBindingKey>,
    input_names: HashMap<&'static str, SemanticShaderInputKey>,
    output_names: HashMap<&'static str, SemanticShaderOutputKey>,
    bindings: HashMap<SemanticShaderBindingKey, SemanticShaderBinding>,
    inputs: HashMap<SemanticShaderInputKey, SemanticShaderInput>,
    outputs: HashMap<SemanticShaderOutputKey, SemanticShaderOutput>,
}

impl ShaderManager {
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
        this.register_binding(semantic_bindings::SCREEN_SIZE);
        this.register_binding(semantic_bindings::SPRITE_TEXTURE);
        this.register_binding(semantic_bindings::SPRITE_SAMPLER);

        this.register_input(semantic_inputs::POSITION);
        this.register_input(semantic_inputs::NORMAL);
        this.register_input(semantic_inputs::UV);
        this.register_input(semantic_inputs::TRANSFORM_ROW_0);
        this.register_input(semantic_inputs::TRANSFORM_ROW_1);
        this.register_input(semantic_inputs::TRANSFORM_ROW_2);
        this.register_input(semantic_inputs::TRANSFORM_ROW_3);
        this.register_input(semantic_inputs::SPRITE_SIZE);
        this.register_input(semantic_inputs::SPRITE_OFFSET);
        this.register_input(semantic_inputs::SPRITE_UV_MIN);
        this.register_input(semantic_inputs::SPRITE_UV_MAX);
        this.register_input(semantic_inputs::SPRITE_COLOR);
        this.register_input(semantic_inputs::GLYPH_THICKNESS);
        this.register_input(semantic_inputs::GLYPH_SMOOTHNESS);

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

    pub fn create_shader(
        &self,
        bind_group_layout_cache: &mut BindGroupLayoutCache,
        source: impl AsRef<str>,
    ) -> Result<ShaderHandle, ShaderInspectionError> {
        let (reflected_shader, shader_module) = self.compile_shader(source)?;

        Ok(self.build_shader(bind_group_layout_cache, shader_module, reflected_shader))
    }

    fn compile_shader(
        &self,
        source: impl AsRef<str>,
    ) -> Result<(ReflectedShader, ShaderModule), ShaderInspectionError> {
        let source = source.as_ref();
        let reflected_shader = inspect_shader(self, source)?;
        let shader_module = self
            .gfx_ctx
            .device
            .create_shader_module(ShaderModuleDescriptor {
                label: None,
                source: ShaderSource::Wgsl(Cow::Borrowed(source)),
            });

        Ok((reflected_shader, shader_module))
    }

    fn build_shader(
        &self,
        bind_group_layout_cache: &mut BindGroupLayoutCache,
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
                    .remove(&group)
                    .unwrap_or_else(|| Vec::new());
                bind_group_layout_cache.create_layout(entries)
            })
            .collect::<Vec<_>>();
        let bind_group_layouts = HashMap::from_iter(
            bind_group_layouts
                .into_iter()
                .enumerate()
                .map(|(group, layout)| (group as u32, layout)),
        );

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

        ShaderHandle::new(Shader {
            shader_module,
            reflected_shader,
            bind_group_layouts,
        })
    }
}
