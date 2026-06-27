//! Semantic BM1366 commands and adapter actions.
//!
//! Reference breadcrumbs:
//! - `reference/esp-miner/components/asic/asic.c`
//! - `reference/esp-miner/components/asic/bm1366.c`
//! - parity checklist rows `ASIC-003`, `ASIC-004`, and `ASIC-008`

use crate::Bm1366ProtocolFault;

use super::{
    observation::{AsicInitStatus, ChipAddress},
    packet::{
        CommandFrame, FrameBytes, JobFrame, CMD_READ, CMD_SET_ADDRESS, CMD_WRITE,
        COMMAND_HEADER_TYPE, GROUP_ALL, GROUP_SINGLE, JOB_HEADER_TYPE,
    },
    registers::{
        frequency_payload, hash_counting_payload, read_register_payload, set_address_payload,
        version_mask_payload, write_register_payload,
    },
    result::BM1366_RESULT_FRAME_LEN,
    work::Bm1366WorkPayload,
};

pub const DEFAULT_BAUD: u32 = 115_200;
pub const MAX_BAUD: u32 = 1_000_000;
pub const ADAPTER_TIMEOUT_MS: u32 = 1_000;
pub const RESET_PULSE_MS: u32 = 100;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Bm1366Command {
    SetVersionMask(VersionMask),
    ReadChipId,
    SetChipAddress(ChipAddress),
    WriteRegister(RegisterWrite),
    SetFrequency(FrequencyPlan),
    SetNonceSpace(NonceSpacePlan),
    SendDiagnosticWork(Bm1366WorkPayload),
}

impl Bm1366Command {
    pub fn adapter_actions(self) -> Result<Vec<Bm1366AdapterAction>, Bm1366ProtocolFault> {
        Ok(vec![Bm1366AdapterAction::WriteFrame(self.frame_bytes()?)])
    }

    pub fn frame_bytes(self) -> Result<FrameBytes, Bm1366ProtocolFault> {
        match self {
            Self::SetVersionMask(version_mask) => command_frame(
                COMMAND_HEADER_TYPE | GROUP_ALL | CMD_WRITE,
                version_mask_payload(version_mask.raw()).as_bytes(),
            ),
            Self::ReadChipId => command_frame(
                COMMAND_HEADER_TYPE | GROUP_ALL | CMD_READ,
                read_register_payload(0x00).as_bytes(),
            ),
            Self::SetChipAddress(address) => command_frame(
                COMMAND_HEADER_TYPE | GROUP_SINGLE | CMD_SET_ADDRESS,
                set_address_payload(address.raw()).as_bytes(),
            ),
            Self::WriteRegister(write) => command_frame(
                COMMAND_HEADER_TYPE | write.group.raw() | CMD_WRITE,
                write_register_payload(write.asic_address.raw(), write.register, write.value)
                    .as_bytes(),
            ),
            Self::SetFrequency(plan) => command_frame(
                COMMAND_HEADER_TYPE | GROUP_ALL | CMD_WRITE,
                frequency_payload(plan.vdo_scale, plan.fb_divider, plan.refdiv, plan.postdiv)
                    .as_bytes(),
            ),
            Self::SetNonceSpace(plan) => command_frame(
                COMMAND_HEADER_TYPE | GROUP_ALL | CMD_WRITE,
                hash_counting_payload(plan.hash_counting_number).as_bytes(),
            ),
            Self::SendDiagnosticWork(payload) => {
                JobFrame::new(JOB_HEADER_TYPE | GROUP_SINGLE | CMD_WRITE, payload.bytes())
                    .map(JobFrame::into_bytes)
            }
        }
    }
}

fn command_frame(header: u8, data: &[u8]) -> Result<FrameBytes, Bm1366ProtocolFault> {
    CommandFrame::new(header, data).map(CommandFrame::into_bytes)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct VersionMask(u32);

impl VersionMask {
    #[must_use]
    pub const fn new(mask: u32) -> Self {
        Self(mask)
    }

    #[must_use]
    pub const fn raw(self) -> u32 {
        self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RegisterWrite {
    pub asic_address: ChipAddress,
    pub register: u8,
    pub value: [u8; 4],
    pub group: RegisterTarget,
}

impl RegisterWrite {
    #[must_use]
    pub const fn all(register: u8, value: [u8; 4]) -> Self {
        Self {
            asic_address: ChipAddress::new(0x00),
            register,
            value,
            group: RegisterTarget::All,
        }
    }

    #[must_use]
    pub const fn single(asic_address: ChipAddress, register: u8, value: [u8; 4]) -> Self {
        Self {
            asic_address,
            register,
            value,
            group: RegisterTarget::Single,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RegisterTarget {
    All,
    Single,
}

impl RegisterTarget {
    const fn raw(self) -> u8 {
        match self {
            Self::All => GROUP_ALL,
            Self::Single => GROUP_SINGLE,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FrequencyPlan {
    pub vdo_scale: u8,
    pub fb_divider: u8,
    pub refdiv: u8,
    pub postdiv: u8,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NonceSpacePlan {
    pub hash_counting_number: u32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Bm1366AdapterAction {
    UseDefaultBaud { baud: u32 },
    UseMaxBaud { baud: u32 },
    WaitTxDone { timeout_ms: u32 },
    ClearRx,
    WriteFrame(FrameBytes),
    ReadExact { len: usize, timeout_ms: u32 },
    ReadChipId { expected_chips: u8, timeout_ms: u32 },
    DelayMs(u32),
    ResetPulse { low_ms: u32, high_ms: u32 },
    HoldResetLow,
    PublishStatus(AsicInitStatus),
}

impl Bm1366AdapterAction {
    pub const USE_DEFAULT_BAUD: Self = Self::UseDefaultBaud { baud: 115_200 };
    pub const USE_MAX_BAUD: Self = Self::UseMaxBaud { baud: 1_000_000 };
    pub const WAIT_TX_DONE: Self = Self::WaitTxDone { timeout_ms: 1_000 };
    /// Adapter constant: ReadExact len 11 with timeout_ms 1_000 for result frames.
    pub const READ_RESULT_FRAME: Self = Self::ReadExact {
        len: BM1366_RESULT_FRAME_LEN,
        timeout_ms: 1_000,
    };
    /// Adapter constant: ResetPulse low_ms 100 and high_ms 100.
    pub const RESET_PULSE: Self = Self::ResetPulse {
        low_ms: 100,
        high_ms: 100,
    };
    pub const HOLD_RESET_LOW: Self = Self::HoldResetLow;

    #[must_use]
    pub const fn read_result_frame() -> Self {
        Self::ReadExact {
            len: BM1366_RESULT_FRAME_LEN,
            timeout_ms: 1_000,
        }
    }

    #[must_use]
    pub const fn read_chip_id_response(expected_chips: u8) -> Self {
        Self::ReadChipId {
            expected_chips,
            timeout_ms: ADAPTER_TIMEOUT_MS,
        }
    }

    #[must_use]
    pub const fn reset_pulse() -> Self {
        Self::ResetPulse {
            low_ms: 100,
            high_ms: 100,
        }
    }
}
