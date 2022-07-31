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

}