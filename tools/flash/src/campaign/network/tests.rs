use std::sync::{Arc, Mutex};

use bitaxe_api::boot_identity::ResetReasonCategory;
use bitaxe_api::{
    ApiSnapshot, ExpectedRuntimeAttestationIdentity, ObservationStateWire,
    OperatorSnapshotRevision, RuntimeBootAttestation, SystemInfoWire,
};
use bitaxe_http_transport::WebSocketReadFailureKind;

use super::super::{CampaignTerminalCategory, MiningCampaignStage};
use super::command_evidence::CommandEffectsEvidence;
use super::model::{
    CampaignNetworkEvidence, NetworkAccumulator, NetworkTransport, SharedSerialState,
    TrustedNetworkTarget, REQUIRED_WINDOWS, WINDOW_MILLIS,
};
use super::serial::NetworkSerialTracker;
use super::NetworkObservationMode;
mod terminal_handoff;

mod startup;
mod watchdog;

const SOURCE: &str = "1111111111111111111111111111111111111111";
const REFERENCE: &str = "2222222222222222222222222222222222222222";
const APP: &str = "3333333333333333333333333333333333333333333333333333333333333333";
const SESSION: &str = "44444444444444444444444444444444";

fn expected() -> ExpectedRuntimeAttestationIdentity {
    ExpectedRuntimeAttestationIdentity {
        firmware_commit: SOURCE.to_owned(),
        reference_commit: REFERENCE.to_owned(),
        app_elf_sha256: APP.to_owned(),
    }
}

#[test]
fn campaign_stages_have_one_closed_network_observation_policy() {
    // Arrange
    let stages = [
        MiningCampaignStage::Observation,
        MiningCampaignStage::JobTransition,
        MiningCampaignStage::LiveShare,
        MiningCampaignStage::Soak,
        MiningCampaignStage::CommandEffects,
    ];

    // Act
    let modes = stages.map(NetworkObservationMode::for_stage);

    // Assert
    assert_eq!(
        modes,
        [
            NetworkObservationMode::NotRequired,
            NetworkObservationMode::NotRequired,
            NetworkObservationMode::Continuity,
            NetworkObservationMode::Continuity,
            NetworkObservationMode::CommandEffects,
        ]
    );
}

fn target() -> TrustedNetworkTarget {
    TrustedNetworkTarget {
        origin: "http://127.0.0.1:80".to_owned(),
        boot_session: "00000000000000000000000000000000".to_owned(),
        boot_ordinal: 0,
        expected: ExpectedRuntimeAttestationIdentity {
            firmware_commit: "0".repeat(40),
            reference_commit: "0".repeat(40),
            app_elf_sha256: "0".repeat(64),
        },
    }
}

fn active_sample(revision: u64, sequence: u64) -> SystemInfoWire {
    active_sample_with_watchdog_sequences(revision, sequence, sequence)
}

fn active_sample_with_watchdog_sequences(
    revision: u64,
    checkpoint_sequence: u64,
    feed_sequence: u64,
) -> SystemInfoWire {
    let mut sample = SystemInfoWire::from_snapshot(&ApiSnapshot::safe_ultra_205());
    sample.operator_snapshot_revision =
        OperatorSnapshotRevision::new(revision).expect("revision must be nonzero");
    sample.mining_paused = false;
    sample.mining_activity = "active".to_owned();
    sample.start_mining_on_boot = false;
    sample.power = 10.0;
    sample.voltage_millivolts = 5_000.0;
    sample.current_milliamps = 2_000.0;
    sample.temp = 60.0;
    sample.fan_rpm = 3_000;
    sample.power_status.state = ObservationStateWire::Fresh;
    sample.voltage_status.state = ObservationStateWire::Fresh;
    sample.current_status.state = ObservationStateWire::Fresh;
    sample.chip_temp_status.state = ObservationStateWire::Fresh;
    sample.fan_rpm_status.state = ObservationStateWire::Fresh;
    sample.runtime_health.supervisor_availability = "available".to_owned();
    sample.runtime_health.checkpoint_health = "healthy".to_owned();
    sample.runtime_health.maybe_checkpoint_sequence = Some(checkpoint_sequence);
    sample.runtime_health.task_watchdog_participation = "participating".to_owned();
    sample.runtime_health.maybe_task_watchdog_reason = Some("feed_fresh".to_owned());
    sample.runtime_health.maybe_task_watchdog_feed_sequence = Some(feed_sequence);
    sample.runtime_health.maybe_task_watchdog_feed_age_millis = Some(100);
    sample.runtime_health.task_watchdog_owner_phase = "waiting_inbox".to_owned();
    sample
}

