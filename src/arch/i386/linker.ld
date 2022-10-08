ENTRY(start)

SECTIONS {
    . = 1M;

    .boot ALIGN(4K) : 
    {
        KEEP(*(.multiboot_header))
    }

    .text ALIGN(4K) : 
    {
        *(.text)
    }
    .data : ALIGN(4K) {
        *(.data)
    }

    .bss : ALIGN(4K) {
        *(.bss)
    }

}