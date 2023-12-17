#![allow(dead_code)]

use core::{
    marker::PhantomData,
    mem::size_of,
    ptr::{read_volatile, write_volatile},
};

use bitflags::bitflags;

use crate::{
    impl_access_at_offset, print,
    x86::io::{io_in_u32, io_out_u16, io_out_u32, io_out_u8},
};

pub use base_address_register::BaseAddressRegister;

use self::base_address_register::MemorySpace;

use super::{PCI_CONFIG_ADDRESS, PCI_CONFIG_DATA, PCI_INVALID_VENDOR};

pub mod base_address_register;
pub mod macros;
pub const DEVICE_INDEX_RANGE: core::ops::Range<u8> = 0..32;

const BUS_SHIFT: usize = 16;
const DEVICE_SHIFT: usize = 11;
const VENDOR_ID_OFFSET: u32 = 0x0;

const DEVICE_ID_OFFSET: u32 = 0x2;
const CONTROL_REGISTER_OFFSET: u32 = 0x4;
const STATUS_REGISTER_OFFSET: u32 = 0x6;
const REVISION_ID_OFFSET: u32 = 0x8;
const CLASS_OFFSET: u32 = 0xB;
const CACHE_LINE_SIZE_OFFSET: u32 = 0xC;
const LATENCY_TIMER_OFFSET: u32 = 0xD;
const HEADER_TYPE_OFFSET: u32 = 0xE;
const BIST_OFFSET: u32 = 0xF;

pub const BASE_ADDRESS_REGISTER_OFFSET: u32 = 0x10;
pub const BASE_ADDRESS_REGISTER_32_BIT_SIZE: u32 = 0x4;
pub const BASE_ADDRESS_REGISTER_64_BIT_SIZE: u32 = 0x8;
pub const BASE_ADDRESS_REGISTERS_COUNT: usize = 6;

const EXPANSION_ROM_BASE_ADDRESS_OFFSET: u32 = 0x30;

const INTERRUPT_LINE_OFFSET: u32 = 0x3C;
const INTERRUPT_PIN_OFFSET: u32 = 0x3D;

pub struct MemoryMappedRegister<T> {
    offset: usize,
    _pin: PhantomData<T>,
}

impl<T> MemoryMappedRegister<T> {
    pub const fn new(offset: usize) -> Self {
        Self {
            offset,
            _pin: PhantomData,
        }
    }

    pub unsafe fn read(&self, memory_mapped_space: &MemorySpace) -> T {
        assert!(self.offset + size_of::<T>() <= memory_mapped_space.size);
        read_volatile((memory_mapped_space.start_ptr.addr() + self.offset) as *const T)
    }

    pub unsafe fn write(&self, memory_mapped_space: &mut MemorySpace, item: T) {
        assert!(self.offset + size_of::<T>() <= memory_mapped_space.size);
        write_volatile(
            (memory_mapped_space.start_ptr.addr() + self.offset) as *mut T,
            item,
        );
    }
}

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

#[derive(Debug, Clone, Copy)]
pub enum ClassCode {
    TooOld,
    MassStorageController,
    NetworkController,
    DisplayController,
    MultimediaDevice,
    MemoryController,
    BridgeDevice,
    SimpleCommunicationControllers,
    BaseSystemPeripherals,
    InputDevices,
    DockingStations,
    PROCESSORS,
    SerialBusControllers,
    WirelessController,
    IntelligentIoControllers,
    SatelliteCommunicationControllers,
    EncryptionDecryptionControllers,
    DataAcquisitionAndSignalProcessing,
    RESERVED(u8),
    UNFIT,
}

impl From<u8> for ClassCode {
    fn from(value: u8) -> Self {
        match value {
            0x0 => ClassCode::TooOld,
            0x1 => ClassCode::MassStorageController,
            0x2 => ClassCode::NetworkController,
            0x3 => ClassCode::DisplayController,
            0x4 => ClassCode::MultimediaDevice,
            0x5 => ClassCode::MemoryController,
            0x6 => ClassCode::BridgeDevice,
            0x7 => ClassCode::SimpleCommunicationControllers,
            0x8 => ClassCode::BaseSystemPeripherals,
            0x9 => ClassCode::InputDevices,
            0xA => ClassCode::DockingStations,
            0xB => ClassCode::PROCESSORS,
            0xC => ClassCode::SerialBusControllers,
            0xD => ClassCode::WirelessController,
            0xE => ClassCode::IntelligentIoControllers,
            0xF => ClassCode::SatelliteCommunicationControllers,
            0x10 => ClassCode::EncryptionDecryptionControllers,
            0x11 => ClassCode::DataAcquisitionAndSignalProcessing,
            0xFF => ClassCode::UNFIT,
            v => ClassCode::RESERVED(v),
        }
    }
}

