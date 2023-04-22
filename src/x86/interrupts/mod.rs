use core::arch::asm;

use modular_bitfield::{
    bitfield,
    specifiers::{B1, B8},
};

use crate::{print, println, x86::interrupts::pic_8259::PIC};

use self::pic_8259::MASTER_INTERRUPT_OFFSET;

use super::{cpu_flags::CpuFlags, PrivilegeLevel};

pub mod idt;
pub mod pic_8259;
pub mod handlers;

#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub enum PciInterruptIndex {
    Timer = MASTER_INTERRUPT_OFFSET,
    Keyboard = MASTER_INTERRUPT_OFFSET + 1,
}

#[repr(C, packed)]
#[derive(Clone, Copy, Debug)]
pub struct InterruptStackFrame {
    instruction_pointer: usize,
    code_segment: usize,
    cpu_flags: CpuFlags,
    stack_pointer: usize,
    stack_segment: usize,
}

pub type ExceptionHandler =
    extern "x86-interrupt" fn(interrupt_stack_frame: &mut InterruptStackFrame, error_code: usize);

pub type InterruptHandler =
    extern "x86-interrupt" fn(interrupt_stack_frame: &mut InterruptStackFrame);

pub unsafe fn enable_interrupt() {
    asm!("sti");
}

pub unsafe fn disable_interrupt() {
    asm!("cli");
}
