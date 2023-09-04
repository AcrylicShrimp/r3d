use super::{GenericBufferAllocation, HostBuffer, PipelineProvider};
use crate::engine::gfx::SemanticShaderInputKey;
use wgpu::{Buffer, BufferAddress};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct RendererVertexBufferLayout {
    pub array_stride: BufferAddress,
    pub attributes: Vec<RendererVertexBufferAttribute>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct RendererVertexBufferAttribute {
    pub key: SemanticShaderInputKey,
    pub offset: BufferAddress,
}

pub trait Renderer {
    fn pipeline_provider(&mut self) -> &mut PipelineProvider;

    fn vertex_count(&self) -> u32;

    fn vertex_buffers(&self) -> Vec<GenericBufferAllocation<Buffer>>;

    fn copy_semantic_per_instance_input(
        &self,
        key: SemanticShaderInputKey,
        allocation: &mut GenericBufferAllocation<HostBuffer>,
    );
}
