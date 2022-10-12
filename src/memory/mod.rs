pub mod simple_allocator;

#[derive(Debug)]
pub struct Frame {
    pub physical_addr: usize,
}

pub trait FrameAllocator {
    fn allocate_frame(&mut self) -> Option<Frame>;
    fn free_frame(&mut self, frame: Frame);
}
