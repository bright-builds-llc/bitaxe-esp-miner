//! Channel jobs share the existing once-per-boot diagnostic admission and lane.
use super::*;

pub(crate) fn claim(
    generation: WorkerGeneration,
    deadline_us: u64,
) -> Result<(), WorkerSessionError> {
    let observation = observation(generation).ok_or(WorkerSessionError::Rejected)?;
    if !crate::bwg_worker_usb::startup_diagnostics::PROGRESS.successful()
        || crate::settings_adapter::start_mining_on_boot()
        || !revocation::is_idle(generation)
        || !observation.wifi_connected
    {
        return Err(WorkerSessionError::Rejected);
    }
    let mut owner = OWNER
        .get()
        .ok_or(WorkerSessionError::Rejected)?
        .lock()
        .map_err(|_| WorkerSessionError::Rejected)?;
    if owner.maybe_binding.is_some() {
        return Err(WorkerSessionError::Rejected);
    }
    let (primary, fallback) = owner
        .maybe_transport
        .as_ref()
        .ok_or(WorkerSessionError::Rejected)?;
    let primary = primary.upgrade().ok_or(WorkerSessionError::Rejected)?;
    let fallback = fallback.upgrade().ok_or(WorkerSessionError::Rejected)?;
    crate::settings_adapter::claim_noise_fence(|| {
        if !EFFECT_OWNERS.claim_diagnostic() {
            return false;
        }
        if !primary.reserve() {
            let _released = EFFECT_OWNERS.release_diagnostic();
            return false;
        }
        if !fallback.reserve() {
            let _released = primary.release();
            let _released = EFFECT_OWNERS.release_diagnostic();
            return false;
        }
        if !revocation::claim_diagnostic(generation, deadline_us / 1000) {
            let _released = fallback.release();
            let _released = primary.release();
            let _released = EFFECT_OWNERS.release_diagnostic();
            return false;
        }
        true
    })
    .map_err(|_| WorkerSessionError::Rejected)?;
    owner.external = true;
    owner.maybe_binding = Some((generation, observation.transport_epoch as u32));
    owner
        .shared
        .binding
        .set((generation, observation.transport_epoch as u32))
        .map_err(|_| WorkerSessionError::Rejected)?;
    Ok(())
}
pub(crate) fn dispatch(generation: WorkerGeneration) -> Result<(), WorkerSessionError> {
    let mut owner = OWNER
        .get()
        .ok_or(WorkerSessionError::Rejected)?
        .lock()
        .map_err(|_| WorkerSessionError::Rejected)?;
    if !owner.external
        || owner.maybe_binding.is_none_or(|(g, e)| {
            g != generation || crate::bwg_worker_usb::maybe_authenticated_epoch() != Some(e)
        })
        || !revocation::diagnostic_live(generation)
    {
        return Err(WorkerSessionError::Rejected);
    }
    dispatch_job(&mut owner)
}
pub(super) fn observe_completion(owner: &mut Owner, generation: WorkerGeneration) {
    let released = owner.maybe_transport.as_ref().is_some_and(|(p, f)| {
        p.upgrade()
            .zip(f.upgrade())
            .is_some_and(|(p, f)| p.release() && f.release())
    }) && revocation::release_diagnostic(generation)
        && EFFECT_OWNERS.release_diagnostic();
    crate::v2_serial_runtime::channel_completed(released);
}
