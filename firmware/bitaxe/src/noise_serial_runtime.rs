//! One network-only work item borrowed from the existing idle primary pool worker.
pub(crate) mod channel;
mod transport;

use crate::production_mining_session::revocation::{self, RevocationReason, WorkerGeneration};
use crate::production_mining_session::NoiseBorrowHandle;
use bitaxe_worker_control::noise::*;
use bitaxe_worker_control::WorkerSessionError;
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use std::sync::{Arc, Mutex, OnceLock, Weak};

static OWNER: OnceLock<Mutex<Owner>> = OnceLock::new();
static EFFECT_OWNERS: EffectFence = EffectFence::new();
static SHARE_SCOPE: AtomicBool = AtomicBool::new(false);

/// Counts ordinary mutation owners through their deferred effects; no waiting lock.
pub(crate) struct MutationGuard {
    _guard: bitaxe_worker_control::noise::MutationGuard<'static>,
}
impl MutationGuard {
    pub(crate) fn acquire() -> Option<Self> {
        EFFECT_OWNERS
            .maybe_mutation()
            .map(|guard| Self { _guard: guard })
    }
}

struct Shared {
    record: Mutex<Option<NoiseRecord>>,
    cancellation: AtomicU32,
    completed: AtomicBool,
    binding: OnceLock<(WorkerGeneration, u32)>,
}
struct Owner {
    shared: Arc<Shared>,
    maybe_transport: Option<(Weak<NoiseBorrowHandle>, Weak<NoiseBorrowHandle>)>,
    external: bool,
    dispatched: bool,
    completion_observed: bool,
    maybe_input: Option<NoiseStart>,
    maybe_binding: Option<(WorkerGeneration, u32)>,
}

pub(crate) fn prepare() -> anyhow::Result<()> {
    let shared = Arc::new(Shared {
        record: Mutex::new(None),
        cancellation: AtomicU32::new(0),
        completed: AtomicBool::new(false),
        binding: OnceLock::new(),
    });
    OWNER
        .set(Mutex::new(Owner {
            shared,
            maybe_transport: None,
            external: false,
            dispatched: false,
            completion_observed: false,
            maybe_input: None,
            maybe_binding: None,
        }))
        .map_err(|_| anyhow::anyhow!("noise_owner_already_prepared"))
}

pub(crate) fn install_transport(
    primary: Weak<NoiseBorrowHandle>,
    fallback: Weak<NoiseBorrowHandle>,
) -> bool {
    let Some(owner) = OWNER.get() else {
        return false;
    };
    let Ok(mut owner) = owner.lock() else {
        return false;
    };
    if owner.maybe_transport.is_some() {
        return false;
    }
    owner.maybe_transport = Some((primary, fallback));
    true
}

#[inline(never)]
fn maybe_take_prepared_job() -> Option<(Arc<Shared>, transport::PreparedInput)> {
    let (shared, input) = {
        let mut owner = OWNER.get()?.lock().ok()?;
        (Arc::clone(&owner.shared), owner.maybe_input.take()?)
    };
    if !allowed(&shared, FailureStage::Admission) {
        return None;
    }
    let Some(input) = transport::maybe_prepare(input) else {
        fail(
            &shared,
            FailureStage::Admission,
            NoiseCategory::EvidenceIncomplete,
            NoiseDetail::Malformed,
        );
        return None;
    };
    Some((shared, input))
}

#[export_name = "bitaxe_noise_serial_owner_entry"]
#[inline(never)]
fn owner_entry() {
    if let Some((shared, input)) = maybe_take_prepared_job() {
        if allowed(&shared, FailureStage::Admission) {
            transport::run(&shared, input);
        }
    }
    // All work-item inputs/socket/crypto have dropped before this function returns.
}
fn job_completed() {
    if let Some(owner) = OWNER.get() {
        if let Ok(owner) = owner.lock() {
            owner.shared.completed.store(true, Ordering::Release);
        }
    }
}
fn dispatch_job(owner: &mut Owner) -> Result<(), WorkerSessionError> {
    if owner.dispatched {
        return Err(WorkerSessionError::Rejected);
    }
    let transport = owner
        .maybe_transport
        .as_ref()
        .and_then(|(primary, _)| Weak::upgrade(primary))
        .ok_or(WorkerSessionError::Rejected)?;
    transport
        .dispatch(
            if owner.external {
                crate::v2_serial_runtime::channel_entry
            } else {
                owner_entry
            },
            job_completed,
        )
        .map_err(|_| WorkerSessionError::Rejected)?;
    owner.dispatched = true;
    Ok(())
}
fn cancel_undispatched(owner: &mut Owner) {
    if !owner.dispatched {
        drop(owner.maybe_input.take());
        if dispatch_job(owner).is_err() {
            fail(
                &owner.shared,
                FailureStage::Cleanup,
                NoiseCategory::Cleanup,
                NoiseDetail::Io,
            );
        }
    }
}

