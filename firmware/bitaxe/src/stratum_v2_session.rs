//! Sole firmware owner for task-gated Stratum V2 production work.

pub(crate) mod transport;

use std::sync::mpsc::{self, TrySendError};
use std::time::{Duration, Instant};

use bitaxe_asic::bm1366::{
    production::{Bm1366ProductionCommand, ProductionWorkPayload},
    result::Bm1366ValidJobIds,
};
use bitaxe_core::runtime_health::TaskWatchdogOwnerSubphase;
use bitaxe_safety::observation::MonotonicMillis;
use bitaxe_stratum::v1::production_session::{
    HardwareSafeStopPurpose, MiningHardwareProfilePreset,
};
use bitaxe_stratum::v2::session::{SessionEvent, SessionFailure, V2Session};
use esp_idf_svc::sys;

use self::transport::{TransportCommand, TransportEvent, TransportFailure, TransportHandle};
use crate::asic_adapter::production::{ProductionAsicExecutor, ProductionReadOutcome};
use crate::settings_adapter::MiningCampaignStage;

const OWNER_STACK_BYTES: usize = 24 * 1024;
const EVENT_CAPACITY: usize = 16;
const PREFLIGHT_TIMEOUT: Duration = Duration::from_secs(60);
const LOOP_CADENCE: Duration = Duration::from_millis(10);
const CAMPAIGN_DURATION: Duration = Duration::from_secs(180);

pub(crate) fn start() -> anyhow::Result<()> {
    std::thread::Builder::new()
        .name("stratum-v2-session".to_owned())
        .stack_size(OWNER_STACK_BYTES)
        .spawn(run_owner)
        .map(|_| ())
        .map_err(Into::into)
}

fn run_owner() {
    let mut watchdog =
        crate::production_mining_session::watchdog::ProductionTaskWatchdog::subscribe(now_ms());
    let admission = match crate::settings_adapter::load_production_campaign_admission() {
        Ok(Some(admission)) if admission.stage == MiningCampaignStage::StratumV2 => admission,
        Ok(Some(_)) => {
            publish_terminal("campaign_mismatch", "not_applicable", false, false);
            return;
        }
        Ok(None) => {
            log::info!("stratum_v2_owner=inactive reason=campaign_absent");
            return;
        }
        Err(error) => {
            log::warn!("stratum_v2_owner=blocked category={}", error.category());
            return;
        }
    };
    let profile = admission
        .maybe_profile
        .unwrap_or(MiningHardwareProfilePreset::Conservative);
    if profile != MiningHardwareProfilePreset::Conservative || admission.maybe_lease.is_none() {
        publish_terminal("campaign_contract", "not_applicable", false, false);
        return;
    }
    let pool_set = match crate::settings_adapter::read_stratum_v2_pool_set() {
        Ok(Some(pool_set)) => pool_set,
        Ok(None) => {
            publish_terminal("pool_configuration", "not_applicable", false, false);
            return;
        }
        Err(error) => {
            log::warn!("stratum_v2_pool=unavailable category={}", error.category());
            publish_terminal("pool_configuration", "not_applicable", false, false);
            return;
        }
    };
    let mut settings_order = Vec::with_capacity(2);
    let (first, second) = if pool_set.prefer_fallback {
        (pool_set.fallback, pool_set.primary)
    } else {
        (pool_set.primary, pool_set.fallback)
    };
    if let Some(settings) = first {
        settings_order.push(settings);
    }
    if let Some(settings) = second {
        if !settings_order.contains(&settings) {
            settings_order.push(settings);
        }
    }
    if settings_order.is_empty() {
        publish_terminal("pool_configuration", "not_applicable", false, false);
        return;
    }
    if !wait_for_preflight(&mut watchdog) {
        publish_terminal("preflight", "not_applicable", false, false);
        return;
    }

    let mut actuation = crate::mining_actuation_adapter::Ultra205MiningActuationAdapter::new();
    watchdog.feed_owner_progress(now_ms(), TaskWatchdogOwnerSubphase::EffectPrepareHardware);
    if let Err(failure) = actuation.prepare(profile.profile()) {
        log::error!(
            "stratum_v2_owner=fail_closed category=hardware_preparation step={} detail={}",
            failure.original().step().label(),
            failure.original().source().category(),
        );
        publish_terminal("hardware_preparation", "not_applicable", false, false);
        return;
    }
    publish_stage("hardware_prepared", 1);

    let mut outcome = OwnerOutcome::TransportWorker;
    for settings in settings_order {
        outcome = run_active_session(settings, &mut watchdog);
        if outcome.accepted() || !outcome.retryable_before_work() {
            break;
        }
    }
    let _ = crate::asic_adapter::production::block_production_dispatch();
    watchdog.feed_owner_progress(now_ms(), TaskWatchdogOwnerSubphase::EffectSafeStopHardware);
    let mut safe_stop_progress = |step: crate::mining_actuation::SafeShutdownStep| {
        watchdog.feed_owner_progress(now_ms(), safe_stop_subphase(step));
    };
    let safe_stop_complete =
        match actuation.safe_stop(HardwareSafeStopPurpose::Terminal, &mut safe_stop_progress) {
            Ok(()) => true,
            Err(failure) => {
                log::error!(
                    "stratum_v2_owner=fail_closed category=safe_stop step={} detail={}",
                    failure.step().label(),
                    failure.source().category(),
                );
                false
            }
        };
    publish_terminal(
        outcome.label(),
        outcome.detail(),
        outcome.accepted(),
        safe_stop_complete,
    );
}

