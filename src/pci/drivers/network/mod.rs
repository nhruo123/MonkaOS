// https://pdos.csail.mit.edu/6.828/2011/readings/hardware/8254x_GBe_SDM.pdf

use core::{arch::asm, mem::size_of};

use thiserror::Error;

use crate::{
    mutex::Mutex,
    pci::{
        config_space::{base_address_register::MemorySpace, BaseAddressRegister, PciConfigSpace},
        drivers::network::registers::{TransmissionCommandRegister, TransmissionStatusRegister},
    },
    println,
};

use self::registers::{
    InterruptMaskRegister, TransmissionControlRegister, TransmissionDescriptor,
    TransmissionIpgRegister, INTERRUPT_MASK, TRANSMIT_CONTROL_REGISTER,
    TRANSMIT_DESCRIPTOR_BASE_HEAD, TRANSMIT_DESCRIPTOR_BASE_HIGH, TRANSMIT_DESCRIPTOR_BASE_LEN,
    TRANSMIT_DESCRIPTOR_BASE_LOW, TRANSMIT_DESCRIPTOR_BASE_TAIL, TRANSMIT_IPG_REGISTER,
};

use super::{DriverError, PciDriver};

mod registers;

const MAX_TRANSMIT_LENGTH: usize = 16288;

#[derive(Error, Debug)]
pub enum NetworkError {
    #[error("Ring Transmission Queue is full")]
    FullTransmissionsQueue,
    #[error("Send buffer is too large")]
    BufferTooLarge,
}

pub type Result<T> = core::result::Result<T, NetworkError>;

pub static NETWORK_DRIVER: Mutex<Option<E1000Driver>> = Mutex::new(None);

pub struct E1000Driver {
    mmio_space: MemorySpace,
}

pub const E1000_DRIVER_ENTRY: PciDriver = PciDriver {
    vendor_id: 0x8086,
    device_id: 0x100E,
    init_device: init_e1000,
};

const TRANSMISSION_DESCRIPTOR_LIST_SIZE: usize = 1 << 8;

static mut TRANSMISSION_DESCRIPTOR_LIST: [TransmissionDescriptor;
    TRANSMISSION_DESCRIPTOR_LIST_SIZE] =
    [TransmissionDescriptor::empty(); TRANSMISSION_DESCRIPTOR_LIST_SIZE];

pub fn init_e1000(pci: &mut PciConfigSpace) -> core::result::Result<(), DriverError> {
    println!("Found e1000, initializing network card..., get_interrupt_line: {}, get_interrupt_pin: {}", pci.get_interrupt_line(), pci.get_interrupt_pin());
    let BaseAddressRegister::MemorySpace(mut memory_space) = pci.base_address_registers[0] else {
        return Err(DriverError::UnexpectedBaseRegisterLayout {
            register: pci.base_address_registers[0],
            index: 0,
        });
    };

    unsafe {
        init_descriptors_list(&mut memory_space);
        NETWORK_DRIVER.lock().replace(E1000Driver {
            mmio_space: memory_space,
        });
    };

    Ok(())
}

unsafe fn init_descriptors_list(memory_space: &mut MemorySpace) {
    TRANSMIT_DESCRIPTOR_BASE_LOW.write(
        memory_space,
        TRANSMISSION_DESCRIPTOR_LIST.as_ptr().addr() as u32,
    );
    TRANSMIT_DESCRIPTOR_BASE_HIGH.write(memory_space, 0);

    TRANSMIT_DESCRIPTOR_BASE_LEN.write(
        memory_space,
        (TRANSMISSION_DESCRIPTOR_LIST.len() * size_of::<TransmissionDescriptor>()) as u64,
    );

    TRANSMIT_DESCRIPTOR_BASE_HEAD.write(memory_space, 0);
    TRANSMIT_DESCRIPTOR_BASE_TAIL.write(memory_space, 0);

    TRANSMIT_CONTROL_REGISTER.write(
        memory_space,
        TransmissionControlRegister::new()
            .with_enabled(true)
            .with_pad_short_packets(true)
            .with_collision_threshold(0xF)
            .with_collision_distance_checked(0x40)
            .unwrap(),
    );

    TRANSMIT_IPG_REGISTER.write(
        memory_space,
        TransmissionIpgRegister::new()
            .with_ipgt(10)
            .with_ipgr1(8)
            .with_ipgr2(6),
    );

    INTERRUPT_MASK.write(
        memory_space,
        InterruptMaskRegister::new().with_transmit_descriptor_written_back(true),
    );
}

impl E1000Driver {
    pub unsafe fn transmit_packet(&mut self, data: &[u8], last_packet: bool) -> Result<()> {
        if data.len() > MAX_TRANSMIT_LENGTH {
            return Err(NetworkError::BufferTooLarge);
        }
        let tail = TRANSMIT_DESCRIPTOR_BASE_TAIL.read(&self.mmio_space);

        let current_descriptor = &mut TRANSMISSION_DESCRIPTOR_LIST[tail as usize];

        if !current_descriptor
            .status
            .contains(TransmissionStatusRegister::DESCRIPTOR_DONE)
        {
            return Err(NetworkError::FullTransmissionsQueue);
        }

        current_descriptor
            .status
            .remove(TransmissionStatusRegister::DESCRIPTOR_DONE);
        current_descriptor.base_address = (data as *const [u8]).addr() as u64;
        current_descriptor.length = data.len() as u16;

        current_descriptor
            .command
            .set(TransmissionCommandRegister::END_OF_PACKET, last_packet);

        TRANSMIT_DESCRIPTOR_BASE_TAIL.write(
            &mut self.mmio_space,
            (tail + 1) % TRANSMISSION_DESCRIPTOR_LIST_SIZE as u64,
        );

        // TODO: IDK why but not printing here breaks transmit_packet.
        // It's not a race condition I looped over 4 million NOPs but still nothing showed up in the network dump
        // ╮(╯ _╰ )╭
        // println!("{}", last_packet);

        Ok(())
    }
}
