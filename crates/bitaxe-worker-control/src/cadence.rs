//! Fixed-memory telemetry measurements. Timestamps are device-local microseconds.

mod store;
mod summary;
pub use store::CadenceRecorder;
pub use summary::{CadenceArmReceipt, CadencePhase, CadenceSnapshot, CadenceState, CadenceSummary};

pub const CAPTURE_DURATION_US: u64 = 60_000_000;

/// The real telemetry owner supplies stage boundaries, never host timing estimates.
#[derive(Clone, Copy, Debug)]
pub struct CadenceIteration {
    pub started_at_us: u64,
    pub live_finished_at_us: u64,
    pub logs_finished_at_us: u64,
    pub finished_at_us: u64,
    pub cpu: u32,
    pub priority: u32,
}

/// Closed publication outcomes. Queue/send completion is recorded separately.
#[derive(Clone, Copy, Debug)]
pub enum CadencePublication {
    Projected,
    Unchanged,
    NoSubscribers,
    ProjectionFailed,
    SerializationFailed,
}

/// Opaque boot-local phase identity retained by the asynchronous sender.
#[derive(Clone, Copy, Debug)]
pub struct CadenceSendToken(usize);

/// Private endpoint observation: must never be copied into public diagnostic evidence.
#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CadenceEndpoint {
    pub schema: &'static str,
    pub ipv4: String,
    pub http_port: u16,
    pub observed_at_us: u64,
    pub boot_ordinal: u64,
    pub generation: u32,
}

#[cfg(test)]
mod tests;

/// Imperative telemetry operations injected into the production iteration wrapper.
pub trait CadenceLoopIo {
    fn now_us(&self) -> u64;
    fn cpu(&self) -> u32;
    fn priority(&self) -> u32;
    fn live(&mut self);
    fn logs(&mut self);
    fn prune(&mut self);
}

/// Runs exactly the existing three stages; the owner retains its scheduling and sleep policy.
pub fn run_iteration(io: &mut impl CadenceLoopIo, recorder: &CadenceRecorder) {
    let started_at_us = io.now_us();
    let cpu = io.cpu();
    let priority = io.priority();
    io.live();
    let live_finished_at_us = io.now_us();
    io.logs();
    let logs_finished_at_us = io.now_us();
    io.prune();
    recorder.iteration(CadenceIteration {
        started_at_us,
        live_finished_at_us,
        logs_finished_at_us,
        finished_at_us: io.now_us(),
        cpu: if cpu == io.cpu() { cpu } else { u32::MAX },
        priority: if priority == io.priority() {
            priority
        } else {
            u32::MAX
        },
    });
}
