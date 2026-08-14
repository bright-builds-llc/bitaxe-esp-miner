use std::fs;
use std::io::{Read, Write};
use std::net::TcpListener;

use bitaxe_api::{ApiSnapshot, ExpectedRuntimeAttestationIdentity, SystemInfoWire};
use bitaxe_http_transport::StrictHttpClient;
use camino::Utf8PathBuf;

use super::super::model::{SharedSerialState, TrustedNetworkTarget};
use super::{
    advance_commands, arm_identify_transaction, arm_ready_after_pause, automated_phase_expired,
    consume_checkpoint_response, consume_cleared_signal, consume_ready_signal,
    rendered_checkpoint_action, respond_identify_checkpoint, take_recovery_pause_request,
    write_required_checkpoint, CheckpointResponse, CommandEffectsEvidence, CommandPhase,
    CommandProgress, IdentifyCheckpointKind, IdentifyCheckpointOutcome, RenderedCheckpointAction,
};
use crate::set_private_directory_mode;

fn private_root() -> (tempfile::TempDir, Utf8PathBuf) {
    let temp = tempfile::tempdir().expect("tempdir");
    let root = Utf8PathBuf::from_path_buf(temp.path().join("attempt")).expect("utf8 attempt path");
    fs::create_dir(&root).expect("create attempt");
    set_private_directory_mode(&root).expect("private attempt");
    (temp, root)
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
fn safe_stopped_pause_arms_ready_without_resume_or_identify() {
    // Arrange
    let (_temp, root) = private_root();
    let mut evidence = CommandEffectsEvidence::new();
    evidence.pause_request_count = 1;

    // Act
    let phase = arm_ready_after_pause(&root, &mut evidence).expect("arm paused readiness");

    // Assert
    assert_eq!(phase, CommandPhase::IdentifyReady);
    assert!(evidence.pause_confirmed);
    assert_eq!(evidence.resume_request_count, 0);
    assert_eq!(evidence.identify_request_count, 0);
    assert!(root.join("identify-ready.required.json").is_file());
}

#[test]
fn ready_signal_is_consumed_without_releasing_pause() {
    // Arrange
    let (_temp, root) = private_root();
    let mut evidence = CommandEffectsEvidence::new();
    evidence.pause_request_count = 1;
    arm_ready_after_pause(&root, &mut evidence).expect("arm paused readiness");

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
    evidence.identify_request_count = 2;
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
fn rendered_confirmation_expires_at_the_exact_effect_deadline() {
    // Arrange
    let started = std::time::Instant::now();
    let expires_at = started + std::time::Duration::from_secs(30);

    // Act
    let before = rendered_checkpoint_action(
        expires_at - std::time::Duration::from_nanos(1),
        expires_at,
        expires_at,
        CheckpointResponse::Confirmed,
        true,
    );
    let exact = rendered_checkpoint_action(
        expires_at,
        expires_at,
        expires_at,
        CheckpointResponse::Confirmed,
        true,
    );

    // Assert
    assert_eq!(before, Ok(RenderedCheckpointAction::Confirmed));
    assert_eq!(exact, Ok(RenderedCheckpointAction::Expired));
}

#[test]
fn rendered_replay_waits_for_the_active_effect_to_expire() {
    // Arrange
    let started = std::time::Instant::now();
    let confirmation_expires_at = started + std::time::Duration::from_secs(30);
    let replay_not_before = confirmation_expires_at + std::time::Duration::from_secs(3);

    // Act
    let before = rendered_checkpoint_action(
        started + std::time::Duration::from_secs(1),
        confirmation_expires_at,
        replay_not_before,
        CheckpointResponse::Replay,
        true,
    );
    let after = rendered_checkpoint_action(
        replay_not_before + std::time::Duration::from_secs(1),
        confirmation_expires_at,
        replay_not_before,
        CheckpointResponse::Replay,
        true,
    );

    // Assert
    assert_eq!(
        before,
        Ok(RenderedCheckpointAction::ReplayAt(replay_not_before))
    );
    assert_eq!(
        after,
        Ok(RenderedCheckpointAction::ReplayAt(
            replay_not_before + std::time::Duration::from_secs(1)
        ))
    );
}

#[test]
fn missed_identify_window_replays_once_after_the_prior_effect_is_inactive() {
    // Arrange
    let (_temp, root) = private_root();
    let listener = TcpListener::bind("127.0.0.1:0").expect("bind identify server");
    let address = listener.local_addr().expect("identify server address");
    let server = std::thread::spawn(move || {
        let mut request_lines = Vec::new();
        for _ in 0..2 {
            let (mut socket, _) = listener.accept().expect("accept identify request");
            let mut bytes = Vec::new();
            let mut chunk = [0_u8; 512];
            loop {
                let count = socket.read(&mut chunk).expect("read identify request");
                if count == 0 {
                    break;
                }
                bytes.extend_from_slice(&chunk[..count]);
                if bytes.windows(4).any(|window| window == b"\r\n\r\n") {
                    break;
                }
            }
            request_lines.push(
                String::from_utf8(bytes)
                    .expect("request is UTF-8")
                    .lines()
                    .next()
                    .expect("request line")
                    .to_owned(),
            );
            socket
                .write_all(b"HTTP/1.1 200 OK\r\nContent-Length: 0\r\n\r\n")
                .expect("write identify response");
        }
        request_lines
    });
    let http = StrictHttpClient::new(&format!("http://{address}")).expect("HTTP client");
    let target = TrustedNetworkTarget {
        origin: format!("http://{address}"),
        boot_session: "0".repeat(32),
        boot_ordinal: 1,
        expected: ExpectedRuntimeAttestationIdentity {
            firmware_commit: "0".repeat(40),
            reference_commit: "0".repeat(40),
            app_elf_sha256: "0".repeat(64),
        },
    };
    let sample = SystemInfoWire::from_snapshot(&ApiSnapshot::safe_ultra_205());
    let serial = SharedSerialState::default();
    let mut evidence = CommandEffectsEvidence::new();
    evidence.pause_confirmed = true;
    evidence.identify_operator_ready_confirmed = true;
    evidence.identify_request_count = 1;
    let started = std::time::Instant::now();
    let replay_not_before = started + std::time::Duration::from_secs(30);
    let mut phase = CommandPhase::IdentifyRendered {
        confirmation_expires_at: started,
        replay_not_before,
    };
    let mut maybe_block_count = None;
    let mut maybe_failure = None;
    write_required_checkpoint(&root, IdentifyCheckpointKind::Rendered)
        .expect("rendered checkpoint");
    respond_identify_checkpoint(
        &root,
        IdentifyCheckpointKind::Rendered,
        IdentifyCheckpointOutcome::Replay,
    )
    .expect("replay response");

    // Act
    advance_commands(
        &http,
        &target,
        &root,
        &sample,
        &serial,
        started + std::time::Duration::from_secs(1),
        CommandProgress {
            phase: &mut phase,
            maybe_block_count: &mut maybe_block_count,
            evidence: &mut evidence,
            maybe_failure: &mut maybe_failure,
        },
    );
    let phase_before_safe_boundary = phase;
    let request_count_before_safe_boundary = evidence.identify_request_count;
    advance_commands(
        &http,
        &target,
        &root,
        &sample,
        &serial,
        replay_not_before,
        CommandProgress {
            phase: &mut phase,
            maybe_block_count: &mut maybe_block_count,
            evidence: &mut evidence,
            maybe_failure: &mut maybe_failure,
        },
    );
    respond_identify_checkpoint(
        &root,
        IdentifyCheckpointKind::Replayed,
        IdentifyCheckpointOutcome::Confirmed,
    )
    .expect("replayed confirmation");
    advance_commands(
        &http,
        &target,
        &root,
        &sample,
        &serial,
        replay_not_before + std::time::Duration::from_secs(1),
        CommandProgress {
            phase: &mut phase,
            maybe_block_count: &mut maybe_block_count,
            evidence: &mut evidence,
            maybe_failure: &mut maybe_failure,
        },
    );

    // Assert
    assert_eq!(
        phase_before_safe_boundary,
        CommandPhase::IdentifyReplayPending {
            starts_at: replay_not_before
        }
    );
    assert_eq!(request_count_before_safe_boundary, 1);
    assert_eq!(phase, CommandPhase::IdentifyCleared);
    assert_eq!(evidence.identify_replay_request_count, 1);
    assert_eq!(evidence.identify_request_count, 3);
    assert!(evidence.identify_rendered_confirmed);
    assert_eq!(maybe_failure, None);
    assert_eq!(
        server.join().expect("identify server thread"),
        [
            "POST /api/system/identify HTTP/1.1",
            "POST /api/system/identify HTTP/1.1",
        ]
    );
}

#[test]
fn expired_rendered_window_waits_for_explicit_replay_or_decline() {
    // Arrange
    let expires_at = std::time::Instant::now();
    let later = expires_at + std::time::Duration::from_secs(24 * 60 * 60);

    // Act
    let pending = rendered_checkpoint_action(
        later,
        expires_at,
        expires_at,
        CheckpointResponse::Pending,
        true,
    );
    let declined = rendered_checkpoint_action(
        later,
        expires_at,
        expires_at,
        CheckpointResponse::Declined,
        true,
    );

    // Assert
    assert_eq!(pending, Ok(RenderedCheckpointAction::Wait));
    assert_eq!(declined, Ok(RenderedCheckpointAction::Declined));
}

#[test]
fn replayed_window_rejects_a_second_replay() {
    // Arrange
    let expires_at = std::time::Instant::now();

    // Act
    let action = rendered_checkpoint_action(
        expires_at,
        expires_at,
        expires_at,
        CheckpointResponse::Replay,
        false,
    );

    // Assert
    assert_eq!(action, Err(()));
}

#[test]
fn human_checkpoint_phases_have_no_elapsed_deadline() {
    // Arrange
    let started = std::time::Instant::now();
    let overnight = started + std::time::Duration::from_secs(24 * 60 * 60);

    // Act
    let ready_expired = automated_phase_expired(CommandPhase::IdentifyReady, started, overnight);
    let rendered_expired = automated_phase_expired(
        CommandPhase::IdentifyRendered {
            confirmation_expires_at: started,
            replay_not_before: started,
        },
        started,
        overnight,
    );
    let replay_pending_expired = automated_phase_expired(
        CommandPhase::IdentifyReplayPending { starts_at: started },
        started,
        overnight,
    );
    let replayed_expired = automated_phase_expired(
        CommandPhase::IdentifyReplayed {
            expires_at: started,
        },
        started,
        overnight,
    );
    let cleared_expired =
        automated_phase_expired(CommandPhase::IdentifyCleared, started, overnight);

    // Assert
    assert!(!ready_expired);
    assert!(!rendered_expired);
    assert!(!replay_pending_expired);
    assert!(!replayed_expired);
    assert!(!cleared_expired);
}

#[test]
fn automated_notification_phase_keeps_its_exact_deadline() {
    // Arrange
    let started = std::time::Instant::now();
    let exact = started + std::time::Duration::from_secs(600);

    // Act
    let before = automated_phase_expired(
        CommandPhase::Notification,
        started,
        exact - std::time::Duration::from_nanos(1),
    );
    let at_deadline = automated_phase_expired(CommandPhase::Notification, started, exact);

    // Assert
    assert!(!before);
    assert!(at_deadline);
}

#[test]
fn confirmation_without_a_required_checkpoint_is_rejected() {
    // Arrange
    let (_temp, root) = private_root();

    // Act
    let result = respond_identify_checkpoint(
        &root,
        IdentifyCheckpointKind::Rendered,
        IdentifyCheckpointOutcome::Confirmed,
    );

    // Assert
    assert!(result.is_err());
}

#[test]
fn replay_is_rejected_for_non_rendered_checkpoints() {
    // Arrange
    let (_temp, root) = private_root();
    write_required_checkpoint(&root, IdentifyCheckpointKind::Ready).expect("required checkpoint");

    // Act
    let result = respond_identify_checkpoint(
        &root,
        IdentifyCheckpointKind::Ready,
        IdentifyCheckpointOutcome::Replay,
    );

    // Assert
    assert!(result.is_err());
    assert!(!root.join("identify-ready.response.json").exists());
}

#[test]
fn recovery_pause_is_claimed_once_without_replacing_the_primary_failure() {
    // Arrange
    let primary = Some(super::CampaignTerminalCategory::CommandRequestFailed);
    let mut request_count = 0;

    // Act
    let first = take_recovery_pause_request(primary, &mut request_count);
    let second = take_recovery_pause_request(primary, &mut request_count);

    // Assert
    assert!(first);
    assert!(!second);
    assert_eq!(request_count, 1);
    assert_eq!(
        primary,
        Some(super::CampaignTerminalCategory::CommandRequestFailed)
    );
}
