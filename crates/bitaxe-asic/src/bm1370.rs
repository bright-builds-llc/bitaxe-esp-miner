//! Pure BM1370 protocol behavior, for the Gamma 601 (ADR-0041 phase E).
//!
//! Reference breadcrumbs:
//! - `reference/esp-miner/components/asic/bm1370.c`
//! - `reference/esp-miner/components/asic/include/bm1370.h`
//! - parity checklist row `ASIC-008`
//!
//! This module owns protocol facts and deterministic planning only: the commands the firmware
//! sends and the frames it receives. Firmware dispatch stays deferred until the Gamma 601 has
//! its own hardware evidence (phase F); nothing here inherits Ultra 205 verification.

pub mod init;
pub mod protocol;
pub mod result;
pub mod shutdown;
pub mod work;

#[cfg(test)]
mod tests;

use thiserror::Error;

/// Upstream BM1370 chip identity.
pub const BM1370_CHIP_ID: u16 = 0x1370;
/// Complete BM1370 receive frame length.
pub const BM1370_RESULT_FRAME_LEN: usize = 11;

/// Closed pure-protocol failures for BM1370 planning and decoding.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
pub enum Bm1370ProtocolFault {
    #[error("bad BM1370 preamble: expected 0x{expected:04x}, got 0x{actual:04x}")]
    BadPreamble { expected: u16, actual: u16 },
    #[error("bad BM1370 CRC")]
    BadCrc,
    #[error("invalid BM1370 length: expected {expected}, got {actual}")]
    InvalidLength { expected: usize, actual: usize },
    #[error("unexpected chip id 0x{chip_id:04x}")]
    UnexpectedChipId { chip_id: u16 },
    #[error("unknown BM1370 register 0x{register:02x}")]
    UnknownRegister { register: u8 },
    #[error("invalid BM1370 job id 0x{job_id:02x}")]
    InvalidJobId { job_id: u8 },
    #[error("invalid BM1370 address interval {address_interval}")]
    InvalidAddressInterval { address_interval: u16 },
    #[error("invalid BM1370 chip count {chip_count}")]
    InvalidChipCount { chip_count: u8 },
    #[error("invalid BM1370 frequency {frequency_quarter_mhz} quarter-MHz")]
    InvalidFrequency { frequency_quarter_mhz: u32 },
    #[error("invalid BM1370 nonce-space input")]
    InvalidNonceSpace,
}
