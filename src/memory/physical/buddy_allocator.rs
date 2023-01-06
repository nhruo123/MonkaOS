use core::{marker::PhantomData, ptr::NonNull, slice::from_raw_parts_mut};

use crate::println;

// google "interrupt safe spinlock"

const PAGE_SIZE: usize = 1024 * 4; // 4 KB

const BLOCK_SIZES: &[usize] = &[
    PAGE_SIZE * 1,
    PAGE_SIZE * 2,
    PAGE_SIZE * 4,
    PAGE_SIZE * 8,
    PAGE_SIZE * 16,
    PAGE_SIZE * 32,
    PAGE_SIZE * 64,
    PAGE_SIZE * 128,
    PAGE_SIZE * 256,
    PAGE_SIZE * 512,
];

// This is a free list of buddy blocks
#[repr(C)]
struct FreeList<'a> {
    next: Option<NonNull<FreeList<'a>>>,
    prev: Option<NonNull<FreeList<'a>>>,
    _phantom: PhantomData<&'a FreeList<'a>>,
}

struct BitMap<'a> {
    inner_array: &'a mut [u8],
}

#[repr(C)]
struct MemoryArea<'a> {
    free_list: Option<NonNull<FreeList<'a>>>,
    bitmap: BitMap<'a>,
    block_size: usize,
    _phantom: PhantomData<&'a FreeList<'a>>,
}

#[repr(C)]
pub struct BuddyAllocator<'a> {
    memory_areas: [MemoryArea<'a>; BLOCK_SIZES.len()],
    size: usize,
    remaining_memory: usize,
    base_address: usize,
}

impl<'a> BuddyAllocator<'a> {
    pub fn new(base_address: *const u8, size: usize) -> Self {
        let largest_block = *BLOCK_SIZES.last().unwrap();

        let aligned_base_address =
            (base_address as usize) + (largest_block - ((base_address as usize) % largest_block));
        let aligned_size = size - (aligned_base_address - (base_address as usize));
        let aligned_size = aligned_size + (largest_block - (aligned_size % largest_block));

        if aligned_size < largest_block {
            panic!("Not enough memory to allocate the biggest buddy, memory provided: {}, required memory: {}",aligned_size, largest_block);
        }

        let mut memory_areas_byte_count: [usize; BLOCK_SIZES.len()] = [0; BLOCK_SIZES.len()];

        let mut total_byte_count = 0;

        // last bitmap is not required so we leave it empty
        for block_size_index in 0..(BLOCK_SIZES.len() - 1) {
            memory_areas_byte_count[block_size_index] =
                ((aligned_size / BLOCK_SIZES[block_size_index]) / 8) / 2;

            total_byte_count += memory_areas_byte_count[block_size_index];
        }

        // for now we will only allocate form the largest page group this wast a lot of space but i need this part working asap
        // TODO: make sure we use the lowest amount of memory

        // round up to the largest block count
        let required_largest_blocks = (total_byte_count + largest_block - 1) / largest_block;

        let final_size = aligned_size - (required_largest_blocks * largest_block);

        // get a pointer to the bitmap memory region
        let mut bitmap_current_address = unsafe { base_address.add(final_size) };

        let memory_areas: [MemoryArea; BLOCK_SIZES.len()] =
            core::array::from_fn(|i| i).map(|index| {
                let byte_count = memory_areas_byte_count[index];

                let bitmap = BitMap::new(unsafe {
                    from_raw_parts_mut(bitmap_current_address as *mut u8, byte_count)
                });

                bitmap_current_address = unsafe { bitmap_current_address.add(byte_count) };

                let free_list = if BLOCK_SIZES[index] == largest_block {
                    Some(FreeList::from_raw_memory(
                        aligned_base_address as *mut u8,
                        final_size,
                        BLOCK_SIZES[index],
                    ))
                } else {
                    None
                };

                MemoryArea {
                    bitmap,
                    free_list,
                    block_size: BLOCK_SIZES[index],
                    _phantom: PhantomData,
                }
            });

        Self {
            memory_areas,
            size: final_size,
            remaining_memory: final_size,
            base_address: aligned_base_address,
        }
    }

    pub fn remaining_memory(&self) -> usize {
        self.remaining_memory
    }
}

impl<'a> MemoryArea<'a> {}

impl<'a> FreeList<'a> {
    fn from_raw_memory(mem: *mut u8, mem_size: usize, block_size: usize) -> NonNull<FreeList<'a>> {
        let mut prev: Option<NonNull<FreeList>> = None;

        for node_offset in (0..mem_size).step_by(block_size) {
            unsafe {
                let node_addr = mem.add(node_offset) as *mut FreeList;
                *node_addr = FreeList {
                    prev,
                    next: None,
                    _phantom: PhantomData,
                };
                if let Some(mut prev_addr) = prev {
                    let prev_ref = prev_addr.as_mut();
                    prev_ref.next = Some(NonNull::new_unchecked(node_addr));
                    prev = prev_ref.next;
                }
            }
        }

        unsafe { NonNull::new_unchecked(mem as *mut FreeList) }
    }
}

impl<'a> BitMap<'a> {
    fn new(inner_array: &'a mut [u8]) -> Self {
        for byte in inner_array.iter_mut() {
            *byte = 0;
        }

        BitMap { inner_array }
    }

    pub fn get_bit(self, index: usize) -> Option<bool> {
        let byte_index = Self::get_byte_index(index);
        let bit_mask: u8 = Self::get_bit_mask(index);

        self.inner_array
            .get(byte_index)
            .map(|byte| (byte & bit_mask) != 0)
    }

    pub fn set_bit(mut self, index: usize, value: bool) -> Option<()> {
        let byte_index = Self::get_byte_index(index);
        let bit_mask: u8 = Self::get_bit_mask(index);

        if let Some(target_byte) = self.inner_array.get_mut(byte_index) {
            if value {
                *target_byte |= bit_mask;
            } else {
                *target_byte &= bit_mask;
            }
            Some(())
        } else {
            None
        }
    }

    fn get_byte_index(index: usize) -> usize {
        index / 8
    }

    fn get_bit_mask(index: usize) -> u8 {
        1 << (index % 8)
    }
}
