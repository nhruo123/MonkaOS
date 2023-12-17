#![allow(dead_code)]

use bitflags::bitflags;

bitflags! {
    #[derive(Default)]
    pub struct TransmissionCommandRegister: u8 {
        const END_OF_PACKET = 1 << 0;
        const IFCS = 1 << 1;
        const IC = 1 << 2;
        const REPORT_STATUS = 1 << 3;
        const REPORT_PACKET_SEND = 1 << 4;
        const DEXT = 1 << 5;
        const VLE = 1 << 6;
        const IDE = 1 << 7;
    }
    #[derive(Default)]
    pub struct TransmissionStatusRegister: u8 {
        // 4 RESERVE BITS
        const DESCRIPTOR_DONE = 1 << 4;
        const EXCESS_COLLISIONS = 1 << 5;
        const LATE_COLLISION = 1 << 6;
        const TRANSMIT_UNDERRUN = 1 << 7;
    }
}

#[derive(Default, Clone, Copy)]
#[repr(C, packed)]
pub struct TransmissionDescriptor {
    pub base_address: u64,
    pub length: u16,
    pub cso: u8,
    pub command: TransmissionCommandRegister,
    pub status: TransmissionStatusRegister,
    pub css: u8,
    pub special: u16,
}

impl TransmissionDescriptor {
    pub const fn empty() -> Self {
        Self {
            base_address: 0,
            command: TransmissionCommandRegister::REPORT_STATUS,
            cso: 0,
            length: 0,
            css: 0,
            special: 0,
            status: TransmissionStatusRegister::DESCRIPTOR_DONE,
        }
    }
}

bitflags! {
    #[derive(Default)]
    pub struct ReceiveStatusRegister: u8 {
        const DESCRIPTOR_DONE = 1 << 0;
        const END_OF_PACKET = 1 << 1;
        const IGNORE_CHECKSUM_INDICATION = 1 << 2;
        const VP = 1 << 3;
        const TCP_CHECKSUM_CALCULATED = 1 << 5;
        const IP_CHECKSUM_CALCULATED = 1 << 6;
        const PIF = 1 << 7;
    }

    #[derive(Default)]
    pub struct ReceiveErrorRegister: u8 {
        const CRC_OR_ALIGNMENT_ERROR = 1 << 0;
        const SYMBOL_ERROR = 1 << 1;
        const SEQUENCE_ERROR = 1 << 2;
        const CARRIER_EXTENSION_ERROR = 1 << 4;
        const TCP_UDP_CHECKSUM_ERROR = 1 << 5;
        const IP_CHECKSUM_ERROR = 1 << 6;
        const RX_DATA_ERROR = 1 << 7;
    }
}

#[derive(Default, Clone, Copy)]
#[repr(C, packed)]
pub struct ReceiveDescriptor {
    pub base_address: u64,
    pub length: u16,
    pub packet_checksum: u16,
    pub status: ReceiveStatusRegister,
    pub errors: ReceiveErrorRegister,
    pub special: u16,
}

impl ReceiveDescriptor {
    pub const fn empty() -> Self {
        Self {
            base_address: 0,
            length: 0,
            packet_checksum: 0,
            status: ReceiveStatusRegister::empty(),
            errors: ReceiveErrorRegister::empty(),
            special: 0,
        }
    }
}
