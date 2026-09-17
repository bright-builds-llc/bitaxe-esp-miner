//! Volatile facts for the fixed Standard channel and signed ordinary-owner job.
mod facts;
mod observer;
use crate::production_mining_session::revocation::{self, WorkerGeneration};
use bitaxe_worker_control::{v2::*, WorkerSessionError};
use std::net::SocketAddrV4;
use std::sync::Mutex;

struct Input {
    endpoint: SocketAddrV4,
    authority: [u8; 32],
    config: Vec<bitaxe_stratum::v2::session::SessionConfig>,
}
struct Data {
    maybe_record: Option<V2Record>,
    maybe_connection: Option<RetainedConnection>,
    maybe_input: Option<Vec<Input>>,
    maybe_binding: Option<(WorkerGeneration, u32)>,
    cancelled: bool,
    release: ShareRelease,
    maybe_dispatch: Option<Dispatch>,
    maybe_work: Option<bitaxe_stratum::v2::work::V2MiningWork>,
    pending_nonces: Vec<(bitaxe_asic::bm1366::result::Bm1366NonceResult, u64)>,
}
#[derive(Clone)]
struct Dispatch {
    sequence: u64,
    at_us: u64,
    id: u8,
    hash: String,
}
static DATA: Mutex<Data> = Mutex::new(Data {
    maybe_record: None,
    maybe_connection: None,
    maybe_input: None,
    maybe_binding: None,
    cancelled: false,
    release: ShareRelease::new(),
    maybe_dispatch: None,
    maybe_work: None,
    pending_nonces: Vec::new(),
});
pub(crate) fn now_us() -> Option<u64> {
    u64::try_from(unsafe { esp_idf_svc::sys::esp_timer_get_time() })
        .ok()
        .filter(|t| *t <= bitaxe_worker_control::noise::MAX_SAFE_INTEGER)
}
pub(crate) fn status(
    generation: WorkerGeneration,
    scope: Scope,
) -> Result<V2Status, WorkerSessionError> {
    let epoch =
        crate::bwg_worker_usb::maybe_authenticated_epoch().ok_or(WorkerSessionError::Rejected)?;
    let station = crate::wifi_adapter::maybe_connected_station_ipv4();
    let mut data = DATA.lock().map_err(|_| WorkerSessionError::Rejected)?;
    let time = now_us();
    if let Some(record) = data.maybe_record.as_mut().filter(|r| r.scope() == scope) {
        record.tick(time);
    }
    let maybe_record = data
        .maybe_record
        .as_ref()
        .filter(|r| r.scope() == scope)
        .map(V2Record::snapshot);
    if maybe_record.is_none() && time.is_none() {
        return Err(WorkerSessionError::Rejected);
    }
    let maybe_connection = maybe_record.as_ref().and(data.maybe_connection.clone());
    let socket = maybe_record
        .as_ref()
        .filter(|r| !r.resources.socket_closed)
        .and(maybe_connection.as_ref())
        .map(|c| c.socket.clone());
    Ok(V2Status {
        schema: "worker-stratum-v2-status-v1",
        scope,
        state: maybe_record.as_ref().map_or(State::Idle, |r| r.state),
        observation: CurrentObservation {
            boot_ordinal: crate::boot_evidence::boot_ordinal(),
            worker_generation: generation.raw().into(),
            serial_transport_epoch: epoch.into(),
            maybe_observed_at_us: time,
            clock_valid: time.is_some(),
            maybe_station_ipv4: station.map(|i| i.to_string()),
            wifi_connected: station.is_some(),
            maybe_socket: socket,
        },
        maybe_connection,
        maybe_record,
    })
}
#[inline(never)]
fn prepare_input(stratum: V2Stratum) -> Option<Vec<Input>> {
    let endpoint = stratum.maybe_socket()?;
    let authority =
        bitaxe_worker_control::noise::canonical_bytes::<32>(&stratum.authority_public_key)?;
    let mut config = Vec::new();
    config.try_reserve_exact(1).ok()?;
    config.push(bitaxe_stratum::v2::session::SessionConfig {
        endpoint_host: endpoint.ip().to_string(),
        endpoint_port: endpoint.port(),
        vendor: "bitaxe".into(),
        hardware_version: "205".into(),
        firmware: crate::semantic_version().into(),
        device_id: String::new(),
        user_identity: stratum.user_identity.to_string(),
        nominal_hashrate: 400_000_000_000.0,
        channel_kind: bitaxe_stratum::v2::messages::ChannelKind::Standard,
        minimum_extranonce_size: 0,
    });
    let mut input = Vec::new();
    input.try_reserve_exact(1).ok()?;
    input.push(Input {
        endpoint,
        authority,
        config,
    });
    Some(input)
}
pub(crate) fn admit(
    generation: WorkerGeneration,
    input: ChannelStart,
) -> Result<V2Status, WorkerSessionError> {
    let observation = status(generation, Scope::Channel)?.observation;
    let deadline = observation
        .maybe_observed_at_us
        .and_then(|t| t.checked_add(AUTHORITY_US))
        .ok_or(WorkerSessionError::Rejected)?;
    let record = V2Record::admit(
        Scope::Channel,
        input.attempt_id,
        &observation,
        Some(deadline),
    )
    .ok_or(WorkerSessionError::Rejected)?;
    let prepared = prepare_input(input.stratum).ok_or(WorkerSessionError::Rejected)?;
    if DATA
        .lock()
        .map_err(|_| WorkerSessionError::Rejected)?
        .maybe_record
        .is_some()
    {
        return Err(WorkerSessionError::Rejected);
    }
    crate::noise_serial_runtime::channel::claim(generation, deadline)?;
    {
        let mut data = DATA.lock().map_err(|_| WorkerSessionError::Rejected)?;
        data.maybe_record = Some(record);
        data.maybe_connection = None;
        data.maybe_input = Some(prepared);
        data.maybe_binding = Some((generation, observation.serial_transport_epoch as u32));
        data.cancelled = false;
        data.maybe_dispatch = None;
    }
    status(generation, Scope::Channel)
}
pub(crate) fn dispatch(generation: WorkerGeneration) -> Result<(), WorkerSessionError> {
    crate::noise_serial_runtime::channel::dispatch(generation)
}
pub(crate) fn cancel() {
    with_record(|r| r.fail(Stage::Revoked, FailureCategory::Authority, now_us()));
    if let Ok(mut data) = DATA.lock() {
        data.cancelled = true;
    }
    crate::noise_serial_runtime::cancel(bitaxe_worker_control::noise::NoiseDetail::CancelRequested);
}
pub(crate) fn channel_poll(generation: WorkerGeneration, epoch: u32) {
    let time = now_us();
    let valid = revocation::diagnostic_live(generation)
        && crate::bwg_worker_usb::maybe_authenticated_epoch() == Some(epoch)
        && time.is_some();
    with_record(|r| {
        r.tick(time);
        if !valid {
            r.fail(
                Stage::Revoked,
                if time.is_none() {
                    FailureCategory::Clock
                } else {
                    FailureCategory::Authority
                },
                time,
            );
        }
    });
    if !valid {
        revocation::revoke_reason_at(
            generation,
            crate::runtime_uptime::millis(),
            revocation::RevocationReason::ControlFailed,
        );
    }
}
#[export_name = "bitaxe_v2_channel_owner_entry"]
#[inline(never)]
pub(crate) fn channel_entry() {
    let maybe_input = DATA.lock().ok().and_then(|mut d| d.maybe_input.take());
    if let Some(input) = maybe_input {
        with_record(|r| {
            r.bind_pool(
                bitaxe_stratum::v1::production_work::PoolSessionGeneration::initial()
                    .next()
                    .raw(),
                bitaxe_stratum::v1::production_session::ProductionTransportEpoch::initial()
                    .next()
                    .raw(),
            );
        });
        observer::run_channel(input);
    }
}
pub(crate) fn channel_completed(released: bool) {
    with_record(|r| {
        if !released {
            r.fail(Stage::WorkerQuiescent, FailureCategory::Cleanup, now_us());
        }
        r.end(Operation::WorkerJoin, now_us(), !released);
        r.joined(now_us(), released);
    });
}
pub(super) fn with_record(operation: impl FnOnce(&mut V2Record)) {
    let maybe_abort = if let Ok(mut data) = DATA.lock() {
        let failure = if let Some(record) = data.maybe_record.as_mut() {
            operation(record);
            record.maybe_failure()
        } else {
            None
        };
        if let Some(failure) = failure {
            data.cancelled = true;
            data.maybe_binding
                .map(|(generation, _)| (generation, failure.category))
        } else {
            None
        }
    } else {
        None
    };
    if let Some((generation, category)) = maybe_abort {
        revocation::revoke_reason_at(
            generation,
            crate::runtime_uptime::millis(),
            if category == FailureCategory::Timeout {
                revocation::RevocationReason::LeaseOrBudgetExpired
            } else {
                revocation::RevocationReason::ControlFailed
            },
        );
    }
}

