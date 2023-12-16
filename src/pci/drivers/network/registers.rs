#![allow(dead_code, unused_variables)]

use crate::pci::config_space::MemoryMappedRegister;
use bitflags::bitflags;
use modular_bitfield::{
    bitfield,
    specifiers::{B1, B10, B16, B2, B5, B6},
    BitfieldSpecifier,
};

#[derive(BitfieldSpecifier, Clone, Copy, Debug)]
#[bits = 2]
pub enum SpeedSelection {
    TenMbs,
    HundredMbs,
    ThousandMbs,
}

#[bitfield]
#[repr(packed, C)]
#[derive(Debug)]
pub struct DeviceControlRegister {
    pub full_duplex: bool,
    #[skip]
    __: B2,
    pub link_reset: B1,
    #[skip]
    __: B1,
    pub auto_speed_detection_enabled: B1,
    pub set_link_up: B1,
    pub ilos: B1,
    #[bits = 2]
    pub speed: SpeedSelection,
    #[skip]
    __: B1,
    pub force_speed: bool,
    pub force_duplex: bool,
    #[skip]
    __: B5,
    pub sdp0_data: B1,
    pub sdp1_data: B1,
    pub advd3wuc: B1,
    pub en_phy_pwr_mgmt: B1,
    pub sdp0_iodir: B1,
    pub sdp1_iodir: B1,
    #[skip]
    __: B2,
    pub device_reset: bool,
    pub rfce: bool,
    pub tfce: bool,
    #[skip]
    __: B1,
    pub vme: bool,
    pub phy_rst: bool,
}

#[derive(BitfieldSpecifier, Clone, Copy, Debug)]
#[bits = 2]
pub enum FunctionID {
    LanA,
    LanB,
}

#[derive(BitfieldSpecifier, Clone, Copy, Debug)]
#[bits = 2]
pub enum PciXBusSpeed {
    LOW,
    MID,
    HIGH,
}

#[bitfield]
#[repr(packed, C)]
#[derive(Debug)]
pub struct DeviceStatusRegister {
    pub full_duplex: bool,
    pub link_up_indication: bool,
    pub function_id: FunctionID,
    pub transmission_paused: bool,
    pub tbimode: bool,
    pub speed: SpeedSelection,
    pub asdv: SpeedSelection,
    #[skip]
    __: B1,
    pub pci66: B1,
    pub bus64: B1,
    pub pcix_mode1: B1,
    #[bits = 2]
    pub pcixspd1: PciXBusSpeed,
    #[skip]
    __: B16,
}

#[bitfield]
#[repr(packed, C)]
#[derive(Debug)]
pub struct TransmissionControlRegister {
    #[skip]
    __: B1,
    pub enabled: bool,
    #[skip]
    __: B1,
    pub pad_short_packets: bool,
    pub collision_threshold: u8,
    pub collision_distance: B10,
    pub swxoff: bool,
    #[skip]
    __: B1,
    pub rtlc: bool,
    pub nrtu: bool,
    #[skip]
    __: B6,
}

#[bitfield]
#[repr(packed, C)]
#[derive(Debug)]
pub struct TransmissionIpgRegister {
    pub ipgt: B10,
    pub ipgr1: B10,
    pub ipgr2: B10,
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
pub struct TransmissionDescriptor {
    pub base_address: u64,
    pub length: u16,
    pub cso: u8,
    pub command: TransmissionCommandRegister,
    pub status: TransmissionStatusRegister,
    pub css: u8,
    pub special: u16,
}

impl TransmissionDescriptor {
    pub const fn empty() -> Self {
        Self {
            base_address: 0,
            command: TransmissionCommandRegister::REPORT_STATUS,
            cso: 0,
            length: 0,
            css: 0,
            special: 0,
            status: TransmissionStatusRegister::DESCRIPTOR_DONE,
        }
    }
}

pub const DEVICE_CONTROL: MemoryMappedRegister<DeviceControlRegister> =
    MemoryMappedRegister::new(0x0000);
pub const DEVICE_STATUS: MemoryMappedRegister<DeviceStatusRegister> =
    MemoryMappedRegister::new(0x0008);

pub const TRANSMIT_CONTROL_REGISTER: MemoryMappedRegister<TransmissionControlRegister> =
    MemoryMappedRegister::new(0x00400);
pub const TRANSMIT_IPG_REGISTER: MemoryMappedRegister<TransmissionIpgRegister> =
    MemoryMappedRegister::new(0x00410);

pub const TRANSMIT_DESCRIPTOR_BASE_LOW: MemoryMappedRegister<u32> =
    MemoryMappedRegister::new(0x3800);
pub const TRANSMIT_DESCRIPTOR_BASE_HIGH: MemoryMappedRegister<u32> =
    MemoryMappedRegister::new(0x3804);
pub const TRANSMIT_DESCRIPTOR_BASE_LEN: MemoryMappedRegister<u64> =
    MemoryMappedRegister::new(0x3808);
pub const TRANSMIT_DESCRIPTOR_BASE_HEAD: MemoryMappedRegister<u64> =
    MemoryMappedRegister::new(0x3810);
pub const TRANSMIT_DESCRIPTOR_BASE_TAIL: MemoryMappedRegister<u64> =
    MemoryMappedRegister::new(0x3818);
