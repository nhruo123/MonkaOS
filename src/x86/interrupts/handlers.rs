use crate::{
    print, println,
    x86::{
        interrupts::{pic_8259::PIC, PciInterruptIndex},
        io::io_in_u8,
    },
};

use super::InterruptStackFrame;

#[allow(dead_code)]
pub extern "x86-interrupt" fn generic_exception_handler(
    interrupt_stack_frame: &mut InterruptStackFrame,
    error_code: usize,
) {
    println!(
        "EXCEPTION! interrupt_stack_frame: {:#X?}, error_code: {:#X?}",
        interrupt_stack_frame, error_code
    );
}
pub extern "x86-interrupt" fn generic_interrupt_handler(
    interrupt_stack_frame: &mut InterruptStackFrame,
) {
    println!(
        "INTERRUPT! interrupt_stack_frame: {:#X?}",
        interrupt_stack_frame
    );
}

pub extern "x86-interrupt" fn double_fault_handler(
    interrupt_stack_frame: &mut InterruptStackFrame,
    error_code: usize,
) {
    panic!(
        "DOUBLE FAULT EXCEPTION! \n{:#x?}\nerror_code: {:#x?}",
        interrupt_stack_frame, error_code
    );
}

pub extern "x86-interrupt" fn general_protection_fault_fault_handler(
    interrupt_stack_frame: &mut InterruptStackFrame,
    error_code: usize,
) {
    panic!(
        "GENERAL PROTECTION FAULT EXCEPTION! \n{:#x?}\nerror_code: {:#x?}",
        interrupt_stack_frame, error_code
    );
}

pub extern "x86-interrupt" fn timer_interrupt_handler(
    _interrupt_stack_frame: &mut InterruptStackFrame,
) {
    // print!(".");

    unsafe {
        PIC.lock()
            .notify_end_of_interrupt(PciInterruptIndex::Timer as u8)
    }
}

pub extern "x86-interrupt" fn keyboard_interrupt_handler(
    _interrupt_stack_frame: &mut InterruptStackFrame,
) {
    let scancode: u8 = unsafe { io_in_u8(0x60) };

    print!("{}", scancode);

    unsafe {
        PIC.lock()
            .notify_end_of_interrupt(PciInterruptIndex::Keyboard as u8);
    }
}
