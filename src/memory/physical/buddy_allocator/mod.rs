mod bitmap;
pub mod buddy_allocator;
mod memory_area;

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
