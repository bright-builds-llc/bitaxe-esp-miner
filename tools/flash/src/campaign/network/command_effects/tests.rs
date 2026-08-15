use std::fs;
use std::io::{Read, Write};
use std::net::TcpListener;

use bitaxe_api::{
    ApiSnapshot, BootSessionId, CommandStatusEffect, CommandStatusFacts, CommandStatusTracker,
    DisplayFrameKind, DisplayRenderOutcome, ExpectedRuntimeAttestationIdentity, SystemInfoWire,
};
use bitaxe_http_transport::StrictHttpClient;
use camino::Utf8PathBuf;

use super::{
    advance_commands, advance_programmatic_commands, arm_cleared_after_natural_expiry,
    arm_identify_transaction, arm_ready_after_paused_dismissal, automated_phase_failure,
    command_state_failure_cause, consume_checkpoint_response, consume_cleared_signal,
    consume_ready_signal, finish_identify_observation, may_reuse_confirmed_safe_stop,
    post_may_have_applied, rendered_checkpoint_action, respond_identify_checkpoint,
    serial_ended_before_terminal, take_recovery_pause_request, terminal_confirmation_timed_out,
    write_required_checkpoint, CheckpointResponse, CommandEffectsEvidence, CommandGenerations,
    CommandPhase, CommandProgress, IdentifyCheckpointKind, IdentifyCheckpointOutcome,
    PauseJoinState, RenderedCheckpointAction, HTTP_DEADLINE, TERMINAL_DEADLINE,
};
use crate::campaign::network::command_witness::CommandTransitionWitness;
use crate::campaign::network::model::{SharedSerialState, TrustedNetworkTarget};
use crate::campaign::CampaignTerminalCategory;
use crate::set_private_directory_mode;

mod continuity;
mod programmatic;
mod replay;
mod sample_validation;
mod terminal;

fn private_root() -> (tempfile::TempDir, Utf8PathBuf) {
    let temp = tempfile::tempdir().expect("tempdir");
    let root = Utf8PathBuf::from_path_buf(temp.path().join("attempt")).expect("utf8 attempt path");
    fs::create_dir(&root).expect("create attempt");
    set_private_directory_mode(&root).expect("private attempt");
    (temp, root)
}

fn trusted_target() -> TrustedNetworkTarget {
    TrustedNetworkTarget {
        origin: "http://127.0.0.1:9".to_owned(),
        boot_session: "0".repeat(32),
        boot_ordinal: 1,
        expected: ExpectedRuntimeAttestationIdentity {
            firmware_commit: "0".repeat(40),
            reference_commit: "0".repeat(40),
            app_elf_sha256: "0".repeat(64),
        },
    }
}

fn successful_command_server() -> (StrictHttpClient, std::thread::JoinHandle<String>) {
    let listener = TcpListener::bind("127.0.0.1:0").expect("bind command server");
    let address = listener.local_addr().expect("command server address");
    let server = std::thread::spawn(move || {
        let (mut socket, _) = listener.accept().expect("accept command request");
        let mut bytes = Vec::new();
        let mut chunk = [0_u8; 512];
        loop {
            let count = socket.read(&mut chunk).expect("read command request");
            if count == 0 {
                break;
            }
            bytes.extend_from_slice(&chunk[..count]);
            if bytes.windows(4).any(|window| window == b"\r\n\r\n") {
                break;
            }
        }
        let request_line = String::from_utf8(bytes)
            .expect("request is UTF-8")
            .lines()
            .next()
            .expect("request line")
            .to_owned();
        socket
            .write_all(b"HTTP/1.1 200 OK\r\nContent-Length: 0\r\n\r\n")
            .expect("write command response");
        request_line
    });
    let http = StrictHttpClient::new(&format!("http://{address}")).expect("HTTP client");
    (http, server)
}

#[test]
fn confirmation_is_consumed_once() {
    // Arrange
    let (_temp, root) = private_root();
    write_required_checkpoint(&root, IdentifyCheckpointKind::Rendered)
        .expect("required checkpoint");

    // Act
    respond_identify_checkpoint(
        &root,
        IdentifyCheckpointKind::Rendered,
        IdentifyCheckpointOutcome::Confirmed,
    )
    .expect("confirmation");
    let accepted = consume_checkpoint_response(&root, IdentifyCheckpointKind::Rendered)
        .expect("consume confirmation");

    // Assert
    assert_eq!(accepted, CheckpointResponse::Confirmed);
    assert!(
        consume_checkpoint_response(&root, IdentifyCheckpointKind::Rendered)
            .expect("second checkpoint is absent")
            == CheckpointResponse::Pending
    );
    assert!(respond_identify_checkpoint(
        &root,
        IdentifyCheckpointKind::Rendered,
        IdentifyCheckpointOutcome::Confirmed,
    )
    .is_err());
}

