use super::*;
use crate::test_link::{frame, Scripted};

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
        (600, frame(BapCommand::Response, "hashrate", "512.3")),
        (
            1_200,
            frame(BapCommand::Acknowledge, "hashrate", "subscribed"),
        ),
        (2_200, frame(BapCommand::Response, "hashrate", "513.0")),
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
