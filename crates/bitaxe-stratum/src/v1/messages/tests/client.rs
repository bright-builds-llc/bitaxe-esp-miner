use serde_json::{json, Value};

use crate::jsonrpc::StratumRequestId;
use crate::v1::messages::StratumV1ClientMessage;

#[test]
fn subscribe_serializes_user_agent() {
    // Arrange
    let message = StratumV1ClientMessage::subscribe(StratumRequestId::new(1), "ultra", "205");

    // Act
    let value = rendered_value(&message);

    // Assert
    assert_eq!(
        value,
        json!({
            "id": 1,
            "method": "mining.subscribe",
            "params": ["bitaxe/ultra/205"]
        })
    );
}

#[test]
fn authorize_serializes_credentials() {
    // Arrange
    let message =
        StratumV1ClientMessage::authorize(StratumRequestId::new(2), "synthetic-user", "x");

    // Act
    let value = rendered_value(&message);

    // Assert
    assert_eq!(
        value,
        json!({
            "id": 2,
            "method": "mining.authorize",
            "params": ["synthetic-user", "x"]
        })
    );
}

#[test]
fn configure_serializes_version_rolling_mask() {
    // Arrange
    let message = StratumV1ClientMessage::ConfigureVersionRolling {
        id: StratumRequestId::new(3),
        mask: 0xffff_fffe,
    };

    // Act
    let value = rendered_value(&message);

    // Assert
    assert_eq!(
        value,
        json!({
            "id": 3,
            "method": "mining.configure",
            "params": [
                ["version-rolling"],
                {"version-rolling.mask": "fffffffe"}
            ]
        })
    );
}

#[test]
fn suggest_difficulty_serializes_pool_value() {
    // Arrange
    let message = StratumV1ClientMessage::suggest_difficulty(StratumRequestId::new(4), 1_000);

    // Act
    let value = rendered_value(&message);

    // Assert
    assert_eq!(
        value,
        json!({
            "id": 4,
            "method": "mining.suggest_difficulty",
            "params": [1_000]
        })
    );
}

#[test]
fn extranonce_subscribe_serializes_empty_params() {
    // Arrange
    let message = StratumV1ClientMessage::extranonce_subscribe(StratumRequestId::new(5));

    // Act
    let value = rendered_value(&message);

    // Assert
    assert_eq!(
        value,
        json!({
            "id": 5,
            "method": "mining.extranonce.subscribe",
            "params": []
        })
    );
}

#[test]
fn pong_serializes_empty_params() {
    // Arrange
    let message = StratumV1ClientMessage::Pong {
        id: StratumRequestId::new(6),
    };

    // Act
    let value = rendered_value(&message);

    // Assert
    assert_eq!(
        value,
        json!({
            "id": 6,
            "method": "pong",
            "params": []
        })
    );
}

#[test]
fn send_version_serializes_response_shape() {
    // Arrange
    let message = StratumV1ClientMessage::SendVersion {
        id: StratumRequestId::new(7),
        version: "bitaxe/ultra/205".to_owned(),
    };

    // Act
    let value = rendered_value(&message);

    // Assert
    assert_eq!(
        value,
        json!({
            "id": 7,
            "result": "bitaxe/ultra/205",
            "error": null
        })
    );
}

#[test]
fn submit_share_serializes_fixed_width_hex_fields() {
    // Arrange
    let message = StratumV1ClientMessage::submit_share(
        StratumRequestId::new(8),
        "synthetic-user",
        "job",
        "00000000",
        0x6470_25b5,
        0x1234_5678,
        0x0000_2000,
    );

    // Act
    let value = rendered_value(&message);

    // Assert
    assert_eq!(
        value,
        json!({
            "id": 8,
            "method": "mining.submit",
            "params": [
                "synthetic-user",
                "job",
                "00000000",
                "647025b5",
                "12345678",
                "00002000"
            ]
        })
    );
}

fn rendered_value(message: &StratumV1ClientMessage) -> Value {
    let line = message
        .to_json_line()
        .expect("client message should serialize");
    assert!(line.ends_with('\n'));
    serde_json::from_str(&line).expect("client message should remain valid JSON")
}
