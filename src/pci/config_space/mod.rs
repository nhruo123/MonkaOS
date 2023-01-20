use alloc::vec::Vec;
use bitflags::bitflags;
use modular_bitfield::bitfield;

use crate::{
    impl_access_at_offset,
    x86::io::{io_in_u16, io_out_u32},
};

use base_address_register::BaseAddressRegister;

use super::{PCI_CONFIG_ADDRESS, PCI_CONFIG_DATA, PCI_INVALID_VENDOR};

mod base_address_register;
mod macros;
const DEVICE_INDEX_RANGE: core::ops::Range<u8> = 0..32;

const BUS_SHIFT: usize = 16;
const DEVICE_SHIFT: usize = 11;

const VENDOR_ID_OFFSET: u32 = 0x0;
const DEVICE_ID_OFFSET: u32 = 0x2;
const CONTROL_REGISTER_OFFSET: u32 = 0x4;
const STATUS_REGISTER_OFFSET: u32 = 0x6;
const REVISION_ID_OFFSET: u32 = 0x8;
const CLASS_CODE_OFFSET: u32 = 0x9;
const CACHE_LINE_SIZE_OFFSET: u32 = 0xC;
const LATENCY_TIMER_OFFSET: u32 = 0xD;
const HEADER_TYPE_OFFSET: u32 = 0xE;
const BIST_OFFSET: u32 = 0xF;

pub const BASE_ADDRESS_REGISTER_OFFSET: u32 = 0x10;
pub const BASE_ADDRESS_REGISTER_32_BIT_SIZE: u32 = 0x4;
pub const BASE_ADDRESS_REGISTER_64_BIT_SIZE: u32 = 0x8;
pub const BASE_ADDRESS_REGISTERS_COUNT: u8 = 6;

const EXPANSION_ROM_BASE_ADDRESS_OFFSET: u32 = 0x30;

const INTERRUPT_PIN_OFFSET: u32 = 0x3C;
const INTERRUPT_LINE_OFFSET: u32 = 0x3D;

bitflags! {
    pub struct HeaderType: u8 {
        const GENERAL_DEVICE = 1 << 0;
        const PCI_TO_PCI_BRIDGE = 1 << 1;
        const PCI_TO_CARD_BUS_BRIDGE = 1 << 2;
        const MULTI_FUNCTION_CARD = 1 << 7;
    }
}

bitflags! {
    pub struct CommandRegister: u16 {
        const IO_SPACE = 1 << 0;
        const MEMORY_SPACE = 1 << 1;
        const BUS_MASTER = 1 << 2;
        const SPECIAL_CYCLE = 1 << 3;
        const MEMORY_WRITE_AND_INVALIDATE = 1 << 4;
        const VGA_PALETTE_SNOOP = 1 << 5;
        const STEPPING_CONTROL = 1 << 6;
        const SERR_ENABLE = 1 << 7;
        const FAST_BACK_TO_BACK_ENABLED = 1 << 8;
    }
}

bitflags! {
    pub struct StatusRegister: u16 {
        // RESERVED
        const CAPABILITIES_LIST = 1 << 4;
        const HIGH_66_HZ = 1 << 5;
        // RESERVED
        const FAST_BACK_TO_BACK = 1 << 7;
        const MASTER_DATA_PARITY_ERROR = 1 << 8;
        // DEVSEL TIMING LEFT OUT FOR NOW
        const SIGNALED_TARGET_ABORT = 1 << 11;
        const RECEIVED_TARGET_ABORT = 1 << 12;
        const RECEIVED_MASTER_ABORT = 1 << 13;
        const SIGNALED_SYSTEM_ERROR = 1 << 14;
        const DETECTED_PARITY_ERROR = 1 << 15;
    }
}

impl_access_at_offset!(u8);
impl_access_at_offset!(u16);
impl_access_at_offset!(u32);

pub struct PciConfigSpace {
    bus_index: u8,
    device_index: u8,
    pub base_address_registers: Vec<BaseAddressRegister>,
}

