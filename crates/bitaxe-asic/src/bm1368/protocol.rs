//! BM1368 semantic commands and exact transmit framing.

use crate::{
    bm1366::{
        command::FrequencyPlan,
        crc::{crc16_false, crc5},
    },
    bm1368::Bm1368ProtocolFault,
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
pub const MAX_BAUD: u32 = 1_000_000;

/// Owned encoded BM1368 frame bytes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Bm1368FrameBytes(Vec<u8>);

impl Bm1368FrameBytes {
    #[must_use]
    pub fn as_slice(&self) -> &[u8] {
        &self.0
    }

    #[must_use]
    pub fn into_vec(self) -> Vec<u8> {
        self.0
    }
}

impl AsRef<[u8]> for Bm1368FrameBytes {
    fn as_ref(&self) -> &[u8] {
        self.as_slice()
    }
}

/// Target selection for a BM1368 register write.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RegisterTarget {
    All,
    Single { asic_address: u8 },
}

/// Pure semantic BM1368 commands.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Bm1368Command {
    SetVersionMask(u32),
    ReadChipId,
    SetChainInactive,
    SetChipAddress(u8),
    WriteRegister {
        target: RegisterTarget,
        register: u8,
        value: [u8; 4],
    },
    SetDifficultyMask([u8; 4]),
    SetFrequency(FrequencyPlan),
    SetNonceSpace(u32),
    SetDefaultBaud,
    SetMaxBaud,
    DelayMs(u32),
}

impl Bm1368Command {
    /// Encodes commands that write an ASIC frame. Delays return `None`.
    pub fn maybe_frame_bytes(self) -> Result<Option<Bm1368FrameBytes>, Bm1368ProtocolFault> {
        let maybe_frame = match self {
            Self::SetVersionMask(mask) => Some(command_frame(
                COMMAND_HEADER_TYPE | GROUP_ALL | CMD_WRITE,
                &version_mask_payload(mask),
            )?),
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
            Self::SetFrequency(plan) => Some(command_frame(
                COMMAND_HEADER_TYPE | GROUP_ALL | CMD_WRITE,
                &register_payload(
                    0x00,
                    0x08,
                    [plan.vdo_scale, plan.fb_divider, plan.refdiv, plan.postdiv],
                ),
            )?),
            Self::SetNonceSpace(hash_counting_number) => Some(command_frame(
                COMMAND_HEADER_TYPE | GROUP_ALL | CMD_WRITE,
                &register_payload(0x00, 0x10, hash_counting_number.to_be_bytes()),
            )?),
            Self::SetDefaultBaud => Some(command_frame(
                COMMAND_HEADER_TYPE | GROUP_ALL | CMD_WRITE,
                &register_payload(0x00, 0x18, [0x00, 0x00, 0x7a, 0x31]),
            )?),
            Self::SetMaxBaud => Some(command_frame(
                COMMAND_HEADER_TYPE | GROUP_ALL | CMD_WRITE,
                &register_payload(0x00, 0x28, [0x11, 0x30, 0x02, 0x00]),
            )?),
            Self::DelayMs(_) => None,
        };

        Ok(maybe_frame)
    }
}

pub(crate) fn command_frame(
    header: u8,
    data: &[u8],
) -> Result<Bm1368FrameBytes, Bm1368ProtocolFault> {
    if data.len() > usize::from(u8::MAX) - 3 {
        return Err(Bm1368ProtocolFault::InvalidLength {
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
    Ok(Bm1368FrameBytes(bytes))
}

pub(crate) fn job_frame(header: u8, data: &[u8]) -> Result<Bm1368FrameBytes, Bm1368ProtocolFault> {
    if data.len() > usize::from(u8::MAX) - 4 {
        return Err(Bm1368ProtocolFault::InvalidLength {
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
    Ok(Bm1368FrameBytes(bytes))
}

const fn version_mask_payload(mask: u32) -> [u8; 6] {
    let versions_to_roll = mask >> 13;
    [
        0x00,
        0xa4,
        0x90,
        0x00,
        (versions_to_roll >> 8) as u8,
        versions_to_roll as u8,
    ]
}

const fn register_payload(address: u8, register: u8, value: [u8; 4]) -> [u8; 6] {
    [address, register, value[0], value[1], value[2], value[3]]
}