fn terminal_sample(revision: u64, sequence: u64) -> SystemInfoWire {
    let mut sample = active_sample(revision, sequence);
    sample.mining_paused = true;
    sample.mining_activity = "paused".to_owned();
    sample
}

#[test]
fn campaign_safety_accepts_exact_legacy_millivolt_boundaries() {
    // Arrange
    let target = target();
    let mut low = active_sample(1, 1);
    low.voltage_millivolts = 4_500.0;
    let mut high = active_sample(2, 2);
    high.voltage_millivolts = 5_500.0;

    // Act
    let results = [
        super::validation::validate_active_prerequisites(&low, &target),
        super::validation::validate_active_prerequisites(&high, &target),
    ];

    // Assert
    assert!(results.iter().all(Result::is_ok));
}

#[test]
fn campaign_safety_rejects_legacy_millivolts_outside_volt_domain() {
    // Arrange
    let target = target();
    let mut low = active_sample(1, 1);
    low.voltage_millivolts = 4_499.0;
    let mut high = active_sample(2, 2);
    high.voltage_millivolts = 5_501.0;

    // Act
    let results = [
        super::validation::validate_active_prerequisites(&low, &target),
        super::validation::validate_active_prerequisites(&high, &target),
    ];

    // Assert
    assert!(results.iter().all(Result::is_err));
}

fn complete_serial() -> SharedSerialState {
    let mut serial = SharedSerialState {
        latest_active_ms: 600_000,
        terminal_consumed: true,
        terminal_pool_persisted: true,
        maximum_active_marker_gap_ms: 1_000,
        ..SharedSerialState::default()
    };
    for window in &mut serial.serial_windows {
        window.observe(1);
        window.observe(2);
    }
    serial
}

fn record_complete_windows(accumulator: &mut NetworkAccumulator) {
    let mut revision = 1_u64;
    for index in 0..REQUIRED_WINDOWS {
        let active_ms = u64::try_from(index).expect("index fits") * WINDOW_MILLIS + 1_000;
        for _ in 0..2 {
            let sample = active_sample(revision, revision);
            accumulator.record_active_sample(NetworkTransport::Http, active_ms, active_ms, &sample);
            revision = revision.saturating_add(1);
            let sample = active_sample(revision, revision);
            accumulator.record_active_sample(
                NetworkTransport::WebSocket,
                active_ms,
                active_ms,
                &sample,
            );
            revision = revision.saturating_add(1);
        }
    }
}

#[test]
fn twenty_complete_windows_and_terminal_state_are_accepted() {
    // Arrange
    let mut accumulator = NetworkAccumulator::new(target());
    record_complete_windows(&mut accumulator);
    let terminal = terminal_sample(100, 100);
    accumulator.record_terminal_sample(NetworkTransport::Http, &terminal);
    accumulator.record_terminal_sample(NetworkTransport::WebSocket, &terminal);

    // Act
    let evidence = accumulator.finish(&complete_serial());

    // Assert
    assert_eq!(evidence.status, "accepted");
    assert_eq!(evidence.covered_window_count, REQUIRED_WINDOWS);
    assert_eq!(evidence.maybe_failure, None);
    assert!(evidence.watchdog_valid);
    assert_eq!(evidence.watchdog_failure, "none");
    assert_eq!(evidence.watchdog_owner_phase, "waiting_inbox");
    assert!(evidence.work_renewal_valid);
}

#[test]
fn unknown_watchdog_owner_phase_fails_closed_without_republishing_free_text() {
    // Arrange
    let mut accumulator = NetworkAccumulator::new(target());
    accumulator.record_active_sample(NetworkTransport::Http, 500, 500, &active_sample(1, 1));
    let mut sample = active_sample(2, 2);
    sample.runtime_health.task_watchdog_owner_phase = "private-phase-42".to_owned();

    // Act
    accumulator.record_active_sample(NetworkTransport::Http, 1_000, 1_000, &sample);
    let evidence = accumulator.finish(&complete_serial());

    // Assert
    assert_eq!(evidence.watchdog_failure, "watchdog_owner_phase_unknown");
    assert_eq!(evidence.watchdog_owner_phase, "unavailable");
}

