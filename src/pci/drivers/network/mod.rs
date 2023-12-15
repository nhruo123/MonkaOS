// https://pdos.csail.mit.edu/6.828/2011/readings/hardware/8254x_GBe_SDM.pdf

use core::{
    alloc::{GlobalAlloc, Layout},
    mem::size_of,
    ptr::write_volatile,
};

use alloc::boxed::Box;
use bitflags::bitflags;

use crate::{
    memory::physical::global_alloc::ALLOCATOR,
    multiboot::memory_map::{self, MemoryMapEntry},
    pci::config_space::{
        base_address_register::MemorySpace, BaseAddressRegister, MemoryMappedRegister,
        PciConfigSpace,
    },
    println,
};

use super::PciDriver;

const TRANSMIT_DESCRIPTOR_BASE_LOW: MemoryMappedRegister<u32> = MemoryMappedRegister::new(0x3800);
const TRANSMIT_DESCRIPTOR_BASE_HIGH: MemoryMappedRegister<u32> = MemoryMappedRegister::new(0x3804);
const TRANSMIT_DESCRIPTOR_BASE_LEN: MemoryMappedRegister<u64> = MemoryMappedRegister::new(0x3808);
const TRANSMIT_DESCRIPTOR_BASE_HEAD: MemoryMappedRegister<u64> = MemoryMappedRegister::new(0x3810);
const TRANSMIT_DESCRIPTOR_BASE_TAIL: MemoryMappedRegister<u64> = MemoryMappedRegister::new(0x3818);

pub const E1000_DRIVER_ENTRY: PciDriver = PciDriver {
    vendor_id: 0x8086,
    device_id: 0x100E,
    init_device: init_e1000,
};

bitflags! {
    #[derive(Default)]
    pub struct CommandRegister: u8 {
        const END_OF_PACKET = 1 << 0;
        const IFCS = 1 << 1;
        const IC = 1 << 2;
        const REPORT_STATUS = 1 << 3;
        const REPORT_PACKET_SEND = 1 << 4;
        const DEXT = 1 << 5;
        const VLE = 1 << 6;
        const IDE = 1 << 7;
    }
}

bitflags! {
    #[derive(Default)]
    pub struct StatusRegister: u8 {
        // 4 RESERVE BITS
        const DESCRIPTOR_DONE = 1 << 4;
        const EXCESS_COLLISIONS = 1 << 5;
        const LATE_COLLISION = 1 << 6;
        const TRANSMIT_UNDERRUN = 1 << 7;
    }
}

#[derive(Default, Clone, Copy)]
#[repr(C, align(32))]
struct TransmissionDescriptor {
    base_address: u64,
    length: u16,
    cso: u8,
    command: CommandRegister,
    status: StatusRegister,
    css: u8,
    special: u16,
}

impl TransmissionDescriptor {
    const fn empty() -> Self {
        Self {
            base_address: 0,
            command: CommandRegister::from_bits(0).unwrap(),
            cso: 0,
            length: 0,
            css: 0,
            special: 0,
            status: StatusRegister::from_bits(0).unwrap(),
        }
    }
}

const TRANSMISSION_DESCRIPTOR_LIST_SIZE: usize = 1 << 8;

static mut TRANSMISSION_DESCRIPTOR_LIST: [TransmissionDescriptor;
    TRANSMISSION_DESCRIPTOR_LIST_SIZE] =
    [TransmissionDescriptor::empty(); TRANSMISSION_DESCRIPTOR_LIST_SIZE];

pub fn init_e1000(pci: &mut PciConfigSpace) {
    println!("I found e1000! yay! \n{:#x?}", pci);
    let BaseAddressRegister::MemorySpace(mut memory_space) = pci.base_address_registers[0] else {
        // TODO: handle errors for now panic
        panic!("Unexpected Base Register type!");
    };

    unsafe {
        init_descriptors_list(&mut memory_space);
    }
}

unsafe fn init_descriptors_list(memory_space: &mut MemorySpace) {
    TRANSMIT_DESCRIPTOR_BASE_LOW.write(
        memory_space,
        TRANSMISSION_DESCRIPTOR_LIST.as_ptr().addr() as u32,
    );

    TRANSMIT_DESCRIPTOR_BASE_LOW.write(
        memory_space,
        (TRANSMISSION_DESCRIPTOR_LIST.len() * size_of::<TransmissionDescriptor>()) as u32,
    );
    
    TRANSMIT_DESCRIPTOR_BASE_HEAD.write(memory_space, 0);
    TRANSMIT_DESCRIPTOR_BASE_TAIL.write(memory_space, 0);
}
