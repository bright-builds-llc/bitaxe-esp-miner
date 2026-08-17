use super::owner_progress::{drive_feedback, OwnerProgressBoundary};
use super::*;
use bitaxe_core::runtime_health::{TaskWatchdogOwnerPhase, TaskWatchdogOwnerSubphase};

pub(super) fn run_owner(
    receiver: Receiver<OwnerInboxMessage>,
    mut adapter: OrdinaryEspProductionSessionAdapter,
) {
    let started_at = Instant::now();
    let mut session = ProductionMiningSession::new();
    record_owner_phase(TaskWatchdogOwnerPhase::Subscribing);
    let mut task_watchdog =
        watchdog::ProductionTaskWatchdog::subscribe(crate::runtime_uptime::millis());
    let mut readiness_schedule = PeriodicDeadline::new(0, PRODUCTION_REREAD_CADENCE_MS)
        .expect("production reread cadence is nonzero");

    loop {
        record_owner_phase(TaskWatchdogOwnerPhase::LoopStart);
        task_watchdog.feed(crate::runtime_uptime::millis());
        let before_wait_ms = elapsed_millis(started_at);
        let wait = Duration::from_millis(
            readiness_schedule
                .next_deadline_ms()
                .saturating_sub(before_wait_ms),
        );
        let wait_millis = u64::try_from(wait.as_millis()).unwrap_or(u64::MAX);
        let maybe_wait_deadline_millis = crate::runtime_uptime::millis().checked_add(wait_millis);
        record_owner_wait(maybe_wait_deadline_millis);
        let maybe_message = match receiver.recv_timeout(wait) {
            Ok(message) => Some(message),
            Err(mpsc::RecvTimeoutError::Timeout) => None,
            Err(mpsc::RecvTimeoutError::Disconnected) => Some(OwnerInboxMessage::Wake(
                ProductionSessionWakeup::ShutdownRequested,
            )),
        };
        let shutdown_requested = matches!(
            &maybe_message,
            Some(OwnerInboxMessage::Wake(
                ProductionSessionWakeup::ShutdownRequested
            ))
        );
        let now_ms = elapsed_millis(started_at);
        let message_reads_readiness = matches!(&maybe_message, Some(OwnerInboxMessage::Wake(_)));
        let message_is_observation_wakeup = matches!(
            &maybe_message,
            Some(OwnerInboxMessage::Wake(
                ProductionSessionWakeup::ObservationsChanged
            ))
        );
        let pending_observations_changed = notifications::take_pending_observations_changed();
        if let Some(message) = maybe_message {
            record_owner_phase(TaskWatchdogOwnerPhase::HandlingInbox);
            task_watchdog.feed_owner_progress(
                crate::runtime_uptime::millis(),
                TaskWatchdogOwnerSubphase::InboxMapping,
            );
            let snapshot = session.snapshot();
            let event = adapter.event_from_inbox(message, now_ms, &snapshot);
            drive_session(
                &mut session,
                &mut adapter,
                &mut task_watchdog,
                event,
                now_ms,
            );
        }
        if pending_observations_changed && !message_is_observation_wakeup && !shutdown_requested {
            record_owner_phase(TaskWatchdogOwnerPhase::HandlingObservation);
            let observation_now_ms = elapsed_millis(started_at);
            let snapshot = session.snapshot();
            let event = adapter.wake_event(
                Some(ProductionSessionWakeup::ObservationsChanged),
                observation_now_ms,
                &snapshot,
                true,
            );
            drive_session(
                &mut session,
                &mut adapter,
                &mut task_watchdog,
                event,
                observation_now_ms,
            );
        }
        if readiness_schedule.is_due(now_ms) {
            record_owner_phase(TaskWatchdogOwnerPhase::HandlingReadiness);
            if !message_reads_readiness {
                let snapshot = session.snapshot();
                let event = adapter.wake_event(None, now_ms, &snapshot, false);
                drive_session(
                    &mut session,
                    &mut adapter,
                    &mut task_watchdog,
                    event,
                    now_ms,
                );
            }
            if readiness_schedule.advance_past(now_ms).is_err() {
                log::error!(
                    "production_mining_session=fail_closed reason=readiness_deadline_overflow"
                );
                let event = adapter.wake_event(
                    Some(ProductionSessionWakeup::ShutdownRequested),
                    now_ms,
                    &session.snapshot(),
                    false,
                );
                drive_session(
                    &mut session,
                    &mut adapter,
                    &mut task_watchdog,
                    event,
                    now_ms,
                );
                record_owner_phase(TaskWatchdogOwnerPhase::PublishingCampaignStatus);
                let _ = adapter.publish_campaign_status(&session.snapshot(), now_ms);
                record_owner_phase(TaskWatchdogOwnerPhase::Shutdown);
                return;
            }
        }
        record_owner_phase(TaskWatchdogOwnerPhase::PublishingCampaignStatus);
        if let Err(error) = adapter.publish_campaign_status(&session.snapshot(), now_ms) {
            log::error!(
                "production_mining_session=fail_closed reason=campaign_status_schedule_{}",
                error.label()
            );
            record_owner_phase(TaskWatchdogOwnerPhase::HandlingInbox);
            let event = adapter.wake_event(
                Some(ProductionSessionWakeup::ShutdownRequested),
                now_ms,
                &session.snapshot(),
                false,
            );
            drive_session(
                &mut session,
                &mut adapter,
                &mut task_watchdog,
                event,
                now_ms,
            );
            record_owner_phase(TaskWatchdogOwnerPhase::Shutdown);
            return;
        }
        task_watchdog.feed(crate::runtime_uptime::millis());
        record_owner_phase(TaskWatchdogOwnerPhase::ServicingHashrate);
        adapter.service_hashrate_monitor(&session.snapshot(), now_ms);
        if shutdown_requested {
            record_owner_phase(TaskWatchdogOwnerPhase::Shutdown);
            return;
        }
    }
}