#[test]
fn command_effects_do_not_inherit_soak_window_requirements() {
    // Arrange
    let effects = CommandEffectsEvidence::new();

    // Act
    let evidence = CampaignNetworkEvidence::from_command_effects(effects, 0, None, None);

    // Assert
    assert_eq!(evidence.required_window_count, 0);
    assert_eq!(evidence.covered_window_count, 0);
}

#[test]
fn missing_http_window_fails_at_the_exact_closed_boundary() {
    // Arrange
    let mut accumulator = NetworkAccumulator::new(target());
    let sample_one = active_sample(1, 1);
    let sample_two = active_sample(2, 2);
    accumulator.record_active_sample(NetworkTransport::WebSocket, 1_000, 1_000, &sample_one);
    accumulator.record_active_sample(NetworkTransport::WebSocket, 2_000, 2_000, &sample_two);
    let serial = complete_serial();

    // Act
    accumulator.close_elapsed_windows(WINDOW_MILLIS, &serial);

    // Assert
    assert_eq!(
        accumulator.maybe_failure,
        Some(CampaignTerminalCategory::HttpWindowIncomplete)
    );
}

#[test]
fn half_open_window_boundaries_assign_exactly_once() {
    // Arrange
    let boundaries = [0, 29_999, 30_000, 599_999, 600_000];

    // Act
    let indexes = boundaries.map(super::validation::window_index);

    // Assert
    assert_eq!(indexes, [0, 0, 1, 19, 19]);
}

#[test]
fn every_individually_missing_http_window_fails_closed() {
    for missing in 0..REQUIRED_WINDOWS {
        // Arrange
        let mut accumulator = NetworkAccumulator::new(target());
        let mut revision = 1_u64;
        for index in 0..REQUIRED_WINDOWS {
            let active_ms = u64::try_from(index).expect("index fits") * WINDOW_MILLIS + 1_000;
            for _ in 0..2 {
                if index != missing {
                    let sample = active_sample(revision, revision);
                    accumulator.record_active_sample(
                        NetworkTransport::Http,
                        active_ms,
                        active_ms,
                        &sample,
                    );
                    revision = revision.saturating_add(1);
                }
                let sample = active_sample(revision, revision);
                accumulator.record_active_sample(
                    NetworkTransport::WebSocket,
                    active_ms,
                    active_ms,
                    &sample,
                );
                revision = revision.saturating_add(1);
            }
        }

        // Act
        let evidence = accumulator.finish(&complete_serial());

        // Assert
        assert_eq!(
            evidence.maybe_failure,
            Some(CampaignTerminalCategory::HttpWindowIncomplete),
            "missing window {missing} must fail",
        );
    }
}

#[test]
fn snapshot_revision_regression_fails_correlation() {
    // Arrange
    let mut accumulator = NetworkAccumulator::new(target());
    accumulator.record_active_sample(NetworkTransport::Http, 1_000, 1_000, &active_sample(2, 1));

    // Act
    accumulator.record_active_sample(NetworkTransport::Http, 2_000, 2_000, &active_sample(1, 2));

    // Assert
    assert_eq!(
        accumulator.maybe_failure,
        Some(CampaignTerminalCategory::NetworkCorrelationFailed)
    );
}

#[test]
fn share_counter_regression_fails_correlation() {
    // Arrange
    let mut accumulator = NetworkAccumulator::new(target());
    let mut first = active_sample(1, 1);
    first.shares_accepted = 2;
    accumulator.record_active_sample(NetworkTransport::Http, 1_000, 1_000, &first);
    let mut regressed = active_sample(2, 2);
    regressed.shares_accepted = 1;

    // Act
    accumulator.record_active_sample(NetworkTransport::Http, 2_000, 2_000, &regressed);

    // Assert
    assert_eq!(
        accumulator.maybe_failure,
        Some(CampaignTerminalCategory::NetworkCorrelationFailed)
    );
}

