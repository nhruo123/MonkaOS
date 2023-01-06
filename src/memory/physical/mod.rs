use super::PhysicalAddress;

pub mod buddy_allocator;

#[derive(Debug)]
pub struct PhysicalMemoryBlock {
    pub start_address: PhysicalAddress,
    pub length: usize,
}

pub trait PhysicalMemoryAllocator {
    fn allocate(&mut self, size: usize) -> Option<PhysicalMemoryBlock>;
    fn free(&mut self, physical_memory_block: PhysicalMemoryBlock);
}