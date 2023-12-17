#![allow(dead_code, unused_variables)]

use crate::pci::config_space::MemoryMappedRegister;
use bitflags::bitflags;
use modular_bitfield::{
    bitfield,
    specifiers::{B1, B10, B15, B16, B2, B3, B4, B5, B6, B13},
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
pub struct InterruptMaskRegister {
    pub transmit_descriptor_written_back: bool,
    pub transmit_queue_empty: bool,
    pub link_status_change: bool,
    pub rxseq: bool,
    pub rxdmt0: bool,
    #[skip]
    __: B1,
    pub receiver_fifo_overrun: bool,
    pub receiver_timer_interrupt: bool,
    #[skip]
    __: B1,
    pub mdac: bool,
    pub rxcfg: bool,
    #[skip]
    __: B1,
    pub phyint: bool,
    pub gpi: B2,
    pub transmit_descriptor_low_threshold_hit: bool,
    pub srpd: bool,
    #[skip]
    __: B15,
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

#[bitfield]
#[repr(packed, C)]
#[derive(Debug)]
pub struct EepromReadRegister {
    pub start_read: bool,
    #[skip]
    __: B3,
    pub done: bool,
    #[skip]
    __: B3,
    pub read_address: u8,
    pub read_data: u16,
}

#[derive(BitfieldSpecifier, Clone, Copy, Debug)]
#[bits = 2]
pub enum LoopBackMode {
    NoLoopBack = 0b00,
    PhyOrExternal = 0b11,
}

#[derive(BitfieldSpecifier, Clone, Copy, Debug)]
#[bits = 2]
pub enum ReceiveDescriptionMinThreshold {
    Half,
    Quarter,
    Eight,
}

#[derive(BitfieldSpecifier, Clone, Copy, Debug)]
#[bits = 2]
pub enum ReceiveBufferSize {
    Bytes2048,
    Bytes1024,
    Bytes512,
    Bytes256,
}

#[bitfield]
#[repr(packed, C)]
#[derive(Debug)]
pub struct ReceiveControlRegister {
    #[skip]
    __: B1,
    pub enabled: bool,
    pub store_bad_packets: bool,
    pub unicast_promiscuous: bool,
    pub multicast_promiscuous: bool,
    pub long_packet_reception: bool,
    pub loopback_mod: LoopBackMode,
    pub receive_description_min_threshold: ReceiveDescriptionMinThreshold,
    #[skip]
    __: B2,
    // TODO: implement enum
    pub mo: B2,
    #[skip]
    __: B1,
    pub accept_broadcast: bool,
    pub receive_buffer_size: ReceiveBufferSize,
    pub vlan_filter: bool,
    pub cfien: bool,
    pub cfi: bool,
    #[skip]
    __: B1,
    pub discard_pause_frames: bool,
    pub pass_mac_control_frames: bool,
    #[skip]
    __: B1,
    pub buffer_size_extension: bool,
    pub strip_ethernet_crc: bool,
    #[skip]
    __: B5,
}

#[bitfield]
#[repr(packed, C)]
#[derive(Debug)]
pub struct ReceiveDelayTimerRegister {
    // every increment is 1.024 ps
    pub delay_timer: u16,
    #[skip]
    __: B15,
    pub flush_partial_descriptor_block: bool,
}

#[derive(BitfieldSpecifier, Clone, Copy, Debug)]
#[bits = 2]
pub enum AddressSelect {
    DestinationAddress,
    SourceAddress,
}

#[bitfield]
#[repr(packed, C)]
#[derive(Debug)]
pub struct ReceiverAddressHighRegister {
    // every increment is 1.024 ps
    pub receiver_address_high: u16,
    pub address_select: AddressSelect,
    #[skip]
    __: B13,
    pub address_valid: bool,
}

pub const DEVICE_CONTROL: MemoryMappedRegister<DeviceControlRegister> =
    MemoryMappedRegister::new(0x0000);
pub const DEVICE_STATUS: MemoryMappedRegister<DeviceStatusRegister> =
    MemoryMappedRegister::new(0x0008);

pub const EEPROM: MemoryMappedRegister<EepromReadRegister> = MemoryMappedRegister::new(0x00014);

pub const INTERRUPT_MASK: MemoryMappedRegister<InterruptMaskRegister> =
    MemoryMappedRegister::new(0x000D0);

pub const RECEIVE_CONTROL_REGISTER: MemoryMappedRegister<ReceiveControlRegister> =
    MemoryMappedRegister::new(0x00100);

pub const TRANSMIT_CONTROL_REGISTER: MemoryMappedRegister<TransmissionControlRegister> =
    MemoryMappedRegister::new(0x00400);
pub const TRANSMIT_IPG_REGISTER: MemoryMappedRegister<TransmissionIpgRegister> =
    MemoryMappedRegister::new(0x00410);

pub const RECEIVE_DESCRIPTOR_BASE_LOW: MemoryMappedRegister<u32> =
    MemoryMappedRegister::new(0x02800);
pub const RECEIVE_DESCRIPTOR_BASE_HIGH: MemoryMappedRegister<u32> =
    MemoryMappedRegister::new(0x02804);
pub const RECEIVE_DESCRIPTOR_LEN: MemoryMappedRegister<u32> = MemoryMappedRegister::new(0x02808);
pub const RECEIVE_DESCRIPTOR_BASE_HEAD: MemoryMappedRegister<u32> =
    MemoryMappedRegister::new(0x02810);
pub const RECEIVE_DESCRIPTOR_BASE_TAIL: MemoryMappedRegister<u32> =
    MemoryMappedRegister::new(0x02818);
pub const RECEIVE_DELAY_TIMER: MemoryMappedRegister<ReceiveDelayTimerRegister> =
    MemoryMappedRegister::new(0x02820);

pub const TRANSMIT_DESCRIPTOR_BASE_LOW: MemoryMappedRegister<u32> =
    MemoryMappedRegister::new(0x3800);
pub const TRANSMIT_DESCRIPTOR_BASE_HIGH: MemoryMappedRegister<u32> =
    MemoryMappedRegister::new(0x3804);
pub const TRANSMIT_DESCRIPTOR_LEN: MemoryMappedRegister<u32> = MemoryMappedRegister::new(0x3808);
pub const TRANSMIT_DESCRIPTOR_BASE_HEAD: MemoryMappedRegister<u32> =
    MemoryMappedRegister::new(0x3810);
pub const TRANSMIT_DESCRIPTOR_BASE_TAIL: MemoryMappedRegister<u32> =
    MemoryMappedRegister::new(0x3818);

pub const MULTICAST_TABLE_ARRAY: MemoryMappedRegister<[u32; 4]> =
    MemoryMappedRegister::new(0x05400);

pub const RECEIVE_ADDRESS_LOW_0: MemoryMappedRegister<u32> = MemoryMappedRegister::new(0x05400);
pub const RECEIVE_ADDRESS_HIGH_0: MemoryMappedRegister<ReceiverAddressHighRegister> = MemoryMappedRegister::new(0x05404);

pub const EEPROM_ETHERNET_ADDRESS_OFFSET: u8 = 0x0;
