#![feature(lang_items)]
#![no_std]
#![no_main]

use core::panic::PanicInfo;
mod vga_buffer;

#[no_mangle]
pub extern "C" fn _start() -> ! {
    println!(
        "Hello World{}, this is `4/3 + 4 * 8` = {}",
        "!",
        4 / 3 + 4 * 8
    );

    loop {}
}

#[panic_handler]
#[no_mangle]
fn panic(info: &PanicInfo) -> ! {
    println!("{}", info);
    loop {}
}
