use modular_bitfield::BitfieldSpecifier;

pub mod gdt;
pub mod interrupts;


#[derive(BitfieldSpecifier)]
#[derive(Clone, Copy, Debug)]
#[bits = 2]
pub enum PrivilegeLevel {
    RingZero = 0,
    RingOne = 1,
    RingTwo = 2,
    RingThree = 3,
}

#[derive(Clone, Copy, Default, Debug)]
#[repr(C, packed)]
pub struct TableDescriptor {
    size: u16,
    offset: u32,
}