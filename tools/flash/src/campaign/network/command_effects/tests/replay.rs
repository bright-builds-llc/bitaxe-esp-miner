use std::io::{Read, Write};
use std::net::TcpListener;

use bitaxe_api::{ApiSnapshot, ExpectedRuntimeAttestationIdentity, SystemInfoWire};
use bitaxe_http_transport::StrictHttpClient;

use super::super::super::model::{SharedSerialState, TrustedNetworkTarget};
use super::*;

#[test]
fn missed_identify_window_replays_once_after_the_prior_effect_is_inactive() {
    // Arrange
    let (_temp, root) = private_root();
    let listener = TcpListener::bind("127.0.0.1:0").expect("bind identify server");
    let address = listener.local_addr().expect("identify server address");
    let server = std::thread::spawn(move || {
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
        let request_line = String::from_utf8(bytes)
            .expect("request is UTF-8")
            .lines()
            .next()
            .expect("request line")
            .to_owned();
        socket
            .write_all(b"HTTP/1.1 200 OK\r\nContent-Length: 0\r\n\r\n")
            .expect("write identify response");
        request_line
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
        effect_inactive_at: replay_not_before,
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
    let replay_effect_inactive_at = match phase {
        CommandPhase::IdentifyReplayed { effect_inactive_at } => effect_inactive_at,
        other => panic!("expected replayed phase, got {other:?}"),
    };
    respond_identify_checkpoint(
        &root,
        IdentifyCheckpointKind::Replayed,
        IdentifyCheckpointOutcome::Confirmed,
    )
    .expect("replayed confirmation");
    let delayed_report = replay_effect_inactive_at + std::time::Duration::from_secs(24 * 60 * 60);
    advance_commands(
        &http,
        &target,
        &root,
        &sample,
        &serial,
        delayed_report,
        CommandProgress {
            phase: &mut phase,
            maybe_block_count: &mut maybe_block_count,
            evidence: &mut evidence,
            maybe_failure: &mut maybe_failure,
        },
    );
    let observed_phase = phase;
    advance_commands(
        &http,
        &target,
        &root,
        &sample,
        &serial,
        delayed_report,
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
    assert_eq!(
        observed_phase,
        CommandPhase::IdentifyObserved {
            clears_at: replay_effect_inactive_at
        }
    );
    assert_eq!(phase, CommandPhase::IdentifyCleared);
    assert_eq!(evidence.identify_replay_request_count, 1);
    assert_eq!(evidence.identify_request_count, 2);
    assert!(evidence.identify_rendered_confirmed);
    assert_eq!(maybe_failure, None);
    assert_eq!(
        server.join().expect("identify server thread"),
        "POST /api/system/identify HTTP/1.1"
    );
}
