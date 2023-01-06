#![crate_type = "staticlib"]
#![feature(lang_items)]
#![no_std]
#![no_builtins]

#[macro_use]
extern crate bitflags;

use core::panic::PanicInfo;

use crate::{
    memory::physical::buddy_allocator::BuddyAllocator,
    multiboot::{memory_map::MemoryEntryType, MultiBootInfo},
};

mod memory;
mod multiboot;
mod vga_buffer;

#[no_mangle]
pub extern "C" fn _start(multiboot_info_ptr: usize) -> ! {
    println!("multiboot_info_ptr: {:x?}", multiboot_info_ptr);
    let multiboot_info = MultiBootInfo::new(multiboot_info_ptr);

    let memory_map_tag = multiboot_info
        .memory_map_tag()
        .expect("Memory Map is missing from multiboot info");

    println!(
        "memory map (of len {}) entries:",
        memory_map_tag.get_memory_map_entries().count()
    );

    let mut largest_mem_addr: usize = 0;
    let mut largest_size: usize = 0;

    for entry in memory_map_tag.get_memory_map_entries() {
        if (entry.memory_type == MemoryEntryType::Available)
            && ((entry.length as usize) > largest_size)
        {
            largest_mem_addr = entry.base_addr as usize;
            largest_size = entry.length as usize;
        }
        println!("{:?}", entry);
    }

    let _buddy_allocator = BuddyAllocator::new(largest_mem_addr as *const u8, largest_size);

    println!("hello form the other side!");
    loop {}
}

#[panic_handler]
#[inline(never)]
#[no_mangle]
fn panic(info: &PanicInfo) -> ! {
    println!("{}", info);
    loop {}
}
