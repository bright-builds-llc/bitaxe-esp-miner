//! Physical singleton wrappers around the shared lock-free authority runtime.
pub(crate) use bitaxe_runtime::revocation::{
    GenerationGate, RevocationReason, RevocationTiming, WorkPermit,
};
pub use bitaxe_runtime::revocation::{WorkerGeneration, HEARTBEAT_CUTOFF_MS};
#[path = "revocation/global.rs"]
mod global;
pub(crate) use global::*;
