// https://pdos.csail.mit.edu/6.828/2011/readings/hardware/8254x_GBe_SDM.pdf

use core::{
    alloc::{GlobalAlloc, Layout},
    mem::size_of,
    ptr::write_volatile,
};

use alloc::boxed::Box;
use bitflags::bitflags;
use modular_bitfield::{
    bitfield,
    specifiers::{B1, B10, B15, B16, B2, B5, B6},
    BitfieldSpecifier,
};

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

const DEVICE_CONTROL: MemoryMappedRegister<DeviceControlRegister> =
    MemoryMappedRegister::new(0x0000);
const DEVICE_STATUS: MemoryMappedRegister<DeviceStatusRegister> = MemoryMappedRegister::new(0x0008);

const TRANSMIT_CONTROL_REGISTER: MemoryMappedRegister<TransmissionControlRegister> =
    MemoryMappedRegister::new(0x00400);
const TRANSMIT_IPG_REGISTER: MemoryMappedRegister<TransmissionIpgRegister> =
    MemoryMappedRegister::new(0x00410);

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

#[derive(BitfieldSpecifier, Clone, Copy, Debug)]
#[bits = 2]
enum SpeedSelection {
    TenMbs,
    HundredMbs,
    ThousandMbs,
}

#[bitfield]
#[repr(packed, C)]
#[derive(Debug)]
pub struct DeviceControlRegister {
    full_duplex: bool,
    #[skip]
    __: B2,
    LINK_RESET: B1,
    #[skip]
    __: B1,
    AUTO_SPEED_DETECTION_ENABLED: B1,
    SET_LINK_UP: B1,
    ILOS: B1,
    #[bits = 2]
    speed: SpeedSelection,
    #[skip]
    __: B1,
    Force_Speed: bool,
    Force_Duplex: bool,
    #[skip]
    __: B5,
    SDP0_DATA: B1,
    SDP1_DATA: B1,
    ADVD3WUC: B1,
    EN_PHY_PWR_MGMT: B1,
    SDP0_IODIR: B1,
    SDP1_IODIR: B1,
    #[skip]
    __: B2,
    device_reset: bool,
    RFCE: bool,
    TFCE: bool,
    #[skip]
    __: B1,
    VME: bool,
    PHY_RST: bool,
}

#[derive(BitfieldSpecifier, Clone, Copy, Debug)]
#[bits = 2]
enum FunctionID {
    LAN_A,
    LAN_B,
}

#[derive(BitfieldSpecifier, Clone, Copy, Debug)]
#[bits = 2]
enum PciXBusSpeed {
    LOW,
    MID,
    HIGH,
}

#[bitfield]
#[repr(packed, C)]
#[derive(Debug)]
pub struct DeviceStatusRegister {
    full_duplex: bool,
    link_up_indication: bool,
    function_id: FunctionID,
    transmission_paused: bool,
    TBIMODE: bool,
    speed: SpeedSelection,
    ASDV: SpeedSelection,
    #[skip]
    __: B1,
    PCI66: B1,
    BUS64: B1,
    PCIX_MODE1: B1,
    #[bits = 2]
    PCIXSPD1: PciXBusSpeed,
    #[skip]
    __: B16,
}

#[bitfield]
#[repr(packed, C)]
#[derive(Debug)]
pub struct TransmissionControlRegister {
    #[skip]
    __: B1,
    enabled: bool,
    #[skip]
    __: B1,
    pad_short_packets: bool,
    collision_threshold: u8,
    collision_distance: B10,
    SWXOFF: bool,
    #[skip]
    __: B1,
    RTLC: bool,
    NRTU: bool,
    #[skip]
    __: B6,
}

#[bitfield]
#[repr(packed, C)]
#[derive(Debug)]
pub struct TransmissionIpgRegister {
    ipgt: B10,
    ipgr1: B10,
    ipgr2: B10,
    #[skip]
    __: B2,
}