pub(crate) fn busy() -> bool {
    EFFECT_OWNERS.busy() && !share_busy()
}

pub(crate) fn observation(generation: WorkerGeneration) -> Option<NoiseObservation> {
    let epoch = crate::bwg_worker_usb::maybe_authenticated_epoch()?;
    let ipv4 = crate::wifi_adapter::maybe_connected_station_ipv4();
    let now = now_us()?;
    Some(NoiseObservation {
        boot_ordinal: crate::boot_evidence::boot_ordinal(),
        worker_generation: generation.raw().into(),
        transport_epoch: epoch.into(),
        observed_at_us: now,
        station_ipv4: ipv4.map(|ip| ip.to_string()),
        wifi_connected: ipv4.is_some(),
    })
}

pub(crate) fn status(generation: WorkerGeneration) -> Result<NoiseStatus, WorkerSessionError> {
    let owner = OWNER
        .get()
        .ok_or(WorkerSessionError::Rejected)?
        .lock()
        .map_err(|_| WorkerSessionError::Rejected)?;
    let record = owner
        .shared
        .record
        .lock()
        .map_err(|_| WorkerSessionError::Rejected)?;
    let observation = observation(generation).ok_or(WorkerSessionError::Rejected)?;
    if record.as_ref().is_some_and(|record| !record.reportable()) {
        return Err(WorkerSessionError::Rejected);
    }
    Ok(record.as_ref().map_or_else(
        || NoiseStatus {
            schema: "worker-noise-diagnostic-status-v2",
            state: NoiseState::Idle,
            observation: observation.clone(),
            job: None,
        },
        |record| record.status(observation.clone()),
    ))
}

pub(crate) fn admit(
    generation: WorkerGeneration,
    input: NoiseStart,
) -> Result<NoiseStatus, WorkerSessionError> {
    let observation = observation(generation).ok_or(WorkerSessionError::Rejected)?;
    if !crate::bwg_worker_usb::startup_diagnostics::PROGRESS.successful()
        || crate::settings_adapter::start_mining_on_boot()
        || !revocation::is_idle(generation)
        || !observation.wifi_connected
    {
        return Err(WorkerSessionError::Rejected);
    }
    let digest = input.input_sha256().ok_or(WorkerSessionError::Rejected)?;
    let record =
        NoiseRecord::admit(&input, &observation, digest).ok_or(WorkerSessionError::Rejected)?;
    let result = record.status(observation.clone());
    let mut owner = OWNER
        .get()
        .ok_or(WorkerSessionError::Rejected)?
        .lock()
        .map_err(|_| WorkerSessionError::Rejected)?;
    if owner.maybe_binding.is_some() {
        return Err(WorkerSessionError::Rejected);
    }
    let transport = owner
        .maybe_transport
        .as_ref()
        .and_then(|(primary, _)| Weak::upgrade(primary))
        .ok_or(WorkerSessionError::Rejected)?;
    let fallback = owner
        .maybe_transport
        .as_ref()
        .and_then(|(_, fallback)| Weak::upgrade(fallback))
        .ok_or(WorkerSessionError::Rejected)?;
    // Serialize the fence against all configuration transactions, not just HTTP parsing.
    crate::settings_adapter::claim_noise_fence(|| {
        if !EFFECT_OWNERS.claim_diagnostic() {
            return false;
        }
        if !transport.reserve() {
            let _released = EFFECT_OWNERS.release_diagnostic();
            return false;
        }
        if !fallback.reserve() {
            let _released = transport.release();
            let _released = EFFECT_OWNERS.release_diagnostic();
            return false;
        }
        if !revocation::claim_diagnostic(generation, record.job().authority_deadline_us / 1000) {
            let _released = fallback.release();
            let _released = transport.release();
            let _released = EFFECT_OWNERS.release_diagnostic();
            return false;
        }
        true
    })
    .map_err(|_| WorkerSessionError::Rejected)?;
    owner.maybe_binding = Some((generation, observation.transport_epoch as u32));
    owner
        .shared
        .binding
        .set((generation, observation.transport_epoch as u32))
        .map_err(|_| WorkerSessionError::Rejected)?;
    owner.maybe_input = Some(input);
    *owner
        .shared
        .record
        .lock()
        .map_err(|_| WorkerSessionError::Rejected)? = Some(record);
    Ok(result)
}

