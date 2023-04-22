// my take on https://www.gnu.org/software/grub/manual/multiboot2/multiboot.html#Boot-information-format with a lil inspection from phil

use self::{
    memory_map::MemoryMapTag,
    tags::{Tag, TagIter, TagType},
};

pub mod memory_map;
pub mod tags;

pub struct MultiBootInfo {
    inner: *const MultiBootInfoInner,
}

#[repr(C)]
struct MultiBootInfoInner {
    total_size: u32,
    __: u32,
}

impl MultiBootInfo {
    pub fn new(address: usize) -> Self {
        Self {
            inner: address as *const MultiBootInfoInner,
        }
    }

    pub fn tags(&self) -> TagIter {
        // tags come right after the inner
        TagIter::new(unsafe { self.inner.offset(1) } as *const _)
    }

    fn get_tag(&self, tag_type: TagType) -> Option<&Tag> {
        self.tags().find(|tag| tag.tag_type == tag_type)
    }

    pub fn memory_map_tag(&self) -> Option<&MemoryMapTag> {
        self.get_tag(TagType::MemoryMap)
            .map(|tag| unsafe { &*(tag as *const Tag as *const MemoryMapTag) })
    }
}