bitflags! {
    #[derive(Default)]
    pub struct TransmissionCommandRegister: u8 {
        const END_OF_PACKET = 1 << 0;
        const IFCS = 1 << 1;
        const IC = 1 << 2;
        const REPORT_STATUS = 1 << 3;
        const REPORT_PACKET_SEND = 1 << 4;
        const DEXT = 1 << 5;
        const VLE = 1 << 6;
        const IDE = 1 << 7;
    }
    #[derive(Default)]
    pub struct TransmissionStatusRegister: u8 {
        // 4 RESERVE BITS
        const DESCRIPTOR_DONE = 1 << 4;
        const EXCESS_COLLISIONS = 1 << 5;
        const LATE_COLLISION = 1 << 6;
        const TRANSMIT_UNDERRUN = 1 << 7;
    }
}

#[derive(Default, Clone, Copy)]
#[repr(C, packed(32))]
struct TransmissionDescriptor {
    base_address: u64,
    length: u16,
    cso: u8,
    command: TransmissionCommandRegister,
    status: TransmissionStatusRegister,
    css: u8,
    special: u16,
}

impl TransmissionDescriptor {
    const fn empty() -> Self {
        let status = unsafe { TransmissionStatusRegister::from_bits_unchecked(1 << 4) };

        Self {
            base_address: 0,
            command: TransmissionCommandRegister::empty(),
            cso: 0,
            length: 0,
            css: 0,
            special: 0,
            status,
        }
    }
}

const TRANSMISSION_DESCRIPTOR_LIST_SIZE: usize = 1 << 8;

static mut TRANSMISSION_DESCRIPTOR_LIST: [TransmissionDescriptor;
    TRANSMISSION_DESCRIPTOR_LIST_SIZE] =
    [TransmissionDescriptor::empty(); TRANSMISSION_DESCRIPTOR_LIST_SIZE];

pub fn init_e1000(pci: &mut PciConfigSpace) {
    println!("I found e1000!");
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

    TRANSMIT_DESCRIPTOR_BASE_LEN.write(
        memory_space,
        (TRANSMISSION_DESCRIPTOR_LIST.len() * size_of::<TransmissionDescriptor>()) as u64,
    );

    TRANSMIT_DESCRIPTOR_BASE_HEAD.write(memory_space, 0);
    TRANSMIT_DESCRIPTOR_BASE_TAIL.write(memory_space, 0);

    TRANSMIT_CONTROL_REGISTER.write(
        memory_space,
        TransmissionControlRegister::new()
            .with_enabled(true)
            .with_pad_short_packets(true)
            .with_collision_threshold(0x10)
            .with_collision_distance_checked(0x200)
            .unwrap(),
    );

    TRANSMIT_IPG_REGISTER.write(
        memory_space,
        TransmissionIpgRegister::new()
            .with_ipgt(10)
            .with_ipgr1(8)
            .with_ipgr2(6),
    );

    transmit_packet("Hello World, This is MonkaOs's e1000 driver!".as_bytes(), memory_space);
}

// TODO: handle errors
unsafe fn transmit_packet(data: &[u8], memory_space: &mut MemorySpace) {
    let tail = TRANSMIT_DESCRIPTOR_BASE_TAIL.read(memory_space);
    let head = TRANSMIT_DESCRIPTOR_BASE_HEAD.read(memory_space);
    println!("TRANSMIT_DESCRIPTOR_BASE_HEAD before: {:?}", head);

    let current_descriptor = &mut TRANSMISSION_DESCRIPTOR_LIST[tail as usize];

    assert!(current_descriptor
        .status
        .contains(TransmissionStatusRegister::DESCRIPTOR_DONE));

    assert!(data.len() <= u16::MAX as usize);

    current_descriptor.status.remove(TransmissionStatusRegister::DESCRIPTOR_DONE);
    current_descriptor.base_address = (data as *const [u8]).addr() as u64;
    current_descriptor.length = data.len() as u16;
    current_descriptor
        .command
        .insert(TransmissionCommandRegister::END_OF_PACKET);

    TRANSMIT_DESCRIPTOR_BASE_TAIL.write(
        memory_space,
        (tail + 1) % TRANSMISSION_DESCRIPTOR_LIST_SIZE as u64,
    );

    println!(
        "TRANSMIT_DESCRIPTOR_BASE_HEAD after: {:?}",
        TRANSMIT_DESCRIPTOR_BASE_HEAD.read(memory_space)
    )
}
