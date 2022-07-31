section .multiboot_header

; Declare constants for the multiboot header.

; 'magic number' lets bootloader find the header
%define MAGIC      0xE85250D6
; this is the Multiboot 'flag' field
%define ARCHITECTURE 0
%define HEADER_LENGTH header_end - header_start
; checksum of above, to prove we are multiboot
%define CHECKSUM  0x100000000 - (0xe85250d6 + 0 + (header_end - header_start))

header_start:
    align 4


    dd MAGIC
    dd 0
    dd HEADER_LENGTH
    dd 0x100000000 - (0xe85250d6 + 0 + (header_end - header_start))

    ; null tag
    dw 0
    dw 0
    dw 8

header_end: