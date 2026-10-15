use super::*;
use crate::test_link::{frame, Scripted};
use bitaxe_core::bap::{plan_command, BapConnectionMode, BapRequestSnapshot};

fn healthy_board() -> Vec<(u64, String)> {
    vec![
        (10, frame(BapCommand::Response, "deviceModel", "Ultra")),
        (11, frame(BapCommand::Response, "asicModel", "BM1366")),
        (
            12,
            frame(BapCommand::Response, "pool", "pool.example.invalid"),
        ),
        (
            13,
            frame(BapCommand::Response, "poolUser", "owner-address.worker"),
        ),
        (600, frame(BapCommand::Response, "shares", "12/0")),
        (
            1_200,
            frame(BapCommand::Acknowledge, "hashrate", "subscribed"),
        ),
        (2_200, frame(BapCommand::Response, "hashrate", "513.00")),
        (
            4_300,
            frame(BapCommand::Acknowledge, "hashrate", "unsubscribed"),
        ),
    ]
}

fn scripted(arrivals: &[(u64, String)]) -> Scripted {
    let borrowed: Vec<(u64, &str)> = arrivals
        .iter()
        .map(|(at, text)| (*at, text.as_str()))
        .collect();
    Scripted::new(&borrowed)
}

#[test]
fn a_healthy_board_completes_the_probe() {
    // Arrange
    let mut link = scripted(&healthy_board());
    // Act
    let record = probe(&mut link, 2).expect("probe");
    // Assert
    assert_eq!(record.outcome, "complete");
    assert_eq!(record.steps.len(), 3);
    assert!(record.maybe_signature.is_none());
}

#[test]
fn the_probe_sends_no_setting() {
    // Arrange
    let mut link = scripted(&healthy_board());
    // Act
    probe(&mut link, 2).expect("probe");
    // Assert
    let commands: Vec<_> = link.sent.iter().map(|frame| frame.command()).collect();
    assert_eq!(
        commands,
        [
            BapCommand::Request,
            BapCommand::Request,
            BapCommand::Subscribe,
            BapCommand::Unsubscribe
        ]
    );
}

#[test]
fn pool_identity_never_reaches_the_record() {
    // Arrange
    let mut link = scripted(&healthy_board());
    // Act
    let record = probe(&mut link, 2).expect("probe");
    // Assert
    let text = serde_json::to_string(&record.steps).expect("json");
    assert!(!text.contains("pool.example.invalid"));
    assert!(!text.contains("owner-address"));
    assert!(text.contains("redacted:"));
}

#[test]
fn a_silent_line_stops_at_the_first_step_with_no_bytes() {
    // Arrange
    let mut link = scripted(&[]);
    // Act
    let record = probe(&mut link, 2).expect("probe");
    // Assert
    assert_eq!(record.outcome, "stop_hardware_blocker");
    assert_eq!(record.steps.len(), 1);
    let signature = record.maybe_signature.expect("signature");
    assert_eq!(signature["step"], "request_system_info");
    assert_eq!(signature["category"], "no_bytes_received");
}

#[test]
fn garbled_bytes_are_distinguished_from_silence() {
    // Arrange
    let mut link = scripted(&[(10, "\u{fffd}\u{fffd}~~\r\n".to_owned())]);
    // Act
    let record = probe(&mut link, 2).expect("probe");
    // Assert
    let signature = record.maybe_signature.expect("signature");
    assert_eq!(signature["category"], "no_parseable_frames");
    assert_eq!(signature["bytes_received"], true);
}

#[test]
fn a_device_rejection_keeps_its_error_code() {
    // Arrange
    let mut link = scripted(&[(
        10,
        frame(BapCommand::Error, "systemInfo", "ap_mode_no_requests"),
    )]);
    // Act
    let record = probe(&mut link, 2).expect("probe");
    // Assert
    let signature = record.maybe_signature.expect("signature");
    assert_eq!(signature["category"], "device_rejected");
    assert_eq!(signature["device_error_code"], "ap_mode_no_requests");
}