#[test]
fn identify_transaction_arms_readiness_without_issuing_a_request() {
    // Arrange
    let (_temp, root) = private_root();
    let evidence = CommandEffectsEvidence::new();

    // Act
    let phase = arm_identify_transaction(&root, &evidence).expect("arm transaction");

    // Assert
    assert_eq!(phase, CommandPhase::IdentifyReady);
    assert_eq!(evidence.identify_request_count, 0);
    assert!(!evidence.identify_operator_ready_confirmed);
    assert!(root.join("identify-ready.required.json").is_file());
    assert!(!root.join("identify-rendered.required.json").exists());
}

#[test]
fn count_preserving_paused_dismissal_arms_ready_without_resume_or_identify() {
    // Arrange
    let (_temp, root) = private_root();
    let mut evidence = CommandEffectsEvidence::new();
    evidence.pause_request_count = 1;
    evidence.pause_confirmed = true;
    evidence.dismiss_request_count = 1;
    evidence.dismiss_confirmed = true;
    evidence.block_count_preserved = true;

    // Act
    let phase = arm_ready_after_paused_dismissal(&root, &evidence).expect("arm paused readiness");

    // Assert
    assert_eq!(phase, CommandPhase::IdentifyReady);
    assert!(evidence.pause_confirmed);
    assert_eq!(evidence.resume_request_count, 0);
    assert_eq!(evidence.identify_request_count, 0);
    assert!(root.join("identify-ready.required.json").is_file());
}

#[test]
fn pause_join_issues_one_dismiss_only_after_http_and_serial_stop() {
    // Arrange
    let (_temp, root) = private_root();
    let (http, server) = successful_command_server();
    let mut sample = SystemInfoWire::from_snapshot(&ApiSnapshot::safe_ultra_205());
    sample.mining_paused = true;
    sample.mining_activity = "paused".to_owned();
    sample.block_found = 7;
    let started = std::time::Instant::now();
    let mut phase = CommandPhase::Pause(PauseJoinState::new(started));
    let mut evidence = CommandEffectsEvidence::new();
    evidence.pause_request_count = 1;
    let mut maybe_block_count = Some(7);
    let mut maybe_failure = None;
    let mut serial = SharedSerialState::default();

    // Act
    advance_commands(
        &http,
        &trusted_target(),
        &root,
        &sample,
        &serial,
        started,
        CommandProgress {
            phase: &mut phase,
            maybe_block_count: &mut maybe_block_count,
            evidence: &mut evidence,
            maybe_failure: &mut maybe_failure,
        },
    );
    let phase_before_serial_stop = phase;
    let requests_before_serial_stop = evidence.dismiss_request_count;
    serial.resumable_pause_safe_stop_confirmed = true;
    advance_commands(
        &http,
        &trusted_target(),
        &root,
        &sample,
        &serial,
        started + std::time::Duration::from_millis(1),
        CommandProgress {
            phase: &mut phase,
            maybe_block_count: &mut maybe_block_count,
            evidence: &mut evidence,
            maybe_failure: &mut maybe_failure,
        },
    );

    // Assert
    assert!(matches!(phase_before_serial_stop, CommandPhase::Pause(_)));
    assert_eq!(requests_before_serial_stop, 0);
    assert_eq!(phase, CommandPhase::PausedDismiss);
    assert!(evidence.pause_confirmed);
    assert_eq!(evidence.dismiss_request_count, 1);
    assert_eq!(evidence.resume_request_count, 0);
    assert_eq!(maybe_failure, None);
    assert_eq!(
        server.join().expect("command server thread"),
        "POST /api/system/blockFound/dismiss HTTP/1.1"
    );
}