pub(super) fn permitted() -> bool {
    let Some((binding, scope, cancelled)) = DATA.lock().ok().and_then(|d| {
        Some((
            d.maybe_binding?,
            d.maybe_record.as_ref()?.scope(),
            d.cancelled,
        ))
    }) else {
        return false;
    };
    let (generation, epoch) = binding;
    !cancelled
        && now_us().is_some()
        && crate::bwg_worker_usb::maybe_authenticated_epoch() == Some(epoch)
        && if scope == Scope::Channel {
            revocation::diagnostic_live(generation)
        } else {
            revocation::permits(Some(generation))
        }
}

pub(crate) trait ShareIo {
    fn authenticated(&mut self);
    fn open(&self) -> bool;
    fn next_write(
        &mut self,
    ) -> Result<Option<(bitaxe_stratum::v2::frame::Frame, revocation::WorkPermit)>, ()>;
    fn frame(&mut self, frame: bitaxe_stratum::v2::frame::Frame);
    fn written(&mut self, sequence: u32);
}
pub(crate) fn run_share_transport(
    endpoint: SocketAddrV4,
    authority: [u8; 32],
    permit: revocation::WorkPermit,
    generation: bitaxe_stratum::v1::production_work::PoolSessionGeneration,
    epoch: bitaxe_stratum::v1::production_session::ProductionTransportEpoch,
    io: &mut dyn ShareIo,
) -> Result<(), ()> {
    if !revocation::permits_work(permit) || !permitted() {
        with_record(|r| {
            r.fail(Stage::Revoked, FailureCategory::Authority, now_us());
            r.begin(Operation::WorkerJoin, now_us());
        });
        return Err(());
    }
    if !endpoint.ip().is_private() {
        with_record(|r| {
            r.fail(Stage::Preparing, FailureCategory::Admission, now_us());
            r.begin(Operation::WorkerJoin, now_us());
        });
        return Err(());
    }
    let mut bound = false;
    with_record(|r| bound = r.bind_pool(generation.raw(), epoch.raw()));
    if !bound {
        with_record(|r| {
            r.fail(Stage::Preparing, FailureCategory::Evidence, now_us());
            r.begin(Operation::WorkerJoin, now_us());
        });
        return Err(());
    }
    observer::run_share(endpoint, authority, io)
}
pub(crate) fn share_completed() {
    if let Ok(mut data) = DATA.lock() {
        data.release.worker_returned();
        if let Some(r) = data.maybe_record.as_mut() {
            r.end(Operation::WorkerJoin, now_us(), false);
            r.joined(now_us(), false);
        }
    }
    maybe_release_share();
}
fn maybe_release_share() {
    if let Ok(mut data) = DATA.lock() {
        if data.release.can_release() {
            if crate::noise_serial_runtime::release_share_fence() {
                data.release.mark_released();
                if let Some(r) = data.maybe_record.as_mut() {
                    r.release_fence(now_us());
                }
            } else if let Some(r) = data.maybe_record.as_mut() {
                r.fail(Stage::WorkerQuiescent, FailureCategory::Cleanup, now_us());
            }
        }
    }
}
pub(crate) fn share_busy() -> bool {
    crate::noise_serial_runtime::share_busy()
}
pub(crate) fn restoration_completed(generation: WorkerGeneration) {
    if let Ok(mut data) = DATA.lock() {
        if data.maybe_record.as_ref().is_some_and(|r| {
            r.scope() == Scope::Share && r.binding().1 == u64::from(generation.raw())
        }) {
            data.release.restored();
        }
    }
    complete_never_started(generation);
    maybe_release_share();
}

