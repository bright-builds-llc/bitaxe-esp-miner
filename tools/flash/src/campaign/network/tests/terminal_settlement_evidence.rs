use bitaxe_http_transport::WebSocketReadFailureKind;

use super::*;

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
    accumulator.terminal_consumed_observed = true;
    accumulator.note_terminal_settlement(TerminalSettlementDecision::RequestSerialClose);
    accumulator.note_terminal_settlement(TerminalSettlementDecision::AcceptAfterSerialClose);
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
    assert!(encoded.contains("mining-campaign-network-continuity-v12"));
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
