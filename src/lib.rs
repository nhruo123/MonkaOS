#![crate_type = "staticlib"]
#![feature(lang_items)]
#![no_std]
#![no_builtins]

use core::panic::PanicInfo;

use crate::{
    memory::simple_allocator::{self, SimpleAllocator},
    multiboot::MultiBootInfo,
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
    for entry in memory_map_tag.get_memory_map_entries() {
        println!("{:?}", entry);
    }

    let mut simple_allocator =
        SimpleAllocator::new(&memory_map_tag.get_memory_map_entries(), 0..0, 0..0);

    println!("we are about to allocate frames!");
    for i in 0..10 {
        let frame = simple_allocator.get_next_frame();
        println!("{} frame is: {:?}", i, frame)
    }

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
