#![no_std]
#![feature(strict_provenance)]
#![feature(abi_x86_interrupt)]
#![feature(error_in_core)]

extern crate alloc;
extern crate bitflags;

use core::panic::PanicInfo;

use crate::{
    memory::physical::{buddy_allocator::buddy_allocator::BuddyAllocator, global_alloc::ALLOCATOR},
    multiboot::{memory_map::MemoryEntryType, MultiBootInfo},
    pci::{
        check_pci_buses,
        drivers::{network::NETWORK_DRIVER, PCI_DRIVERS},
    },
    x86::{
        gdt::load_gdt,
        hlt, hlt_loop,
        interrupts::{enable_interrupt, idt::load_idt, pic_8259::PIC},
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
            (device_driver.init_device)(device).unwrap();
            println!(
                "Found known device, vendor_id:{:#x}, device_id:{:#x}, class_code: {:?}, header_type: {:?}",
                device.vendor_id, device.device_id,device.get_class_code(), device.get_header_type(),
            );
        } else {
            println!(
                "Found unknown device, vendor_id:{:#x}, device_id:{:#x}, class_code: {:?}, header_type: {:?}",
                device.vendor_id, device.device_id,device.get_class_code(), device.get_header_type(),
            );
        }
    }

    {
        unsafe {
            PIC.lock().init();
            PIC.lock().master.write_mask(0xFE);
            PIC.lock().slave.write_mask(0xFF);

            // PIC.lock().master.write_mask(0x00);
            // PIC.lock().slave.write_mask(0x00);

            enable_interrupt();
        };
    }

    unsafe {
        println!("transmit head at: {}", NETWORK_DRIVER.lock().as_ref().unwrap().get_head());
        NETWORK_DRIVER
            .lock()
            .as_mut()
            .unwrap()
            .transmit_packet("Hello world".as_bytes(), true)
            .unwrap();

        println!("new transmit head at: {}", NETWORK_DRIVER.lock().as_ref().unwrap().get_head())
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
