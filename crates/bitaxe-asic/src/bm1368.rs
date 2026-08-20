//! Pure BM1368 protocol behavior.
//!
//! Reference breadcrumbs:
//! - `reference/esp-miner/components/asic/bm1368.c`
//! - `reference/esp-miner/components/asic/include/bm1368.h`
//! - parity checklist row `ASIC-009`
//!
//! This module owns protocol facts and deterministic planning only. Firmware
//! dispatch remains deferred until a supported BM1368 board has hardware
//! evidence.

pub mod init;
pub mod protocol;
pub mod result;
pub mod work;

#[cfg(test)]
mod tests;

use thiserror::Error;

/// Upstream BM1368 chip identity.
pub const BM1368_CHIP_ID: u16 = 0x1368;
/// Complete BM1368 receive frame length.
pub const BM1368_RESULT_FRAME_LEN: usize = 11;

/// Closed pure-protocol failures for BM1368 planning and decoding.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
pub enum Bm1368ProtocolFault {
    #[error("bad BM1368 preamble: expected 0x{expected:04x}, got 0x{actual:04x}")]
    BadPreamble { expected: u16, actual: u16 },
    #[error("bad BM1368 CRC")]
    BadCrc,
    #[error("invalid BM1368 length: expected {expected}, got {actual}")]
    InvalidLength { expected: usize, actual: usize },
    #[error("unknown BM1368 register 0x{register:02x}")]
    UnknownRegister { register: u8 },
    #[error("invalid BM1368 job id 0x{job_id:02x}")]
    InvalidJobId { job_id: u8 },
    #[error("invalid BM1368 core id {core_id}")]
    InvalidCoreId { core_id: u8 },
    #[error("invalid BM1368 address interval {address_interval}")]
    InvalidAddressInterval { address_interval: u16 },
    #[error("invalid BM1368 chip count {chip_count}")]
    InvalidChipCount { chip_count: u8 },
    #[error("invalid BM1368 frequency {frequency_quarter_mhz} quarter-MHz")]
    InvalidFrequency { frequency_quarter_mhz: u32 },
}
