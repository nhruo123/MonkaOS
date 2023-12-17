#![allow(dead_code, unused_variables)]

use core::arch::asm;

pub unsafe fn io_out_u32(port: u16, value: u32) {
    asm!("out dx, eax", in("dx") port, in("eax") value);
}

pub unsafe fn io_out_u16(port: u16, value: u16) {
    asm!("out dx, ax", in("dx") port, in("ax") value);
}

pub unsafe fn io_out_u8(port: u16, value: u8) {
    asm!("out dx, al", in("dx") port, in("al") value);
}

pub unsafe fn io_in_u32(port: u16) -> u32 {
    let mut value;

    asm!("in eax, dx", in("dx") port, out("eax") value);

    value
}

pub unsafe fn io_in_u16(port: u16) -> u16 {
    let mut value;

    asm!("in ax, dx", in("dx") port, out("ax") value);

    value
}

pub unsafe fn io_in_u8(port: u16) -> u8 {
    let mut value;

    asm!("in al, dx", in("dx") port, out("al") value);

    value
}