#[test]
fn equal_snapshot_revisions_remain_correlated() {
    // Arrange
    let mut accumulator = NetworkAccumulator::new(target());

    // Act
    accumulator.record_active_sample(NetworkTransport::Http, 1_000, 1_000, &active_sample(2, 1));
    accumulator.record_active_sample(NetworkTransport::Http, 2_000, 2_000, &active_sample(2, 2));

    // Assert
    assert_eq!(accumulator.maybe_failure, None);
}

#[test]
fn missing_asic_poll_renewal_fails_the_closed_window() {
    // Arrange
    let mut accumulator = NetworkAccumulator::new(target());
    record_complete_windows(&mut accumulator);
    let mut serial = complete_serial();
    serial.serial_windows[7] = Default::default();

    // Act
    let evidence = accumulator.finish(&serial);

    // Assert
    assert_eq!(
        evidence.maybe_failure,
        Some(CampaignTerminalCategory::WorkRenewalMissing)
    );
}

#[test]
fn terminal_pool_reread_must_confirm_persistence() {
    // Arrange
    let mut accumulator = NetworkAccumulator::new(target());
    record_complete_windows(&mut accumulator);
    let terminal = terminal_sample(100, 100);
    accumulator.record_terminal_sample(NetworkTransport::Http, &terminal);
    accumulator.record_terminal_sample(NetworkTransport::WebSocket, &terminal);
    let mut serial = complete_serial();
    serial.terminal_pool_persisted = false;

    // Act
    let evidence = accumulator.finish(&serial);

    // Assert
    assert_eq!(
        evidence.maybe_failure,
        Some(CampaignTerminalCategory::PoolPersistenceUnconfirmed)
    );
}

#[test]
fn recovery_pause_is_one_shot_and_preserves_the_earliest_failure() {
    // Arrange
    let mut accumulator = NetworkAccumulator::new(target());
    accumulator.fail(CampaignTerminalCategory::WatchdogUnresponsive);

    // Act
    let first_request = accumulator.take_recovery_pause_request();
    accumulator.fail(CampaignTerminalCategory::TerminalStateUnconfirmed);
    let second_request = accumulator.take_recovery_pause_request();

    // Assert
    assert!(first_request);
    assert!(!second_request);
    assert_eq!(accumulator.recovery_pause_request_count, 1);
    assert_eq!(
        accumulator.maybe_failure,
        Some(CampaignTerminalCategory::WatchdogUnresponsive)
    );
}

#[test]
fn terminal_safe_stop_observations_remain_recorded_after_an_earlier_failure() {
    // Arrange
    let mut accumulator = NetworkAccumulator::new(target());
    accumulator.fail(CampaignTerminalCategory::HttpWindowIncomplete);
    let terminal = terminal_sample(100, 100);

    // Act
    accumulator.record_terminal_sample(NetworkTransport::Http, &terminal);
    accumulator.record_terminal_sample(NetworkTransport::WebSocket, &terminal);

    // Assert
    assert!(accumulator.terminal_http_valid);
    assert!(accumulator.terminal_websocket_valid);
    assert_eq!(
        accumulator.maybe_failure,
        Some(CampaignTerminalCategory::HttpWindowIncomplete)
    );
}

#[test]
fn same_session_repeated_origin_is_admitted_but_https_and_conflicts_are_rejected() {
    // Arrange
    let mut tracker = NetworkSerialTracker::new(expected());
    let shared = Arc::new(Mutex::new(SharedSerialState::default()));
    let first = RuntimeBootAttestation::new(
        SESSION,
        7,
        ResetReasonCategory::SoftwareCpu,
        1_000,
        SOURCE,
        REFERENCE,
        APP,
        "v5.5.4",
    )
    .expect("first attestation");
    let second = RuntimeBootAttestation::new(
        SESSION,
        7,
        ResetReasonCategory::SoftwareCpu,
        2_000,
        SOURCE,
        REFERENCE,
        APP,
        "v5.5.4",
    )
    .expect("second attestation");
    let accepted = format!(
        "{}\n{}\nruntime_origin session={SESSION} boot_ordinal=7 device_url=http://127.0.0.1:80 redacted=true\nruntime_origin session={SESSION} boot_ordinal=7 device_url=http://127.0.0.1:80 redacted=true\n",
        first.marker(),
        second.marker(),
    );

    // Act
    tracker.observe(accepted.as_bytes(), &shared);
    let target = tracker.maybe_trusted_target();
    tracker.observe(
        format!("runtime_origin session={SESSION} boot_ordinal=7 device_url=https://127.0.0.1 redacted=true\n").as_bytes(),
        &shared,
    );

    // Assert
    assert!(target.is_some());
    assert_eq!(
        shared.lock().expect("shared state").maybe_failure,
        Some(CampaignTerminalCategory::NetworkTargetUnavailable)
    );
}

