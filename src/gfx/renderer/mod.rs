use super::{
    semantic_bindings,
    semantic_inputs::{self},
    CachedPipeline, Material, PipelineCache, ShaderManager,
};
use crate::object::{ObjectHierarchy, ObjectId};
use parking_lot::RwLockReadGuard;
use wgpu::{BindGroup, Buffer, BufferAddress, RenderPass, VertexStepMode};
use zerocopy::AsBytes;

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

pub struct RenderingCommand<'r> {
    pub pipeline: CachedPipeline,
    pub material: RwLockReadGuard<'r, Material>,
    pub instance_count: u32,
    pub vertex_count: u32,
    pub bind_group_provider: &'r dyn BindGroupProvider,
    pub per_vertex_buffers: Vec<GenericBufferAllocation<Buffer>>,
    pub per_instance_buffer: Option<GenericBufferAllocation<Buffer>>,
}

impl<'r> RenderingCommand<'r> {
    pub fn render(
        &'r self,
        render_pass: &mut RenderPass<'r>,
        camera_transform_bind_group: &'r BindGroup,
        screen_size_bind_group: &'r BindGroup,
    ) {
        render_pass.set_pipeline(self.pipeline.as_ref());

        for binding in &self.material.shader.reflected_shader.bindings {
            let key = if let Some(key) = binding.semantic_binding {
                key
            } else {
                continue;
            };

            match key {
                semantic_bindings::KEY_CAMERA_TRANSFORM => {
                    render_pass.set_bind_group(binding.group, camera_transform_bind_group, &[]);
                }
                semantic_bindings::KEY_SCREEN_SIZE => {
                    render_pass.set_bind_group(binding.group, screen_size_bind_group, &[]);
                }
                _ => {
                    if let Some(bind_group) = self.bind_group_provider.bind_group(0, key) {
                        render_pass.set_bind_group(binding.group, &bind_group, &[]);
                    }
                }
            }
        }

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

        render_pass.draw(0..self.vertex_count, 0..self.instance_count);
    }
}

pub fn build_rendering_command<'r>(
    object_id: ObjectId,
    object_hierarchy: &ObjectHierarchy,
    renderer: &'r mut dyn Renderer,
    bind_group_provider: &'r dyn BindGroupProvider,
    per_instance_data_provider: &dyn PerInstanceDataProvider,
    shader_mgr: &ShaderManager,
    pipeline_cache: &mut PipelineCache,
    frame_buffer_allocator: &mut FrameBufferAllocator,
) -> Option<RenderingCommand<'r>> {
    let matrix = object_hierarchy.matrix(object_id);

    let pipeline_provider = renderer.pipeline_provider();
    let pipeline =
        if let Some(pipeline) = pipeline_provider.obtain_pipeline(shader_mgr, pipeline_cache) {
            pipeline
        } else {
            return None;
        };
    let material = if let Some(material) = pipeline_provider.material() {
        material.clone()
    } else {
        return None;
    };
    let material = material.read();

    let instance_count = renderer.instance_count();
    let per_instance_buffer = frame_buffer_allocator.alloc_staging_buffer(
        material.shader.reflected_shader.per_instance_input.stride
            * instance_count as BufferAddress,
    );

    for instance in 0..instance_count {
        let per_instance_buffer = per_instance_buffer.slice(
            material.shader.reflected_shader.per_instance_input.stride * instance as BufferAddress,
            material.shader.reflected_shader.per_instance_input.stride,
        );

        for (&key, input_data) in &material.semantic_inputs {
            if input_data.step_mode != VertexStepMode::Instance {
                continue;
            }

            let size = material.shader.reflected_shader.per_instance_input.elements
                [input_data.index]
                .attribute
                .format
                .size();
            let allocation = &mut per_instance_buffer.slice(input_data.offset, size);

            match key {
                semantic_inputs::KEY_TRANSFORM_ROW_0 => {
                    allocation.copy_from_slice(matrix.row(0).as_bytes())
                }
                semantic_inputs::KEY_TRANSFORM_ROW_1 => {
                    allocation.copy_from_slice(matrix.row(1).as_bytes())
                }
                semantic_inputs::KEY_TRANSFORM_ROW_2 => {
                    allocation.copy_from_slice(matrix.row(2).as_bytes())
                }
                semantic_inputs::KEY_TRANSFORM_ROW_3 => {
                    allocation.copy_from_slice(matrix.row(3).as_bytes())
                }
                _ => {
                    per_instance_data_provider.copy_per_instance_data(instance, key, allocation);
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
    }

    let per_instance_buffer = frame_buffer_allocator.commit_staging_buffer(per_instance_buffer);

    Some(RenderingCommand {
        pipeline,
        instance_count,
        vertex_count: renderer.vertex_count(),
        bind_group_provider,
        per_vertex_buffers: renderer.vertex_buffers(),
        per_instance_buffer,
        material: renderer.pipeline_provider().material().unwrap().read(),
    })
}
