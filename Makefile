arch ?= i386
kernel := build/kernal-$(arch).bin
iso := build/os-$(arch).iso

target ?= i386-unknown-none
rust_os := target/$(target)/debug/libmonkaos_grub.rlib

linker_script := src/arch/$(arch)/linker.ld
grub_cfg := src/arch/$(arch)/grub.cfg
assembly_source_files := $(wildcard src/arch/$(arch)/*.s)
assembly_object_files := $(patsubst src/arch/$(arch)/%.s, build/arch/$(arch)/%.o, $(assembly_source_files))

.PHONY: all clean run iso kernal

all: $(kernel)

clean:
	@rm -r build

run: $(iso)
	@qemu-system-$(arch) -cdrom $(iso)

iso: $(iso)

$(iso): $(kernel) $(grub_cfg)
	@mkdir -p build/isofiles/boot/grub
	@cp $(kernel) build/isofiles/boot/kernel.bin
	@cp $(grub_cfg) build/isofiles/boot/grub
	@grub-mkrescue -o $(iso) build/isofiles 2> /dev/null
	@rm -r build/isofiles

	
$(kernel): kernel $(assembly_object_files) $(linker_script)
	@ld --gc-sections -m elf_$(arch) -n -T $(linker_script) -o $(kernel) $(assembly_object_files) $(rust_os)

kernel:
	@cargo build

build/arch/$(arch)/%.o: src/arch/$(arch)/%.s
	@mkdir -p $(shell dirname $@)
	@nasm -f elf32 $< -o $@