fn record_owner_phase(phase: TaskWatchdogOwnerPhase) {
    crate::task_watchdog_observation::record_owner_phase(phase);
}

fn record_owner_wait(maybe_deadline_millis: Option<u64>) {
    crate::task_watchdog_observation::record_owner_wait(maybe_deadline_millis);
}

fn drive_session(
    session: &mut ProductionMiningSession,
    adapter: &mut OrdinaryEspProductionSessionAdapter,
    task_watchdog: &mut watchdog::ProductionTaskWatchdog,
    initial_event: ProductionSessionEvent,
    now_ms: u64,
) {
    let result = drive_feedback(
        initial_event,
        |event| session.handle(event),
        |effect, heartbeat| adapter.maybe_execute(effect, now_ms, heartbeat),
        |boundary, maybe_effect| {
            let maybe_subphase = match boundary {
                OwnerProgressBoundary::EventStarted => {
                    Some(TaskWatchdogOwnerSubphase::SessionEvaluation)
                }
                OwnerProgressBoundary::EffectStarted => {
                    let effect = maybe_effect.expect("effect-start boundary carries its effect");
                    Some(effect_subphase(effect))
                }
                OwnerProgressBoundary::EventHandled
                | OwnerProgressBoundary::EffectHeartbeat
                | OwnerProgressBoundary::EffectCompleted => None,
            };
            let now_millis = crate::runtime_uptime::millis();
            if let Some(subphase) = maybe_subphase {
                task_watchdog.feed_owner_progress(now_millis, subphase);
            } else {
                task_watchdog.feed(now_millis);
            }
        },
    );
    if let Err(error) = result {
        log::error!("production_mining_session=fail_closed reason=engine_error error={error}");
    }
}

fn effect_subphase(effect: &ProductionSessionEffect) -> TaskWatchdogOwnerSubphase {
    match effect {
        ProductionSessionEffect::PrepareHardware { .. } => {
            TaskWatchdogOwnerSubphase::EffectPrepareHardware
        }
        ProductionSessionEffect::ReadPoolConfiguration => {
            TaskWatchdogOwnerSubphase::EffectReadPoolConfiguration
        }
        ProductionSessionEffect::ConnectPool { .. } => TaskWatchdogOwnerSubphase::EffectConnectPool,
        ProductionSessionEffect::WritePoolLine { .. } => {
            TaskWatchdogOwnerSubphase::EffectWritePoolLine
        }
        ProductionSessionEffect::ApplyVersionMask { .. } => {
            TaskWatchdogOwnerSubphase::EffectApplyVersionMask
        }
        ProductionSessionEffect::DispatchAsic { .. } => {
            TaskWatchdogOwnerSubphase::EffectDispatchChip
        }
        ProductionSessionEffect::PollAsic { .. } => TaskWatchdogOwnerSubphase::EffectPollChip,
        ProductionSessionEffect::BlockSubmissions => {
            TaskWatchdogOwnerSubphase::EffectBlockSubmissions
        }
        ProductionSessionEffect::InvalidateWorkAndSubmissions => {
            TaskWatchdogOwnerSubphase::EffectInvalidateWorkAndSubmissions
        }
        ProductionSessionEffect::StopAsicInteraction => {
            TaskWatchdogOwnerSubphase::EffectStopChipInteraction
        }
        ProductionSessionEffect::ClosePoolConnection { .. } => {
            TaskWatchdogOwnerSubphase::EffectClosePoolConnection
        }
        ProductionSessionEffect::SafeStopHardware { .. } => {
            TaskWatchdogOwnerSubphase::EffectSafeStopHardware
        }
        ProductionSessionEffect::RecordScoreboard { .. } => {
            TaskWatchdogOwnerSubphase::EffectRecordScoreboard
        }
        ProductionSessionEffect::RecordBlockFound => {
            TaskWatchdogOwnerSubphase::EffectRecordBlockFound
        }
        ProductionSessionEffect::Publish(_) => TaskWatchdogOwnerSubphase::EffectPublish,
    }
}