#[test]
fn paused_dismissal_readback_precedes_identify_readiness() {
    // Arrange
    let (_temp, root) = private_root();
    let http = StrictHttpClient::new("http://127.0.0.1:9").expect("HTTP client");
    let mut sample = SystemInfoWire::from_snapshot(&ApiSnapshot::safe_ultra_205());
    sample.block_found = 7;
    sample.show_new_block = false;
    let mut phase = CommandPhase::PausedDismiss;
    let mut evidence = CommandEffectsEvidence::new();
    evidence.pause_request_count = 1;
    evidence.pause_confirmed = true;
    evidence.dismiss_request_count = 1;
    let mut maybe_block_count = Some(7);
    let mut maybe_failure = None;

    // Act
    advance_commands(
        &http,
        &trusted_target(),
        &root,
        &sample,
        &SharedSerialState::default(),
        std::time::Instant::now(),
        CommandProgress {
            phase: &mut phase,
            maybe_block_count: &mut maybe_block_count,
            evidence: &mut evidence,
            maybe_failure: &mut maybe_failure,
        },
    );

    // Assert
    assert_eq!(phase, CommandPhase::IdentifyReady);
    assert!(evidence.dismiss_confirmed);
    assert!(evidence.block_count_preserved);
    assert!(root.join("identify-ready.required.json").is_file());
    assert_eq!(maybe_failure, None);
}

#[test]
fn ready_signal_is_consumed_without_releasing_pause() {
    // Arrange
    let (_temp, root) = private_root();
    let mut evidence = CommandEffectsEvidence::new();
    evidence.pause_request_count = 1;
    evidence.pause_confirmed = true;
    evidence.dismiss_request_count = 1;
    evidence.dismiss_confirmed = true;
    evidence.block_count_preserved = true;
    arm_ready_after_paused_dismissal(&root, &evidence).expect("arm paused readiness");

    // Act
    let absent = consume_ready_signal(&root, &mut evidence).expect("absent signal");
    respond_identify_checkpoint(
        &root,
        IdentifyCheckpointKind::Ready,
        IdentifyCheckpointOutcome::Confirmed,
    )
    .expect("ready signal");
    let present = consume_ready_signal(&root, &mut evidence).expect("consume signal");

    // Assert
    assert_eq!(absent, CheckpointResponse::Pending);
    assert_eq!(present, CheckpointResponse::Confirmed);
    assert!(evidence.identify_operator_ready_confirmed);
    assert_eq!(evidence.resume_request_count, 0);
    assert_eq!(evidence.identify_request_count, 0);
    assert!(root.join("identify-ready.consumed.json").is_file());
}

#[test]
fn cleared_signal_is_the_only_transition_that_releases_resume() {
    // Arrange
    let (_temp, root) = private_root();
    let mut evidence = CommandEffectsEvidence::new();
    evidence.pause_confirmed = true;
    evidence.identify_operator_ready_confirmed = true;
    evidence.identify_request_count = 1;
    evidence.identify_rendered_confirmed = true;
    write_required_checkpoint(&root, IdentifyCheckpointKind::Cleared).expect("required checkpoint");

    // Act
    let absent = consume_cleared_signal(&root, &mut evidence).expect("absent signal");
    respond_identify_checkpoint(
        &root,
        IdentifyCheckpointKind::Cleared,
        IdentifyCheckpointOutcome::Confirmed,
    )
    .expect("cleared signal");
    let present = consume_cleared_signal(&root, &mut evidence).expect("consume signal");

    // Assert
    assert_eq!(absent, CheckpointResponse::Pending);
    assert_eq!(present, CheckpointResponse::Confirmed);
    assert!(evidence.identify_cleared_confirmed);
    assert_eq!(evidence.resume_request_count, 1);
    assert!(root.join("identify-cleared.consumed.json").is_file());
}

#[test]
fn malformed_confirmation_fails_closed() {
    // Arrange
    let (_temp, root) = private_root();
    write_required_checkpoint(&root, IdentifyCheckpointKind::Cleared).expect("required checkpoint");
    let response = root.join("identify-cleared.response.json");
    crate::write_private_new_bytes(
        &response,
        br#"{"schema":"wrong","checkpoint":"cleared","status":"confirmed"}"#,
    )
    .expect("malformed confirmation");

    // Act
    let result = consume_checkpoint_response(&root, IdentifyCheckpointKind::Cleared);

    // Assert
    assert!(result.is_err());
}

#[test]
fn declined_rendered_checkpoint_is_a_typed_response() {
    // Arrange
    let (_temp, root) = private_root();
    write_required_checkpoint(&root, IdentifyCheckpointKind::Rendered)
        .expect("required checkpoint");
    respond_identify_checkpoint(
        &root,
        IdentifyCheckpointKind::Rendered,
        IdentifyCheckpointOutcome::Declined,
    )
    .expect("declined response");

    // Act
    let result = consume_checkpoint_response(&root, IdentifyCheckpointKind::Rendered);

    // Assert
    assert_eq!(result, Ok(CheckpointResponse::Declined));
}

