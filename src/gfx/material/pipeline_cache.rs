use super::{CachedPipelineLayout, ShaderHandle, ShaderManager};
use crate::gfx::GfxContextHandle;
use std::{
    collections::HashMap,
    hash::Hash,
    sync::{Arc, Weak},
};
use wgpu::{
    BufferAddress, DepthStencilState, Device, FragmentState, PrimitiveState, RenderPipeline,
    RenderPipelineDescriptor, VertexAttribute, VertexBufferLayout, VertexState, VertexStepMode,
};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct BufferLayout {
    pub array_stride: BufferAddress,
    pub step_mode: VertexStepMode,
    pub attributes: Vec<VertexAttribute>,
}

#[derive(Clone, PartialEq, Eq, Hash)]
pub struct PipelineKey {
    pub layout: CachedPipelineLayout,
    pub shader: ShaderHandle,
    pub buffer_layouts: Vec<BufferLayout>,
    pub primitive: PrimitiveState,
    pub depth_stencil: Option<DepthStencilState>,
}

impl PipelineKey {
    pub fn create_pipeline(&self, device: &Device, shader_mgr: &ShaderManager) -> RenderPipeline {
        let buffers = Vec::from_iter(self.buffer_layouts.iter().map(|buffer| VertexBufferLayout {
            array_stride: buffer.array_stride,
            step_mode: buffer.step_mode,
            attributes: &buffer.attributes,
        }));
        let max_target_location = self
            .shader
            .reflected_shader
            .outputs
            .iter()
            .map(|output| output.location)
            .max()
            .unwrap_or(0);
        let mut targets = (0..=max_target_location).map(|_| None).collect::<Vec<_>>();

        for output in &self.shader.reflected_shader.outputs {
            let target = output.semantic_output.and_then(|key| {
                shader_mgr
                    .get_semantic_output(key)
                    .map(|output| output.target.clone())
            });
            targets[output.location as usize] = target;
        }

        device.create_render_pipeline(&RenderPipelineDescriptor {
            label: None,
            layout: Some(self.layout.as_ref()),
            vertex: VertexState {
                module: &self.shader.shader_module,
                entry_point: &self.shader.reflected_shader.vertex_entry_point_name,
                buffers: &buffers,
            },
            primitive: self.primitive,
            depth_stencil: self.depth_stencil.clone(),
            multisample: Default::default(),
            fragment: Some(FragmentState {
                module: &self.shader.shader_module,
                entry_point: &self.shader.reflected_shader.fragment_entry_point_name,
                targets: &targets,
            }),
            multiview: None,
        })
    }
}

#[derive(Debug, Clone)]
pub struct CachedPipeline {
    pipeline: Arc<RenderPipeline>,
}

impl CachedPipeline {
    pub fn new(pipeline: Arc<RenderPipeline>) -> Self {
        Self { pipeline }
    }
}

impl AsRef<RenderPipeline> for CachedPipeline {
    fn as_ref(&self) -> &RenderPipeline {
        &self.pipeline
    }
}

impl PartialEq for CachedPipeline {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.pipeline, &other.pipeline)
    }
}

impl Eq for CachedPipeline {}

impl Hash for CachedPipeline {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        Arc::as_ptr(&self.pipeline).hash(state);
    }
}

pub struct PipelineCache {
    gfx_ctx: GfxContextHandle,
    caches: HashMap<PipelineKey, Weak<RenderPipeline>>,
}

impl PipelineCache {
    pub fn new(gfx_ctx: GfxContextHandle) -> Self {
        Self {
            gfx_ctx,
            caches: HashMap::new(),
        }
    }

    pub fn create_pipeline(
        &mut self,
        shader_mgr: &ShaderManager,
        layout: CachedPipelineLayout,
        shader: ShaderHandle,
        buffer_layouts: Vec<BufferLayout>,
        primitive: PrimitiveState,
        depth_stencil: Option<DepthStencilState>,
    ) -> CachedPipeline {
        let key = PipelineKey {
            layout,
            shader,
            buffer_layouts,
            primitive,
            depth_stencil,
        };

        if let Some(pipeline) = self.caches.get(&key).and_then(|weak| weak.upgrade()) {
            return CachedPipeline::new(pipeline);
        }

        let pipeline = Arc::new(key.create_pipeline(&self.gfx_ctx.device, shader_mgr));
        self.caches.insert(key, Arc::downgrade(&pipeline));

        CachedPipeline::new(pipeline)
    }
}
