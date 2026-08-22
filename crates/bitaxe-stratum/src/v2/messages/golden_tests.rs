use serde::Deserialize;

use super::{OpenStandardMiningChannel, ServerMessage, SetupConnection, SubmitSharesStandard};
use crate::v2::frame::Frame;

const FIXTURE: &str = include_str!("../../../fixtures/stratum-v2-protocol-vectors.json");

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Fixture {
    schema_version: String,
    reference_commit: String,
    sources: Vec<String>,
    cases: Vec<Case>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Case {
    name: String,
    wire_hex: String,
}

#[test]
fn protocol_fixture_is_provenance_bound_and_matches_pinned_wire_bytes() {
    // Arrange
    let fixture: Fixture = serde_json::from_str(FIXTURE).expect("fixture JSON");
    let expected_names = [
        "setup_connection",
        "open_standard_channel",
        "submit_standard",
        "setup_success",
        "set_target",
    ];

    // Act
    let actual_names = fixture
        .cases
        .iter()
        .map(|case| case.name.as_str())
        .collect::<Vec<_>>();

    // Assert
    assert_eq!(fixture.schema_version, "bitaxe-stratum-v2-vectors-v1");
    assert_eq!(
        fixture.reference_commit,
        "c1915b0a63bfabebdb95a515cedfee05146c1d50"
    );
    assert_eq!(fixture.sources.len(), 2);
    assert_eq!(actual_names, expected_names);
    assert_eq!(encoded_setup(), fixture_bytes(&fixture, "setup_connection"));
    assert_eq!(
        encoded_open_standard(),
        fixture_bytes(&fixture, "open_standard_channel")
    );
    assert_eq!(
        encoded_submit_standard(),
        fixture_bytes(&fixture, "submit_standard")
    );
    assert!(matches!(
        decode_case(&fixture, "setup_success"),
        ServerMessage::SetupConnectionSuccess(_)
    ));
    assert!(matches!(
        decode_case(&fixture, "set_target"),
        ServerMessage::SetTarget(_)
    ));
}

fn encoded_setup() -> Vec<u8> {
    SetupConnection {
        endpoint_host: "pool".to_owned(),
        endpoint_port: 3333,
        vendor: "bitaxe".to_owned(),
        hardware_version: "BM1366".to_owned(),
        firmware: String::new(),
        device_id: String::new(),
        flags: 1,
    }
    .encode()
    .expect("setup")
    .encode()
}

fn encoded_open_standard() -> Vec<u8> {
    OpenStandardMiningChannel {
        request_id: 1,
        user_identity: "worker".to_owned(),
        nominal_hashrate: 1.0e12,
        maximum_target: [0xff; 32],
    }
    .encode()
    .expect("open")
    .encode()
}

fn encoded_submit_standard() -> Vec<u8> {
    SubmitSharesStandard {
        channel_id: 1,
        sequence_number: 2,
        job_id: 3,
        nonce: 4,
        ntime: 5,
        version: 6,
    }
    .encode()
    .expect("submit")
    .encode()
}

fn decode_case(fixture: &Fixture, name: &str) -> ServerMessage {
    let frame = Frame::parse(&fixture_bytes(fixture, name)).expect("fixture frame");
    ServerMessage::decode(&frame).expect("fixture message")
}

fn fixture_bytes(fixture: &Fixture, name: &str) -> Vec<u8> {
    let raw = &fixture
        .cases
        .iter()
        .find(|case| case.name == name)
        .expect("fixture case")
        .wire_hex;
    assert_eq!(raw.len() % 2, 0);
    raw.as_bytes()
        .chunks_exact(2)
        .map(|pair| {
            let text = std::str::from_utf8(pair).expect("hex text");
            u8::from_str_radix(text, 16).expect("hex byte")
        })
        .collect()
}