#[test]
fn a_subscription_without_updates_is_not_a_pass() {
    // Arrange
    let mut arrivals = healthy_board();
    arrivals.retain(|(_, text)| !text.contains("513.0"));
    let mut link = scripted(&arrivals);
    // Act
    let record = probe(&mut link, 2).expect("probe");
    // Assert
    let signature = record.maybe_signature.expect("signature");
    assert_eq!(signature["step"], "subscribe_hashrate");
    assert_eq!(signature["category"], "no_subscription_update");
}

#[test]
fn listening_is_bounded() {
    // Arrange
    let mut link = scripted(&healthy_board());
    // Act
    probe(&mut link, 3_600).expect("probe");
    // Assert: the subscription listened no longer than the bound plus its answers.
    assert!(
        link.now <= (MAXIMUM_LISTEN_SECONDS * 1_000) + 3 * 3_000 + 2_000,
        "{}",
        link.now
    );
}

/// The firmware's own planner, connected to Wi-Fi, with a synthetic snapshot.
fn planned_responses(operation: &Operation) -> usize {
    let snapshot = BapRequestSnapshot {
        device_model: "205".into(),
        asic_model: "BM1366".into(),
        pool_endpoint: "pool.example.invalid".into(),
        pool_port: 3333,
        pool_user: "synthetic".into(),
        shares_accepted: 12,
        shares_rejected: 0,
        block_height: 1,
        found_block: 0,
        show_new_block: false,
    };
    let frame = operation.frame().expect("frame");
    plan_command(&frame, BapConnectionMode::Connected, Some(&snapshot))
        .expect("plan")
        .responses()
        .len()
}

#[test]
fn every_probe_request_is_one_the_firmware_answers() {
    for step in Step::ALL {
        if let Operation::Request(_) = step.operation() {
            assert!(planned_responses(&step.operation()) > 0, "{}", step.label());
        }
    }
}

#[test]
fn a_hashrate_request_is_not_answered_by_the_firmware() {
    assert_eq!(
        planned_responses(&Operation::Request(BapParameter::Hashrate)),
        0
    );
}

fn first_signature(arrivals: &[(u64, String)]) -> serde_json::Value {
    let mut link = scripted(arrivals);
    probe(&mut link, 2)
        .expect("probe")
        .maybe_signature
        .expect("a failed step")
}

#[test]
fn a_board_version_device_model_is_a_value_mismatch() {
    // Arrange: ordinal 006's answer, before the reference family name was used.
    let mut arrivals = healthy_board();
    arrivals[0] = (10, frame(BapCommand::Response, "deviceModel", "205"));
    // Act
    let signature = first_signature(&arrivals);
    // Assert
    assert_eq!(
        (signature["step"].as_str(), signature["category"].as_str()),
        (Some("request_system_info"), Some("value_mismatch"))
    );
}

#[test]
fn an_answer_for_another_parameter_is_refused() {
    // Arrange
    let mut arrivals = healthy_board();
    arrivals[4] = (600, frame(BapCommand::Response, "hashrate", "513.00"));
    arrivals.insert(5, (601, frame(BapCommand::Response, "shares", "12/0")));
    // Act
    let signature = first_signature(&arrivals);
    // Assert
    assert_eq!(
        (signature["step"].as_str(), signature["category"].as_str()),
        (Some("request_shares"), Some("unexpected_parameter"))
    );
}

#[test]
fn a_hashrate_without_two_decimals_is_a_value_mismatch() {
    // Arrange
    let mut arrivals = healthy_board();
    arrivals[6] = (2_200, frame(BapCommand::Response, "hashrate", "513"));
    // Act
    let signature = first_signature(&arrivals);
    // Assert
    assert_eq!(signature["category"].as_str(), Some("value_mismatch"));
}
