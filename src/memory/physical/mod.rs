use super::PhysicalAddress;

pub mod buddy_allocator;

pub type Result<T> = core::result::Result<T, AllocatorError>;

#[derive(Debug)]
pub struct PhysicalMemoryBlock {
    pub base_address: PhysicalAddress,
    pub size: usize,
}

pub trait PhysicalMemoryAllocator {
    fn allocate(&mut self, size: usize) -> Option<PhysicalMemoryBlock>;
    fn free(&mut self, physical_memory_block: PhysicalMemoryBlock);
}

#[derive(Debug, Clone)]
pub enum AllocatorError {
    OutOfMemory,
    UnsupportedSize,
    FreeOutOfBounds,
}