pub(crate) fn dispatch(generation: WorkerGeneration) -> Result<(), WorkerSessionError> {
    let mut owner = OWNER
        .get()
        .ok_or(WorkerSessionError::Rejected)?
        .lock()
        .map_err(|_| WorkerSessionError::Rejected)?;
    let Some((bound_generation, epoch)) = owner.maybe_binding else {
        return Err(WorkerSessionError::Rejected);
    };
    if bound_generation.raw() != generation.raw()
        || !revocation::diagnostic_live(generation)
        || crate::bwg_worker_usb::maybe_authenticated_epoch() != Some(epoch)
    {
        return Err(WorkerSessionError::Rejected);
    }
    let now = now_us().ok_or(WorkerSessionError::Rejected)?;
    let permitted = owner
        .shared
        .record
        .lock()
        .map_err(|_| WorkerSessionError::Rejected)?
        .as_mut()
        .is_some_and(|record| record.dispatch(now));
    if !permitted {
        return Err(WorkerSessionError::Rejected);
    }
    if owner.external || owner.maybe_input.is_none() {
        return Err(WorkerSessionError::Rejected);
    }
    dispatch_job(&mut owner)
}

pub(crate) fn cancel(detail: NoiseDetail) {
    let Some(owner) = OWNER.get() else {
        return;
    };
    let Ok(mut owner) = owner.lock() else {
        return;
    };
    if !busy() {
        return;
    }
    let detail = match owner
        .maybe_binding
        .map(|(generation, _)| revocation::diagnostic_reason(generation))
    {
        Some(RevocationReason::HeartbeatTimeout) => NoiseDetail::HeartbeatExpired,
        Some(RevocationReason::LeaseOrBudgetExpired) => NoiseDetail::Timeout,
        _ => detail,
    };
    latch(&owner.shared, detail);
    if let Some((generation, _)) = owner.maybe_binding {
        revocation::revoke_reason_at(
            generation,
            crate::runtime_uptime::millis(),
            RevocationReason::RestorationRequested,
        );
    }
    cancel_undispatched(&mut owner);
    observe_cancellation(&owner.shared);
}

pub(crate) fn poll() {
    let Some(owner) = OWNER.get() else {
        return;
    };
    let Ok(mut owner) = owner.try_lock() else {
        return;
    };
    if owner.external && owner.completion_observed {
        return;
    }
    let Some((generation, epoch)) = owner.maybe_binding else {
        return;
    };
    if !revocation::diagnostic_live(generation) {
        latch(
            &owner.shared,
            match revocation::diagnostic_reason(generation) {
                RevocationReason::HeartbeatTimeout => NoiseDetail::HeartbeatExpired,
                RevocationReason::LeaseOrBudgetExpired => NoiseDetail::Timeout,
                _ => NoiseDetail::SessionReplaced,
            },
        );
    } else if crate::bwg_worker_usb::maybe_authenticated_epoch() != Some(epoch) {
        latch(&owner.shared, NoiseDetail::SessionReplaced);
    }
    if owner.external {
        crate::v2_serial_runtime::channel_poll(generation, epoch);
    }
    observe_cancellation(&owner.shared);
    if let Ok(mut record) = owner.shared.record.lock() {
        if let Some(record) = record.as_mut() {
            let maybe_now = now_us();
            if !record.observe_clock(maybe_now) {
                latch(
                    &owner.shared,
                    if maybe_now.is_none() {
                        NoiseDetail::ClockDiscontinuity
                    } else {
                        NoiseDetail::Timeout
                    },
                );
            }
        }
    }
    if owner.shared.cancellation.load(Ordering::Acquire) != 0 && busy() {
        revocation::revoke_reason_at(
            generation,
            crate::runtime_uptime::millis(),
            RevocationReason::RestorationRequested,
        );
        cancel_undispatched(&mut owner);
    }
    if owner.completion_observed || !owner.shared.completed.load(Ordering::Acquire) {
        return;
    }
    owner.completion_observed = true;
    if owner.external {
        channel::observe_completion(&mut owner, generation);
        return;
    }
    let mut release_proved = false;
    if let Ok(mut record) = owner.shared.record.lock() {
        if let Some(record) = record.as_mut() {
            record.joined(now_us().unwrap_or(0), false);
            release_proved = !record.active();
        }
    }
    let worker_released = release_proved
        && owner
            .maybe_transport
            .as_ref()
            .is_some_and(|(primary, fallback)| {
                let (Some(primary), Some(fallback)) = (primary.upgrade(), fallback.upgrade())
                else {
                    return false;
                };
                primary.release() && fallback.release()
            });
    if worker_released && revocation::release_diagnostic(generation) {
        let _released = EFFECT_OWNERS.release_diagnostic();
    }
}

