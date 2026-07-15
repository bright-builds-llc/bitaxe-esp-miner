//! Session-scoped boot evidence and serial-only runtime heartbeat.

use std::{
    sync::{Mutex, OnceLock},
    thread,
    time::Duration,
};

use bitaxe_api::boot_identity::{
    boot_identity_marker, runtime_origin_marker, ResetReasonCategory, BOOT_EVIDENCE_INTERVAL_MS,
    ORIGIN_REPLAY_WINDOW_MS,
};
use bitaxe_api::logs::{
    RuntimeHeartbeatModel, ACCEPTED_STATE_REPLAY_INTERVAL_MS, ACCEPTED_STATE_REPLAY_WINDOW_MS,
};
use bitaxe_api::BootSessionId;
use esp_idf_svc::sys;

use crate::{asic_adapter, log_buffer, rtc_boot_ordinal, runtime_uptime};

static BOOT_SESSION: OnceLock<BootSessionNonce> = OnceLock::new();
static HEARTBEAT_MODEL: OnceLock<Mutex<RuntimeHeartbeatModel>> = OnceLock::new();
static BOOT_ORDINAL: OnceLock<u64> = OnceLock::new();
static RESET_REASON: OnceLock<ResetReasonCategory> = OnceLock::new();
static CONNECTED_ORIGIN: OnceLock<Mutex<Option<ConnectedOriginReplay>>> = OnceLock::new();
const OBSERVER_THREAD_STACK_BYTES: usize = 8 * 1024;
const OBSERVER_THREAD_NAME: &str = "runtime-observer";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct BootSessionNonce([u32; 4]);

#[derive(Debug, Clone, PartialEq, Eq)]
struct ConnectedOriginReplay {
    device_url: String,
    next_deadline_ms: u64,
    ends_at_ms: u64,
}

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

/// Creates the per-boot session and starts the sole boot-lifetime observer.
pub fn initialize_observer() {
    let nonce = *BOOT_SESSION.get_or_init(BootSessionNonce::from_hardware_rng);
    let reset_reason = *RESET_REASON.get_or_init(rtc_boot_ordinal::reset_reason_category);
    let transition = rtc_boot_ordinal::initialize(reset_reason);
    let ordinal = *BOOT_ORDINAL.get_or_init(|| transition.record.ordinal);
    HEARTBEAT_MODEL.get_or_init(|| Mutex::new(RuntimeHeartbeatModel::new(nonce.0)));
    CONNECTED_ORIGIN.get_or_init(|| Mutex::new(None));

    emit_boot_identity(nonce, ordinal, reset_reason, runtime_uptime::millis());

    let result = thread::Builder::new()
        .name(OBSERVER_THREAD_NAME.to_owned())
        .stack_size(OBSERVER_THREAD_STACK_BYTES)
        .spawn(observe_boot_lifetime);
    if let Err(error) = result {
        log::warn!("runtime_observer=unavailable reason=thread_spawn_failed error={error}");
    }
}

/// Publishes the connected HTTP origin into the bounded boot-evidence replay.
pub fn publish_connected_origin(device_url: String) {
    let now_ms = runtime_uptime::millis();
    let replay = ConnectedOriginReplay {
        device_url: device_url.clone(),
        next_deadline_ms: now_ms.saturating_add(BOOT_EVIDENCE_INTERVAL_MS),
        ends_at_ms: now_ms.saturating_add(ORIGIN_REPLAY_WINDOW_MS),
    };
    let cell = CONNECTED_ORIGIN.get_or_init(|| Mutex::new(None));
    let Ok(mut maybe_replay) = cell.lock() else {
        log::warn!("runtime_origin=unavailable reason=mutex_poisoned");
        return;
    };
    *maybe_replay = Some(replay);
    drop(maybe_replay);
    emit_runtime_origin(&device_url);
}

/// Records boot proof in Plan 13 evidence mode.
pub fn record_booted() {
    if !asic_adapter::accepted_state_snapshot_enabled() {
        return;
    }
    record(boot_session(), BootEvidenceState::Booted);
}

/// Latches listener readiness and conditionally records dedicated Plan 13 proof.
pub fn record_listener_armed() {
    let model = heartbeat_model();
    let Ok(mut model) = model.lock() else {
        log::warn!("runtime_heartbeat=unavailable reason=mutex_poisoned");
        return;
    };
    model.arm_listener();
    drop(model);

    if asic_adapter::accepted_state_snapshot_enabled() {
        record(boot_session(), BootEvidenceState::ListenerArmed);
    }
}

/// Returns the typed operator-snapshot session backed by the existing hardware-RNG nonce.
pub fn operator_snapshot_boot_session() -> BootSessionId {
    BootSessionId::from_words(boot_session().0)
}

fn boot_session() -> BootSessionNonce {
    *BOOT_SESSION.get_or_init(BootSessionNonce::from_hardware_rng)
}

fn heartbeat_model() -> &'static Mutex<RuntimeHeartbeatModel> {
    HEARTBEAT_MODEL.get_or_init(|| Mutex::new(RuntimeHeartbeatModel::new(boot_session().0)))
}

fn record(nonce: BootSessionNonce, state: BootEvidenceState) {
    let marker = evidence_marker(nonce, state);
    log::info!("{marker}");
    log_buffer::append_runtime_log_line(&marker);
}

