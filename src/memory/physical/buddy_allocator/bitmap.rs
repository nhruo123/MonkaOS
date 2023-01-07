pub struct BitMap<'a> {
    inner_array: &'a mut [u8],
}

impl<'a> BitMap<'a> {
    pub fn new(inner_array: &'a mut [u8]) -> Self {
        for byte in inner_array.iter_mut() {
            *byte = 0;
        }

        BitMap { inner_array }
    }

    pub fn get_bit(&self, index: usize) -> Option<bool> {
        let byte_index = Self::get_byte_index(index);
        let bit_mask: u8 = Self::get_bit_mask(index);

        self.inner_array
            .get(byte_index)
            .map(|byte| (byte & bit_mask) != 0)
    }

    pub fn set_bit(&mut self, index: usize, value: bool) -> Option<()> {
        let byte_index = Self::get_byte_index(index);
        let bit_mask: u8 = Self::get_bit_mask(index);

        if let Some(target_byte) = self.inner_array.get_mut(byte_index) {
            if value {
                *target_byte |= bit_mask;
            } else {
                *target_byte &= !bit_mask;
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
