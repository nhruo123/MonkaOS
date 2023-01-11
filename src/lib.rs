#![crate_type = "staticlib"]
#![feature(lang_items)]
#![no_std]
#![no_builtins]

#[macro_use]
extern crate bitflags;
extern crate alloc;

use core::panic::PanicInfo;

use alloc::vec::Vec;

use crate::{
    memory::physical::{buddy_allocator::buddy_allocator::BuddyAllocator, global_alloc::ALLOCATOR},
    multiboot::{memory_map::MemoryEntryType, MultiBootInfo},
};

mod memory;
mod multiboot;
mod mutex;
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

    let mut buddy_allocator = BuddyAllocator::new(largest_mem_addr, largest_size);
    println!("-----MEM TEST-----");

    let page1 = buddy_allocator.allocate(1024 * 1024).unwrap();
    let page2 = buddy_allocator.allocate(1024 * 1024).unwrap();
    let page3 = buddy_allocator.allocate(1024 * 1024).unwrap();
    let page4 = buddy_allocator.allocate(1).unwrap();

    println!("page1: {:#x?}", page1.base_address);
    println!("page2: {:#x?}", page2.base_address);
    println!("page3: {:#x?}", page3.base_address);
    println!("page4: {:#x?}", page4.base_address);

    buddy_allocator.free(page4).unwrap();
    buddy_allocator.free(page3).unwrap();
    buddy_allocator.free(page2).unwrap();
    buddy_allocator.free(page1).unwrap();

    let page1 = buddy_allocator.allocate(1024 * 1024).unwrap();
    let page2 = buddy_allocator.allocate(1024 * 1024).unwrap();
    let page3 = buddy_allocator.allocate(1024 * 1024).unwrap();
    let page4 = buddy_allocator.allocate(1).unwrap();

    println!("page1: {:#x?}", page1.base_address);
    println!("page2: {:#x?}", page2.base_address);
    println!("page3: {:#x?}", page3.base_address);
    println!("page4: {:#x?}", page4.base_address);

    buddy_allocator.free(page4).unwrap();
    buddy_allocator.free(page3).unwrap();
    buddy_allocator.free(page2).unwrap();
    buddy_allocator.free(page1).unwrap();

    {
        let mut alloc = ALLOCATOR.lock();
        alloc.init(buddy_allocator);

        let first_alloc = alloc.allocate(10).unwrap();
        println!("first_alloc: {:#x?}", first_alloc);

        alloc.free(first_alloc).unwrap();
        let first_alloc = alloc.allocate(10).unwrap();
        println!("first_alloc: {:#x?}", first_alloc);

        alloc.free(first_alloc).unwrap();
    };

    let mut v = Vec::new();

    for i in 0..100 {
        v.push(i);
    }
    println!("This is a vec after push: {:?}", v);

    for _ in 0..100 {
        v.pop();
    }
    println!("This is a vec after pop: {:?}", v);

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
