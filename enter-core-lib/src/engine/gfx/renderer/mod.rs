use super::{CachedPipeline, MaterialHandle, PipelineCache, ShaderManager};
use wgpu::{Buffer, RenderPass, VertexStepMode};

mod device_buffer;
mod frame_buffer_allocator;
mod generic_buffer_pool;
mod host_buffer;
mod pipeline_provider;
mod renderer;
mod renderer_impls;

pub use device_buffer::*;
pub use frame_buffer_allocator::*;
pub use generic_buffer_pool::*;
pub use host_buffer::*;
pub use pipeline_provider::*;
pub use renderer::*;
pub use renderer_impls::*;

pub struct RenderingCommand {
    pub pipeline: CachedPipeline,
    pub material: MaterialHandle,
    pub vertex_count: u32,
    pub per_instance_buffer: Option<GenericBufferAllocation<Buffer>>,
    pub per_vertex_buffers: Vec<GenericBufferAllocation<Buffer>>,
}

impl RenderingCommand {
    pub fn render<'r, 'c: 'r>(&'c self, render_pass: &mut RenderPass<'r>) {
        render_pass.set_pipeline(self.pipeline.as_ref());

        for bind_group_index in self.material.bind_properties.values() {
            let bind_group_holder = &self.material.bind_group_holders[bind_group_index.group_index];

            if let Some(bind_group) = bind_group_holder.bind_group.as_ref() {
                render_pass.set_bind_group(bind_group_holder.group, bind_group, &[]);
            }
        }

        for (index, buffer) in self.per_vertex_buffers.iter().enumerate() {
            render_pass.set_vertex_buffer(index as u32, buffer.as_slice());
        }

        if let Some(buffer) = &self.per_instance_buffer {
            render_pass.set_vertex_buffer(self.per_vertex_buffers.len() as u32, buffer.as_slice());
        }

        render_pass.draw(0..self.vertex_count, 0..1);
    }
}

pub fn build_rendering_command(
    renderer: &mut dyn Renderer,
    shader_mgr: &ShaderManager,
    pipeline_cache: &mut PipelineCache,
    frame_buffer_allocator: &mut FrameBufferAllocator,
) -> Option<RenderingCommand> {
    let pipeline_provider = renderer.pipeline_provider();

    let pipeline =
        if let Some(pipeline) = pipeline_provider.obtain_pipeline(shader_mgr, pipeline_cache) {
            pipeline
        } else {
            return None;
        };
    let material = if let Some(material) = pipeline_provider.material() {
        material
    } else {
        return None;
    };

    let per_instance_buffer = frame_buffer_allocator
        .alloc_staging_buffer(material.shader.reflected_shader.per_instance_input.stride);

    for (key, input_data) in &material.semantic_inputs {
        match input_data.step_mode {
            VertexStepMode::Vertex => {}
            VertexStepMode::Instance => {
                let size = material.shader.reflected_shader.per_instance_input.elements
                    [input_data.index]
                    .attribute
                    .format
                    .size();

                renderer.copy_semantic_per_instance_input(
                    *key,
                    &mut per_instance_buffer.slice(input_data.offset, size),
                );
            }
        }
    }

    for property in material.per_instance_properties.values() {
        if let Some(value) = &property.value {
            per_instance_buffer
                .slice(property.offset, value.to_vertex_format().size())
                .copy_from_slice(value.as_bytes());
        }
    }

    let per_instance_buffer = frame_buffer_allocator.commit_staging_buffer(per_instance_buffer);

    Some(RenderingCommand {
        pipeline,
        material,
        vertex_count: renderer.vertex_count(),
        per_instance_buffer,
        per_vertex_buffers: renderer.vertex_buffers(),
    })
}
