use crate::engine::gfx::Material;
use wgpu::{Buffer, BufferAddress, RenderPass, VertexStepMode};

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

pub struct RenderingCommand<'r> {
    pub vertex_count: u32,
    pub instance_count: u32,
    pub render_pass: RenderPass<'r>,
    pub per_instance_buffer: Option<GenericBufferAllocation<Buffer>>,
    pub per_vertex_buffer: Option<GenericBufferAllocation<Buffer>>,
}

impl<'r> RenderingCommand<'r> {
    pub fn render(&'r mut self) {
        if let Some(buffer) = &self.per_vertex_buffer {
            self.render_pass.set_vertex_buffer(
                0,
                buffer
                    .buffer()
                    .slice(buffer.offset()..buffer.offset() + buffer.size().get()),
            );
        }

        if let Some(buffer) = &self.per_instance_buffer {
            self.render_pass.set_vertex_buffer(
                1,
                buffer
                    .buffer()
                    .slice(buffer.offset()..buffer.offset() + buffer.size().get()),
            );
        }

        self.render_pass
            .draw(0..self.vertex_count, 0..self.instance_count);
    }
}

pub fn render_with<'r, T>(
    mut render_pass: RenderPass<'r>,
    material: &'r Material,
    renderer: &'r mut dyn Renderer<RenderData = T>,
    render_data: impl Iterator<Item = &'r T> + Clone,
    frame_buffer_allocator: &'r mut FrameBufferAllocator,
) -> RenderingCommand<'r> {
    let render_data_count = render_data.clone().count();

    if render_data_count == 0 {
        return RenderingCommand {
            vertex_count: 0,
            instance_count: 0,
            render_pass,
            per_instance_buffer: None,
            per_vertex_buffer: None,
        };
    }

    let vertex_count = renderer.vertex_count();
    let per_instance_buffer = frame_buffer_allocator.alloc_staging_buffer(
        material.shader.reflected_shader.per_instance_input.stride
            * render_data_count as BufferAddress,
    );
    let per_vertex_buffer = frame_buffer_allocator.alloc_staging_buffer(
        material.shader.reflected_shader.per_vertex_input.stride
            * vertex_count as BufferAddress
            * render_data_count as BufferAddress,
    );

    for (index, render_data) in render_data.enumerate() {
        let per_instance = per_instance_buffer.slice(
            material.shader.reflected_shader.per_instance_input.stride * index as BufferAddress,
            material.shader.reflected_shader.per_instance_input.stride,
        );
        let per_vertex = per_vertex_buffer.slice(
            material.shader.reflected_shader.per_vertex_input.stride
                * vertex_count as BufferAddress
                * index as BufferAddress,
            material.shader.reflected_shader.per_vertex_input.stride
                * vertex_count as BufferAddress,
        );

        for (key, input_data) in &material.semantic_inputs {
            match input_data.step_mode {
                VertexStepMode::Vertex => {
                    let size = material.shader.reflected_shader.per_vertex_input.elements
                        [input_data.index]
                        .attribute
                        .format
                        .size();

                    for vertex_index in 0..vertex_count {
                        renderer.copy_semantic_per_vertex_input(
                            *key,
                            render_data,
                            vertex_index,
                            &mut per_vertex.slice(
                                input_data.offset
                                    + material.shader.reflected_shader.per_vertex_input.stride
                                        * vertex_index as BufferAddress,
                                size,
                            ),
                        );
                    }
                }
                VertexStepMode::Instance => {
                    let size = material.shader.reflected_shader.per_instance_input.elements
                        [input_data.index]
                        .attribute
                        .format
                        .size();

                    renderer.copy_semantic_per_instance_input(
                        *key,
                        render_data,
                        &mut per_instance.slice(input_data.offset, size),
                    );
                }
            }
        }

        for property in material.per_instance_properties.values() {
            if let Some(value) = &property.value {
                per_instance
                    .slice(property.offset, value.to_vertex_format().size())
                    .copy_from_slice(value.as_bytes());
            }
        }
    }

    render_pass.set_pipeline(&material.shader.render_pipeline);

    for bind_group_index in material.bind_properties.values() {
        let bind_group_holder = &material.bind_group_holders[bind_group_index.group_index];

        if let Some(bind_group) = bind_group_holder.bind_group.as_ref() {
            render_pass.set_bind_group(bind_group_holder.group, bind_group, &[]);
        }
    }

    let per_vertex_buffer = frame_buffer_allocator.commit_staging_buffer(per_vertex_buffer);
    let per_instance_buffer = frame_buffer_allocator.commit_staging_buffer(per_instance_buffer);

    RenderingCommand {
        vertex_count,
        instance_count: render_data_count as u32,
        render_pass,
        per_instance_buffer,
        per_vertex_buffer,
    }
}
