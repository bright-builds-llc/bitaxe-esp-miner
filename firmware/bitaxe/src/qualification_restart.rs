//! Idle-only software restart. The existing control owner calls this after reply confirmation.
use crate::production_mining_session::revocation::{self, WorkerGeneration};
use bitaxe_worker_control::{QualificationRestartContext, WorkerSessionError};

pub(crate) fn context(
    generation: WorkerGeneration,
) -> Result<Option<QualificationRestartContext>, WorkerSessionError> {
    if !crate::bwg_worker_usb::startup_diagnostics::PROGRESS.successful()
        || crate::settings_adapter::start_mining_on_boot()
        || !revocation::is_idle(generation)
    {
        return Ok(None);
    }
    let Some(transport_epoch) = crate::bwg_worker_usb::maybe_authenticated_epoch() else {
        return Ok(None);
    };
    Ok(Some(QualificationRestartContext {
        boot_ordinal: crate::boot_evidence::operator_snapshot_boot_ordinal(),
        worker_generation: generation.raw(),
        transport_epoch,
    }))
}

pub(crate) fn restart(
    generation: WorkerGeneration,
    expected: QualificationRestartContext,
    expires_at_ms: u64,
) -> Result<(), WorkerSessionError> {
    if context(generation)? != Some(expected)
        || crate::runtime_uptime::millis() >= expires_at_ms
        || crate::bwg_worker_usb::maybe_authenticated_epoch() != Some(expected.transport_epoch)
        || !revocation::claim_idle_restart(generation)
    {
        return Err(WorkerSessionError::Rejected);
    }
    if !crate::bwg_worker_usb::claim_restart_epoch(expected.transport_epoch) {
        // Cancellation won the native epoch. Release only our claimed idle state;
        // no reservation, mining cleanup or new generation can be affected.
        let _released = revocation::abort_idle_restart(generation);
        return Err(WorkerSessionError::Rejected);
    }
    // The generation fence excludes work and the epoch CAS committed reset over
    // cancellation. There is no fallible operation or delayed task before reset.
    unsafe { esp_idf_svc::sys::esp_restart() }
}
