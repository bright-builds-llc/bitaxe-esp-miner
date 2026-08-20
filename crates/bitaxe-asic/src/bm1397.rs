//! Pure BM1397 protocol behavior.
//!
//! Reference breadcrumbs:
//! - `reference/esp-miner/components/asic/bm1397.c`
//! - `reference/esp-miner/components/asic/include/bm1397.h`
//! - `reference/esp-miner/main/device_config.h`
//! - parity checklist row `ASIC-010`
//!
//! This module contains no UART, GPIO, timing, or device effects. Firmware
//! dispatch remains deferred until a supported BM1397 board has hardware
//! evidence and an independently authorized adapter.

pub mod frequency;
pub mod init;
pub mod protocol;
pub mod result;
pub mod work;

#[cfg(test)]
mod tests;

use thiserror::Error;

/// Upstream BM1397 chip identity.
pub const BM1397_CHIP_ID: u16 = 0x1397;
/// Complete BM1397 receive frame length.
pub const BM1397_RESULT_FRAME_LEN: usize = 9;

/// Closed pure-protocol failures for BM1397 planning and decoding.
#[derive(Debug, Error, Clone, Copy, PartialEq, Eq)]
pub enum Bm1397ProtocolFault {
    #[error("bad BM1397 preamble: expected 0x{expected:04x}, got 0x{actual:04x}")]
    BadPreamble { expected: u16, actual: u16 },
    #[error("bad BM1397 CRC")]
    BadCrc,
    #[error("invalid BM1397 length: expected {expected}, got {actual}")]
    InvalidLength { expected: usize, actual: usize },
    #[error("unknown BM1397 register 0x{register:02x}")]
    UnknownRegister { register: u8 },
    #[error("invalid BM1397 job id 0x{job_id:02x}")]
    InvalidJobId { job_id: u8 },
    #[error("duplicate BM1397 nonce 0x{nonce:08x}")]
    DuplicateNonce { nonce: u32 },
    #[error("invalid BM1397 address interval {address_interval}")]
    InvalidAddressInterval { address_interval: u16 },
    #[error("invalid BM1397 chip count {chip_count}")]
    InvalidChipCount { chip_count: u8 },
    #[error("invalid BM1397 frequency {frequency_quarter_mhz} quarter-MHz")]
    InvalidFrequency { frequency_quarter_mhz: u32 },
}
