/*
   read here for more info:
       https://www.ics.uci.edu/~harris/ics216/pci/PCI_22.pdf
       http://www.csit-sun.pub.ro/~cpop/Documentatie_SMP/Intel_Microprocessor_Systems/Intel_ProcessorNew/Intel%20White%20Paper/Accessing%20PCI%20Express%20Configuration%20Registers%20Using%20Intel%20Chipsets.pdf
       https://en.wikipedia.org/wiki/PCI_configuration_space
*/

use alloc::vec::Vec;

use self::config_space::{PciConfigSpace, DEVICE_INDEX_RANGE};

pub mod config_space;
pub mod drivers;

const PCI_CONFIG_ADDRESS: u16 = 0xCF8;
const PCI_CONFIG_DATA: u16 = 0xCFC;

const PCI_INVALID_VENDOR: u16 = u16::MAX;
const PCI_BUS_INDEX_RANGE: core::ops::RangeInclusive<u8> = 0..=225;

pub fn check_pci_buses() -> Vec<PciConfigSpace> {
    let mut result = Vec::new();
    for bus_index in PCI_BUS_INDEX_RANGE {
        for device_index in DEVICE_INDEX_RANGE {
            if let Some(config) = PciConfigSpace::new(bus_index, device_index)
                .filter(|config| config.get_vendor_id() != PCI_INVALID_VENDOR)
            {
                result.push(config);
            }
        }
    }

    result
}