#[test]
fn changed_session_attestation_after_admission_fails_closed() {
    // Arrange
    let mut tracker = NetworkSerialTracker::new(expected());
    let shared = Arc::new(Mutex::new(SharedSerialState::default()));
    let first = RuntimeBootAttestation::new(
        SESSION,
        7,
        ResetReasonCategory::SoftwareCpu,
        1_000,
        SOURCE,
        REFERENCE,
        APP,
        "v5.5.4",
    )
    .expect("first attestation");
    let second = RuntimeBootAttestation::new(
        SESSION,
        7,
        ResetReasonCategory::SoftwareCpu,
        2_000,
        SOURCE,
        REFERENCE,
        APP,
        "v5.5.4",
    )
    .expect("second attestation");
    let changed = RuntimeBootAttestation::new(
        "55555555555555555555555555555555",
        8,
        ResetReasonCategory::SoftwareCpu,
        3_000,
        SOURCE,
        REFERENCE,
        APP,
        "v5.5.4",
    )
    .expect("changed attestation");
    let admitted = format!(
        "{}\n{}\nruntime_origin session={SESSION} boot_ordinal=7 device_url=http://127.0.0.1:80 redacted=true\n",
        first.marker(),
        second.marker(),
    );
    tracker.observe(admitted.as_bytes(), &shared);
    assert!(tracker.maybe_trusted_target().is_some());

    // Act
    tracker.observe(format!("{}\n", changed.marker()).as_bytes(), &shared);

    // Assert
    assert_eq!(
        shared.lock().expect("shared state").maybe_failure,
        Some(CampaignTerminalCategory::NetworkTargetUnavailable)
    );
}

#[test]
fn network_evidence_serialization_contains_only_closed_aggregates() {
    // Arrange
    let mut accumulator = NetworkAccumulator::new(target());
    accumulator.note_websocket_connect_failure();
    accumulator.note_websocket_peer_close();
    accumulator.note_websocket_failure(WebSocketReadFailureKind::Io);
    accumulator.note_websocket_failure(WebSocketReadFailureKind::Protocol);
    accumulator.note_websocket_failure(WebSocketReadFailureKind::Capacity);
    accumulator.note_websocket_failure(WebSocketReadFailureKind::Other);
    record_complete_windows(&mut accumulator);
    let terminal = terminal_sample(100, 100);
    accumulator.record_terminal_sample(NetworkTransport::Http, &terminal);
    accumulator.record_terminal_sample(NetworkTransport::WebSocket, &terminal);
    let evidence = accumulator.finish(&complete_serial());

    // Act
    let encoded = serde_json::to_string(&evidence).expect("evidence should serialize");

    // Assert
    for prohibited in [
        "127.0.0.1",
        "device_url",
        "boot_session",
        "poolUser",
        "ssid",
        "windows",
        "sequence",
        "poll_request_count",
        "ConnectionReset",
        "ResetWithoutClosingHandshake",
        "AttackAttempt",
    ] {
        assert!(!encoded.contains(prohibited));
    }
    assert!(encoded.contains("mining-campaign-network-continuity-v7"));
    assert!(encoded.contains("http_startup_transition_count"));
    assert!(encoded.contains("websocket_startup_transition_count"));
    assert!(encoded.contains("http_initial_active_observed"));
    assert!(encoded.contains("websocket_initial_active_observed"));
    assert_eq!(evidence.websocket_connect_failure_count, 1);
    assert_eq!(evidence.websocket_peer_close_count, 1);
    assert_eq!(evidence.websocket_io_failure_count, 1);
    assert_eq!(evidence.websocket_protocol_failure_count, 1);
    assert_eq!(evidence.websocket_capacity_failure_count, 1);
    assert_eq!(evidence.websocket_other_failure_count, 1);
}
