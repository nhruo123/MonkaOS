use super::PhysicalAddress;

pub mod buddy_allocator;
pub mod global_alloc;
mod fixed_block_allocator;
mod inline_free_list;

pub type Result<T> = core::result::Result<T, AllocatorError>;

#[derive(Debug)]
pub struct PhysicalMemoryBlock {
    pub base_address: PhysicalAddress,
    pub size: usize,
}

pub trait PhysicalMemoryAllocator {
    fn allocate(&mut self, size: usize) -> Result<PhysicalMemoryBlock>;
    fn free(&mut self, physical_memory_block: PhysicalMemoryBlock) -> Result<()>;
}

#[derive(Debug, Clone)]
pub enum AllocatorError {
    OutOfMemory,
    UnsupportedSize,
    FreeOutOfBounds,
    UninitializedAllocator
}