fn wait_for_preflight(
    watchdog: &mut crate::production_mining_session::watchdog::ProductionTaskWatchdog,
) -> bool {
    // SAFETY: the ESP-IDF heap capability query is read-only.
    if unsafe { sys::heap_caps_get_total_size(sys::MALLOC_CAP_SPIRAM) } == 0 {
        return false;
    }
    let deadline = Instant::now() + PREFLIGHT_TIMEOUT;
    while Instant::now() < deadline {
        watchdog.feed(now_ms());
        let wifi_ready = crate::wifi_adapter::current_wifi_snapshot().wifi_status == "connected";
        let safety_ready = crate::safety_adapter::observation_snapshot()
            .is_ultra_205_mining_safe_at(MonotonicMillis::new(now_ms()));
        if wifi_ready
            && safety_ready
            && crate::safety_adapter::safety_actuation_available()
            && crate::asic_adapter::production::production_handle_available()
        {
            return true;
        }
        std::thread::sleep(Duration::from_millis(100));
    }
    false
}

fn run_active_session(
    settings: crate::settings_adapter::V2PoolSettings,
    watchdog: &mut crate::production_mining_session::watchdog::ProductionTaskWatchdog,
) -> OwnerOutcome {
    let session_config = settings.session.clone();
    let (event_sender, event_receiver) = mpsc::sync_channel(EVENT_CAPACITY);
    let transport = match TransportHandle::spawn(settings, move |event| {
        if event_sender.send(event).is_err() {
            log::warn!("stratum_v2_transport_event=discarded reason=owner_unavailable");
        }
    }) {
        Ok(transport) => transport,
        Err(_) => return OwnerOutcome::TransportWorker,
    };
    let mut session = match V2Session::new(session_config) {
        Ok(session) => session,
        Err(_) => return OwnerOutcome::Protocol,
    };
    let mut executor = ProductionAsicExecutor::new();
    let mut valid_jobs = Bm1366ValidJobIds::empty();
    let started = Instant::now();
    let mut sequence = 1_u64;
    loop {
        watchdog.feed_owner_progress(now_ms(), TaskWatchdogOwnerSubphase::SessionEvaluation);
        if started.elapsed() >= CAMPAIGN_DURATION {
            return OwnerOutcome::Deadline;
        }
        if !crate::safety_adapter::observation_snapshot()
            .is_ultra_205_mining_safe_at(MonotonicMillis::new(now_ms()))
        {
            return OwnerOutcome::Safety;
        }
        match event_receiver.recv_timeout(LOOP_CADENCE) {
            Ok(event) => {
                if let Some(outcome) = handle_transport_event(
                    event,
                    &mut session,
                    &transport,
                    &mut executor,
                    &mut valid_jobs,
                    &mut sequence,
                ) {
                    return outcome;
                }
            }
            Err(mpsc::RecvTimeoutError::Timeout) => {}
            Err(mpsc::RecvTimeoutError::Disconnected) => return OwnerOutcome::TransportWorker,
        }
        match executor.try_read_production_result(&valid_jobs, 1) {
            Ok(ProductionReadOutcome::JobNonce(result)) => match session.observe_nonce(result) {
                Ok(Some(event)) => {
                    if let Some(outcome) = apply_session_event(
                        event,
                        &transport,
                        &mut executor,
                        &mut valid_jobs,
                        &mut sequence,
                    ) {
                        return outcome;
                    }
                }
                Ok(None) => {}
                Err(_) => return OwnerOutcome::Protocol,
            },
            Ok(
                ProductionReadOutcome::Pending
                | ProductionReadOutcome::Discarded(_)
                | ProductionReadOutcome::RegisterReadProof(_),
            ) => {}
            Err(_) => return OwnerOutcome::Asic,
        }
    }
}

