use super::{GenericBufferAllocation, GenericBufferPool, HostBuffer};
use crate::engine::gfx::GfxContextHandle;
use std::mem::replace;
use wgpu::{
    util::StagingBelt, Buffer, BufferAddress, BufferSize, CommandBuffer, CommandEncoder,
    CommandEncoderDescriptor, Device,
};

/// A buffer allocator that can be used to allocate buffers for a single frame.
pub struct FrameBufferAllocator {
    gfx_context: GfxContextHandle,
    staging_belt: StagingBelt,
    staging_belt_encoder: CommandEncoder,
    host_buffer_list: GenericBufferPool<HostBuffer>,
    device_buffer_list: GenericBufferPool<Buffer>,
}

impl FrameBufferAllocator {
    /// The size of a single page in the buffer list. It is currently set to 1 MiB.
    pub const PAGE_SIZE: BufferSize = unsafe { BufferSize::new_unchecked(1 * 1024 * 1024) };

    pub fn new(gfx_context: GfxContextHandle) -> FrameBufferAllocator {
        Self {
            staging_belt: StagingBelt::new(Self::PAGE_SIZE.get()),
            staging_belt_encoder: create_staging_belt_encoder(&gfx_context.device),
            host_buffer_list: GenericBufferPool::new(Self::PAGE_SIZE),
            device_buffer_list: GenericBufferPool::new(Self::PAGE_SIZE),
            gfx_context,
        }
    }

    pub fn alloc_staging_buffer(
        &mut self,
        size: BufferAddress,
    ) -> GenericBufferAllocation<HostBuffer> {
        if size == 0 {
            GenericBufferAllocation::empty()
        } else {
            self.host_buffer_list
                .allocate(&self.gfx_context.device, unsafe {
                    BufferSize::new_unchecked(size)
                })
        }
    }

    pub fn commit_staging_buffer(
        &mut self,
        allocation: GenericBufferAllocation<HostBuffer>,
    ) -> Option<GenericBufferAllocation<Buffer>> {
        if allocation.is_empty() {
            return None;
        }

        let device_allocation = self
            .device_buffer_list
            .allocate(&self.gfx_context.device, allocation.size());
        let mut view = self.staging_belt.write_buffer(
            &mut self.staging_belt_encoder,
            device_allocation.buffer(),
            device_allocation.offset(),
            device_allocation.size(),
            &self.gfx_context.device,
        );

        allocation.with_data(|data| {
            view.copy_from_slice(
                &data[allocation.offset() as usize
                    ..(allocation.offset() + allocation.size().get()) as usize],
            )
        });

        Some(device_allocation)
    }

    pub fn finish(&mut self) -> CommandBuffer {
        replace(
            &mut self.staging_belt_encoder,
            create_staging_belt_encoder(&self.gfx_context.device),
        )
        .finish()
    }

    pub fn recall(&mut self) {
        self.staging_belt.recall();
        self.host_buffer_list.recall();
        self.device_buffer_list.recall();
    }
}

fn create_staging_belt_encoder(device: &Device) -> CommandEncoder {
    device.create_command_encoder(&CommandEncoderDescriptor {
        label: Some("[frame buffer allocator] staging belt encoder"),
    })
}
