global inner_load_gdt

inner_load_gdt:
    mov eax, [esp + 4]
    lgdt [eax]

    mov eax, [esp + 8]
    push eax
    push .reload_gdt
    retf

    .reload_gdt:

    mov ax, [esp + 12]
    mov ss, ax
    mov ds, ax
    mov es, ax
    mov fs, ax
    mov gs, ax
    
    ret