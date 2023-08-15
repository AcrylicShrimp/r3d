use super::{GenericBuffer, GenericBufferEmpty, GenericBufferMut};
use std::{cell::RefCell, sync::Arc};
use wgpu::{BufferSize, Device};

pub struct HostBuffer {
    buffer: RefCell<Vec<u8>>,
}

impl GenericBuffer for HostBuffer {
    fn allocate(_: &Device, size: BufferSize) -> Arc<Self> {
        Arc::new(Self {
            buffer: RefCell::new(vec![0; size.get() as usize]),
        })
    }
}

impl GenericBufferMut for HostBuffer {
    fn with_data(&self, f: impl FnOnce(&[u8])) {
        f(&self.buffer.borrow())
    }

    fn with_data_mut(&self, f: impl FnOnce(&mut [u8])) {
        f(&mut self.buffer.borrow_mut())
    }
}

impl GenericBufferEmpty for HostBuffer {
    fn empty() -> Arc<Self> {
        Arc::new(Self {
            buffer: RefCell::new(Vec::new()),
        })
    }
}
