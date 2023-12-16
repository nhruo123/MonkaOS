use super::{
    buddy_allocator::buddy_allocator::BuddyAllocator, inline_free_list::InlineFreeList,
    AllocatorError, PhysicalMemoryBlock, Result,
};

const BLOCK_SIZES: &[usize] = &[8, 16, 32, 64, 128, 256, 512, 1024];

pub struct FixedBlockAllocator {
    free_lists: [InlineFreeList<()>; BLOCK_SIZES.len()],
    buddy_allocator: Option<BuddyAllocator>,
}

impl FixedBlockAllocator {
    pub const fn new() -> Self {
        const EMPTY_FREE_LIST: InlineFreeList<()> = InlineFreeList::<()>::new();
        const FREE_LISTS: [InlineFreeList<()>; BLOCK_SIZES.len()] =
            [EMPTY_FREE_LIST; BLOCK_SIZES.len()];

        FixedBlockAllocator {
            free_lists: FREE_LISTS,
            buddy_allocator: None,
        }
    }

    pub fn init(&mut self, buddy_allocator: BuddyAllocator) {
        self.buddy_allocator = Some(buddy_allocator);
    }

    pub fn allocate(&mut self, size: usize) -> Result<PhysicalMemoryBlock> {
        match Self::get_free_list_index(size) {
            Some(index) => {
                let adder = match self.free_lists[index].pop_head() {
                    Some(result) => result,
                    None => {
                        self.grow_free_list(index)?;
                        self.free_lists[index].pop_head().unwrap()
                    }
                };
                Ok(PhysicalMemoryBlock {
                    base_address: adder,
                    size: BLOCK_SIZES[index],
                })
            }
            None => self
                .buddy_allocator
                .as_mut()
                .ok_or(AllocatorError::UninitializedAllocator)?
                .allocate(size),
        }
    }

    pub fn free(&mut self, block_to_free: PhysicalMemoryBlock) -> Result<()> {
        match Self::get_free_list_index(block_to_free.size) {
            Some(index) => {
                self.free_lists[index].push_head(block_to_free.base_address, ());
                Ok(())
            }
            None => self
                .buddy_allocator
                .as_mut()
                .ok_or(AllocatorError::UninitializedAllocator)?
                .free(block_to_free),
        }
    }

    fn get_free_list_index(required_size: usize) -> Option<usize> {
        BLOCK_SIZES.iter().position(|&size| size >= required_size)
    }

    fn grow_free_list(&mut self, block_index: usize) -> Result<()> {
        let inner_allocator = self
            .buddy_allocator
            .as_mut()
            .ok_or(AllocatorError::UninitializedAllocator)?;

        let new_chunk = inner_allocator.allocate(inner_allocator.smallest_block_size())?;
        let free_list_block_size = BLOCK_SIZES[block_index];
        let free_list = &mut self.free_lists[block_index];

        for block_index in 0..new_chunk.size / free_list_block_size {
            free_list.push_head(
                new_chunk.base_address + block_index * free_list_block_size,
                (),
            );
        }

        Ok(())
    }
}
