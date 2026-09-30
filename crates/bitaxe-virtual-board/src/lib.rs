//! Deterministic functional Ultra 205 devices; no host filesystem or physical I/O.
//!
//! Observations are produced here; authority and safety decisions remain in the
//! production runtime. Dynamics are explicit assumptions, not electrical proof.
pub mod asic;
pub mod board;
pub mod memory;
pub mod peripherals;
pub mod scheduler;
pub mod storage;
pub mod transport;

pub use board::{BoardConfig, BoardEvent, VirtualBoard};
pub use peripherals::{Dynamics, SensorSnapshot};

/// Stable model identity bound into validation evidence.
pub const MODEL_VERSION: &str = "ultra205-functional-v1";

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum ModelError {
    #[error("unsupported virtual operation: {0}")]
    Unsupported(&'static str),
    #[error("invalid virtual operation: {0}")]
    Invalid(&'static str),
    #[error("virtual device unavailable: {0}")]
    Unavailable(&'static str),
    #[error("virtual bus is occupied until {until_ms}ms")]
    Busy { until_ms: u64 },
    #[error("virtual persistence rejected")]
    Persistence,
}