pub(crate) use facts::{dispatched, nonce_observed, validated_frame, work_ready};

pub(crate) fn seed_share(
    generation: WorkerGeneration,
    grant: &bitaxe_worker_control::WorkerLeaseGrant,
) -> Result<(), WorkerSessionError> {
    if grant.maybe_v2().is_none() {
        return Ok(());
    }
    let attempt = grant
        .maybe_qualification_attempt()
        .ok_or(WorkerSessionError::Rejected)?;
    let observation = status(generation, Scope::Share)?.observation;
    let record = V2Record::admit(Scope::Share, attempt.id().to_owned(), &observation, None)
        .ok_or(WorkerSessionError::Rejected)?;
    let mut data = DATA.lock().map_err(|_| WorkerSessionError::Rejected)?;
    if data.maybe_record.is_some() {
        return Err(WorkerSessionError::Rejected);
    }
    crate::noise_serial_runtime::claim_share_fence()?;
    data.release = ShareRelease::new();
    data.maybe_record = Some(record);
    data.maybe_binding = Some((generation, observation.serial_transport_epoch as u32));
    data.maybe_connection = None;
    data.cancelled = false;
    data.maybe_dispatch = None;
    data.maybe_work = None;
    data.pending_nonces.clear();
    Ok(())
}
pub(crate) fn budget_armed(generation: WorkerGeneration, arming_epoch_ms: u64, limit_ms: u32) {
    with_record(|r| {
        if r.scope() == Scope::Share && r.binding().1 == u64::from(generation.raw()) {
            r.budget_armed(arming_epoch_ms, limit_ms, now_us());
        }
    });
}

