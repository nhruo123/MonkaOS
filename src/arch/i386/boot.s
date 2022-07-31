global start



section .bss
align 4096
page_directory:
    resb 4096
page_table:
    resb 4096
bottom_stack:
    resb 64 ; leave space for stack
top_stack:



section .text
bits 32

panic:    
    mov dword [0xb8000], 0x4f524f45
    mov dword [0xb8004], 0x4f3a4f52
    mov word [0xb8008], 0x4f20
    mov byte  [0xb800a], al
    mov byte [0xb800b], 0x4f
    hlt

check_multiboot:
    ; test if multiboot magic is present 
    cmp eax, 0x36d76289
    jne .no_multiboot
    ret
.no_multiboot:
    mov al, "0"
    jmp panic


set_paging:
    mov eax, page_table
    or eax, 0b11 ; rw+p
    mov [page_directory], eax

    mov ecx, 0

    ; identity map first 4M of memory
    .map_page_table: 
    
    mov eax, 0x1000 ; 4K of memory
    mul ecx
    or eax, 0b11 ; w/r+p
    mov [page_table + ecx * 4], eax
    
    inc ecx
    cmp ecx, 512
    jnz .map_page_table

    ret

enable_paging:
    mov eax, page_directory
    mov cr3, eax

    mov eax, cr0
    or eax, 0x80000001
    mov cr0, eax

    ret

start:
    mov esp, top_stack ; setup stack pointer

    call check_multiboot
    call set_paging
    call enable_paging

    extern _start
    call _start

    mov dword [0xb8000], 0x2f4b2f4f ; print OK
    hlt