#[derive(Copy, Clone)]
pub struct PciConfigSpace {
    bus_index: u8,
    device_index: u8,
    pub vendor_id: u16,
    pub device_id: u16,
    pub base_address_registers: [BaseAddressRegister; BASE_ADDRESS_REGISTERS_COUNT],
}

impl PciConfigSpace {
    fn read_offset_u8(&self, offset: u32) -> u8 {
        let aligned_offset = offset & (!0b11);
        (self.read_offset_u32(aligned_offset) >> (8 * (offset & 0b11))) as u8
    }

    fn read_offset_u16(&self, offset: u32) -> u16 {
        debug_assert!(offset & 0b1 == 0);

        let aligned_offset: u32 = offset & (!0b11);
        (self.read_offset_u32(aligned_offset) >> (8 * (offset & 0b11))) as u16
    }

    fn read_offset_u32(&self, offset: u32) -> u32 {
        debug_assert!(offset & 0b11 == 0);

        unsafe {
            io_out_u32(PCI_CONFIG_ADDRESS, self.get_base_addr() + offset);
            io_in_u32(PCI_CONFIG_DATA)
        }
    }

    fn write_offset_u8(&self, offset: u32, value: u8) {
        let aligned_offset: u32 = offset & (!0b11);

        let masked_orig_value = self.read_offset_u32(offset) & !(0xFF << 8 * (offset & 0b11));
        let shifted_value = (value as u32) << 8 * (offset & 0b11);

        self.write_offset_u32(aligned_offset, masked_orig_value | shifted_value);
    }

    fn write_offset_u16(&self, offset: u32, value: u16) {
        debug_assert!(offset & 0b1 == 0);
        let aligned_offset: u32 = offset & (!0b11);

        let masked_orig_value = self.read_offset_u32(offset) & !(0xFFFF << 8 * (offset & 0b11));
        let shifted_value = (value as u32) << 8 * (offset & 0b11);

        self.write_offset_u32(aligned_offset, masked_orig_value | shifted_value);
    }

    fn write_offset_u32(&self, offset: u32, value: u32) {
        debug_assert!(offset & 0b11 == 0);

        unsafe {
            crate::x86::io::io_out_u32(PCI_CONFIG_ADDRESS, self.get_base_addr() + offset);
            io_out_u32(PCI_CONFIG_DATA, value);
        }
    }

    pub fn new(bus_index: u8, device_index: u8) -> Option<Self> {
        let mut config_space = Self {
            bus_index,
            device_index,
            device_id: 0,
            vendor_id: PCI_INVALID_VENDOR,
            base_address_registers: [BaseAddressRegister::EmptyEntry; BASE_ADDRESS_REGISTERS_COUNT],
        };

        let vendor_id = config_space.get_vendor_id();

        if !DEVICE_INDEX_RANGE.contains(&device_index) || vendor_id == PCI_INVALID_VENDOR {
            return None;
        }

        config_space.vendor_id = vendor_id;
        config_space.device_id = config_space.get_device_id();

        config_space.init_bar();

        let mut new_config_space = config_space.get_command_register();
        new_config_space.set(CommandRegister::BUS_MASTER, true);
        config_space.set_command_register(new_config_space);

        Some(config_space)
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

    pub fn get_base_address_register(&self, index: usize) -> u32 {
        debug_assert!(index < BASE_ADDRESS_REGISTERS_COUNT);

        self.read_offset_u32(
            BASE_ADDRESS_REGISTER_OFFSET + index as u32 * BASE_ADDRESS_REGISTER_32_BIT_SIZE,
        )
    }

    pub fn set_base_address_register(&self, index: usize, value: u32) {
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

    pub fn get_class_code(&self) -> ClassCode {
        self.read_offset_u8(CLASS_OFFSET).into()
    }
}

impl core::fmt::Debug for PciConfigSpace {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("PciConfigSpace")
            .field("bus_index", &self.bus_index)
            .field("device_index", &self.device_index)
            .field("vendor_id", &self.vendor_id)
            .field("device_id", &self.device_id)
            .field("base_address_registers", &self.base_address_registers)
            .finish()
    }
}
