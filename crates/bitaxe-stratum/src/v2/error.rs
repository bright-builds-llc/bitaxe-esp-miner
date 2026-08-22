use thiserror::Error;

#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum StratumV2Error {
    #[error("Stratum V2 frame header is truncated")]
    TruncatedHeader,
    #[error("Stratum V2 payload length {actual} exceeds the {maximum}-byte bound")]
    PayloadTooLarge { actual: usize, maximum: usize },
    #[error("Stratum V2 frame length mismatch: expected {expected}, received {actual}")]
    FrameLengthMismatch { expected: usize, actual: usize },
    #[error("Stratum V2 message type 0x{0:02x} is unsupported")]
    UnsupportedMessageType(u8),
    #[error("Stratum V2 field {field} is malformed: {reason}")]
    InvalidField {
        field: &'static str,
        reason: &'static str,
    },
    #[error("Stratum V2 payload is truncated while reading {field}")]
    TruncatedField { field: &'static str },
    #[error("Stratum V2 payload contains trailing bytes")]
    TrailingPayload,
    #[error("Stratum V2 Noise handshake state is invalid")]
    InvalidNoiseState,
    #[error("Stratum V2 Noise handshake failed")]
    NoiseHandshake,
    #[error("Stratum V2 Noise authentication failed")]
    NoiseAuthentication,
    #[error("Stratum V2 Noise cipher nonce budget is exhausted")]
    NoiseNonceExhausted,
}
