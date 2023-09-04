use std::{cmp::Ordering, sync::Arc};
use wgpu::{Buffer, BufferAddress, BufferSize, BufferSlice, Device};

/// Represents a buffer that can be used to allocate sub buffers from.
pub trait GenericBuffer
where
    Self: Sized,
{
    /// Allocates a new buffer with the given size.
    fn allocate(device: &Device, size: BufferSize) -> Arc<Self>;
}

pub trait GenericBufferMut: GenericBuffer {
    fn with_data(&self, f: impl FnOnce(&[u8]));

    fn with_data_mut(&self, f: impl FnOnce(&mut [u8]));
}

pub trait GenericBufferEmpty: GenericBuffer {
    fn empty() -> Arc<Self>;
}

/// Represents a sub buffer that was allocated from a buffer.
pub struct GenericBufferAllocation<T>
where
    T: GenericBuffer,
{
    buffer: Arc<T>,
    offset: BufferAddress,
    size: BufferSize,
}

impl<T> GenericBufferAllocation<T>
where
    T: GenericBuffer,
{
    pub fn new(buffer: T, offset: BufferAddress, size: BufferSize) -> Self {
        Self {
            buffer: buffer.into(),
            offset,
            size,
        }
    }

    pub fn buffer(&self) -> &Arc<T> {
        &self.buffer
    }

    pub fn offset(&self) -> BufferAddress {
        self.offset
    }

    pub fn size(&self) -> BufferSize {
        self.size
    }

    pub fn slice(&self, offset: BufferAddress, size: BufferAddress) -> Self {
        debug_assert!(offset + size <= self.size.get());

        Self {
            buffer: self.buffer.clone(),
            offset: self.offset + offset,
            size: if size == 0 {
                BufferSize::MAX
            } else {
                unsafe { BufferSize::new_unchecked(size) }
            },
        }
    }
}

impl<T> GenericBufferAllocation<T>
where
    T: GenericBufferMut,
{
    pub fn with_data(&self, f: impl FnOnce(&[u8])) {
        self.buffer.with_data(|data| {
            f(&data[self.offset as usize..(self.offset + self.size.get()) as usize])
        });
    }

    pub fn with_data_mut(&self, f: impl FnOnce(&mut [u8])) {
        self.buffer.with_data_mut(|data| {
            f(&mut data[self.offset as usize..(self.offset + self.size.get()) as usize])
        });
    }

    pub fn copy_from_slice(&mut self, slice: &[u8]) {
        self.with_data_mut(|data| {
            data[..slice.len()].copy_from_slice(slice);
        });
    }
}

impl<T> GenericBufferAllocation<T>
where
    T: GenericBufferEmpty,
{
    pub fn empty() -> Self {
        Self {
            buffer: T::empty(),
            offset: 0,
            size: BufferSize::MAX,
        }
    }

    pub fn is_empty(&self) -> bool {
        self.size == BufferSize::MAX
    }
}

impl GenericBufferAllocation<Buffer> {
    pub fn as_slice<'a>(&'a self) -> BufferSlice<'a> {
        self.buffer
            .slice(self.offset..self.offset + self.size.get())
    }
}

impl Clone for GenericBufferAllocation<Buffer> {
    fn clone(&self) -> Self {
        Self {
            buffer: self.buffer.clone(),
            offset: self.offset,
            size: self.size,
        }
    }
}

/// Represents a page in a buffer list.
pub struct GenericBufferPage<T> {
    /// The buffer that this page belongs to.
    buffer: Arc<T>,
    /// The size of the page.
    size: BufferSize,
    /// The amount of bytes that are already allocated in this page.
    allocated: BufferAddress,
}

impl<T> GenericBufferPage<T>
where
    T: GenericBuffer,
{
    pub fn new(device: &Device, size: BufferSize) -> Self {
        Self {
            buffer: T::allocate(device, size),
            size,
            allocated: 0,
        }
    }

    /// Returns the amount of bytes that are still available in this page.
    pub fn available_size(&self) -> u64 {
        self.size.get() - self.allocated
    }

    /// Allocates a new sub buffer from this page.
    pub fn allocate(&mut self, size: BufferSize) -> GenericBufferAllocation<T> {
        debug_assert!(size.get() <= self.available_size());

        let offset = self.allocated;
        self.allocated += size.get();

        GenericBufferAllocation {
            buffer: self.buffer.clone(),
            offset,
            size,
        }
    }
}

/// Holds a list of buffers of type `T` and allocates sub buffers from them.
pub struct GenericBufferPool<T>
where
    T: GenericBuffer,
{
    /// The size of a single page in the buffer list.
    page_size: BufferSize,
    /// A list of buffers. It is guaranteed that the buffers are always sorted by size in ascending order.
    pages: Vec<GenericBufferPage<T>>,
}

impl<T> GenericBufferPool<T>
where
    T: GenericBuffer,
{
    pub fn new(page_size: BufferSize) -> Self {
        Self {
            page_size,
            pages: Vec::new(),
        }
    }

    /// Mark all pages as unused.
    pub fn recall(&mut self) {
        // TODO: Drop some pages to prevent memory leaks.
        for page in &mut self.pages {
            page.allocated = 0;
        }
    }

    /// Allocates a new buffer with the given size. It may allocate a new page if no page with enough space is available.
    pub fn allocate(&mut self, device: &Device, size: BufferSize) -> GenericBufferAllocation<T> {
        let result = self.pages.binary_search_by(|page| {
            let available_size = page.available_size();
            let size = size.get();
            available_size.cmp(&size)
        });
        let index = match result {
            Ok(index) => index,
            Err(index) => {
                if index == self.pages.len() {
                    self.append_page(device, self.page_size.max(size))
                } else {
                    index
                }
            }
        };

        let mut updated_page = self.pages.remove(index);
        let allocation = updated_page.allocate(size);

        let new_page_index = self
            .pages
            .binary_search_by(|page| {
                if page.available_size() < updated_page.available_size() {
                    Ordering::Less
                } else {
                    Ordering::Greater
                }
            })
            .err()
            .unwrap();
        self.pages.insert(new_page_index, updated_page);

        allocation
    }

    /// Appends a new page to the buffer list. Returns the index of the new page.
    fn append_page(&mut self, device: &Device, page_size: BufferSize) -> usize {
        let index = self
            .pages
            .binary_search_by(|page| {
                // We never return `Equal` here, because we want to insert the new page in correct order.
                if page.available_size() < page_size.get() {
                    Ordering::Less
                } else {
                    Ordering::Greater
                }
            })
            .err()
            .unwrap();

        let page = GenericBufferPage::new(device, page_size);
        self.pages.insert(index, page);

        index
    }
}
