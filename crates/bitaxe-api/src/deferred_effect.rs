//! Pure failure model for firmware-owned deferred effects.

/// Failure to transfer an effect to the process-lifetime firmware worker.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DeferredEffectQueueUnavailable;