fn observe_boot_lifetime() {
    let replay_enabled = asic_adapter::accepted_state_snapshot_enabled();
    let started_at_ms = runtime_uptime::millis();
    let replay_ends_at_ms = started_at_ms.saturating_add(ACCEPTED_STATE_REPLAY_WINDOW_MS);
    let mut maybe_replay_deadline_ms =
        replay_enabled.then(|| started_at_ms.saturating_add(ACCEPTED_STATE_REPLAY_INTERVAL_MS));
    let mut identity_deadline_ms = started_at_ms.saturating_add(BOOT_EVIDENCE_INTERVAL_MS);

    loop {
        let now_ms = runtime_uptime::millis();
        emit_due_heartbeat(now_ms);
        if now_ms >= identity_deadline_ms {
            emit_boot_identity(boot_session(), boot_ordinal(), reset_reason(), now_ms);
            identity_deadline_ms = now_ms.saturating_add(BOOT_EVIDENCE_INTERVAL_MS);
        }
        emit_due_runtime_origin(now_ms);

        if maybe_replay_deadline_ms
            .is_some_and(|deadline_ms| now_ms >= deadline_ms && now_ms < replay_ends_at_ms)
        {
            for line in log_buffer::accepted_state_replay_lines() {
                log::info!("{line}");
            }
            maybe_replay_deadline_ms = Some(
                now_ms
                    .saturating_add(ACCEPTED_STATE_REPLAY_INTERVAL_MS)
                    .min(replay_ends_at_ms),
            );
        }
        if maybe_replay_deadline_ms == Some(replay_ends_at_ms) {
            maybe_replay_deadline_ms = None;
        }

        let next_heartbeat_ms = next_heartbeat_deadline();
        let next_wake_ms = maybe_replay_deadline_ms
            .map_or(next_heartbeat_ms, |replay_ms| {
                replay_ms.min(next_heartbeat_ms)
            })
            .min(identity_deadline_ms)
            .min(next_origin_deadline());
        let sleep_ms = next_wake_ms.saturating_sub(runtime_uptime::millis());
        if sleep_ms > 0 {
            thread::sleep(Duration::from_millis(sleep_ms));
        } else {
            thread::yield_now();
        }
    }
}

fn boot_ordinal() -> u64 {
    *BOOT_ORDINAL
        .get()
        .expect("boot ordinal is initialized before the observer starts")
}

fn reset_reason() -> ResetReasonCategory {
    *RESET_REASON
        .get()
        .expect("reset reason is initialized before the observer starts")
}

fn emit_boot_identity(
    nonce: BootSessionNonce,
    ordinal: u64,
    reason: ResetReasonCategory,
    uptime_ms: u64,
) {
    let marker = boot_identity_marker(nonce.0, ordinal, reason, uptime_ms);
    log::info!("{marker}");
}

fn emit_runtime_origin(device_url: &str) {
    let marker = runtime_origin_marker(boot_session().0, boot_ordinal(), device_url);
    log::info!("{marker}");
}

fn emit_due_runtime_origin(now_ms: u64) {
    let cell = CONNECTED_ORIGIN.get_or_init(|| Mutex::new(None));
    let Ok(mut maybe_replay) = cell.lock() else {
        return;
    };
    let Some(replay) = maybe_replay.as_mut() else {
        return;
    };
    if now_ms >= replay.ends_at_ms {
        *maybe_replay = None;
        return;
    }
    if now_ms >= replay.next_deadline_ms {
        let device_url = replay.device_url.clone();
        replay.next_deadline_ms = now_ms.saturating_add(BOOT_EVIDENCE_INTERVAL_MS);
        drop(maybe_replay);
        emit_runtime_origin(&device_url);
    }
}

fn next_origin_deadline() -> u64 {
    let cell = CONNECTED_ORIGIN.get_or_init(|| Mutex::new(None));
    let Ok(maybe_replay) = cell.lock() else {
        return u64::MAX;
    };
    maybe_replay.as_ref().map_or(u64::MAX, |replay| {
        replay.next_deadline_ms.min(replay.ends_at_ms)
    })
}

fn emit_due_heartbeat(now_ms: u64) {
    let model = heartbeat_model();
    let Ok(mut model) = model.lock() else {
        log::warn!("runtime_heartbeat=unavailable reason=mutex_poisoned");
        return;
    };
    let maybe_sample = model.take_due(now_ms);
    drop(model);

    if let Some(sample) = maybe_sample {
        let marker = sample.marker();
        log::info!("{marker}");
    }
}

fn next_heartbeat_deadline() -> u64 {
    let model = heartbeat_model();
    let Ok(model) = model.lock() else {
        return runtime_uptime::millis().saturating_add(1_000);
    };
    model.next_deadline_ms()
}

fn evidence_marker(nonce: BootSessionNonce, state: BootEvidenceState) -> String {
    format!(
        "plan13_boot_evidence session={} state={} redacted=true",
        nonce.as_hex(),
        state.label()
    )
}

#[cfg(test)]
mod tests {
    use super::{evidence_marker, BootEvidenceState, BootSessionNonce};

    #[test]
    fn boot_evidence_marker_is_fixed_width_and_redacted() {
        // Arrange
        let nonce = BootSessionNonce([0, 1, u32::MAX, 0x1234_abcd]);

        // Act
        let marker = evidence_marker(nonce, BootEvidenceState::ListenerArmed);

        // Assert
        assert_eq!(
            marker,
            "plan13_boot_evidence session=0000000000000001ffffffff1234abcd state=listener_armed redacted=true"
        );
    }
}