impl PciConfigSpace {
    pub fn new(bus_index: u8, device_index: u8) -> Option<Self> {
        let mut config_space = Self {
            bus_index,
            device_index,
            base_address_registers: Vec::new(),
        };

        if DEVICE_INDEX_RANGE.contains(&device_index)
            && config_space.get_vendor_id() != PCI_INVALID_VENDOR
        {
            config_space.init_bar();
            Some(config_space)
        } else {
            None
        }
    }

    fn get_base_addr(&self) -> u32 {
        0x80_000_000
            | ((self.bus_index as u32) << BUS_SHIFT)
            | ((self.device_index as u32) << DEVICE_SHIFT)
    }

    pub fn get_vendor_id(&self) -> u16 {
        self.read_offset_u16(VENDOR_ID_OFFSET)
    }

    pub fn get_device_id(&self) -> u16 {
        self.read_offset_u16(DEVICE_ID_OFFSET)
    }

    pub fn get_command_register(&self) -> CommandRegister {
        unsafe {
            CommandRegister::from_bits_unchecked(self.read_offset_u16(CONTROL_REGISTER_OFFSET))
        }
    }

    pub fn set_command_register(&self, command_register: CommandRegister) {
        self.write_offset_u16(CONTROL_REGISTER_OFFSET, command_register.bits())
    }

    pub fn get_status_register(&self) -> StatusRegister {
        unsafe { StatusRegister::from_bits_unchecked(self.read_offset_u16(STATUS_REGISTER_OFFSET)) }
    }

    pub fn set_status_register(&self, command_register: StatusRegister) {
        self.write_offset_u16(STATUS_REGISTER_OFFSET, command_register.bits())
    }

    pub fn get_revision_id(&self) -> u8 {
        self.read_offset_u8(REVISION_ID_OFFSET)
    }

    pub fn get_header_type(&self) -> HeaderType {
        unsafe { HeaderType::from_bits_unchecked(self.read_offset_u8(HEADER_TYPE_OFFSET)) }
    }

    pub fn get_interrupt_line(&self) -> u8 {
        self.read_offset_u8(INTERRUPT_LINE_OFFSET)
    }

    pub fn set_interrupt_line(&self, value: u8) {
        self.write_offset_u8(INTERRUPT_LINE_OFFSET, value)
    }

    pub fn get_interrupt_pin(&self) -> u8 {
        self.read_offset_u8(INTERRUPT_PIN_OFFSET)
    }

    pub fn get_base_address_register(&self, index: u8) -> u32 {
        debug_assert!(index < BASE_ADDRESS_REGISTERS_COUNT);

        self.read_offset_u32(
            BASE_ADDRESS_REGISTER_OFFSET + index as u32 * BASE_ADDRESS_REGISTER_32_BIT_SIZE,
        )
    }

    pub fn set_base_address_register(&self, index: u8, value: u32) {
        debug_assert!(index < BASE_ADDRESS_REGISTERS_COUNT);

        self.write_offset_u32(
            BASE_ADDRESS_REGISTER_OFFSET + index as u32 * BASE_ADDRESS_REGISTER_32_BIT_SIZE,
            value,
        );
    }

    pub fn get_expansion_rom_register(&self) -> u32 {
        self.read_offset_u32(EXPANSION_ROM_BASE_ADDRESS_OFFSET)
    }

    pub fn set_expansion_rom_register(&self, value: u32) {
        self.write_offset_u32(EXPANSION_ROM_BASE_ADDRESS_OFFSET, value);
    }
}

impl core::fmt::Debug for PciConfigSpace {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("PciConfigSpace")
            .field("bus_index", &self.bus_index)
            .field("device_index", &self.device_index)
            .field("vendor_id", &self.get_vendor_id())
            .field("device_id", &self.get_device_id())
            .field("base_address_registers", &self.base_address_registers)
            .finish()
    }
}
