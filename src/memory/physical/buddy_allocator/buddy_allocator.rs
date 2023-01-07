use core::{marker::PhantomData, slice::from_raw_parts_mut};

use crate::memory::physical::{AllocatorError, PhysicalMemoryBlock, Result};

use super::{bitmap::BitMap, free_list::FreeList, memory_area::MemoryArea, BLOCK_SIZES};

#[repr(C)]
pub struct BuddyAllocator<'a> {
    memory_areas: [MemoryArea<'a>; BLOCK_SIZES.len()],
    size: usize,
    remaining_memory: usize,
    base_address: usize,
}

impl<'a> BuddyAllocator<'a> {
    pub fn new(base_address: usize, size: usize) -> Self {
        let largest_block = *BLOCK_SIZES.last().unwrap();

        // we must be biggest block aligned for xor trick to work at buddy pair search
        let aligned_base_address = base_address + (largest_block - (base_address % largest_block));
        let aligned_size = size - (aligned_base_address - base_address);

        let mut memory_areas_byte_count: [usize; BLOCK_SIZES.len()] = [0; BLOCK_SIZES.len()];

        let mut total_byte_count = 0;

        for block_size_index in 0..BLOCK_SIZES.len() {
            // we divide the size of the allocator by the block size and then by 8 to count bytes and by 2 because we pair buddies together
            let divisor = BLOCK_SIZES[block_size_index] * 2 * 8;
            memory_areas_byte_count[block_size_index] = (aligned_size + divisor - 1) / divisor;

            total_byte_count += memory_areas_byte_count[block_size_index];
        }

        let mut fittest_block_size = largest_block;
        for &block_size in BLOCK_SIZES {
            if block_size > size {
                fittest_block_size = block_size;
                break;
            }
        }

        // round up to the largest block count
        let required_fit_blocks = (total_byte_count + fittest_block_size - 1) / fittest_block_size;

        let final_size = aligned_size - (required_fit_blocks * fittest_block_size);

        // get a pointer to the bitmap memory region
        let mut bitmap_current_address = unsafe { (base_address as *const u8).add(final_size) };

        let memory_areas: [MemoryArea; BLOCK_SIZES.len()] =
            core::array::from_fn(|i| i).map(|index| {
                let byte_count = memory_areas_byte_count[index];

                let bitmap = BitMap::new(unsafe {
                    from_raw_parts_mut(bitmap_current_address as *mut u8, byte_count)
                });

                bitmap_current_address = unsafe { bitmap_current_address.add(byte_count) };

                MemoryArea {
                    bitmap,
                    free_list: FreeList::new_empty(),
                    block_size: BLOCK_SIZES[index],
                    merge_buddies: index != (BLOCK_SIZES.len() - 1),
                    _phantom: PhantomData,
                }
            });

        let mut new_allocator = Self {
            memory_areas,
            size: final_size,
            remaining_memory: 0,
            base_address: aligned_base_address,
        };

        let mut remaining_heap_size = final_size;
        let mut current_block_size_index = BLOCK_SIZES.len() - 1;

        while remaining_heap_size >= *BLOCK_SIZES.first().unwrap() {
            if remaining_heap_size < BLOCK_SIZES[current_block_size_index] {
                current_block_size_index -= 1;
                continue;
            }

            new_allocator
                .free(PhysicalMemoryBlock {
                    base_address: new_allocator.base_address + new_allocator.size
                        - remaining_heap_size,
                    size: BLOCK_SIZES[current_block_size_index],
                })
                .unwrap();

            remaining_heap_size -= BLOCK_SIZES[current_block_size_index];
        }

        new_allocator
    }

    pub fn remaining_memory(&self) -> usize {
        self.remaining_memory
    }

    pub fn allocate(&mut self, size: usize) -> Result<PhysicalMemoryBlock> {
        if self.remaining_memory < size {
            return Err(AllocatorError::OutOfMemory);
        }

        let area_index = if let Some(area_index) = self.fit_arena_index(size) {
            area_index
        } else {
            return Err(AllocatorError::UnsupportedSize);
        };

        self.remaining_memory -= self.memory_areas[area_index].block_size;

        return Ok(PhysicalMemoryBlock {
            base_address: self.inner_alloc(area_index),
            size: self.memory_areas[area_index].block_size,
        });
    }

    fn inner_alloc(&mut self, area_index: usize) -> usize {
        assert!(area_index < self.memory_areas.len());

        if let Some(meme_block) = self.memory_areas[area_index].allocate_block(self.base_address) {
            meme_block
        } else {
            let meme_block = self.inner_alloc(area_index + 1);

            self.memory_areas[area_index]
                .free_block(meme_block, self.base_address)
                .expect(
                    "We can't find valid memory block while splitting from largess memory blocks",
                );
            meme_block + self.memory_areas[area_index].block_size
        }
    }

    pub fn free(&mut self, block_to_free: PhysicalMemoryBlock) -> Result<()> {
        if block_to_free.base_address > self.memory_areas.last().unwrap().block_size + self.size {
            return Err(AllocatorError::FreeOutOfBounds);
        }

        let mut area_index = if let Some(area_index) = self.fit_arena_index(block_to_free.size) {
            area_index
        } else {
            return Err(AllocatorError::UnsupportedSize);
        };

        self.remaining_memory += self.memory_areas[area_index].block_size;

        while area_index < self.memory_areas.len()
            && self.memory_areas[area_index]
                .free_block(
                    // we want the lower buddy in a pair if we are chain freeing
                    block_to_free.base_address & !(self.memory_areas[area_index].block_size - 1),
                    self.base_address,
                )
                .unwrap()
        {
            area_index += 1;
        }

        Ok(())
    }

    fn fit_arena_index(&self, size: usize) -> Option<usize> {
        for index in 0..self.memory_areas.len() {
            if size <= self.memory_areas[index].block_size {
                return Some(index);
            }
        }
        None
    }
}
