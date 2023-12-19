use core::{fmt::Formatter, mem::size_of};

use alloc::vec::{self, Vec};
use crc::Crc;

#[derive(Debug, Clone, Copy)]
#[repr(C, packed)]
pub struct EthernetAddress {
    pub bytes: [u8; 6],
}

impl EthernetAddress {
    pub fn broadcast() -> Self {
        Self { bytes: [0xff; 6] }
    }
}

impl core::fmt::Display for EthernetAddress {
    fn fmt(&self, f: &mut Formatter) -> core::fmt::Result {
        let _ = write!(
            f,
            "{:<02X}:{:<02X}:{:<02X}:{:<02X}:{:<02X}:{:<02X}",
            self.bytes[0],
            self.bytes[1],
            self.bytes[2],
            self.bytes[3],
            self.bytes[4],
            self.bytes[5]
        );
        Ok(())
    }
}

#[derive(Debug, Clone, Copy)]
#[repr(u16)]
pub enum EitherType {
    Ipv4 = 0x0800,
    Arp = 0x0806,
}

#[repr(C, packed)]
pub struct EthernetFrame<'a> {
    pub destination_address: EthernetAddress,
    pub source_address: EthernetAddress,
    pub ether_type: EitherType,
    pub data: &'a [u8],
}

impl <'a> EthernetFrame <'a> {
    pub fn to_bytes(&self, add_crc: bool) -> Vec<u8> {
        let mut vec = Vec::with_capacity(size_of::<EthernetFrame>() + self.data.len());

        vec.extend_from_slice(&self.destination_address.bytes);
        vec.extend_from_slice(&self.source_address.bytes);
        vec.extend_from_slice(&(self.ether_type as u16).to_be_bytes());
        vec.extend_from_slice(self.data);

        if add_crc {
            let crc = Crc::<u32>::new(&crc::CRC_32_ISO_HDLC);
            let mut digest = crc.digest();
            digest.update(&vec);
            vec.extend_from_slice(&digest.finalize().to_be_bytes());
        }

        vec
    }
}
