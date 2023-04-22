use core::{marker::PhantomData, mem::size_of};

use super::tags::TagType;

#[derive(Debug)]
#[repr(C)]
pub struct MemoryMapTag {
    tag_type: TagType,
    size: u32,
    entry_size: u32,
    entry_version: u32,
    first_entry: MemoryMapEntry,
}

#[derive(Debug)]
#[repr(C)]
pub struct MemoryMapEntry {
    pub base_addr: u64,
    pub length: u64,
    pub memory_type: MemoryEntryType,
    __: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[repr(u32)]
pub enum MemoryEntryType {
    Available = 1,
    Reserved = 2,
    AcpiAvailable = 3, // TODO: find out what is Acpi
    ReservedHibernate = 4,
    Defective = 5,
}

impl MemoryMapTag {
    pub fn get_memory_map_entries(&self) -> MemoryMapEntryIter {
        let first_entry_ptr = (&self.first_entry) as *const MemoryMapEntry;
        let entry_count = (self.size - size_of::<MemoryMapTag>() as u32
            + size_of::<MemoryMapEntry>() as u32)
            / self.entry_size;
        MemoryMapEntryIter::new(entry_count, self.entry_size, first_entry_ptr)
    }

    pub fn get_available_memory_map_entries(&self) -> impl Iterator<Item = &MemoryMapEntry> {
        self.get_memory_map_entries()
            .filter(|entry| entry.memory_type == MemoryEntryType::Available)
    }
}

#[derive(Debug, Clone)]
pub struct MemoryMapEntryIter<'a> {
    current_entry: u32,
    entry_count: u32,
    entry_size: u32,
    first_entry_ptr: *const MemoryMapEntry,
    phantom: PhantomData<&'a MemoryMapEntry>,
}

impl<'a> MemoryMapEntryIter<'a> {
    pub fn new(entry_count: u32, entry_size: u32, first_entry_ptr: *const MemoryMapEntry) -> Self {
        Self {
            current_entry: 0,
            entry_count,
            entry_size,
            first_entry_ptr,
            phantom: PhantomData,
        }
    }
}

impl<'a> Iterator for MemoryMapEntryIter<'a> {
    type Item = &'a MemoryMapEntry;

    fn next(&mut self) -> Option<Self::Item> {
        if self.current_entry >= self.entry_count {
            None
        } else {
            let new_entry_addr =
                (self.first_entry_ptr as usize) + (self.current_entry * self.entry_size) as usize;
            let entry = unsafe { &*(new_entry_addr as *const MemoryMapEntry) };

            self.current_entry += 1;
            Some(entry)
        }
    }
}
