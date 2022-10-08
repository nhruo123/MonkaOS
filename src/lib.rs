#![crate_type="staticlib"]
#![feature(lang_items)]
#![no_std]
#![no_builtins]

use core::panic::PanicInfo;

use vga_buffer::{Writer, VgaColor, Color, Buffer};
mod vga_buffer;

#[no_mangle]
pub extern "C" fn _start() -> ! {

    println!("Hello world{}, this is 1+2*3/4 = {}", "!", 1+2*3/4);

    loop {}
}

#[panic_handler]
#[inline(never)]
#[no_mangle]
fn panic(info: &PanicInfo) -> ! {
    println!("{}", info);
    loop {}
}