pub(crate) fn capture_failure() {
    with_record(|r| r.fail(Stage::WorkerQuiescent, FailureCategory::Evidence, now_us()));
}

/// These events are observation-time facts. The unchanged qualification atoms
/// retain the original effect timestamps and alone prove the three-second bound.
pub(crate) fn poll_safety_facts() {
    let Some(timing) = revocation::timing(crate::runtime_uptime::millis()) else {
        return;
    };
    with_record(|r| {
        if r.scope() != Scope::Share || r.binding().1 != u64::from(timing.generation) {
            return;
        }
        for (seen, stage) in [
            (timing.maybe_gate_closed_ms.is_some(), Stage::Revoked),
            (timing.maybe_shutdown_started_ms.is_some(), Stage::Shutdown),
            (timing.shutdown_complete, Stage::Cooled),
        ] {
            if seen && !r.has_stage(stage) {
                r.event(stage, now_us(), None, None, None, None);
            }
        }
    });
}

pub(crate) fn protocol_rejected(message_type: u8, reason: bitaxe_stratum::v2::standard::Rejected) {
    use bitaxe_stratum::v2::{messages::MessageType, standard::Rejected};
    let stage = match MessageType::try_from(message_type) {
        Ok(MessageType::SetupConnectionSuccess) => Stage::Setup,
        Ok(MessageType::OpenStandardMiningChannelSuccess) => Stage::Channel,
        Ok(MessageType::NewMiningJob) => Stage::Job,
        Ok(MessageType::SetTarget) => Stage::Target,
        Ok(MessageType::SetNewPrevHash) => Stage::WorkReady,
        Ok(MessageType::SubmitSharesSuccess | MessageType::SubmitSharesError) => Stage::Accepted,
        _ => Stage::Setup,
    };
    let category = match reason {
        Rejected::Channel => FailureCategory::ChannelMismatch,
        Rejected::Job => FailureCategory::JobMismatch,
        Rejected::Nonce => FailureCategory::InvalidNonce,
        Rejected::Capacity => FailureCategory::Evidence,
        Rejected::Acknowledgement => FailureCategory::RejectedShare,
        _ => FailureCategory::Protocol,
    };
    with_record(|r| r.fail(stage, category, now_us()));
}

pub(crate) fn complete_never_started(generation: WorkerGeneration) {
    if revocation::permits(Some(generation)) || !crate::noise_serial_runtime::ordinary_pools_idle()
    {
        return;
    }
    with_record(|r| {
        if r.scope() != Scope::Share
            || r.binding().1 != u64::from(generation.raw())
            || r.pool_binding().is_some()
            || r.has_stage(Stage::WorkerQuiescent)
        {
            return;
        }
        if r.maybe_failure().is_none() {
            r.fail(Stage::Preparing, FailureCategory::Admission, now_us());
        }
        r.begin(Operation::WorkerJoin, now_us());
        r.end(Operation::WorkerJoin, now_us(), false);
        r.joined(now_us(), false);
    });
    if let Ok(mut data) = DATA.lock() {
        if data
            .maybe_record
            .as_ref()
            .is_some_and(|r| r.has_stage(Stage::WorkerQuiescent))
        {
            data.release.worker_returned();
        }
    }
}
pub(crate) fn asic_failed(
    generation: bitaxe_stratum::v1::production_work::PoolSessionGeneration,
    failure: bitaxe_stratum::v1::production_session::ProductionAsicFailure,
) {
    use bitaxe_stratum::v1::production_session::ProductionAsicFailure;
    with_record(|r| {
        if r.scope() == Scope::Share && r.pool_binding().is_some_and(|(g, _)| g == generation.raw())
        {
            r.fail(
                if failure == ProductionAsicFailure::Poll {
                    Stage::Nonce
                } else {
                    Stage::AsicDispatch
                },
                FailureCategory::Safety,
                now_us(),
            );
        }
    });
}

pub(crate) fn waiting_for_network(generation: WorkerGeneration) -> bool {
    DATA.lock().is_ok_and(|d| {
        d.maybe_record.as_ref().is_some_and(|r| {
            r.scope() == Scope::Share && r.binding().1 == u64::from(generation.raw())
        }) && d.release.waiting_for_worker()
    })
}
