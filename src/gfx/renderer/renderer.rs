use super::{GenericBufferAllocation, HostBuffer};
use crate::gfx::{CachedPipeline, Material, SemanticShaderBindingKey, SemanticShaderInputKey};
use parking_lot::RwLockReadGuard;
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
    fn pipeline(&self) -> CachedPipeline;

    fn material(&self) -> RwLockReadGuard<Material>;

    fn instance_count(&self) -> u32;

    fn vertex_count(&self) -> u32;

    fn bind_group_provider(&self) -> &dyn BindGroupProvider;

    fn vertex_buffer_provider(&self) -> &dyn VertexBufferProvider;

    fn instance_data_provider(&self) -> &dyn InstanceDataProvider;
}

pub trait BindGroupProvider {
    fn bind_group(&self, instance: u32, key: SemanticShaderBindingKey) -> Option<&BindGroup>;
}

pub struct VertexBuffer<'a> {
    pub slot: u32,
    pub buffer: &'a GenericBufferAllocation<Buffer>,
}

pub trait VertexBufferProvider {
    fn vertex_buffer_count(&self) -> u32;
    fn vertex_buffer(&self, key: SemanticShaderInputKey) -> Option<VertexBuffer>;
}

pub trait InstanceDataProvider {
    fn copy_per_instance_data(
        &self,
        instance: u32,
        key: SemanticShaderInputKey,
        buffer: &mut GenericBufferAllocation<HostBuffer>,
    );
}