fn handle_transport_event(
    event: TransportEvent,
    session: &mut V2Session,
    transport: &TransportHandle,
    executor: &mut ProductionAsicExecutor,
    valid_jobs: &mut Bm1366ValidJobIds,
    sequence: &mut u64,
) -> Option<OwnerOutcome> {
    match event {
        TransportEvent::Established => match session.start() {
            Ok(event) => apply_session_event(event, transport, executor, valid_jobs, sequence),
            Err(_) => Some(OwnerOutcome::Protocol),
        },
        TransportEvent::Message(message) => match session.handle(message) {
            Ok(events) => {
                for event in events {
                    if let Some(outcome) =
                        apply_session_event(event, transport, executor, valid_jobs, sequence)
                    {
                        return Some(outcome);
                    }
                }
                None
            }
            Err(_) => Some(OwnerOutcome::Protocol),
        },
        TransportEvent::Failed(failure) => Some(OwnerOutcome::Transport(failure)),
        TransportEvent::Closed => Some(OwnerOutcome::TransportClosed),
    }
}

fn apply_session_event(
    event: SessionEvent,
    transport: &TransportHandle,
    executor: &mut ProductionAsicExecutor,
    valid_jobs: &mut Bm1366ValidJobIds,
    sequence: &mut u64,
) -> Option<OwnerOutcome> {
    match event {
        SessionEvent::Outbound(frame) => match transport.try_send(TransportCommand::Send(frame)) {
            Ok(()) => None,
            Err(TrySendError::Full(_) | TrySendError::Disconnected(_)) => {
                Some(OwnerOutcome::TransportQueue)
            }
        },
        SessionEvent::ChannelReady { .. } => {
            publish_stage("channel_ready", advance_sequence(sequence));
            None
        }
        SessionEvent::Work(work) => {
            *valid_jobs = Bm1366ValidJobIds::single(work.asic_job_id);
            let command = Bm1366ProductionCommand::SendProductionWork(ProductionWorkPayload::new(
                work.asic_job_id,
                work.fields,
            ));
            match executor.maybe_execute(command, valid_jobs) {
                Ok(_) => {
                    publish_stage("work_dispatched", advance_sequence(sequence));
                    None
                }
                Err(_) => Some(OwnerOutcome::Asic),
            }
        }
        SessionEvent::TargetUpdated { .. } => {
            publish_stage("target_updated", advance_sequence(sequence));
            None
        }
        SessionEvent::ShareAccepted { accepted_count } if accepted_count > 0 => {
            publish_stage("share_accepted", advance_sequence(sequence));
            Some(OwnerOutcome::Accepted)
        }
        SessionEvent::ShareAccepted { .. } | SessionEvent::ShareRejected => {
            Some(OwnerOutcome::ShareRejected)
        }
        SessionEvent::Failed(failure) => Some(OwnerOutcome::Session(failure)),
        SessionEvent::Stopped => Some(OwnerOutcome::Protocol),
    }
}

