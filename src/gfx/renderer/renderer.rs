use super::{GenericBufferAllocation, HostBuffer, PipelineProvider};
use crate::gfx::{SemanticShaderBindingKey, SemanticShaderInputKey};
use wgpu::{BindGroup, Buffer, BufferAddress};

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

    fn instance_count(&self) -> u32;

    fn vertex_count(&self) -> u32;

    fn vertex_buffers(&self) -> Vec<GenericBufferAllocation<Buffer>>;
}

pub trait BindGroupProvider {
    fn bind_group(&self, instance: u32, key: SemanticShaderBindingKey) -> Option<&BindGroup>;
}

pub trait PerInstanceDataProvider {
    fn copy_per_instance_data(
        &self,
        instance: u32,
        key: SemanticShaderInputKey,
        buffer: &mut GenericBufferAllocation<HostBuffer>,
    );
}
