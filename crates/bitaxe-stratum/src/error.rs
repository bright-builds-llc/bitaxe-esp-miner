use thiserror::Error;

/// Errors returned while parsing or serializing Stratum v1 JSON-RPC messages.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum StratumV1Error {
    #[error("invalid Stratum v1 JSON-RPC message")]
    InvalidJson,
    #[error("missing Stratum v1 field: {0}")]
    MissingField(&'static str),
    #[error("invalid Stratum v1 field {field}: {reason}")]
    InvalidField {
        field: &'static str,
        reason: &'static str,
    },
    #[error("unknown Stratum v1 method: {method}")]
    UnknownMethod { method: String },
    #[error("invalid Stratum v1 params for {method}")]
    InvalidParams { method: &'static str },
    #[error("failed to serialize Stratum v1 message")]
    SerializationFailed,
}
