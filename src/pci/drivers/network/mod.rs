// https://pdos.csail.mit.edu/6.828/2011/readings/hardware/8254x_GBe_SDM.pdf

use core::{
    alloc::{GlobalAlloc, Layout},
    arch::asm,
    fmt::Formatter,
    hint,
    mem::size_of,
    ops::DerefMut,
};

use thiserror::Error;

use crate::{
    memory::physical::global_alloc::ALLOCATOR,
    mutex::Mutex,
    network_stack::ethernet::EthernetAddress,
    pci::{
        config_space::{base_address_register::MemorySpace, BaseAddressRegister, PciConfigSpace},
        drivers::network::{
            descriptors::{ReceiveErrorRegister, ReceiveStatusRegister},
            interrupts::generic_e1000_interrupt,
        },
    },
    println,
    x86::interrupts::idt::{load_idt, IDT},
};

use self::{
    descriptors::{
        ReceiveDescriptor, TransmissionCommandRegister, TransmissionDescriptor,
        TransmissionStatusRegister,
    },
    registers::{
        EepromReadRegister, InterruptMaskRegister, LoopBackMode, ReceiveBufferSize,
        ReceiveControlRegister, ReceiveDescriptionMinThreshold, ReceiverAddressHighRegister,
        TransmissionControlRegister, TransmissionIpgRegister, EEPROM,
        EEPROM_ETHERNET_ADDRESS_OFFSET, INTERRUPT_MASK, MULTICAST_TABLE_ARRAY,
        RECEIVE_ADDRESS_HIGH_0, RECEIVE_ADDRESS_LOW_0, RECEIVE_CONTROL_REGISTER,
        RECEIVE_DESCRIPTOR_BASE_HEAD, RECEIVE_DESCRIPTOR_BASE_HIGH, RECEIVE_DESCRIPTOR_BASE_LOW,
        RECEIVE_DESCRIPTOR_BASE_TAIL, RECEIVE_DESCRIPTOR_LEN, TRANSMIT_CONTROL_REGISTER,
        TRANSMIT_DESCRIPTOR_BASE_HEAD, TRANSMIT_DESCRIPTOR_BASE_HIGH, TRANSMIT_DESCRIPTOR_BASE_LOW,
        TRANSMIT_DESCRIPTOR_BASE_TAIL, TRANSMIT_DESCRIPTOR_LEN, TRANSMIT_IPG_REGISTER,
    },
};

use super::{DriverError, PciDriver};

mod descriptors;
mod interrupts;
mod registers;

const MAX_TRANSMIT_LENGTH: usize = 16384;
const MAX_RECEIVE_LENGTH: usize = 16384;

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
    pci_config_space: PciConfigSpace,
    ethernet_address: EthernetAddress,
    mmio_space: MemorySpace,
}

pub const E1000_DRIVER_ENTRY: PciDriver = PciDriver {
    vendor_id: 0x8086,
    device_id: 0x100E,
    init_device: init_e1000,
};

const TRANSMISSION_DESCRIPTOR_LIST_SIZE: usize = 1 << 8;
const RECEIVE_DESCRIPTOR_LIST_SIZE: usize = 1 << 8;

#[repr(C, align(16))]
struct TransmissionDescriptorList {
    transmission_descriptor_list: [TransmissionDescriptor; TRANSMISSION_DESCRIPTOR_LIST_SIZE],
}

static mut TRANSMISSION_DESCRIPTOR_LIST: TransmissionDescriptorList = TransmissionDescriptorList {
    transmission_descriptor_list: [TransmissionDescriptor::empty();
        TRANSMISSION_DESCRIPTOR_LIST_SIZE],
};

#[repr(C, align(16))]
struct ReceiveDescriptorList {
    receive_descriptor_list: [ReceiveDescriptor; RECEIVE_DESCRIPTOR_LIST_SIZE],
}

static mut RECEIVE_DESCRIPTOR_LIST: ReceiveDescriptorList = ReceiveDescriptorList {
    receive_descriptor_list: [ReceiveDescriptor::empty(); RECEIVE_DESCRIPTOR_LIST_SIZE],
};

