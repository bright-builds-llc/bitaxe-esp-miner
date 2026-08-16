use super::owner_progress::drive_feedback;
use super::*;

pub(super) fn run_owner(
    receiver: Receiver<OwnerInboxMessage>,
    mut adapter: OrdinaryEspProductionSessionAdapter,
) {
    let started_at = Instant::now();
    let mut session = ProductionMiningSession::new();
    let mut task_watchdog =
        watchdog::ProductionTaskWatchdog::subscribe(crate::runtime_uptime::millis());
    let mut readiness_schedule = PeriodicDeadline::new(0, PRODUCTION_REREAD_CADENCE_MS)
        .expect("production reread cadence is nonzero");

    loop {
        task_watchdog.feed(crate::runtime_uptime::millis());
        let before_wait_ms = elapsed_millis(started_at);
        let wait = Duration::from_millis(
            readiness_schedule
                .next_deadline_ms()
                .saturating_sub(before_wait_ms),
        );
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
                adapter.publish_campaign_status(&session.snapshot(), now_ms);
                return;
            }
        }
        adapter.publish_campaign_status(&session.snapshot(), now_ms);
        task_watchdog.feed(crate::runtime_uptime::millis());
        adapter.service_hashrate_monitor(&session.snapshot(), now_ms);
        if shutdown_requested {
            return;
        }
    }
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
        |effect| adapter.maybe_execute(effect, now_ms),
        |_| task_watchdog.feed(crate::runtime_uptime::millis()),
    );
    if let Err(error) = result {
        log::error!("production_mining_session=fail_closed reason=engine_error error={error}");
    }
}
