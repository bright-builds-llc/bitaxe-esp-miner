//! Generation-stamped authority and retained timing observations.
use super::{RevocationReason, GENERATION_SHIFT};

#[derive(Clone, Copy)]
pub struct RevocationTiming {
    pub generation: u32,
    pub revocation_reason: RevocationReason,
    pub last_valid_heartbeat_ms: u32,
    pub maybe_gate_closed_ms: Option<u32>,
    pub maybe_shutdown_started_ms: Option<u32>,
    pub active_ms: u32,
    pub generation_elapsed_ms: u32,
    pub active_limit_ms: Option<u32>,
    pub shutdown_budget_ms: u32,
    pub work_gate_remaining_ms: Option<u32>,
    pub shutdown_stage: u32,
    pub shutdown_complete: bool,
    pub submitted: u32,
    pub accepted: u32,
    pub rejected: u32,
    pub nonce_work_correlations: u32,
    pub work_dispatched: u32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct WorkerGeneration(pub(super) u32);

impl WorkerGeneration {
    pub const fn raw(self) -> u32 {
        self.0 >> GENERATION_SHIFT
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct WorkPermit {
    pub(super) maybe_generation: Option<WorkerGeneration>,
    pub(super) epoch: u32,
}

impl WorkPermit {
    pub const fn maybe_generation(self) -> Option<WorkerGeneration> {
        self.maybe_generation
    }
}
