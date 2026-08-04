//! Sole producer for bounded runtime statistics history.

use std::{thread, time::Duration};

use anyhow::{Context, Result};
use bitaxe_core::runtime_orchestration::PeriodicDeadline;

pub const STATISTICS_CADENCE_MS: u64 = 1_000;
const PRODUCER_THREAD_NAME: &str = "statistics";
const PRODUCER_THREAD_STACK_BYTES: usize = 8 * 1024;

pub fn start() -> Result<()> {
    thread::Builder::new()
        .name(PRODUCER_THREAD_NAME.to_owned())
        .stack_size(PRODUCER_THREAD_STACK_BYTES)
        .spawn(run)
        .context("spawn statistics producer")?;
    log::info!("statistics_runtime=started cadence_ms={STATISTICS_CADENCE_MS}");
    Ok(())
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
