use core::{arch::asm, mem::size_of};

use lazy_static::lazy_static;
// this is x86 legacy magic
// read here for info: https://wiki.osdev.org/GDT
use modular_bitfield::{
    bitfield,
    specifiers::{B1, B13, B2, B4},
};

use super::{PrivilegeLevel, TableDescriptor};

#[bitfield]
#[derive(Clone, Copy, Default, Debug)]
#[repr(C, packed)]
pub struct GDTEntryFlags {
    limit_high: B4,
    reserved: B1,
    long_mode: B1,
    size: B1,
    granularity: B1,
}

#[bitfield]
#[derive(Clone, Copy, Default, Debug)]
#[repr(C, packed)]
struct GDTEntryAccessByte {
    accessed: B1,
    read_write: B1,
    direction: B1,
    executable: B1,
    descriptor_type: B1,
    privilege_level: PrivilegeLevel,
    present: bool,
}

#[bitfield]
#[derive(Clone, Copy, Debug, Default)]
#[repr(C, packed)]
pub struct SegmentSelector {
    pub privilege_level: PrivilegeLevel,
    pub is_local: bool,
    pub index: B13,
}

impl SegmentSelector {
    pub fn get_code_segment_selector() -> Self {
        SegmentSelector::new()
            .with_index(2)
            .with_is_local(false)
            .with_privilege_level(PrivilegeLevel::RingZero)
    }
}

#[derive(Default, Clone, Copy, Debug)]
#[repr(C, packed)]
struct GDTEntry {
    limit_lower: u16,
    base_lower: u16,
    base_middle: u8,
    access_byte: GDTEntryAccessByte,
    flags: GDTEntryFlags,
    base_high: u8,
}

impl GDTEntry {
    pub fn create_segment(
        base_adder: u32,
        limit: u32,
        access_byte: GDTEntryAccessByte,
        mut flags: GDTEntryFlags,
    ) -> Self {
        let base_lower = base_adder as u16;
        let base_middle = (base_adder >> 16) as u8;
        let base_high = (base_adder >> 24) as u8;

        let limit_lower = limit as u16;
        let limit_high = (limit >> 16) as u8;

        flags.set_limit_high(limit_high & 0xF);

        Self {
            limit_lower,
            base_lower,
            base_middle,
            access_byte,
            flags,
            base_high,
        }
    }

    pub fn null_segment() -> Self {
        Self::default()
    }

    pub fn code_segment() -> Self {
        let mut access_byte = GDTEntryAccessByte::new()
            .with_present(true)
            .with_descriptor_type(1)
            .with_executable(1)
            .with_direction(1)
            .with_read_write(1);

        let mut flags = GDTEntryFlags::new()
            .with_granularity(1)
            .with_long_mode(0)
            .with_size(1);

        Self::create_segment(0, u32::MAX, access_byte, flags)
    }

    pub fn data_segment() -> Self {
        let mut access_byte = GDTEntryAccessByte::new()
            .with_present(true)
            .with_descriptor_type(1)
            .with_executable(0)
            .with_direction(0)
            .with_read_write(1);

        let mut flags = GDTEntryFlags::new()
            .with_granularity(1)
            .with_long_mode(0)
            .with_size(1);

        Self::create_segment(0, u32::MAX, access_byte, flags)
    }
}

const GDT_SIZE: usize = 3;

lazy_static! {
    static ref GDT: [GDTEntry; GDT_SIZE] = [
        GDTEntry::null_segment(),
        GDTEntry::code_segment(),
        GDTEntry::data_segment()
    ];
}

extern "C" {
    fn inner_load_gdt(gdt_descriptor_ptr: usize, code_segment: usize, data_segment: usize);
}

pub fn load_gdt() {
    unsafe {
        let gdt_descriptor = TableDescriptor {
            offset: (&*GDT as *const _) as u32,
            size: (GDT.len() * size_of::<GDTEntry>()) as u16 - 1,
        };
        let gdt_descriptor_ptr = &gdt_descriptor as *const _ as usize;
        let code_segment = 1 * size_of::<GDTEntry>();
        let data_segment = 2 * size_of::<GDTEntry>();

        inner_load_gdt(gdt_descriptor_ptr, code_segment, data_segment);
    }
}

pub unsafe fn get_cs() -> u16 {
    let cs: u16;

    asm!("MOV {segment:x}, cs", segment = out(reg) cs,  options(nomem, nostack, preserves_flags));

    return cs;
}