fn now_us() -> Option<u64> {
    u64::try_from(unsafe { esp_idf_svc::sys::esp_timer_get_time() })
        .ok()
        .filter(|now| *now <= MAX_SAFE_INTEGER)
}
fn latch(shared: &Shared, detail: NoiseDetail) {
    let code = match detail {
        NoiseDetail::CancelRequested => 1,
        NoiseDetail::SessionReplaced => 2,
        NoiseDetail::HeartbeatExpired => 3,
        NoiseDetail::ClockDiscontinuity => 5,
        NoiseDetail::DeliveryAmbiguous => 6,
        _ => 4,
    };
    let _first = shared
        .cancellation
        .compare_exchange(0, code, Ordering::AcqRel, Ordering::Acquire);
}
fn observe_cancellation(shared: &Shared) {
    let detail = match shared.cancellation.load(Ordering::Acquire) {
        0 => return,
        1 => NoiseDetail::CancelRequested,
        2 => NoiseDetail::SessionReplaced,
        3 => NoiseDetail::HeartbeatExpired,
        5 => NoiseDetail::ClockDiscontinuity,
        6 => NoiseDetail::DeliveryAmbiguous,
        _ => NoiseDetail::Timeout,
    };
    fail(
        shared,
        FailureStage::Cleanup,
        NoiseCategory::AuthorityLost,
        detail,
    );
}
fn fail(shared: &Shared, stage: FailureStage, category: NoiseCategory, detail: NoiseDetail) {
    if let Ok(mut record) = shared.record.lock() {
        if let Some(record) = record.as_mut() {
            record.fail(NoiseFailure::new(stage, category, detail, now_us()));
        }
    }
}
fn allowed(shared: &Shared, stage: FailureStage) -> bool {
    if let Some((generation, epoch)) = shared.binding.get() {
        if !revocation::diagnostic_live(*generation) {
            latch(
                shared,
                match revocation::diagnostic_reason(*generation) {
                    RevocationReason::HeartbeatTimeout => NoiseDetail::HeartbeatExpired,
                    RevocationReason::LeaseOrBudgetExpired => NoiseDetail::Timeout,
                    _ => NoiseDetail::SessionReplaced,
                },
            );
        } else if crate::bwg_worker_usb::maybe_authenticated_epoch() != Some(*epoch) {
            latch(shared, NoiseDetail::SessionReplaced);
        }
    } else {
        return false;
    }
    observe_cancellation(shared);
    let Ok(mut record) = shared.record.lock() else {
        return false;
    };
    let Some(record) = record.as_mut() else {
        return false;
    };
    let Some(now) = now_us() else {
        record.fail(NoiseFailure::new(
            stage,
            NoiseCategory::ClockInvalid,
            NoiseDetail::ClockDiscontinuity,
            None,
        ));
        return false;
    };
    record.tick(now);
    record.permitted()
}

pub(crate) use transport::rng::{prepare as prepare_rng, Failure as RngFailure};

pub(crate) fn ordinary_pools_idle() -> bool {
    OWNER
        .get()
        .and_then(|o| o.try_lock().ok())
        .and_then(|o| o.maybe_transport.clone())
        .is_some_and(|(p, f)| {
            p.upgrade()
                .zip(f.upgrade())
                .is_some_and(|(p, f)| p.is_idle() && f.is_idle())
        })
}

pub(crate) fn share_busy() -> bool {
    SHARE_SCOPE.load(Ordering::Acquire)
}
pub(crate) fn claim_share_fence() -> Result<(), WorkerSessionError> {
    crate::settings_adapter::claim_noise_fence(|| {
        if !EFFECT_OWNERS.claim_diagnostic() {
            return false;
        }
        SHARE_SCOPE.store(true, Ordering::Release);
        true
    })
    .map_err(|_| WorkerSessionError::Rejected)
}
pub(crate) fn release_share_fence() -> bool {
    if !share_busy() || !EFFECT_OWNERS.release_diagnostic() {
        return false;
    }
    SHARE_SCOPE.store(false, Ordering::Release);
    true
}
