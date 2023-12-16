use alloc::boxed::Box;
use thiserror::Error;

use self::network::E1000_DRIVER_ENTRY;

use super::config_space::{PciConfigSpace, BaseAddressRegister};

pub mod network;


#[derive(Error, Debug)]
pub enum DriverError {
    #[error(transparent)]
    InitializationError(#[from] Box<dyn core::error::Error>),
    #[error("Unexpected Base Register Layout found at index {index}, register: {register:?}")]
    UnexpectedBaseRegisterLayout{register: BaseAddressRegister, index: usize},
}


pub type PciInitFunction = fn(&mut PciConfigSpace) -> Result<(), DriverError>;

pub struct PciDriver {
    pub device_id: u16,
    pub vendor_id: u16,
    pub init_device: PciInitFunction,
}

pub static PCI_DRIVERS: &'static [PciDriver] = &[E1000_DRIVER_ENTRY];
