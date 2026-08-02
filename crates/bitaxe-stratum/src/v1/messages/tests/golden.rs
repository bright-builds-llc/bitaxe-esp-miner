use std::collections::BTreeSet;

use serde::Deserialize;
use serde_json::Value;

use crate::jsonrpc::StratumRequestId;
use crate::v1::messages::{
    parse_server_message, ExtranonceAssignment, PoolDifficulty, StratumResponseError,
    StratumV1ClientMessage, StratumV1ServerMessage, VersionMask,
};

const REFERENCE_COMMIT: &str = "c1915b0a63bfabebdb95a515cedfee05146c1d50";
const EXPECTED_CASE_NAMES: [&str; 23] = [
    "client-subscribe",
    "client-authorize",
    "client-configure-version-rolling",
    "client-suggest-difficulty",
    "client-extranonce-subscribe",
    "client-pong",
    "client-version-response",
    "client-submit",
    "pool-notify",
    "pool-set-difficulty",
    "pool-set-extranonce",
    "pool-set-version-mask",
    "pool-reconnect",
    "pool-show-message",
    "pool-get-version",
    "pool-ping",
    "pool-subscribe-result",
    "pool-configure-result",
    "pool-success-result",
    "pool-false-result",
    "pool-error-result-string",
    "pool-error-result-array",
    "pool-error-result-object",
];

#[derive(Debug, Deserialize)]
struct ProtocolFixture {
    checklist_ids: Vec<String>,
    source_file: String,
    reference_commit: String,
    cases: Vec<ProtocolCase>,
}

#[derive(Debug, Deserialize)]
struct ProtocolCase {
    name: String,
    method: String,
    direction: String,
    json: Value,
}

#[test]
fn protocol_fixture_is_pinned_and_covers_every_str_002_message_family() {
    // Arrange
    let fixture = protocol_fixture();
    let actual_names = fixture
        .cases
        .iter()
        .map(|case| case.name.as_str())
        .collect::<BTreeSet<_>>();
    let expected_names = EXPECTED_CASE_NAMES.into_iter().collect::<BTreeSet<_>>();

    // Act
    let has_unique_names = actual_names.len() == fixture.cases.len();
    let directions_are_closed = fixture
        .cases
        .iter()
        .all(|case| matches!(case.direction.as_str(), "client-to-pool" | "pool-to-client"));
    let names_match_directions = fixture.cases.iter().all(|case| {
        case.name.starts_with("client-") == (case.direction == "client-to-pool")
            && case.name.starts_with("pool-") == (case.direction == "pool-to-client")
    });
    let methods_are_named = fixture.cases.iter().all(|case| !case.method.is_empty());

    // Assert
    assert_eq!(fixture.checklist_ids, ["STR-002"]);
    assert_eq!(
        fixture.source_file,
        "reference/esp-miner/components/stratum/stratum_api.c"
    );
    assert_eq!(fixture.reference_commit, REFERENCE_COMMIT);
    assert!(has_unique_names, "golden case names must be unique");
    assert!(
        directions_are_closed,
        "golden directions must use closed labels"
    );
    assert!(
        names_match_directions,
        "every golden case must be exercised in its named direction"
    );
    assert!(methods_are_named, "golden methods must be named");
    assert_eq!(actual_names, expected_names);
}

#[test]
fn client_messages_serialize_to_reference_derived_golden_shapes() {
    // Arrange
    let fixture = protocol_fixture();
    let client_cases = fixture
        .cases
        .iter()
        .filter(|case| case.direction == "client-to-pool");

    // Act and Assert
    for case in client_cases {
        let message = client_message(case.name.as_str());
        let line = message
            .to_json_line()
            .expect("golden client message should serialize");
        let actual: Value =
            serde_json::from_str(&line).expect("serialized client line should be JSON");
        assert_eq!(actual, case.json, "golden mismatch for {}", case.name);
    }
}

#[test]
fn pool_messages_parse_as_reference_derived_golden_variants() {
    // Arrange
    let fixture = protocol_fixture();
    let pool_cases = fixture
        .cases
        .iter()
        .filter(|case| case.direction == "pool-to-client");

    // Act and Assert
    for case in pool_cases {
        let input =
            serde_json::to_string(&case.json).expect("golden pool message should serialize");
        let actual = parse_server_message(&input).expect("golden pool message should parse");
        assert_pool_case(case.name.as_str(), actual);
    }
}

fn protocol_fixture() -> ProtocolFixture {
    serde_json::from_str(include_str!("../../../../fixtures/v1/protocol-cases.json"))
        .expect("protocol fixture should parse")
}