#[test]
fn rendered_checkpoint_replay_is_a_typed_response() {
    // Arrange
    let (_temp, root) = private_root();
    write_required_checkpoint(&root, IdentifyCheckpointKind::Rendered)
        .expect("required checkpoint");
    crate::write_private_new_bytes(
        &root.join("identify-rendered.response.json"),
        br#"{"schema":"bitaxe-identify-checkpoint-v3","checkpoint":"rendered","status":"replay"}"#,
    )
    .expect("replay response");

    // Act
    let result = consume_checkpoint_response(&root, IdentifyCheckpointKind::Rendered);

    // Assert
    assert_eq!(result, Ok(CheckpointResponse::Replay));
}

#[test]
fn rendered_confirmation_remains_valid_after_the_effect_deadline() {
    // Arrange
    let (_temp, root) = private_root();
    let started = std::time::Instant::now();
    let effect_inactive_at = started + std::time::Duration::from_secs(30);
    let delayed_report = effect_inactive_at + std::time::Duration::from_secs(24 * 60 * 60);
    let mut evidence = CommandEffectsEvidence::new();
    evidence.identify_request_count = 1;
    let mut phase = CommandPhase::IdentifyRendered { effect_inactive_at };
    let mut maybe_failure = None;

    // Act
    let before = rendered_checkpoint_action(
        effect_inactive_at - std::time::Duration::from_nanos(1),
        effect_inactive_at,
        CheckpointResponse::Confirmed,
        true,
    );
    let delayed = rendered_checkpoint_action(
        delayed_report,
        effect_inactive_at,
        CheckpointResponse::Confirmed,
        true,
    );
    finish_identify_observation(
        effect_inactive_at,
        &mut phase,
        &mut evidence,
        &mut maybe_failure,
    );
    let before_clear = arm_cleared_after_natural_expiry(
        &root,
        effect_inactive_at - std::time::Duration::from_nanos(1),
        effect_inactive_at,
    );
    let after_clear = arm_cleared_after_natural_expiry(&root, delayed_report, effect_inactive_at);

    // Assert
    assert_eq!(before, Ok(RenderedCheckpointAction::Confirmed));
    assert_eq!(delayed, Ok(RenderedCheckpointAction::Confirmed));
    assert_eq!(
        phase,
        CommandPhase::IdentifyObserved {
            clears_at: effect_inactive_at
        }
    );
    assert_eq!(before_clear, Ok(false));
    assert_eq!(after_clear, Ok(true));
    assert!(root.join("identify-cleared.required.json").is_file());
    assert_eq!(evidence.identify_request_count, 1);
    assert!(evidence.identify_rendered_confirmed);
    assert_eq!(maybe_failure, None);
}

#[test]
fn rendered_replay_waits_for_the_active_effect_to_expire() {
    // Arrange
    let started = std::time::Instant::now();
    let effect_inactive_at = started + std::time::Duration::from_secs(33);

    // Act
    let before = rendered_checkpoint_action(
        started + std::time::Duration::from_secs(1),
        effect_inactive_at,
        CheckpointResponse::Replay,
        true,
    );
    let after = rendered_checkpoint_action(
        effect_inactive_at + std::time::Duration::from_secs(1),
        effect_inactive_at,
        CheckpointResponse::Replay,
        true,
    );

    // Assert
    assert_eq!(
        before,
        Ok(RenderedCheckpointAction::ReplayAt(effect_inactive_at))
    );
    assert_eq!(
        after,
        Ok(RenderedCheckpointAction::ReplayAt(
            effect_inactive_at + std::time::Duration::from_secs(1)
        ))
    );
}

#[test]
fn rendered_report_waits_for_explicit_replay_or_decline() {
    // Arrange
    let expires_at = std::time::Instant::now();
    let later = expires_at + std::time::Duration::from_secs(24 * 60 * 60);

    // Act
    let pending = rendered_checkpoint_action(later, expires_at, CheckpointResponse::Pending, true);
    let declined =
        rendered_checkpoint_action(later, expires_at, CheckpointResponse::Declined, true);

    // Assert
    assert_eq!(pending, Ok(RenderedCheckpointAction::Wait));
    assert_eq!(declined, Ok(RenderedCheckpointAction::Declined));
}

