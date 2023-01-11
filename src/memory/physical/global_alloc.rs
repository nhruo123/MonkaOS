use core::alloc::GlobalAlloc;

use crate::{mutex::Mutex, println};

use super::fixed_block_allocator::FixedBlockAllocator;

#[global_allocator]
pub static ALLOCATOR: Mutex<FixedBlockAllocator> = Mutex::new(FixedBlockAllocator::new());

unsafe impl GlobalAlloc for Mutex<FixedBlockAllocator> {
    unsafe fn alloc(&self, layout: core::alloc::Layout) -> *mut u8 {
        let mut alloc = self.lock();

        alloc
            .allocate(layout.size().max(layout.align()))
            .map_err(|err| println!("{:?}", err))
            .map_or(core::ptr::null_mut(), |block| block.base_address as *mut u8)
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: core::alloc::Layout) {
        let mut alloc = self.lock();

        let result = alloc.free(super::PhysicalMemoryBlock {
            base_address: ptr as usize,
            size: layout.size().max(layout.align()),
        });

        match result {
            Ok(_) => (),
            Err(err) => println!("{:?}", err),
        }
    }
}
