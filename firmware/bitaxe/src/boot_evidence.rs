//! Session-scoped boot evidence and serial-only runtime heartbeat.

use std::{
    sync::{
        atomic::{AtomicBool, Ordering},
        Mutex, OnceLock,
    },
    thread,
    time::Duration,
};

use bitaxe_api::boot_identity::{
    boot_identity_marker, runtime_origin_marker, ResetReasonCategory, BOOT_EVIDENCE_INTERVAL_MS,
};
use bitaxe_api::logs::{
    RuntimeHeartbeatModel, ACCEPTED_STATE_REPLAY_INTERVAL_MS, ACCEPTED_STATE_REPLAY_WINDOW_MS,
};
use bitaxe_api::{
    provisioning::PROVISIONING_NETWORK_READY_MARKER, BootSessionId, RuntimeBootAttestation,
    UsbBootBaseline, UsbBootProfileReason, UsbBootTransport,
};
use esp_idf_svc::sys;

use crate::{asic_adapter, log_buffer, rtc_boot_ordinal, runtime_uptime};

mod usb_profile;

static BOOT_SESSION: OnceLock<BootSessionNonce> = OnceLock::new();
static HEARTBEAT_MODEL: OnceLock<Mutex<RuntimeHeartbeatModel>> = OnceLock::new();
static BOOT_ORDINAL: OnceLock<u64> = OnceLock::new();
static RESET_REASON: OnceLock<ResetReasonCategory> = OnceLock::new();
static CONNECTED_ORIGIN: OnceLock<Mutex<Option<ConnectedOriginReplay>>> = OnceLock::new();
static RUNTIME_ATTESTATION: OnceLock<Mutex<Option<RuntimeAttestationReplay>>> = OnceLock::new();
static PROVISIONING_NETWORK_READY: OnceLock<Mutex<Option<u64>>> = OnceLock::new();
static SELF_TEST_RECEIPT: OnceLock<Mutex<Option<SelfTestReceiptReplay>>> = OnceLock::new();
static DIAGNOSTIC_REPLAY_REQUESTED: AtomicBool = AtomicBool::new(false);
const OBSERVER_THREAD_STACK_BYTES: usize = 8 * 1024;
const OBSERVER_THREAD_NAME: &str = "runtime-observer";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct BootSessionNonce([u32; 4]);

#[derive(Debug, Clone, PartialEq, Eq)]
struct ConnectedOriginReplay {
    device_url: String,
    next_deadline_ms: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct SelfTestReceiptReplay {
    marker: String,
    next_deadline_ms: u64,
}

impl SelfTestReceiptReplay {
    fn new(marker: String, now_ms: u64) -> Self {
        Self {
            marker,
            next_deadline_ms: now_ms.saturating_add(BOOT_EVIDENCE_INTERVAL_MS),
        }
    }

    fn maybe_take_due(&mut self, now_ms: u64) -> Option<String> {
        if now_ms < self.next_deadline_ms {
            return None;
        }
        self.next_deadline_ms = now_ms.saturating_add(BOOT_EVIDENCE_INTERVAL_MS);
        Some(self.marker.clone())
    }
}

impl ConnectedOriginReplay {
    fn new(device_url: String, now_ms: u64) -> Self {
        Self {
            device_url,
            next_deadline_ms: now_ms.saturating_add(BOOT_EVIDENCE_INTERVAL_MS),
        }
    }

