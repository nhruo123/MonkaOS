use modular_bitfield::{
    bitfield,
    specifiers::{B1, B8},
};

use crate::println;

use super::PrivilegeLevel;

pub mod idt;

// https://en.wikipedia.org/wiki/FLAGS_register
#[bitfield]
#[derive(Clone, Copy, Debug)]
#[repr(C, packed)]
pub struct CPUFlags {
    carry: bool,
    _reserved_1: B1,
    parity: bool,
    _reserved_2: B1,
    adjust: bool,
    _reserved_3: B1,
    zero: bool,
    sign: bool,
    trap: bool,
    interrupt_enabled: bool,
    direction: bool,
    overflow: bool,
    io_privilege_level: PrivilegeLevel,
    nested_task: bool,
    mode: bool,
    resume: bool,
    virtual_8086_mode: bool,
    smap_access_check: bool,
    virtual_interrupt: bool,
    virtual_interrupt_pending: bool,
    cpuid_usable: bool,
    _reserved_4: B8,
    aes_key_schedual_loaded: bool,
    _reserved_5: B1,
}

#[repr(C, packed)]
#[derive(Clone, Copy, Debug)]
pub struct InterruptStackFrame {
    instruction_pointer: usize,
    code_segment: usize,
    cpu_flags: CPUFlags,
    stack_pointer: usize,
    stack_segment: usize,
}

pub type ExceptionHandler =
    extern "x86-interrupt" fn(interrupt_stack_frame: &mut InterruptStackFrame, error_code: usize);

pub type InterruptHandler =
    extern "x86-interrupt" fn(interrupt_stack_frame: &mut InterruptStackFrame);

extern "x86-interrupt" fn generic_exception_handler(
    interrupt_stack_frame: &mut InterruptStackFrame,
    error_code: usize,
) {
    println!(
        "EXCEPTION!!! interrupt_stack_frame: ${:#X?}, error_code: {:#X?}",
        interrupt_stack_frame, error_code
    )
}
extern "x86-interrupt" fn generic_interrupt_handler(
    interrupt_stack_frame: &mut InterruptStackFrame,
) {
    println!(
        "INTERRUPT!!! interrupt_stack_frame: ${:#X?}",
        interrupt_stack_frame
    );

    loop {
        
    }
}
