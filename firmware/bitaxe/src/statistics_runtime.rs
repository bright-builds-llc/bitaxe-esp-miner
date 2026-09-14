//! Sole producer for bounded runtime statistics history.

use std::{thread, time::Duration};

use bitaxe_core::runtime_orchestration::PeriodicDeadline;

pub(crate) mod diagnostics;
mod lifecycle;
mod native;

pub const STATISTICS_CADENCE_MS: u64 = 1_000;
const PRODUCER_THREAD_NAME: &str = "statistics";
const PRODUCER_THREAD_STACK_BYTES: usize = 8 * 1024;

pub(crate) struct PreparedStatistics(lifecycle::Prepared);

#[derive(Debug)]
pub(crate) enum PreparationFailure {
    AlreadyPrepared,
    Configuration,
    Spawn,
}

pub(crate) fn prepare() -> Result<PreparedStatistics, PreparationFailure> {
    if !diagnostics::STARTUP.begin(PRODUCER_THREAD_STACK_BYTES as u32) {
        return Err(PreparationFailure::AlreadyPrepared);
    }
    let Some(caps) = native::stack_capabilities() else {
        diagnostics::STARTUP.config_failed(PRODUCER_THREAD_STACK_BYTES as u32);
        return Err(PreparationFailure::Configuration);
    };
    diagnostics::STARTUP.observe_before(caps, native::heap(caps));
    let result = lifecycle::prepare(|gate| {
        thread::Builder::new()
            .name(PRODUCER_THREAD_NAME.to_owned())
            .stack_size(PRODUCER_THREAD_STACK_BYTES)
            .spawn(move || {
                if gate.wait() {
                    run();
                }
            })
    });
    diagnostics::STARTUP.finish(
        native::heap(caps),
        result
            .as_ref()
            .map(|_| ())
            .map_err(|error| error.raw_os_error()),
    );
    result
        .map(PreparedStatistics)
        .map_err(|_| PreparationFailure::Spawn)
}

impl PreparedStatistics {
    pub(crate) fn activate(mut self) -> bool {
        if !self.0.activate() {
            return false;
        }
        diagnostics::STARTUP.active();
        log::info!("statistics_runtime=started cadence_ms={STATISTICS_CADENCE_MS}");
        true
    }
}

impl Drop for PreparedStatistics {
    fn drop(&mut self) {
        if self.0.is_prepared() {
            diagnostics::STARTUP.cancelled();
        }
    }
}

fn run() -> ! {
    let started_at_ms = crate::runtime_uptime::millis();
    let mut schedule = PeriodicDeadline::new(started_at_ms, STATISTICS_CADENCE_MS)
        .expect("statistics cadence is nonzero");
    if schedule.advance_past(started_at_ms).is_err() {
        halt_after_deadline_overflow();
    }

    loop {
        thread::sleep(duration_until(schedule.next_deadline_ms()));
        let now_ms = crate::runtime_uptime::millis();
        if !schedule.is_due(now_ms) {
            continue;
        }

        let frequency_seconds = crate::settings_adapter::statistics_frequency_seconds();
        crate::runtime_snapshot::record_statistics_sample(now_ms, frequency_seconds);
        let advance = match schedule.advance_past(crate::runtime_uptime::millis()) {
            Ok(advance) => advance,
            Err(_) => halt_after_deadline_overflow(),
        };
        if advance.missed_slots() > 0 {
            log::warn!(
                "statistics_runtime=overrun category=deadline_missed slots={}",
                advance.missed_slots()
            );
        }
    }
}

fn duration_until(deadline_ms: u64) -> Duration {
    Duration::from_millis(deadline_ms.saturating_sub(crate::runtime_uptime::millis()))
}

fn halt_after_deadline_overflow() -> ! {
    log::error!("statistics_runtime=fault category=deadline_overflow action=halt");
    loop {
        thread::park();
    }
}
