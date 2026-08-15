use std::io::{Read, Write};
use std::net::TcpListener;
use std::time::Instant;

use bitaxe_http_transport::{StrictHttpClient, WebSocketRead, WebSocketReadFailureKind};

use super::super::{consume_optional_websocket_read, fetch_command_status, fetch_system_info};
use super::*;
use crate::campaign::network::command_effects::record_command_failure;
use crate::campaign::network::model::{
    CommandFailureCause, CommandFailureDiagnostic, CommandFailurePhase,
};

fn malformed_json_server() -> (StrictHttpClient, std::thread::JoinHandle<()>) {
    let listener = TcpListener::bind("127.0.0.1:0").expect("bind malformed server");
    let address = listener.local_addr().expect("malformed server address");
    let server = std::thread::spawn(move || {
        let (mut socket, _) = listener.accept().expect("accept malformed request");
        let mut bytes = Vec::new();
        let mut chunk = [0_u8; 512];
        loop {
            let count = socket.read(&mut chunk).expect("read malformed request");
            if count == 0 {
                break;
            }
            bytes.extend_from_slice(&chunk[..count]);
            if bytes.windows(4).any(|window| window == b"\r\n\r\n") {
                break;
            }
        }
        socket
            .write_all(b"HTTP/1.1 200 OK\r\nContent-Length: 1\r\n\r\n{")
            .expect("write malformed response");
    });
    let http = StrictHttpClient::new(&format!("http://{address}")).expect("HTTP client");
    (http, server)
}

#[test]
fn transient_http_status_loss_waits_for_the_existing_phase_deadline() {
    // Arrange
    let http = StrictHttpClient::new("http://127.0.0.1:9").expect("idle HTTP client");

    // Act
    let system_info = fetch_system_info(&http);
    let command_status = fetch_command_status(&http);

    // Assert
    assert!(matches!(system_info, Ok(None)));
    assert!(matches!(command_status, Ok(None)));
}

#[test]
fn malformed_successful_http_status_response_still_fails_closed() {
    // Arrange
    let (http, server) = malformed_json_server();

    // Act
    let result = fetch_system_info(&http);
    server.join().expect("malformed server");

    // Assert
    assert_eq!(
        result,
        Err(CampaignTerminalCategory::NetworkCorrelationFailed)
    );
}

#[test]
fn websocket_io_loss_reconnects_without_retaining_partial_bytes() {
    // Arrange
    let mut pending = b"partial".to_vec();
    let mut witness = CommandTransitionWitness::default();

    // Act
    let retained = consume_optional_websocket_read(
        Err(WebSocketReadFailureKind::Io),
        &"0".repeat(32),
        &mut pending,
        &mut witness,
    );

    // Assert
    assert_eq!(retained, Ok(false));
    assert!(pending.is_empty());
}

#[test]
fn malformed_websocket_transition_still_fails_closed() {
    // Arrange
    let mut pending = Vec::new();
    let mut witness = CommandTransitionWitness::default();

    // Act
    let malformed = consume_optional_websocket_read(
        Ok(WebSocketRead::Text(
            b"command_status_transition {invalid}\n".to_vec(),
        )),
        &"0".repeat(32),
        &mut pending,
        &mut witness,
    );

    // Assert
    assert_eq!(malformed, Err(()));
}

#[test]
fn command_failure_diagnostic_has_closed_redaction_safe_vocabularies() {
    // Arrange
    let phases = [
        CommandFailurePhase::Notification,
        CommandFailurePhase::Pause,
        CommandFailurePhase::Dismiss,
        CommandFailurePhase::IdentifyStart,
        CommandFailurePhase::IdentifyRendered,
        CommandFailurePhase::IdentifyCleared,
        CommandFailurePhase::ResumeIntent,
        CommandFailurePhase::ResumeActive,
        CommandFailurePhase::Terminal,
    ];
    let causes = [
        CommandFailureCause::SerialWitness,
        CommandFailureCause::PhaseDeadline,
        CommandFailureCause::WebsocketWitness,
        CommandFailureCause::HttpSystemInfo,
        CommandFailureCause::HttpCommandStatus,
        CommandFailureCause::HttpSampleValidation,
        CommandFailureCause::CommandRequest,
        CommandFailureCause::CommandStateMachine,
        CommandFailureCause::TerminalDeadline,
        CommandFailureCause::SerialEnded,
        CommandFailureCause::QuorumIncomplete,
    ];

    // Act
    let phase_values = phases.map(|phase| {
        serde_json::to_value(CommandFailureDiagnostic::new(
            phase,
            CommandFailureCause::PhaseDeadline,
        ))
        .expect("phase diagnostic")["phase"]
            .as_str()
            .expect("phase label")
            .to_owned()
    });
    let cause_values = causes.map(|cause| {
        serde_json::to_value(CommandFailureDiagnostic::new(
            CommandFailurePhase::Pause,
            cause,
        ))
        .expect("cause diagnostic")["cause"]
            .as_str()
            .expect("cause label")
            .to_owned()
    });

    // Assert
    assert_eq!(phase_values.len(), 9);
    assert_eq!(cause_values.len(), 11);
    assert!(phase_values.iter().all(|value| !value.is_empty()));
    assert!(cause_values.iter().all(|value| !value.is_empty()));
}

#[test]
fn command_failure_diagnostic_preserves_the_first_failure_through_recovery() {
    // Arrange
    let mut maybe_failure = None;
    let mut maybe_diagnostic = None;

    // Act
    record_command_failure(
        &mut maybe_failure,
        &mut maybe_diagnostic,
        CommandPhase::ProgrammaticPause(PauseJoinState::new(Instant::now())),
        CommandFailureCause::WebsocketWitness,
        CampaignTerminalCategory::NetworkCorrelationFailed,
    );
    record_command_failure(
        &mut maybe_failure,
        &mut maybe_diagnostic,
        CommandPhase::ProgrammaticDismiss,
        CommandFailureCause::HttpCommandStatus,
        CampaignTerminalCategory::TerminalStateUnconfirmed,
    );

    // Assert
    assert_eq!(
        maybe_failure,
        Some(CampaignTerminalCategory::NetworkCorrelationFailed)
    );
    assert_eq!(
        maybe_diagnostic,
        Some(CommandFailureDiagnostic::new(
            CommandFailurePhase::Pause,
            CommandFailureCause::WebsocketWitness,
        ))
    );
}

#[test]
fn command_request_category_maps_to_the_specific_diagnostic_cause() {
    // Arrange
    let request = CampaignTerminalCategory::CommandRequestFailed;
    let state = CampaignTerminalCategory::NetworkCorrelationFailed;

    // Act
    let request_cause = command_state_failure_cause(request);
    let state_cause = command_state_failure_cause(state);

    // Assert
    assert_eq!(request_cause, CommandFailureCause::CommandRequest);
    assert_eq!(state_cause, CommandFailureCause::CommandStateMachine);
}
