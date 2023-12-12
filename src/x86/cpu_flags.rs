use core::{arch::asm, fmt::Debug};

use modular_bitfield::{
    bitfield,
    specifiers::{B1, B8},
};

use super::PrivilegeLevel;

// https://en.wikipedia.org/wiki/FLAGS_register
#[bitfield]
#[derive(Clone, Copy)]
#[repr(C, packed)]
pub struct CpuFlags {
    pub carry: bool,
    #[skip]
    __: B1,
    pub parity: bool,
    #[skip]
    __: B1,
    adjust: bool,
    #[skip]
    __: B1,
    pub zero: bool,
    pub sign: bool,
    pub trap: bool,
    pub interrupt_enabled: bool,
    pub direction: bool,
    pub overflow: bool,
    pub io_privilege_level: PrivilegeLevel,
    pub nested_task: bool,
    pub mode: bool,
    pub resume: bool,
    pub virtual_8086_mode: bool,
    pub smap_access_check: bool,
    pub virtual_interrupt: bool,
    pub virtual_interrupt_pending: bool,
    pub cpuid_usable: bool,
    #[skip]
    __: B8,
    pub aes_key_schedule_loaded: bool,
    #[skip]
    __: B1,
}

pub unsafe fn get_cpu_flags() -> CpuFlags {
    let mut output: u32;

    asm!(
        "PUSHFD",
        "POP {output}",

        output = out(reg) output
    );

    CpuFlags::from_bytes(output.to_ne_bytes())
}

pub unsafe fn set_cpu_flags(cpu_flags: CpuFlags) {
    let input: u32 = u32::from_ne_bytes(cpu_flags.into_bytes());

    asm!(
        "PUSH {input}",
        "POPFD",

        input = in(reg) input
    );
}

pub unsafe fn set_specific_cpu_flags(cpu_flags: CpuFlags) {
    let input: u32 = u32::from_ne_bytes(cpu_flags.into_bytes());

    asm!(
        "PUSHFD",
        "POP EAX",
        "OR EAX, {input}",
        "PUSH EAX",
        "POPFD",

        input = in(reg) input
    );
}

pub unsafe fn unset_specific_cpu_flags(cpu_flags: CpuFlags) {
    let input: u32 = !u32::from_ne_bytes(cpu_flags.into_bytes());

    asm!(
        "PUSHFD",
        "POP EAX",
        "AND EAX, {input}",
        "PUSH EAX",
        "POPFD",

        input = in(reg) input
    );
}

impl Debug for CpuFlags {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("CpuFlags")
            .field("value", &u32::from_ne_bytes(self.bytes))
            .finish()
    }
}