fn advance_sequence(sequence: &mut u64) -> u64 {
    let current = *sequence;
    *sequence = (*sequence).saturating_add(1);
    current
}

fn publish_stage(stage: &str, sequence: u64) {
    crate::info_retained(&format!(
        "stratum_v2_runtime={{\"schema\":\"bitaxe-stratum-v2-runtime-v1\",\"stage\":\"{stage}\",\"sequence\":{sequence}}}"
    ));
}

fn publish_terminal(category: &str, detail: &str, accepted: bool, safe_stop_complete: bool) {
    crate::info_retained(&format!(
        "stratum_v2_terminal={{\"schema\":\"bitaxe-stratum-v2-terminal-v1\",\"category\":\"{category}\",\"detail\":\"{detail}\",\"accepted\":{accepted},\"safe_stop_complete\":{safe_stop_complete}}}"
    ));
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum OwnerOutcome {
    Accepted,
    Asic,
    Deadline,
    Protocol,
    Safety,
    ShareRejected,
    Transport(TransportFailure),
    TransportClosed,
    TransportQueue,
    TransportWorker,
    Session(SessionFailure),
}

impl OwnerOutcome {
    const fn accepted(self) -> bool {
        matches!(self, Self::Accepted)
    }

    const fn label(self) -> &'static str {
        match self {
            Self::Accepted => "accepted",
            Self::Asic => "asic",
            Self::Deadline => "deadline",
            Self::Protocol => "protocol",
            Self::Safety => "safety",
            Self::ShareRejected => "share_rejected",
            Self::Transport(_) => "transport",
            Self::TransportClosed => "transport_closed",
            Self::TransportQueue => "transport_queue",
            Self::TransportWorker => "transport_worker",
            Self::Session(_) => "session",
        }
    }

    const fn detail(self) -> &'static str {
        match self {
            Self::Transport(TransportFailure::Resolve) => "resolve",
            Self::Transport(TransportFailure::Connect) => "connect",
            Self::Transport(TransportFailure::Configure) => "configure",
            Self::Transport(TransportFailure::Handshake) => "handshake",
            Self::Transport(TransportFailure::Write) => "write",
            Self::Transport(TransportFailure::Read) => "read",
            Self::Transport(TransportFailure::Frame) => "frame",
            _ => "not_applicable",
        }
    }

    const fn retryable_before_work(self) -> bool {
        matches!(
            self,
            Self::Transport(
                TransportFailure::Resolve
                    | TransportFailure::Connect
                    | TransportFailure::Configure
                    | TransportFailure::Handshake
            ) | Self::TransportWorker
        )
    }
}

fn now_ms() -> u64 {
    crate::runtime_uptime::millis()
}

const fn safe_stop_subphase(
    step: crate::mining_actuation::SafeShutdownStep,
) -> TaskWatchdogOwnerSubphase {
    use crate::mining_actuation::SafeShutdownStep;
    match step {
        SafeShutdownStep::StopDispatch => TaskWatchdogOwnerSubphase::SafeStopStopDispatch,
        SafeShutdownStep::ReduceFrequencyAndResetNonce => {
            TaskWatchdogOwnerSubphase::SafeStopReduceFrequencyAndNonceState
        }
        SafeShutdownStep::HoldResetLow => TaskWatchdogOwnerSubphase::SafeStopAssertControlLineLow,
        SafeShutdownStep::DisableCoreVoltage => TaskWatchdogOwnerSubphase::SafeStopDisableCoreRail,
        SafeShutdownStep::DisableAsic => TaskWatchdogOwnerSubphase::SafeStopDisableChip,
        SafeShutdownStep::SetFanDutyTo100Percent => {
            TaskWatchdogOwnerSubphase::SafeStopSetCoolingMaximum
        }
        SafeShutdownStep::WaitForFreshTemperatureAtOrBelow45C => {
            TaskWatchdogOwnerSubphase::SafeStopWaitForCoolingProof
        }
        SafeShutdownStep::SetFanDutyTo30Percent => {
            TaskWatchdogOwnerSubphase::SafeStopSetCoolingPaused
        }
    }
}
