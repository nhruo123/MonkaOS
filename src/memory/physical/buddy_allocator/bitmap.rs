pub struct BitMap {
    inner_array: *mut u8,
    byte_count: usize,
}

impl BitMap {
    pub fn new(base_address: usize, byte_count: usize) -> Self {
        let base_address = base_address as *mut u8;

        unsafe {
            for byte_index in 0..byte_count {
                *base_address.add(byte_index) = 0;
            }
        }

        Self {
            inner_array: base_address,
            byte_count,
        }
    }

    pub fn get_bit(&self, index: usize) -> Option<bool> {
        let byte_index = Self::get_byte_index(index);
        let bit_mask: u8 = Self::get_bit_mask(index);

        if byte_index < self.byte_count {
            unsafe { Some((*self.inner_array.add(byte_index) & bit_mask) != 0) }
        } else {
            None
        }
    }

    pub fn set_bit(&mut self, index: usize, value: bool) -> Option<()> {
        let byte_index = Self::get_byte_index(index);
        let bit_mask: u8 = Self::get_bit_mask(index);

        if byte_index < self.byte_count {
            unsafe {
                if value {
                    (*self.inner_array.add(byte_index)) |= bit_mask;
                } else {
                    (*self.inner_array.add(byte_index)) &= !bit_mask;
                };
            }

            Some(())
        } else {
            None
        }
    }

    fn get_byte_index(index: usize) -> usize {
        index / 8
    }

    fn get_bit_mask(index: usize) -> u8 {
        1 << (index % 8)
    }
}