pub fn init_e1000(pci: &mut PciConfigSpace) -> core::result::Result<(), DriverError> {
    println!(
        "Found E1000 like card (vendor_id:{:#X} , device_id: {:#X}), initializing network card...",
        pci.vendor_id, pci.device_id
    );

    unsafe {
        NETWORK_DRIVER
            .lock()
            .deref_mut()
            .replace(E1000Driver::new(pci)?);
    };

    println!(
        "E1000 initialized!, my ethernet address is: {}",
        NETWORK_DRIVER.lock().as_ref().unwrap().ethernet_address
    );

    Ok(())
}

impl E1000Driver {
    unsafe fn new(pci: &mut PciConfigSpace) -> core::result::Result<Self, DriverError> {
        let BaseAddressRegister::MemorySpace(memory_space) = pci.base_address_registers[0] else {
            return Err(DriverError::UnexpectedBaseRegisterLayout {
                register: pci.base_address_registers[0],
                index: 0,
            });
        };

        let mut new_driver = E1000Driver {
            pci_config_space: pci.clone(),
            mmio_space: memory_space,
            ethernet_address: EthernetAddress { bytes: [0; 6] },
        };

        new_driver.init_transmit();
        new_driver.init_receive();

        Ok(new_driver)
    }

    unsafe fn init_transmit(&mut self) {
        TRANSMIT_DESCRIPTOR_BASE_LOW.write(
            &mut self.mmio_space,
            TRANSMISSION_DESCRIPTOR_LIST
                .transmission_descriptor_list
                .as_ptr()
                .addr() as u32,
        );
        TRANSMIT_DESCRIPTOR_BASE_HIGH.write(&mut self.mmio_space, 0);

        TRANSMIT_DESCRIPTOR_LEN.write(
            &mut self.mmio_space,
            (TRANSMISSION_DESCRIPTOR_LIST
                .transmission_descriptor_list
                .len()
                * size_of::<TransmissionDescriptor>()) as u32,
        );

        TRANSMIT_DESCRIPTOR_BASE_HEAD.write(&mut self.mmio_space, 0);
        TRANSMIT_DESCRIPTOR_BASE_TAIL.write(&mut self.mmio_space, 0);
        TRANSMIT_CONTROL_REGISTER.write(
            &mut self.mmio_space,
            TransmissionControlRegister::new()
                .with_enabled(true)
                .with_pad_short_packets(true)
                .with_collision_threshold(0xF)
                .with_collision_distance_checked(0x40)
                .unwrap(),
        );

        TRANSMIT_IPG_REGISTER.write(
            &mut self.mmio_space,
            TransmissionIpgRegister::new()
                .with_ipgt(10)
                .with_ipgr1(8)
                .with_ipgr2(6),
        );

        // INTERRUPT_MASK.write(
        //     &mut memory_space,
        //     InterruptMaskRegister::new().with_transmit_descriptor_written_back(true),
        // );
        // IDT.lock()[32 + pci.get_interrupt_line()].set_handler_fn(generic_e1000_interrupt);
        // load_idt();
    }

