//! BM1397 semantic commands and exact transmit framing.

use crate::{
    bm1366::crc::{crc16_false, crc5},
    bm1397::Bm1397ProtocolFault,
};

pub const COMMAND_PREAMBLE: [u8; 2] = [0x55, 0xaa];
pub const COMMAND_HEADER_TYPE: u8 = 0x40;
pub const JOB_HEADER_TYPE: u8 = 0x20;
pub const GROUP_ALL: u8 = 0x10;
pub const GROUP_SINGLE: u8 = 0x00;
pub const CMD_SET_ADDRESS: u8 = 0x00;
pub const CMD_WRITE: u8 = 0x01;
pub const CMD_READ: u8 = 0x02;
pub const CMD_INACTIVE: u8 = 0x03;
pub const DEFAULT_BAUD: u32 = 115_749;
pub const MAX_BAUD: u32 = 3_125_000;

/// Owned encoded BM1397 frame bytes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Bm1397FrameBytes(Vec<u8>);

impl Bm1397FrameBytes {
    #[must_use]
    pub fn as_slice(&self) -> &[u8] {
        &self.0
    }

    #[must_use]
    pub fn into_vec(self) -> Vec<u8> {
        self.0
    }
}

impl AsRef<[u8]> for Bm1397FrameBytes {
    fn as_ref(&self) -> &[u8] {
        self.as_slice()
    }
}

/// Target selection for a BM1397 register write.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RegisterTarget {
    All,
    Single { asic_address: u8 },
}

/// Pure semantic BM1397 command or delay.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Bm1397Command {
    ReadChipId,
    SetChainInactive,
    SetChipAddress(u8),
    WriteRegister {
        target: RegisterTarget,
        register: u8,
        value: [u8; 4],
    },
    SetDifficultyMask([u8; 4]),
    SetDefaultBaud,
    SetMaxBaud,
    VersionMaskPlaceholder(u32),
    DelayMs(u32),
}

impl Bm1397Command {
    /// Encodes commands that write an ASIC frame. Delays and the upstream
    /// version-mask placeholder return `None`.
    pub fn maybe_frame_bytes(self) -> Result<Option<Bm1397FrameBytes>, Bm1397ProtocolFault> {
        let maybe_frame = match self {
            Self::ReadChipId => Some(command_frame(
                COMMAND_HEADER_TYPE | GROUP_ALL | CMD_READ,
                &[0x00, 0x00],
            )?),
            Self::SetChainInactive => Some(command_frame(
                COMMAND_HEADER_TYPE | GROUP_ALL | CMD_INACTIVE,
                &[0x00, 0x00],
            )?),
            Self::SetChipAddress(address) => Some(command_frame(
                COMMAND_HEADER_TYPE | GROUP_SINGLE | CMD_SET_ADDRESS,
                &[address, 0x00],
            )?),
            Self::WriteRegister {
                target,
                register,
                value,
            } => {
                let (group, address) = match target {
                    RegisterTarget::All => (GROUP_ALL, 0x00),
                    RegisterTarget::Single { asic_address } => (GROUP_SINGLE, asic_address),
                };
                Some(command_frame(
                    COMMAND_HEADER_TYPE | group | CMD_WRITE,
                    &register_payload(address, register, value),
                )?)
            }
            Self::SetDifficultyMask(mask) => Some(command_frame(
                COMMAND_HEADER_TYPE | GROUP_ALL | CMD_WRITE,
                &register_payload(0x00, 0x14, mask),
            )?),
            Self::SetDefaultBaud => Some(command_frame(
                COMMAND_HEADER_TYPE | GROUP_ALL | CMD_WRITE,
                &register_payload(0x00, 0x18, [0x00, 0x00, 0x7a, 0x31]),
            )?),
            Self::SetMaxBaud => Some(command_frame(
                COMMAND_HEADER_TYPE | GROUP_ALL | CMD_WRITE,
                &register_payload(0x00, 0x18, [0x00, 0x00, 0x60, 0x31]),
            )?),
            Self::VersionMaskPlaceholder(_) | Self::DelayMs(_) => None,
        };

        Ok(maybe_frame)
    }
}

pub(crate) fn command_frame(
    header: u8,
    data: &[u8],
) -> Result<Bm1397FrameBytes, Bm1397ProtocolFault> {
    if data.len() > usize::from(u8::MAX) - 3 {
        return Err(Bm1397ProtocolFault::InvalidLength {
            expected: usize::from(u8::MAX) - 3,
            actual: data.len(),
        });
    }

    let mut bytes = Vec::with_capacity(data.len() + 5);
    bytes.extend_from_slice(&COMMAND_PREAMBLE);
    bytes.push(header);
    bytes.push((data.len() + 3) as u8);
    bytes.extend_from_slice(data);
    bytes.push(crc5(&bytes[2..]));
    Ok(Bm1397FrameBytes(bytes))
}

pub(crate) fn job_frame(header: u8, data: &[u8]) -> Result<Bm1397FrameBytes, Bm1397ProtocolFault> {
    if data.len() > usize::from(u8::MAX) - 4 {
        return Err(Bm1397ProtocolFault::InvalidLength {
            expected: usize::from(u8::MAX) - 4,
            actual: data.len(),
        });
    }

    let mut bytes = Vec::with_capacity(data.len() + 6);
    bytes.extend_from_slice(&COMMAND_PREAMBLE);
    bytes.push(header);
    bytes.push((data.len() + 4) as u8);
    bytes.extend_from_slice(data);
    bytes.extend_from_slice(&crc16_false(&bytes[2..]).to_be_bytes());
    Ok(Bm1397FrameBytes(bytes))
}

const fn register_payload(address: u8, register: u8, value: [u8; 4]) -> [u8; 6] {
    [address, register, value[0], value[1], value[2], value[3]]
}
