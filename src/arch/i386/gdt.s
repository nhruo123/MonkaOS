global inner_load_gdt

inner_load_gdt:
    mov eax, [esp + 4]
    lgdt [eax]

    mov ax, [esp + 8]
    ; TODO: FIX STATIC GDT CODE ENTRY
    jmp 0x8:.reload_gdt

    .reload_gdt:

    mov ax, [esp + 12]
    mov ss, ax
    mov ds, ax
    mov es, ax
    mov fs, ax
    mov gs, ax
    
    ret