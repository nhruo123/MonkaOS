#[macro_export]
macro_rules! impl_access_at_offset {
    ($t:ty) => {
        ::paste::paste! {

            impl PciConfigSpace {
                fn [<read_offset_ $t>](&self, offset: u32) -> $t {
                    use crate::x86::io::{[<io_in_ $t>]};
                    unsafe {
                        crate::x86::io::io_out_u32(PCI_CONFIG_ADDRESS, self.get_base_addr() + offset);
                        [<io_in_ $t>](PCI_CONFIG_DATA)
                    }
                }

                fn [<write_offset_ $t>](&self, offset: u32, value: $t) {
                    use crate::x86::io::{[<io_out_ $t>]};
                    unsafe {
                        crate::x86::io::io_out_u32(PCI_CONFIG_ADDRESS, self.get_base_addr() + offset);
                        [<io_out_ $t>](PCI_CONFIG_DATA, value)
                    }
                }

            }
        }
    };
}
