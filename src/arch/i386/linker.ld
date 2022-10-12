ENTRY(start)

SECTIONS {
    . = 1M;

    .boot ALIGN(4K) : 
    {
        KEEP(*(.multiboot_header))
    }

    .text ALIGN(4K) : 
    {
        *(.text .text.*)
    }
    .data : ALIGN(4K) {
        *(.data)
    }
    .rodata : ALIGN(4K) {
        *(.rodata .rodata.*)
    }

    .data.rel.ro : ALIGN(4K) {
        *(.data.rel.ro.local*) *(.data.rel.ro .data.rel.ro.*)
    }

    .bss : ALIGN(4K) {
        *(.bss)
    }

}