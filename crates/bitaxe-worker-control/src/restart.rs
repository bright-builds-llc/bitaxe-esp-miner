//! Small restart bindings; they never grant work or change durable accounting.

/// Fresh native facts captured by the session adapter at restart admission.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct QualificationRestartContext {
    pub boot_ordinal: u64,
    pub worker_generation: u32,
    pub transport_epoch: u32,
}