fn client_message(name: &str) -> StratumV1ClientMessage {
    match name {
        "client-subscribe" => {
            StratumV1ClientMessage::subscribe(StratumRequestId::new(1), "ultra", "205")
        }
        "client-authorize" => {
            StratumV1ClientMessage::authorize(StratumRequestId::new(2), "synthetic-user", "x")
        }
        "client-configure-version-rolling" => StratumV1ClientMessage::ConfigureVersionRolling {
            id: StratumRequestId::new(3),
            mask: u32::MAX,
        },
        "client-suggest-difficulty" => {
            StratumV1ClientMessage::suggest_difficulty(StratumRequestId::new(4), 1_000)
        }
        "client-extranonce-subscribe" => {
            StratumV1ClientMessage::extranonce_subscribe(StratumRequestId::new(5))
        }
        "client-pong" => StratumV1ClientMessage::Pong {
            id: StratumRequestId::new(6),
        },
        "client-version-response" => StratumV1ClientMessage::SendVersion {
            id: StratumRequestId::new(7),
            version: "v0.0.0-synthetic".to_owned(),
        },
        "client-submit" => StratumV1ClientMessage::submit_share(
            StratumRequestId::new(8),
            "synthetic-user",
            "job",
            "00000000",
            0x6470_25b5,
            0x1234_5678,
            0x0000_2000,
        ),
        _ => panic!("unmapped client golden case: {name}"),
    }
}

fn assert_pool_case(name: &str, actual: StratumV1ServerMessage) {
    match name {
        "pool-notify" => assert!(matches!(
            actual,
            StratumV1ServerMessage::Notify(notify)
                if notify.job_id == "0"
                    && notify.version == 0x2000_0004
                    && notify.nbits == 0x1705_ae3a
                    && notify.ntime == 0x6470_25b5
                    && notify.clean_jobs
        )),
        "pool-set-difficulty" => assert_eq!(
            actual,
            StratumV1ServerMessage::SetDifficulty(PoolDifficulty {
                difficulty: 4_294_967_295.0,
            })
        ),
        "pool-set-extranonce" => assert_eq!(
            actual,
            StratumV1ServerMessage::SetExtranonce(ExtranonceAssignment {
                extranonce1: "deadbeef".to_owned(),
                extranonce2_len: 8,
            })
        ),
        "pool-set-version-mask" => assert_eq!(
            actual,
            StratumV1ServerMessage::SetVersionMask(VersionMask { mask: 0x1fff_e000 })
        ),
        "pool-reconnect" => assert_eq!(actual, StratumV1ServerMessage::ClientReconnect),
        "pool-show-message" => assert_eq!(
            actual,
            StratumV1ServerMessage::ClientShowMessage("synthetic pool message".to_owned())
        ),
        "pool-get-version" => assert_eq!(actual, StratumV1ServerMessage::ClientGetVersion),
        "pool-ping" => assert!(matches!(
            actual,
            StratumV1ServerMessage::Ping { maybe_id: Some(id) } if id.raw() == 9
        )),
        "pool-subscribe-result" => assert!(matches!(
            actual,
            StratumV1ServerMessage::Response(response)
                if response.success
                    && response.maybe_extranonce == Some(ExtranonceAssignment {
                        extranonce1: "4de05269".to_owned(),
                        extranonce2_len: 8,
                    })
        )),
        "pool-configure-result" => assert!(matches!(
            actual,
            StratumV1ServerMessage::Response(response)
                if response.success
                    && response.maybe_version_mask == Some(VersionMask { mask: 0x1fff_e000 })
        )),
        "pool-success-result" => assert_response(actual, 10, true, None),
        "pool-false-result" => assert_response(
            actual,
            11,
            false,
            Some(StratumResponseError {
                maybe_code: None,
                message: "stale share".to_owned(),
            }),
        ),
        "pool-error-result-string" => assert_response(
            actual,
            12,
            false,
            Some(StratumResponseError {
                maybe_code: None,
                message: "authorization rejected".to_owned(),
            }),
        ),
        "pool-error-result-array" => assert_response(
            actual,
            13,
            false,
            Some(StratumResponseError {
                maybe_code: Some(21),
                message: "job not found".to_owned(),
            }),
        ),
        "pool-error-result-object" => assert_response(
            actual,
            14,
            false,
            Some(StratumResponseError {
                maybe_code: Some(22),
                message: "duplicate share".to_owned(),
            }),
        ),
        _ => panic!("unmapped pool golden case: {name}"),
    }
}

fn assert_response(
    actual: StratumV1ServerMessage,
    expected_id: u64,
    expected_success: bool,
    expected_error: Option<StratumResponseError>,
) {
    let StratumV1ServerMessage::Response(response) = actual else {
        panic!("expected response message");
    };
    assert_eq!(
        response.maybe_id.map(StratumRequestId::raw),
        Some(expected_id)
    );
    assert_eq!(response.success, expected_success);
    assert_eq!(response.maybe_error, expected_error);
}
