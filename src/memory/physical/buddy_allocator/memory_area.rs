use core::marker::PhantomData;

use super::{bitmap::BitMap, free_list::FreeList};

#[repr(C)]
pub struct MemoryArea<'a> {
    pub free_list: FreeList<'a>,
    pub bitmap: BitMap<'a>,
    pub block_size: usize,
    pub merge_buddies: bool,
    pub _phantom: PhantomData<&'a FreeList<'a>>,
}

impl<'a> MemoryArea<'a> {
    pub fn allocate_block(&mut self, base_addr: usize) -> Option<usize> {
        if let Some(start_address) = self.free_list.pop() {
            let buddy_index = self.get_buddy_bitmap_index(base_addr, start_address);

            // we want to store 1 only if 1 of the blocks is missing
            // so if we find 1 we don't have a pair so we set it to 0
            let current_bitmap_status = self
                .bitmap
                .get_bit(buddy_index)
                .expect("bitmap out of range");

            self.bitmap
                .set_bit(buddy_index, !current_bitmap_status)
                .unwrap();

            return Some(start_address);
        } else {
            return None;
        }
    }

    fn get_buddy_bitmap_index(&self, base_addr: usize, block_addr: usize) -> usize {
        ((block_addr - base_addr) / self.block_size) / 2
    }

    // returns true if blocks have been merged
    pub fn free_block(&mut self, freed_memory: usize, alloc_base_addr: usize) -> Option<bool> {
        debug_assert!(freed_memory >= alloc_base_addr);

        let buddy_addr = freed_memory ^ self.block_size;
        let buddy_index = self.get_buddy_bitmap_index(alloc_base_addr, freed_memory);
        let current_bitmap_status = self.bitmap.get_bit(buddy_index)?;
        self.bitmap.set_bit(buddy_index, !current_bitmap_status)?;

        // are we about to merge 2 blocks
        if self.merge_buddies && current_bitmap_status {
            self.free_list
                .remove_at_address(buddy_addr);
        } else {
            self.free_list.insert(freed_memory);
        }

        return Some(current_bitmap_status);
    }
}
