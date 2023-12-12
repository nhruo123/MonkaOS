#![crate_type = "staticlib"]
#![feature(lang_items)]
#![no_std]
#![no_builtins]
#![feature(abi_x86_interrupt)]
#![feature(default_alloc_error_handler)]
#![feature(core_intrinsics)]

extern crate alloc;
extern crate bitflags;

use core::{mem::size_of, panic::PanicInfo};

use crate::{
    memory::physical::{buddy_allocator::buddy_allocator::BuddyAllocator, global_alloc::ALLOCATOR},
    multiboot::{memory_map::MemoryEntryType, MultiBootInfo},
    pci::check_pci_buses,
    x86::{
        gdt::load_gdt,
        hlt_loop,
        interrupts::{
            enable_interrupt,
            idt::{load_idt, InterruptDescriptorTable},
            pic_8259::PIC,
        },
    },
};

mod memory;
mod multiboot;
mod mutex;
mod pci;
mod vga_buffer;
mod x86;
mod drivers;

#[no_mangle]
pub extern "C" fn _start(multiboot_info_ptr: usize) -> ! {
    load_gdt();
    load_idt();

    let multiboot_info = MultiBootInfo::new(multiboot_info_ptr);

    let memory_map_tag = multiboot_info
        .memory_map_tag()
        .expect("Memory Map is missing from multiboot info");

    let mut largest_mem_addr: usize = 0;
    let mut largest_size: usize = 0;

    for entry in memory_map_tag.get_memory_map_entries() {
        if (entry.memory_type == MemoryEntryType::Available)
            && ((entry.length as usize) > largest_size)
        {
            largest_mem_addr = entry.base_addr as usize;
            largest_size = entry.length as usize;
        }
    }

    let buddy_allocator = BuddyAllocator::new(largest_mem_addr, largest_size);

    {
        let mut alloc = ALLOCATOR.lock();
        alloc.init(buddy_allocator);
    };

    let mut pci_devices = check_pci_buses();

    for device in &mut pci_devices {
        println!("{:#x?}", device);
    }

    {
        unsafe {
            PIC.lock().init();
            PIC.lock().master.write_mask(0xFE);
            PIC.lock().slave.write_mask(0xFF);
            enable_interrupt();
        };
    }
    println!("hello form the other side!");
    hlt_loop();
}

#[panic_handler]
#[inline(never)]
#[no_mangle]
fn panic(info: &PanicInfo) -> ! {
    println!("{}", info);
    hlt_loop();
}
