use crate::{
    pci::drivers::network::NETWORK_DRIVER,
    println,
    x86::interrupts::{pic_8259::PIC, InterruptStackFrame},
};

pub extern "x86-interrupt" fn generic_e1000_interrupt(
    _interrupt_stack_frame: &mut InterruptStackFrame,
) {
    println!("Received e1000 interrupt");

    unsafe {
        PIC.lock().notify_end_of_interrupt(
            32 + NETWORK_DRIVER
                .bypass()
                .as_ref()
                .unwrap()
                .pci_config_space
                .get_interrupt_line(),
        );
    }
}
