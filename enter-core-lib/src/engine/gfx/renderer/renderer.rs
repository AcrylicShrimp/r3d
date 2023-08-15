use super::{GenericBufferAllocation, HostBuffer};
use crate::engine::gfx::SemanticShaderInputKey;

pub trait Renderer {
    type RenderData;

    fn vertex_count(&self) -> u32;

    fn copy_semantic_per_instance_input(
        &self,
        key: SemanticShaderInputKey,
        data: &Self::RenderData,
        allocation: &mut GenericBufferAllocation<HostBuffer>,
    );

    fn copy_semantic_per_vertex_input(
        &self,
        key: SemanticShaderInputKey,
        data: &Self::RenderData,
        vertex_index: u32,
        allocation: &mut GenericBufferAllocation<HostBuffer>,
    );
}