    unsafe fn init_receive(&mut self) {
        for byte_index in 0..self.ethernet_address.bytes.len() / 2 {
            EEPROM.write(
                &mut self.mmio_space,
                EepromReadRegister::new()
                    .with_read_address(EEPROM_ETHERNET_ADDRESS_OFFSET + byte_index as u8)
                    .with_start_read(true),
            );

            while !EEPROM.read(&mut self.mmio_space).done() {
                hint::spin_loop();
            }

            let value = EEPROM.read(&mut self.mmio_space).read_data().to_le_bytes();

            self.ethernet_address.bytes[byte_index * 2] = value[0];
            self.ethernet_address.bytes[byte_index * 2 + 1] = value[1];
        }

        RECEIVE_ADDRESS_LOW_0.write(
            &mut self.mmio_space,
            u32::from_le_bytes(self.ethernet_address.bytes[..4].try_into().unwrap()),
        );

        RECEIVE_ADDRESS_HIGH_0.write(
            &mut self.mmio_space,
            ReceiverAddressHighRegister::new()
                .with_receiver_address_high(u16::from_le_bytes(
                    self.ethernet_address.bytes[4..].try_into().unwrap(),
                ))
                .with_address_valid(true),
        );

        MULTICAST_TABLE_ARRAY.write(&mut self.mmio_space, [0; 4]);

        RECEIVE_DESCRIPTOR_BASE_LOW.write(
            &mut self.mmio_space,
            (&RECEIVE_DESCRIPTOR_LIST.receive_descriptor_list).as_ptr() as u32,
        );

        RECEIVE_DESCRIPTOR_BASE_HIGH.write(&mut self.mmio_space, 0);

        RECEIVE_DESCRIPTOR_LEN.write(
            &mut self.mmio_space,
            (RECEIVE_DESCRIPTOR_LIST.receive_descriptor_list.len() * size_of::<ReceiveDescriptor>())
                as u32,
        );

        RECEIVE_DESCRIPTOR_BASE_HEAD.write(&mut self.mmio_space, 0);
        RECEIVE_DESCRIPTOR_BASE_TAIL.write(
            &mut self.mmio_space,
            (RECEIVE_DESCRIPTOR_LIST.receive_descriptor_list.len() - 1) as u32,
        );

        for descriptor in &mut RECEIVE_DESCRIPTOR_LIST.receive_descriptor_list {
            let new_page = ALLOCATOR
                .alloc(Layout::from_size_align(MAX_RECEIVE_LENGTH, MAX_RECEIVE_LENGTH).unwrap());
            assert!(!new_page.is_null(), "out of memory");
            descriptor.base_address = new_page as u64;
            descriptor.status = ReceiveStatusRegister::empty();
        }

        RECEIVE_CONTROL_REGISTER.write(
            &mut self.mmio_space,
            ReceiveControlRegister::new()
                .with_enabled(true)
                .with_loopback_mod(LoopBackMode::NoLoopBack)
                .with_store_bad_packets(true)
                .with_multicast_promiscuous(true)
                .with_unicast_promiscuous(true)
                .with_receive_description_min_threshold(ReceiveDescriptionMinThreshold::Quarter)
                .with_accept_broadcast(true)
                .with_buffer_size_extension(true)
                .with_receive_buffer_size(ReceiveBufferSize::Bytes1024)
                .with_strip_ethernet_crc(false),
        );

        // TODO: init interrupts

        INTERRUPT_MASK.write(
            &mut self.mmio_space,
            InterruptMaskRegister::new().with_receiver_timer_interrupt(true),
        );

        IDT.lock()[32 + self.pci_config_space.get_interrupt_line()]
            .set_handler_fn(generic_e1000_interrupt);
        load_idt();
    }

    // TODO: add an abstraction above this method to send packet of any size and await data sent
    // prevent stack buffer
    pub unsafe fn transmit_packet(&mut self, data: &[u8], last_packet: bool) -> Result<()> {
        if data.len() > MAX_TRANSMIT_LENGTH {
            return Err(NetworkError::BufferTooLarge);
        }

        let tail = TRANSMIT_DESCRIPTOR_BASE_TAIL.read(&self.mmio_space);

        let current_descriptor =
            &mut TRANSMISSION_DESCRIPTOR_LIST.transmission_descriptor_list[tail as usize];

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

        current_descriptor.command = TransmissionCommandRegister::END_OF_PACKET;

        TRANSMIT_DESCRIPTOR_BASE_TAIL.write(
            &mut self.mmio_space,
            (tail + 1) % TRANSMISSION_DESCRIPTOR_LIST_SIZE as u32,
        );

        Ok(())
    }

    pub unsafe fn get_receive_packet_test(&self) {
        let tail_orig = RECEIVE_DESCRIPTOR_BASE_TAIL.read(&self.mmio_space);
        let head_orig = RECEIVE_DESCRIPTOR_BASE_HEAD.read(&self.mmio_space);

        while tail_orig == RECEIVE_DESCRIPTOR_BASE_TAIL.read(&self.mmio_space)
            && head_orig == RECEIVE_DESCRIPTOR_BASE_HEAD.read(&self.mmio_space)
        {
            hint::spin_loop();
        }
    }

    pub fn get_address(&self) -> EthernetAddress {
        self.ethernet_address.clone()
    }
}
