use super::{CommandRegister, PciConfigSpace, BASE_ADDRESS_REGISTERS_COUNT};

const BASE_ADDRESS_REGISTERS_TYPE_MASK: u32 = 0b1;
const BASE_ADDRESS_REGISTERS_MEMORY_SPACE_TYPE_MASK: u32 = 0b11 << 1;
const BASE_ADDRESS_REGISTERS_PREFETCHABLE_MASK: u32 = 0b1 << 3;

const BASE_ADDRESS_REGISTERS_IO_SIZE_MASK: u32 = !0b11;
const BASE_ADDRESS_REGISTERS_MEMORY_SIZE_MASK: u32 = !0b1111;

#[derive(Debug)]
pub enum BaseAddressRegister {
    IoSpace {
        start_ptr: *const u8,
        size: usize,
    },
    MemorySpace {
        prefetchable: bool,
        start_ptr: *const u8,
        size: usize,
    },
}

impl PciConfigSpace {
    // we can only set address values so when we read back we will negate them and get the size
    //https://www.ics.uci.edu/~harris/ics216/pci/PCI_22.pdf page 224 for more info
    pub(super) fn init_bar(&mut self) {
        let mut command_register = self.get_command_register();
        command_register.set(CommandRegister::IO_SPACE, false);
        command_register.set(CommandRegister::MEMORY_SPACE, false);
        self.set_command_register(command_register.clone());

        for mut register_index in 0..BASE_ADDRESS_REGISTERS_COUNT {
            let original_register_value = self.get_base_address_register(register_index);

            if original_register_value & BASE_ADDRESS_REGISTERS_TYPE_MASK != 0 {
                self.parse_io_space_bar(register_index, original_register_value);

                // the device requested 0 space no BAR detected
            } else {
                match original_register_value & BASE_ADDRESS_REGISTERS_MEMORY_SPACE_TYPE_MASK >> 1 {
                    0b00 => {
                        self.parse_32bit_mmio_space_bar(register_index, original_register_value);
                    }
                    0b10 => {
                        // this register is twice as long
                        register_index += 1;
                        self.parse_64bit_mmio_space_bar(register_index, original_register_value);
                    }
                    _ => (),
                }
            }

            // clean after config parse function deleted the original_register_value
            self.set_base_address_register(register_index, original_register_value);
        }

        command_register.set(CommandRegister::IO_SPACE, true);
        command_register.set(CommandRegister::MEMORY_SPACE, true);
        self.set_command_register(command_register);
    }

    fn parse_io_space_bar(&mut self, register_index: u8, original_register_value: u32) {
        self.set_base_address_register(register_index, u32::MAX);

        let (size, overflow) = (!(self.get_base_address_register(register_index)
            & BASE_ADDRESS_REGISTERS_IO_SIZE_MASK))
            .overflowing_add(1);

        if overflow {
            return;
        }

        self.base_address_registers
            .push(BaseAddressRegister::IoSpace {
                start_ptr: (original_register_value & BASE_ADDRESS_REGISTERS_IO_SIZE_MASK)
                    as *const u8,
                size: size as usize,
            });
    }

    fn parse_32bit_mmio_space_bar(&mut self, register_index: u8, original_register_value: u32) {
        self.set_base_address_register(register_index, u32::MAX);

        let (size, overflow) = (!(self.get_base_address_register(register_index)
            & BASE_ADDRESS_REGISTERS_MEMORY_SIZE_MASK))
            .overflowing_add(1);

        if overflow {
            return;
        }

        self.base_address_registers
            .push(BaseAddressRegister::MemorySpace {
                prefetchable: original_register_value & BASE_ADDRESS_REGISTERS_PREFETCHABLE_MASK
                    != 0,
                start_ptr: (original_register_value & BASE_ADDRESS_REGISTERS_MEMORY_SIZE_MASK)
                    as *const u8,
                size: size as usize,
            });
    }

    fn parse_64bit_mmio_space_bar(&mut self, register_index: u8, original_register_value: u32) {
        let original_second_register_contents = self.get_base_address_register(register_index + 1);

        self.set_base_address_register(register_index, u32::MAX);
        self.set_base_address_register(register_index + 1, u32::MAX);

        let long_register_value = (self.get_base_address_register(register_index) as u64)
            | ((self.get_base_address_register(register_index + 1) as u64) << 32);

        let (size, overflow) = (!(long_register_value
            & BASE_ADDRESS_REGISTERS_MEMORY_SIZE_MASK as u64))
            .overflowing_add(1);

        self.set_base_address_register(register_index + 1, original_second_register_contents);

        if overflow {
            return;
        }

        self.base_address_registers
            .push(BaseAddressRegister::MemorySpace {
                prefetchable: original_register_value & BASE_ADDRESS_REGISTERS_PREFETCHABLE_MASK
                    != 0,
                start_ptr: (original_register_value & BASE_ADDRESS_REGISTERS_MEMORY_SIZE_MASK)
                    as *const u8,
                size: size as usize,
            });
    }
}
