//! Session-scoped, replayable Plan 13 boot evidence.

use std::{
    sync::OnceLock,
    thread,
    time::{Duration, Instant},
};

use bitaxe_api::logs::{ACCEPTED_STATE_REPLAY_INTERVAL_MS, ACCEPTED_STATE_REPLAY_WINDOW_MS};
use esp_idf_svc::sys;

use crate::{asic_adapter, log_buffer};

static BOOT_SESSION: OnceLock<BootSessionNonce> = OnceLock::new();
const REPLAY_THREAD_STACK_BYTES: usize = 8 * 1024;
const REPLAY_THREAD_NAME: &str = "plan13-boot-replay";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct BootSessionNonce([u32; 4]);

impl BootSessionNonce {
    fn from_hardware_rng() -> Self {
        Self([
            unsafe { sys::esp_random() },
            unsafe { sys::esp_random() },
            unsafe { sys::esp_random() },
            unsafe { sys::esp_random() },
        ])
    }

    fn as_hex(self) -> String {
        let [first, second, third, fourth] = self.0;
        format!("{first:08x}{second:08x}{third:08x}{fourth:08x}")
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum BootEvidenceState {
    Booted,
    ListenerArmed,
}

impl BootEvidenceState {
    const fn label(self) -> &'static str {
        match self {
            Self::Booted => "booted",
            Self::ListenerArmed => "listener_armed",
        }
    }
}

/// Generates the per-boot nonce and records boot proof in Plan 13 evidence mode.
pub fn initialize_and_record_booted() {
    if !asic_adapter::accepted_state_snapshot_enabled() {
        return;
    }
    let nonce = BOOT_SESSION.get_or_init(BootSessionNonce::from_hardware_rng);
    record(*nonce, BootEvidenceState::Booted);
    start_replay_task();
}

/// Records listener readiness against the same per-boot nonce.
pub fn record_listener_armed() {
    if !asic_adapter::accepted_state_snapshot_enabled() {
        return;
    }
    let nonce = BOOT_SESSION.get_or_init(BootSessionNonce::from_hardware_rng);
    record(*nonce, BootEvidenceState::ListenerArmed);
}

fn record(nonce: BootSessionNonce, state: BootEvidenceState) {
    let marker = marker(nonce, state);
    log::info!("{marker}");
    log_buffer::append_runtime_log_line(&marker);
}

fn start_replay_task() {
    let result = thread::Builder::new()
        .name(REPLAY_THREAD_NAME.to_owned())
        .stack_size(REPLAY_THREAD_STACK_BYTES)
        .spawn(replay_until_window_end);
    if let Err(error) = result {
        log::warn!(
            "plan13_boot_evidence_replay=unavailable reason=thread_spawn_failed error={error}"
        );
    }
}

fn replay_until_window_end() {
    let started_at = Instant::now();
    let mut tick = 1;
    while let Some(deadline) = replay_deadline(tick) {
        let elapsed = started_at.elapsed();
        if elapsed >= Duration::from_millis(ACCEPTED_STATE_REPLAY_WINDOW_MS) {
            return;
        }
        if deadline > elapsed {
            thread::sleep(deadline - elapsed);
        }
        if started_at.elapsed() >= Duration::from_millis(ACCEPTED_STATE_REPLAY_WINDOW_MS) {
            return;
        }
        for line in log_buffer::accepted_state_replay_lines() {
            log::info!("{line}");
        }
        tick += 1;
    }
}

fn replay_deadline(tick: u64) -> Option<Duration> {
    let elapsed_ms = ACCEPTED_STATE_REPLAY_INTERVAL_MS.checked_mul(tick)?;
    (elapsed_ms < ACCEPTED_STATE_REPLAY_WINDOW_MS).then(|| Duration::from_millis(elapsed_ms))
}

fn marker(nonce: BootSessionNonce, state: BootEvidenceState) -> String {
    format!(
        "plan13_boot_evidence session={} state={} redacted=true",
        nonce.as_hex(),
        state.label()
    )
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use super::{marker, replay_deadline, BootEvidenceState, BootSessionNonce};

    #[test]
    fn boot_evidence_marker_is_fixed_width_and_redacted() {
        // Arrange
        let nonce = BootSessionNonce([0, 1, u32::MAX, 0x1234_abcd]);

        // Act
        let marker = marker(nonce, BootEvidenceState::ListenerArmed);

        // Assert
        assert_eq!(
            marker,
            "plan13_boot_evidence session=0000000000000001ffffffff1234abcd state=listener_armed redacted=true"
        );
    }

    #[test]
    fn replay_schedule_stays_inside_the_plan13_window() {
        // Arrange
        let first_tick = 1;
        let latest_tick = 187;
        let expired_tick = 188;

        // Act
        let first_deadline = replay_deadline(first_tick);
        let latest_deadline = replay_deadline(latest_tick);
        let expired_deadline = replay_deadline(expired_tick);

        // Assert
        assert_eq!(first_deadline, Some(Duration::from_millis(10_000)));
        assert_eq!(latest_deadline, Some(Duration::from_millis(1_870_000)));
        assert_eq!(expired_deadline, None);
    }
}
