use super::RendererVertexBufferLayout;
use crate::engine::gfx::{
    BufferLayout, CachedPipeline, MaterialHandle, PipelineCache, ShaderManager,
};
use wgpu::{DepthStencilState, PrimitiveState, VertexAttribute, VertexStepMode};

// TODO: Should we make buffer layouts and states to be shared across all renderer instances?
pub struct PipelineProvider {
    is_dirty: bool,
    pipeline: Option<CachedPipeline>,
    material: Option<MaterialHandle>,
    buffer_layouts: Vec<RendererVertexBufferLayout>,
    primitive: Option<PrimitiveState>,
    depth_stencil: Option<DepthStencilState>,
}

impl PipelineProvider {
    pub fn new() -> Self {
        Self {
            is_dirty: true,
            pipeline: None,
            material: None,
            buffer_layouts: Vec::new(),
            primitive: None,
            depth_stencil: None,
        }
    }

    pub fn material(&self) -> Option<MaterialHandle> {
        self.material.clone()
    }

    pub fn set_material(&mut self, material: MaterialHandle) {
        self.is_dirty = true;
        self.material = Some(material);
    }

    pub fn set_buffer_layouts(&mut self, buffer_layouts: Vec<RendererVertexBufferLayout>) {
        self.is_dirty = true;
        self.buffer_layouts = buffer_layouts;
    }

    pub fn set_primitive(&mut self, primitive: PrimitiveState) {
        self.is_dirty = true;
        self.primitive = Some(primitive);
    }

    pub fn set_depth_stencil(&mut self, depth_stencil: Option<DepthStencilState>) {
        self.is_dirty = true;
        self.depth_stencil = depth_stencil;
    }

    pub fn obtain_pipeline(
        &mut self,
        shader_mgr: &ShaderManager,
        pipeline_cache: &mut PipelineCache,
    ) -> Option<CachedPipeline> {
        if !self.is_dirty {
            if let Some(pipeline) = self.pipeline.clone() {
                return Some(pipeline);
            }
        }

        let material = if let Some(material) = &self.material {
            material
        } else {
            return None;
        };

        if self.buffer_layouts.len() == 0 {
            return None;
        }

        let primitive = if let Some(primitive) = self.primitive.clone() {
            primitive
        } else {
            return None;
        };
        let mut buffer_layouts =
            Vec::from_iter(self.buffer_layouts.iter().map(|layout| BufferLayout {
                array_stride: layout.array_stride,
                step_mode: VertexStepMode::Vertex,
                attributes: Vec::from_iter(layout.attributes.iter().filter_map(|attribute| {
                    let input = match shader_mgr.get_semantic_input(attribute.key) {
                        Some(input) => input,
                        None => {
                            return None;
                        }
                    };

                    if input.step_mode != VertexStepMode::Vertex {
                        return None;
                    }

                    let material_input = match material.semantic_inputs.get(&attribute.key) {
                        Some(input) => input,
                        None => {
                            return None;
                        }
                    };

                    Some(VertexAttribute {
                        format: input.format,
                        offset: attribute.offset,
                        shader_location: material_input.shader_location,
                    })
                })),
            }));
        let per_instance_attributes = Vec::from_iter(
            material
                .shader
                .reflected_shader
                .per_instance_input
                .elements
                .iter()
                .map(|element| element.attribute.clone()),
        );

        buffer_layouts.push(BufferLayout {
            array_stride: material.shader.reflected_shader.per_instance_input.stride,
            step_mode: VertexStepMode::Instance,
            attributes: per_instance_attributes,
        });

        let pipeline = pipeline_cache.create_pipeline(
            shader_mgr,
            material.pipeline_layout.clone(),
            material.shader.clone(),
            buffer_layouts,
            primitive,
            self.depth_stencil.clone(),
        );

        self.is_dirty = false;
        self.pipeline = Some(pipeline.clone());

        Some(pipeline)
    }
}
