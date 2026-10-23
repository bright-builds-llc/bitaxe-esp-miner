//! BM1370 semantic commands and their exact transmit frames.
//!
//! The framing (preamble, header, length, CRC5 or CRC16-FALSE) is the BM13xx framing shared
//! with the BM1366 (`bm1366::packet`); only the commands and their payloads are BM1370's.

use crate::{
    bm1366::{
        command::FrequencyPlan,
        packet::{
            CommandFrame, FrameBytes, CMD_INACTIVE, CMD_READ, CMD_SET_ADDRESS, CMD_WRITE,
            COMMAND_HEADER_TYPE, GROUP_ALL, GROUP_SINGLE,
        },
    },
    bm1370::Bm1370ProtocolFault,
};

/// The UART rate init runs at: upstream's `UART_FREQ`. The BM1370 is never sent its own
/// default-baud register, unlike the BM1368.
pub const DEFAULT_BAUD: u32 = 115_200;
/// The UART rate `SetMaxBaud` switches the chip to.
pub const MAX_BAUD: u32 = 1_000_000;

const REGISTER_DIFFICULTY_MASK: u8 = 0x14;
const REGISTER_FREQUENCY: u8 = 0x08;
const REGISTER_HASH_COUNTING: u8 = 0x10;
const REGISTER_UART_BAUD: u8 = 0x28;
const REGISTER_VERSION_MASK: u8 = 0xa4;

/// Target selection for a BM1370 register write.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RegisterTarget {
    All,
    Single { asic_address: u8 },
}

/// Pure semantic BM1370 commands.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Bm1370Command {
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
    SetMaxBaud,
    DelayMs(u32),
}

impl Bm1370Command {
    /// Encodes commands that write an ASIC frame. Delays return `None`.
    pub fn maybe_frame_bytes(self) -> Result<Option<FrameBytes>, Bm1370ProtocolFault> {
        let write_all = COMMAND_HEADER_TYPE | GROUP_ALL | CMD_WRITE;
        let (header, payload) = match self {
            Self::SetVersionMask(mask) => (write_all, version_mask_payload(mask).to_vec()),
            Self::ReadChipId => (COMMAND_HEADER_TYPE | GROUP_ALL | CMD_READ, vec![0x00, 0x00]),
            Self::SetChainInactive => (
                COMMAND_HEADER_TYPE | GROUP_ALL | CMD_INACTIVE,
                vec![0x00, 0x00],
            ),
            Self::SetChipAddress(address) => (
                COMMAND_HEADER_TYPE | GROUP_SINGLE | CMD_SET_ADDRESS,
                vec![address, 0x00],
            ),
            Self::WriteRegister {
                target,
                register,
                value,
            } => {
                let (group, address) = match target {
                    RegisterTarget::All => (GROUP_ALL, 0x00),
                    RegisterTarget::Single { asic_address } => (GROUP_SINGLE, asic_address),
                };
                (
                    COMMAND_HEADER_TYPE | group | CMD_WRITE,
                    register_payload(address, register, value).to_vec(),
                )
            }
            Self::SetDifficultyMask(mask) => (
                write_all,
                register_payload(0x00, REGISTER_DIFFICULTY_MASK, mask).to_vec(),
            ),
            Self::SetFrequency(plan) => (
                write_all,
                register_payload(
                    0x00,
                    REGISTER_FREQUENCY,
                    [plan.vdo_scale, plan.fb_divider, plan.refdiv, plan.postdiv],
                )
                .to_vec(),
            ),
            Self::SetNonceSpace(hash_counting_number) => (
                write_all,
                register_payload(
                    0x00,
                    REGISTER_HASH_COUNTING,
                    hash_counting_number.to_be_bytes(),
                )
                .to_vec(),
            ),
            Self::SetMaxBaud => (
                write_all,
                register_payload(0x00, REGISTER_UART_BAUD, [0x11, 0x30, 0x02, 0x00]).to_vec(),
            ),
            Self::DelayMs(_) => return Ok(None),
        };
        CommandFrame::new(header, &payload)
            .map(|frame| Some(frame.into_bytes()))
            .map_err(|_| Bm1370ProtocolFault::InvalidLength {
                expected: usize::from(u8::MAX) - 3,
                actual: payload.len(),
            })
    }
}

/// The version-rolling mask register: the mask's rolled bits, shifted down 13.
const fn version_mask_payload(mask: u32) -> [u8; 6] {
    let versions_to_roll = mask >> 13;
    [
        0x00,
        REGISTER_VERSION_MASK,
        0x90,
        0x00,
        (versions_to_roll >> 8) as u8,
        versions_to_roll as u8,
    ]
}

const fn register_payload(address: u8, register: u8, value: [u8; 4]) -> [u8; 6] {
    [address, register, value[0], value[1], value[2], value[3]]
}
