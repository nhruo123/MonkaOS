#![allow(dead_code)]

use core::marker::PhantomData;

#[repr(u32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum TagType {
    End = 0,
    MemoryMap = 6,
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct Tag {
    pub tag_type: TagType,
    size: u32,
}

pub struct TagIter<'a> {
    pub current_tag: *const Tag,
    // The struct is build like an linked list so we want a pointer to the next tag but we want to return a
    // borrowed Tag but then we don't have a lifetime, but you can't define a lifetime without using it so we added this PhantomData
    phantom: PhantomData<&'a Tag>,
}

impl<'a> TagIter<'a> {
    pub fn new(first: *const Tag) -> Self {
        Self {
            current_tag: first,
            phantom: PhantomData,
        }
    }
}

impl<'a> Iterator for TagIter<'a> {
    type Item = &'a Tag;

    fn next(&mut self) -> Option<Self::Item> {
        match unsafe { &*self.current_tag } {
            Tag {
                size: 8,
                tag_type: TagType::End,
            } => None,
            tag => {
                let mut next_tag_addr = self.current_tag as usize;
                // make sure its 8 byte aligned
                // if we are aligned we do nothing else we skip to the next correct alignment
                next_tag_addr += ((tag.size + 7) & !7) as usize;

                self.current_tag = next_tag_addr as *const Tag;

                Some(tag)
            }
        }
    }
}
