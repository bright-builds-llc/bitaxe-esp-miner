//! BM1366 transmit packet framing.
//!
//! Reference breadcrumb: `reference/esp-miner/components/asic/bm1366.c:_send_BM1366`,
//! parity checklist rows `ASIC-001` and `ASIC-006`.

use crate::Bm1366ProtocolFault;

use super::crc::{crc16_false, crc5};

pub const BM1366_COMMAND_PREAMBLE: [u8; 2] = [0x55, 0xaa];
pub const COMMAND_HEADER_TYPE: u8 = 0x40;
pub const JOB_HEADER_TYPE: u8 = 0x20;
pub const GROUP_ALL: u8 = 0x10;
pub const GROUP_SINGLE: u8 = 0x00;
pub const CMD_SET_ADDRESS: u8 = 0x00;
pub const CMD_WRITE: u8 = 0x01;
pub const CMD_READ: u8 = 0x02;
pub const CMD_INACTIVE: u8 = 0x03;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FrameBytes(Vec<u8>);

impl FrameBytes {
    pub fn as_slice(&self) -> &[u8] {
        &self.0
    }

    pub fn into_vec(self) -> Vec<u8> {
        self.0
    }
}

impl AsRef<[u8]> for FrameBytes {
    fn as_ref(&self) -> &[u8] {
        self.as_slice()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommandFrame {
    bytes: FrameBytes,
}

impl CommandFrame {
    pub fn new(header: u8, data: &[u8]) -> Result<Self, Bm1366ProtocolFault> {
        let data_len = data.len();
        if data_len > usize::from(u8::MAX) - 3 {
            return Err(Bm1366ProtocolFault::InvalidLength {
                expected: usize::from(u8::MAX) - 3,
                actual: data_len,
            });
        }

        let total_len = data_len + 5;
        let length_field = data_len + 3;
        let mut bytes = Vec::with_capacity(total_len);
        bytes.extend_from_slice(&BM1366_COMMAND_PREAMBLE);
        bytes.push(header);
        bytes.push(length_field as u8);
        bytes.extend_from_slice(data);
        let crc = crc5(&bytes[2..]);
        bytes.push(crc);

        Ok(Self {
            bytes: FrameBytes(bytes),
        })
    }

    pub fn bytes(&self) -> &[u8] {
        self.bytes.as_slice()
    }

    pub fn into_bytes(self) -> FrameBytes {
        self.bytes
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct JobFrame {
    bytes: FrameBytes,
}

impl JobFrame {
    pub fn new(header: u8, data: &[u8]) -> Result<Self, Bm1366ProtocolFault> {
        let data_len = data.len();
        if data_len > usize::from(u8::MAX) - 4 {
            return Err(Bm1366ProtocolFault::InvalidLength {
                expected: usize::from(u8::MAX) - 4,
                actual: data_len,
            });
        }

        let total_len = data_len + 6;
        let length_field = data_len + 4;
        let mut bytes = Vec::with_capacity(total_len);
        bytes.extend_from_slice(&BM1366_COMMAND_PREAMBLE);
        bytes.push(header);
        bytes.push(length_field as u8);
        bytes.extend_from_slice(data);
        let crc = crc16_false(&bytes[2..]);
        bytes.extend_from_slice(&crc.to_be_bytes());

        Ok(Self {
            bytes: FrameBytes(bytes),
        })
    }

    pub fn bytes(&self) -> &[u8] {
        self.bytes.as_slice()
    }

    pub fn into_bytes(self) -> FrameBytes {
        self.bytes
    }
}
