use core::ops::Range;

use crate::multiboot::memory_map::{MemoryMapEntryIter, MemoryEntryType};

use super::{Frame, FrameAllocator};

// this is the simplest allocator I could build it has bad performance it leaks memory but it will help us bootstrap
pub struct SimpleAllocator<'a> {
    next_frame_index: usize,
    areas: MemoryMapEntryIter<'a>,
    kernel_range: Range<usize>,
    multiboot_range: Range<usize>,
}

impl<'a> SimpleAllocator<'a> {
    pub fn new(
        areas: &MemoryMapEntryIter<'a>,
        kernel_range: Range<usize>,
        multiboot_range: Range<usize>,
    ) -> Self {
        Self {
            next_frame_index: 0,
            areas: areas.clone(),
            kernel_range,
            multiboot_range,
        }
    }

    pub fn get_next_frame(&mut self) -> Option<Frame> {
        let mut frame_iter = self
            .areas
            .clone()
            .filter(|area| area.memory_type == MemoryEntryType::Available)
            .map(|area| area.base_addr as usize..(area.base_addr + area.length) as usize)
            .flat_map(|range| range.step_by(4096))
            .filter(|addr| {
                !self.multiboot_range.contains(addr) && !self.kernel_range.contains(addr)
            })
            .map(|physical_addr| Frame { physical_addr });

        let frame = frame_iter.nth(self.next_frame_index);
        self.next_frame_index += 1;

        frame
    }
}

impl<'a> FrameAllocator for SimpleAllocator<'a> {
    fn allocate_frame(&mut self) -> Option<Frame> {
        self.get_next_frame()
    }

    fn free_frame(&mut self, _frame: Frame) {
        // do nothing ¯\_(ツ)_/¯ we leak it for now
    }
}