    fn maybe_take_due(&mut self, now_ms: u64) -> Option<String> {
        if now_ms < self.next_deadline_ms {
            return None;
        }
        self.next_deadline_ms = now_ms.saturating_add(BOOT_EVIDENCE_INTERVAL_MS);
        Some(self.device_url.clone())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct RuntimeAttestationIdentity {
    firmware_commit: String,
    reference_commit: String,
    app_elf_sha256: String,
    esp_idf_version: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct RuntimeAttestationReplay {
    identity: RuntimeAttestationIdentity,
    next_deadline_ms: u64,
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
}

impl BootEvidenceState {
    const fn label(self) -> &'static str {
        match self {
            Self::Booted => "booted",
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
    RUNTIME_ATTESTATION.get_or_init(|| Mutex::new(None));
    PROVISIONING_NETWORK_READY.get_or_init(|| Mutex::new(None));
    SELF_TEST_RECEIPT.get_or_init(|| Mutex::new(None));
    usb_profile::initialize();
    if asic_adapter::accepted_state_snapshot_enabled() {
        request_diagnostic_replay();
    }

    emit_boot_identity(nonce, ordinal, reset_reason, runtime_uptime::millis());

    let result = thread::Builder::new()
        .name(OBSERVER_THREAD_NAME.to_owned())
        .stack_size(OBSERVER_THREAD_STACK_BYTES)
        .spawn(observe_boot_lifetime);
    if let Err(error) = result {
        log::warn!("runtime_observer=unavailable reason=thread_spawn_failed error={error}");
    }
}

/// Registers a validated persisted self-test receipt for serial-only replay.
pub fn register_self_test_receipt(lease: u64, outcome: &'static str) {
    if lease == 0 || !matches!(outcome, "cancelled" | "passed") {
        log::warn!("self_test_receipt_replay=unavailable reason=invalid_receipt");
        return;
    }
    let marker = format!("self_test_receipt outcome={outcome} lease={lease:016x}");
    let cell = SELF_TEST_RECEIPT.get_or_init(|| Mutex::new(None));
    let Ok(mut maybe_replay) = cell.lock() else {
        log::warn!("self_test_receipt_replay=unavailable reason=mutex_poisoned");
        return;
    };
    *maybe_replay = Some(SelfTestReceiptReplay::new(marker, runtime_uptime::millis()));
}

/// Requests bounded replay of privacy-safe retained diagnostics for this boot.
pub fn request_diagnostic_replay() {
    DIAGNOSTIC_REPLAY_REQUESTED.store(true, Ordering::Release);
}

/// Publishes and replays the selected USB transport and application-owner reason.
pub fn publish_usb_boot_profile(
    transport: UsbBootTransport,
    reason: UsbBootProfileReason,
    baseline: UsbBootBaseline,
) {
    usb_profile::publish(
        transport,
        reason,
        baseline,
        crate::firmware_commit().to_owned(),
        crate::app_elf_sha256(),
        boot_ordinal(),
        runtime_uptime::millis(),
        BOOT_EVIDENCE_INTERVAL_MS,
    );
}

/// Begins replaying exact-package ready-state proof for late monitor attachment.
pub fn publish_runtime_boot_attestation(
    firmware_commit: &str,
    reference_commit: &str,
    app_elf_sha256: &str,
    esp_idf_version: &str,
) {
    let identity = RuntimeAttestationIdentity {
        firmware_commit: firmware_commit.to_owned(),
        reference_commit: reference_commit.to_owned(),
        app_elf_sha256: app_elf_sha256.to_owned(),
        esp_idf_version: esp_idf_version.to_owned(),
    };
    if let Err(error) = runtime_attestation(&identity, runtime_uptime::millis()) {
        log::warn!("runtime_boot_attestation=unavailable reason=invalid_identity error={error}");
        return;
    }

    let now_ms = runtime_uptime::millis();
    let replay = RuntimeAttestationReplay {
        identity: identity.clone(),
        next_deadline_ms: now_ms.saturating_add(BOOT_EVIDENCE_INTERVAL_MS),
    };
    let cell = RUNTIME_ATTESTATION.get_or_init(|| Mutex::new(None));
    let Ok(mut maybe_replay) = cell.lock() else {
        log::warn!("runtime_boot_attestation=unavailable reason=mutex_poisoned");
        return;
    };
    *maybe_replay = Some(replay);
    drop(maybe_replay);
    emit_runtime_attestation(&identity, now_ms);
}

/// Begins recurring closed-category proof after AP, DHCP, and DNS readiness.
pub fn publish_provisioning_network_ready() {
    let now_ms = runtime_uptime::millis();
    let cell = PROVISIONING_NETWORK_READY.get_or_init(|| Mutex::new(None));
    let Ok(mut maybe_deadline) = cell.lock() else {
        log::warn!("provisioning_network_ready=unavailable reason=mutex_poisoned");
        return;
    };
    *maybe_deadline = Some(now_ms.saturating_add(BOOT_EVIDENCE_INTERVAL_MS));
    drop(maybe_deadline);
    emit_provisioning_network_ready();
}

/// Stops readiness replay whenever the configuration network is no longer active.
pub fn clear_provisioning_network_ready() {
    let cell = PROVISIONING_NETWORK_READY.get_or_init(|| Mutex::new(None));
    let Ok(mut maybe_deadline) = cell.lock() else {
        log::warn!("provisioning_network_ready=unavailable reason=mutex_poisoned");
        return;
    };
    *maybe_deadline = None;
}

/// Publishes the currently connected HTTP origin for private late USB observers.
///
/// Unlike effect deadlines, this observation must remain available for the full
/// connection lifetime: a human may begin an independently replayable display
/// UAT hours after the programmatic campaign completed.
pub fn publish_connected_origin(device_url: String) {
    let now_ms = runtime_uptime::millis();
    let replay = ConnectedOriginReplay::new(device_url.clone(), now_ms);
    let cell = CONNECTED_ORIGIN.get_or_init(|| Mutex::new(None));
    let Ok(mut maybe_replay) = cell.lock() else {
        log::warn!("runtime_origin=unavailable reason=mutex_poisoned");
        return;
    };
    *maybe_replay = Some(replay);
    drop(maybe_replay);
    emit_runtime_origin(&device_url);
}

/// Stops publishing an origin as soon as station connectivity is lost.
pub fn clear_connected_origin() {
    let cell = CONNECTED_ORIGIN.get_or_init(|| Mutex::new(None));
    let Ok(mut maybe_replay) = cell.lock() else {
        log::warn!("runtime_origin=unavailable reason=mutex_poisoned");
        return;
    };
    *maybe_replay = None;
}

/// Records boot proof in Plan 13 evidence mode.
pub fn record_booted() {
    if !asic_adapter::accepted_state_snapshot_enabled() {
        return;
    }
    record(boot_session(), BootEvidenceState::Booted);
}

/// Returns the typed operator-snapshot session backed by the existing hardware-RNG nonce.
pub fn operator_snapshot_boot_session() -> BootSessionId {
    BootSessionId::from_words(boot_session().0)
}

/// Returns the reset-retained ordinal for the current boot.
pub fn operator_snapshot_boot_ordinal() -> u64 {
    boot_ordinal()
}

/// Returns the closed reset category for the current boot.
pub fn operator_snapshot_reset_reason_category() -> ResetReasonCategory {
    reset_reason()
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
    let started_at_ms = runtime_uptime::millis();
    let mut replay_ends_at_ms = started_at_ms;
    let mut replay_started = false;
    let mut maybe_replay_deadline_ms = None;
    let mut identity_deadline_ms = started_at_ms.saturating_add(BOOT_EVIDENCE_INTERVAL_MS);

    loop {
        let now_ms = runtime_uptime::millis();
        if !replay_started && DIAGNOSTIC_REPLAY_REQUESTED.load(Ordering::Acquire) {
            replay_started = true;
            replay_ends_at_ms = now_ms.saturating_add(ACCEPTED_STATE_REPLAY_WINDOW_MS);
            // Replaying immediately and then periodically closes the monitor-attachment
            // race without extending the bounded diagnostic lifetime.
            maybe_replay_deadline_ms = Some(now_ms);
        }
        emit_due_heartbeat(now_ms);
        if now_ms >= identity_deadline_ms {
            emit_boot_identity(boot_session(), boot_ordinal(), reset_reason(), now_ms);
            identity_deadline_ms = now_ms.saturating_add(BOOT_EVIDENCE_INTERVAL_MS);
        }
        emit_due_runtime_origin(now_ms);
        emit_due_runtime_attestation(now_ms);
        emit_due_provisioning_network_ready(now_ms);
        emit_due_self_test_receipt(now_ms);
        usb_profile::emit_due(now_ms);

        if maybe_replay_deadline_ms
            .is_some_and(|deadline_ms| now_ms >= deadline_ms && now_ms < replay_ends_at_ms)
        {
            for line in log_buffer::diagnostic_replay_lines() {
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
            .min(next_origin_deadline())
            .min(next_attestation_deadline());
        let next_wake_ms = next_wake_ms.min(next_provisioning_network_ready_deadline());
        let next_wake_ms = next_wake_ms.min(next_self_test_receipt_deadline());
        let next_wake_ms = next_wake_ms.min(usb_profile::next_deadline());
        let sleep_ms = next_wake_ms.saturating_sub(runtime_uptime::millis());
        if sleep_ms > 0 {
            thread::sleep(Duration::from_millis(sleep_ms));
        } else {
            thread::yield_now();
        }
    }
}

fn emit_due_self_test_receipt(now_ms: u64) {
    let cell = SELF_TEST_RECEIPT.get_or_init(|| Mutex::new(None));
    let Ok(mut maybe_replay) = cell.lock() else {
        return;
    };
    let Some(replay) = maybe_replay.as_mut() else {
        return;
    };
    if let Some(marker) = replay.maybe_take_due(now_ms) {
        drop(maybe_replay);
        log::info!("{marker}");
    }
}

fn next_self_test_receipt_deadline() -> u64 {
    let cell = SELF_TEST_RECEIPT.get_or_init(|| Mutex::new(None));
    let Ok(maybe_replay) = cell.lock() else {
        return u64::MAX;
    };
    maybe_replay
        .as_ref()
        .map_or(u64::MAX, |replay| replay.next_deadline_ms)
}

fn emit_provisioning_network_ready() {
    log::info!("{PROVISIONING_NETWORK_READY_MARKER}");
}

fn emit_due_provisioning_network_ready(now_ms: u64) {
    let cell = PROVISIONING_NETWORK_READY.get_or_init(|| Mutex::new(None));
    let Ok(mut maybe_deadline) = cell.lock() else {
        return;
    };
    let Some(deadline) = *maybe_deadline else {
        return;
    };
    if now_ms < deadline {
        return;
    }
    *maybe_deadline = Some(now_ms.saturating_add(BOOT_EVIDENCE_INTERVAL_MS));
    drop(maybe_deadline);
    emit_provisioning_network_ready();
}

fn next_provisioning_network_ready_deadline() -> u64 {
    let cell = PROVISIONING_NETWORK_READY.get_or_init(|| Mutex::new(None));
    let Ok(maybe_deadline) = cell.lock() else {
        return u64::MAX;
    };
    maybe_deadline.unwrap_or(u64::MAX)
}

fn runtime_attestation(
    identity: &RuntimeAttestationIdentity,
    uptime_ms: u64,
) -> Result<RuntimeBootAttestation, bitaxe_api::RuntimeBootAttestationError> {
    RuntimeBootAttestation::new(
        &boot_session().as_hex(),
        boot_ordinal(),
        reset_reason(),
        uptime_ms,
        &identity.firmware_commit,
        &identity.reference_commit,
        &identity.app_elf_sha256,
        &identity.esp_idf_version,
    )
}

fn emit_runtime_attestation(identity: &RuntimeAttestationIdentity, uptime_ms: u64) {
    match runtime_attestation(identity, uptime_ms) {
        Ok(attestation) => log::info!("{}", attestation.marker()),
        Err(error) => {
            log::warn!(
                "runtime_boot_attestation=unavailable reason=invalid_identity error={error}"
            );
        }
    }
}

fn emit_due_runtime_attestation(now_ms: u64) {
    let cell = RUNTIME_ATTESTATION.get_or_init(|| Mutex::new(None));
    let Ok(mut maybe_replay) = cell.lock() else {
        return;
    };
    let Some(replay) = maybe_replay.as_mut() else {
        return;
    };
    if now_ms < replay.next_deadline_ms {
        return;
    }
    let identity = replay.identity.clone();
    replay.next_deadline_ms = now_ms.saturating_add(BOOT_EVIDENCE_INTERVAL_MS);
    drop(maybe_replay);
    emit_runtime_attestation(&identity, now_ms);
}

fn next_attestation_deadline() -> u64 {
    let cell = RUNTIME_ATTESTATION.get_or_init(|| Mutex::new(None));
    let Ok(maybe_replay) = cell.lock() else {
        return u64::MAX;
    };
    maybe_replay
        .as_ref()
        .map_or(u64::MAX, |replay| replay.next_deadline_ms)
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
    if let Some(device_url) = replay.maybe_take_due(now_ms) {
        drop(maybe_replay);
        emit_runtime_origin(&device_url);
    }
}

fn next_origin_deadline() -> u64 {
    let cell = CONNECTED_ORIGIN.get_or_init(|| Mutex::new(None));
    let Ok(maybe_replay) = cell.lock() else {
        return u64::MAX;
    };
    maybe_replay
        .as_ref()
        .map_or(u64::MAX, |replay| replay.next_deadline_ms)
}

fn emit_due_heartbeat(now_ms: u64) {
    let model = heartbeat_model();
    let Ok(mut model) = model.lock() else {
        log::warn!("runtime_heartbeat=unavailable reason=mutex_poisoned");
        return;
    };
    let maybe_sample = model.maybe_take_due(now_ms);
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
    use super::{evidence_marker, BootEvidenceState, BootSessionNonce, ConnectedOriginReplay};

    #[test]
    fn boot_evidence_marker_is_fixed_width_and_redacted() {
        // Arrange
        let nonce = BootSessionNonce([0, 1, u32::MAX, 0x1234_abcd]);

        // Act
        let marker = evidence_marker(nonce, BootEvidenceState::Booted);

        // Assert
        assert_eq!(
            marker,
            "plan13_boot_evidence session=0000000000000001ffffffff1234abcd state=booted redacted=true"
        );
    }

    #[test]
    fn connected_origin_remains_due_after_an_unbounded_human_delay() {
        // Arrange
        let mut replay = ConnectedOriginReplay::new("http://private-device".to_owned(), 1_000);

        // Act
        let observed = replay.maybe_take_due(24 * 60 * 60 * 1_000);

        // Assert
        assert_eq!(observed.as_deref(), Some("http://private-device"));
        assert!(replay.next_deadline_ms > 24 * 60 * 60 * 1_000);
    }
}
