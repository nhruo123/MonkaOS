#![crate_type = "staticlib"]
#![feature(lang_items)]
#![no_std]
#![no_builtins]
#![feature(strict_provenance)]
#![feature(const_trait_impl)]
#![feature(const_option)]
#![feature(abi_x86_interrupt)]
#![feature(default_alloc_error_handler)]
#![feature(core_intrinsics)]

extern crate alloc;
extern crate bitflags;

use core::{mem::size_of, panic::PanicInfo};

use crate::{
    memory::physical::{buddy_allocator::buddy_allocator::BuddyAllocator, global_alloc::ALLOCATOR},
    multiboot::{memory_map::MemoryEntryType, MultiBootInfo},
    pci::{check_pci_buses, drivers::PCI_DRIVERS},
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
        if let Some(device_driver) = PCI_DRIVERS.iter().find(|driver_entry| {
            driver_entry.device_id == device.device_id && driver_entry.vendor_id == device.vendor_id
        }) {
            (device_driver.init_device)(device);
        } else {
            println!(
                "Found unknown device, vendor_id:{:#x}, device_id:{:#x}",
                device.vendor_id, device.device_id
            );
        }
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
