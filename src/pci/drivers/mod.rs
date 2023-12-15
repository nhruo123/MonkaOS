use self::network::E1000_DRIVER_ENTRY;

use super::config_space::PciConfigSpace;

mod network;

enum DriverError {
       
}

pub type PciInitFunction = fn(&mut PciConfigSpace) -> ();

pub struct PciDriver {
    pub device_id: u16,
    pub vendor_id: u16,
    pub init_device: PciInitFunction,
}


pub static PCI_DRIVERS: &'static [PciDriver] = &[
    E1000_DRIVER_ENTRY
];