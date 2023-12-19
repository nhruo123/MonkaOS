use core::mem::size_of;

use alloc::vec::Vec;

// https://datatracker.ietf.org/doc/html/rfc826
use super::ethernet::EitherType;

#[repr(u16)]
#[derive(Debug, Clone, Copy)]
pub enum Operation {
    Request = 1,
    Replay = 2,
}

#[repr(u16)]
#[derive(Debug, Clone, Copy)]
pub enum HardwareType {
    Ethernet = 1,
}

#[repr(C, packed)]
pub struct ArpPacket<'a> {
    pub hardware_type: HardwareType,
    pub protocol_type: EitherType,
    pub hardware_len: u8,
    pub protocol_len: u8,
    pub operation: Operation,
    pub sender_hardware_address: &'a [u8],
    pub sender_protocol_address: &'a [u8],
    pub target_hardware_address: &'a [u8],
    pub target_protocol_address: &'a [u8],
}

impl<'a> ArpPacket<'a> {
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut vec = Vec::with_capacity(
            size_of::<ArpPacket>()
                + self.hardware_len as usize * 2
                + self.protocol_len as usize * 2,
        );

        vec.extend_from_slice(&(self.hardware_type as u16).to_be_bytes());
        vec.extend_from_slice(&(self.protocol_type as u16).to_be_bytes());
        vec.push(self.hardware_len);
        vec.push(self.protocol_len);
        vec.extend_from_slice(&(self.operation as u16).to_be_bytes());
        vec.extend_from_slice(self.sender_hardware_address);
        vec.extend_from_slice(self.sender_protocol_address);
        vec.extend_from_slice(self.target_hardware_address);
        vec.extend_from_slice(self.target_protocol_address);

        vec
    }
}
