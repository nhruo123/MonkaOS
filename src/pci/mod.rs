/*
   read here for more info:
       https://www.ics.uci.edu/~harris/ics216/pci/PCI_22.pdf
       http://www.csit-sun.pub.ro/~cpop/Documentatie_SMP/Intel_Microprocessor_Systems/Intel_ProcessorNew/Intel%20White%20Paper/Accessing%20PCI%20Express%20Configuration%20Registers%20Using%20Intel%20Chipsets.pdf
       https://en.wikipedia.org/wiki/PCI_configuration_space
*/

use alloc::vec::Vec;
use modular_bitfield::{
    bitfield,
    specifiers::{B2, B5, B6},
};

use crate::{
    println,
    x86::io::{io_in_u16, io_in_u32, io_out_u32},
};

use self::config_space::PciConfigSpace;

pub mod config_space;

const PCI_CONFIG_ADDRESS: u16 = 0xCF8;
const PCI_CONFIG_DATA: u16 = 0xCFC;

const PCI_INVALID_VENDOR: u16 = u16::MAX;

pub fn check_pci_buses() -> Vec<PciConfigSpace> {
    let mut result = Vec::new();
    for bus_index in 0..=255 {
        for device_index in 0..32 {
            if let Some(config) = PciConfigSpace::new(bus_index, device_index)
                .filter(|config| config.get_vendor_id() != PCI_INVALID_VENDOR)
            {
                result.push(config);
            }
        }
    }

    result
}
