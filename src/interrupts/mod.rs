bitflags! {
    struct IDTEntryFlags: u8 {
        const GATE_TYPE         = 0b00001111;
        const ZERO              = 0b00010000;
        const PRIVILEGE_LEVEL   = 0b01100000;
        const PRESENT           = 0b10000000;
    }
}

#[repr(C)]
struct IDtEntry {
    lower_half_offset: u16,
    segment_selector: u16,
    zero: u8,
    flags: IDTEntryFlags,
    higher_half_offset: u16,
}
