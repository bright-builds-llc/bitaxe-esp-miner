use thiserror::Error;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
pub enum Bm1366ProtocolFault {
    #[error("bad BM1366 preamble: expected 0x{expected:04x}, got 0x{actual:04x}")]
    BadPreamble { expected: u16, actual: u16 },
    #[error("bad BM1366 CRC")]
    BadCrc,
    #[error("invalid BM1366 length: expected {expected}, got {actual}")]
    InvalidLength { expected: usize, actual: usize },
    #[error("unknown BM1366 register 0x{register:02x}")]
    UnknownRegister { register: u8 },
    #[error("invalid BM1366 job id 0x{job_id:02x}")]
    InvalidJobId { job_id: u8 },
    #[error("BM1366 chip count mismatch: expected {expected}, got {actual}")]
    ChipCountMismatch { expected: u8, actual: u8 },
    #[error("BM1366 preflight missing: {reason}")]
    PreflightMissing { reason: &'static str },
}
