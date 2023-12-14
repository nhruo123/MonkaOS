// https://pdos.csail.mit.edu/6.828/2011/readings/hardware/8254x_GBe_SDM.pdf

use crate::{pci::config_space::PciConfigSpace, println};

use super::PciDriver;

pub const E1000_DRIVER_ENTRY: PciDriver = PciDriver {
    vendor_id: 0x8086,
    device_id: 0x100E,
    init_device: init_e1000,
};

pub fn init_e1000(pci: &mut PciConfigSpace) {
    println!("I found e1000! yay!");
}
