use super::GenericBuffer;
use std::sync::Arc;
use wgpu::{Buffer, BufferDescriptor, BufferSize, BufferUsages, Device};

impl GenericBuffer for Buffer {
    fn allocate(device: &Device, size: BufferSize) -> Arc<Self> {
        Arc::new(device.create_buffer(&BufferDescriptor {
            label: None,
            size: size.get(),
            usage: BufferUsages::COPY_DST | BufferUsages::VERTEX,
            mapped_at_creation: false,
        }))
    }
}