#[test]
fn replayed_window_rejects_a_second_replay() {
    // Arrange
    let expires_at = std::time::Instant::now();

    // Act
    let action =
        rendered_checkpoint_action(expires_at, expires_at, CheckpointResponse::Replay, false);

    // Assert
    assert_eq!(action, Err(()));
}

#[test]
fn human_checkpoint_phases_have_no_elapsed_deadline() {
    // Arrange
    let started = std::time::Instant::now();
    let overnight = started + std::time::Duration::from_secs(24 * 60 * 60);

    // Act
    let ready_failure = automated_phase_failure(CommandPhase::IdentifyReady, started, overnight);
    let rendered_failure = automated_phase_failure(
        CommandPhase::IdentifyRendered {
            effect_inactive_at: started,
        },
        started,
        overnight,
    );
    let replay_pending_failure = automated_phase_failure(
        CommandPhase::IdentifyReplayPending { starts_at: started },
        started,
        overnight,
    );
    let replayed_failure = automated_phase_failure(
        CommandPhase::IdentifyReplayed {
            effect_inactive_at: started,
        },
        started,
        overnight,
    );
    let observed_failure = automated_phase_failure(
        CommandPhase::IdentifyObserved { clears_at: started },
        started,
        overnight,
    );
    let cleared_failure =
        automated_phase_failure(CommandPhase::IdentifyCleared, started, overnight);

    // Assert
    assert_eq!(ready_failure, None);
    assert_eq!(rendered_failure, None);
    assert_eq!(replay_pending_failure, None);
    assert_eq!(replayed_failure, None);
    assert_eq!(observed_failure, None);
    assert_eq!(cleared_failure, None);
}

#[test]
fn automated_notification_phase_keeps_its_exact_deadline() {
    // Arrange
    let started = std::time::Instant::now();
    let exact = started + std::time::Duration::from_secs(600);

    // Act
    let before = automated_phase_failure(
        CommandPhase::Notification,
        started,
        exact - std::time::Duration::from_nanos(1),
    );
    let at_deadline = automated_phase_failure(CommandPhase::Notification, started, exact);

    // Assert
    assert_eq!(before, None);
    assert_eq!(
        at_deadline,
        Some(super::CampaignTerminalCategory::NetworkCorrelationFailed)
    );
}

#[test]
fn resume_phases_keep_distinct_typed_deadlines() {
    // Arrange
    let started = std::time::Instant::now();

    // Act
    let intent = automated_phase_failure(
        CommandPhase::ResumeIntent,
        started,
        started + std::time::Duration::from_secs(15),
    );
    let activation = automated_phase_failure(
        CommandPhase::ResumeActive,
        started,
        started + std::time::Duration::from_secs(180),
    );

    // Assert
    assert_eq!(
        intent,
        Some(super::CampaignTerminalCategory::ResumeIntentUnconfirmed)
    );
    assert_eq!(
        activation,
        Some(super::CampaignTerminalCategory::ResumeReactivationTimedOut)
    );
}

#[test]
fn resume_intent_is_confirmed_before_active_reactivation() {
    // Arrange
    let (_temp, root) = private_root();
    let http = StrictHttpClient::new("http://127.0.0.1:9").expect("HTTP client");
    let mut sample = SystemInfoWire::from_snapshot(&ApiSnapshot::safe_ultra_205());
    sample.mining_paused = false;
    sample.mining_activity = "paused".to_owned();
    let mut phase = CommandPhase::ResumeIntent;
    let mut evidence = CommandEffectsEvidence::new();
    evidence.resume_request_count = 1;
    let mut maybe_block_count = None;
    let mut maybe_failure = None;

    // Act
    advance_commands(
        &http,
        &trusted_target(),
        &root,
        &sample,
        &SharedSerialState::default(),
        std::time::Instant::now(),
        CommandProgress {
            phase: &mut phase,
            maybe_block_count: &mut maybe_block_count,
            evidence: &mut evidence,
            maybe_failure: &mut maybe_failure,
        },
    );

    // Assert
    assert_eq!(phase, CommandPhase::ResumeActive);
    assert!(evidence.resume_intent_confirmed);
    assert!(!evidence.resume_confirmed);
    assert!(!evidence.active_after_resume);
    assert_eq!(maybe_failure, None);
}
