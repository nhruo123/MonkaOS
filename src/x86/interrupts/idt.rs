use core::{arch::asm, borrow::BorrowMut, marker::PhantomData, mem::size_of, ops::DerefMut};

use lazy_static::lazy_static;
use modular_bitfield::{
    bitfield,
    specifiers::{B1, B2, B4},
};

use crate::{
    mutex::Mutex,
    println,
    x86::{
        gdt::{self, SegmentSelector},
        PrivilegeLevel, TableDescriptor,
    },
};

use super::{generic_interrupt_handler, ExceptionHandler, InterruptHandler};

// this is taken from https://docs.rs/x86_64/latest/src/x86_64/structures/idt.rs.html#10-635
// this seems like a very clean solution to our problem
macro_rules! impl_set_handler_fn {
    ($type:ty) => {
        impl IDTEntry<$type> {
            pub fn set_handler_fn(&mut self, handler: $type) {
                unsafe { self.set_handler_addr(handler as usize) }
            }
        }
    };
}

#[bitfield]
#[derive(Clone, Copy, Default, Debug)]
#[repr(C)]
pub struct IDTEntryFlags {
    gate_type: B4,
    zero: B1,
    privilege_level: PrivilegeLevel,
    present: bool,
}

#[derive(Clone, Copy)]
#[repr(C)]
pub struct IDTEntry<F> {
    lower_half_offset: u16,
    segment_selector: SegmentSelector,
    zero: u8,
    flags: IDTEntryFlags,
    higher_half_offset: u16,
    _phantom: PhantomData<F>,
}

impl<F> IDTEntry<F> {
    fn zeroed() -> Self {
        Self {
            lower_half_offset: 0,
            segment_selector: SegmentSelector::new(),
            zero: 0,
            flags: IDTEntryFlags::new(),
            higher_half_offset: 0,
            _phantom: PhantomData,
        }
    }

    unsafe fn set_handler_addr(&mut self, addr: usize) {
        self.lower_half_offset = addr as u16;
        self.higher_half_offset = (addr >> 16) as u16;

        self.segment_selector
            .set_index(unsafe { gdt::get_cs() } >> 3);

        self.flags.set_present(true);
        // gate type too lazy to create enum for single use
        self.flags.set_gate_type(0b1110);
        self.flags.set_privilege_level(PrivilegeLevel::RingZero);
    }
}

impl_set_handler_fn!(ExceptionHandler);
impl_set_handler_fn!(InterruptHandler);

const IDT_LENGTH: usize = 256;

#[repr(C)]
#[repr(align(16))]
pub struct InterruptDescriptorTable {
    pub division_error: IDTEntry<InterruptHandler>,
    pub debug: IDTEntry<InterruptHandler>,
    pub non_maskable_interrupt: IDTEntry<InterruptHandler>,
    pub breakpoint: IDTEntry<InterruptHandler>,
    pub overflow: IDTEntry<InterruptHandler>,
    pub bound_range_exceeded: IDTEntry<InterruptHandler>,
    pub invalid_opcode: IDTEntry<InterruptHandler>,
    pub device_not_available: IDTEntry<InterruptHandler>,
    pub double_fault: IDTEntry<ExceptionHandler>,
    pub coprocessor_segment_overrun: IDTEntry<InterruptHandler>,
    pub invalid_tss: IDTEntry<ExceptionHandler>,
    pub segment_not_present: IDTEntry<ExceptionHandler>,
    pub stack_segment_fault: IDTEntry<ExceptionHandler>,
    pub general_protection_fault: IDTEntry<ExceptionHandler>,
    pub page_fault: IDTEntry<ExceptionHandler>,
    _reserved_1: IDTEntry<InterruptHandler>,
    pub x87_floating_point_exception: IDTEntry<InterruptHandler>,
    pub alignment_check: IDTEntry<ExceptionHandler>,
    pub machine_check: IDTEntry<InterruptHandler>,
    pub simd_floating_point_exception: IDTEntry<InterruptHandler>,
    pub virtualization_exception: IDTEntry<InterruptHandler>,
    pub control_protection_exception: IDTEntry<ExceptionHandler>,
    _reserved_2: [IDTEntry<InterruptHandler>; 6],
    pub hypervisor_injection_exception: IDTEntry<ExceptionHandler>,
    pub vmm_communication_exception: IDTEntry<ExceptionHandler>,
    pub security_exception: IDTEntry<ExceptionHandler>,
    _reserved_3: IDTEntry<InterruptHandler>,

    interrupts: [IDTEntry<InterruptHandler>; IDT_LENGTH - 32],
}

impl InterruptDescriptorTable {
    pub fn new() -> Self {
        Self {
            division_error: IDTEntry::zeroed(),
            debug: IDTEntry::zeroed(),
            non_maskable_interrupt: IDTEntry::zeroed(),
            breakpoint: IDTEntry::zeroed(),
            overflow: IDTEntry::zeroed(),
            bound_range_exceeded: IDTEntry::zeroed(),
            invalid_opcode: IDTEntry::zeroed(),
            device_not_available: IDTEntry::zeroed(),
            double_fault: IDTEntry::zeroed(),
            coprocessor_segment_overrun: IDTEntry::zeroed(),
            invalid_tss: IDTEntry::zeroed(),
            segment_not_present: IDTEntry::zeroed(),
            stack_segment_fault: IDTEntry::zeroed(),
            general_protection_fault: IDTEntry::zeroed(),
            page_fault: IDTEntry::zeroed(),
            _reserved_1: IDTEntry::zeroed(),
            x87_floating_point_exception: IDTEntry::zeroed(),
            alignment_check: IDTEntry::zeroed(),
            machine_check: IDTEntry::zeroed(),
            simd_floating_point_exception: IDTEntry::zeroed(),
            virtualization_exception: IDTEntry::zeroed(),
            control_protection_exception: IDTEntry::zeroed(),
            _reserved_2: [IDTEntry::zeroed(); 6],
            hypervisor_injection_exception: IDTEntry::zeroed(),
            vmm_communication_exception: IDTEntry::zeroed(),
            _reserved_3: IDTEntry::zeroed(),
            security_exception: IDTEntry::zeroed(),

            interrupts: [IDTEntry::zeroed(); IDT_LENGTH - 32],
        }
    }
}

lazy_static! {
    static ref IDT: InterruptDescriptorTable = {
        let mut idt = InterruptDescriptorTable::new();

        idt.breakpoint.set_handler_fn(generic_interrupt_handler);

        idt
    };
}

pub fn load_idt() {
    debug_assert!(size_of::<IDTEntry<ExceptionHandler>>() == 8);

    debug_assert!(
        IDT_LENGTH * size_of::<IDTEntry<ExceptionHandler>>()
            == size_of::<InterruptDescriptorTable>()
    );

    let idt_descriptor = TableDescriptor {
        offset: (&*IDT as *const _) as u32,
        size: (size_of::<InterruptDescriptorTable>()) as u16 - 1,
    };

    let idt_descriptor_ptr = &idt_descriptor as *const _ as usize;
    unsafe {
        asm!("lidt [{idt_descriptor_ptr}]", idt_descriptor_ptr = in(reg) idt_descriptor_ptr,options(nostack, preserves_flags));
    }
}
