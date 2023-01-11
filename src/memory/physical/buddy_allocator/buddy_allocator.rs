use crate::memory::physical::{
    inline_free_list::InlineFreeList, AllocatorError, PhysicalMemoryAllocator, PhysicalMemoryBlock,
    Result,
};

use super::{bitmap::BitMap, memory_area::MemoryArea, BLOCK_SIZES};

pub struct BuddyAllocator {
    memory_areas: [MemoryArea; BLOCK_SIZES.len()],
    size: usize,
    remaining_memory: usize,
    base_address: usize,
}

impl BuddyAllocator {
    pub fn new(base_address: usize, size: usize) -> Self {
        let largest_block_size = *BLOCK_SIZES.last().unwrap();

        // we must be biggest block aligned for xor trick to work at buddy pair search
        let aligned_base_address = base_address + (base_address % largest_block_size);

        let size = size - (aligned_base_address - base_address);
        let base_address = aligned_base_address;

        let mut memory_areas_byte_count: [usize; BLOCK_SIZES.len()] = [0; BLOCK_SIZES.len()];

        let mut total_bitmap_bytes = 0;

        for block_size_index in 0..BLOCK_SIZES.len() {
            // we divide the size of the allocator by the block size and then by 8 to count bytes and by 2 because we pair buddies together
            let divisor = BLOCK_SIZES[block_size_index] * 2 * 8;
            memory_areas_byte_count[block_size_index] = (size + divisor - 1) / divisor;

            total_bitmap_bytes += memory_areas_byte_count[block_size_index];
        }

        let fittest_block_size_for_bitmaps = BLOCK_SIZES
            .iter()
            .find(|&&block_size| block_size > total_bitmap_bytes)
            .map(|&block_size| block_size)
            .unwrap_or(largest_block_size);

        // round up to the largest block count
        let required_fit_blocks = (total_bitmap_bytes + fittest_block_size_for_bitmaps - 1)
            / fittest_block_size_for_bitmaps;

        let size = size - (required_fit_blocks * fittest_block_size_for_bitmaps);

        // get a pointer to the bitmap memory region
        let mut bitmap_current_address = unsafe { (base_address as *const u8).add(size) };

        let memory_areas: [MemoryArea; BLOCK_SIZES.len()] =
            core::array::from_fn(|i| i).map(|index| {
                let byte_count = memory_areas_byte_count[index];

                let bitmap = BitMap::new(bitmap_current_address as usize, byte_count);

                bitmap_current_address = unsafe { bitmap_current_address.add(byte_count) };

                MemoryArea {
                    bitmap,
                    free_list: InlineFreeList::new(),
                    block_size: BLOCK_SIZES[index],
                    merge_buddies: index != (BLOCK_SIZES.len() - 1),
                }
            });

        let mut new_allocator = Self {
            memory_areas,
            size,
            remaining_memory: 0,
            base_address,
        };

        new_allocator.populate_memory();

        new_allocator
    }

    fn populate_memory(&mut self) {
        let smallest_block_size = self.memory_areas.first().unwrap().block_size;

        let mut remaining_size = self.size;
        let mut next_block_address = self.base_address;

        let mut current_block_size_index = BLOCK_SIZES.len() - 1;

        while remaining_size >= smallest_block_size {
            if remaining_size < BLOCK_SIZES[current_block_size_index] {
                current_block_size_index -= 1;
                continue;
            }

            self.free(PhysicalMemoryBlock {
                base_address: next_block_address,
                size: self.memory_areas[current_block_size_index].block_size,
            })
            .unwrap();

            remaining_size -= self.memory_areas[current_block_size_index].block_size;
            next_block_address += self.memory_areas[current_block_size_index].block_size;
        }
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

    pub fn smallest_block_size(&self) -> usize {
        self.memory_areas.first().unwrap().block_size
    }
}

impl PhysicalMemoryAllocator for BuddyAllocator {
    fn allocate(&mut self, size: usize) -> Result<PhysicalMemoryBlock> {
        self.allocate(size)
    }

    fn free(&mut self, physical_memory_block: PhysicalMemoryBlock) -> Result<()> {
        self.free(physical_memory_block)
    }
}
