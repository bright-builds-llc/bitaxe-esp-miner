use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use base64::Engine;
use bitaxe_worker_control::{DeviceIdentity, FirmwareSourceCommit, PossessionRequest};
use serde_json::Value;
use sha2::{Digest, Sha256};

const REQUEST: &str = concat!(
    "{\"profile\":\"bwg-worker-possession/0.1\",",
    "\"requestId\":\"pos_initial_01\",",
    "\"command\":\"prove_possession\",",
    "\"payload\":{",
    "\"purpose\":\"initial_admission\",",
    "\"possessionNonce\":\"BBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBB\",",
    "\"challengeBindingSha256\":\"CCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCC\",",
    "\"controllerCapabilitySha256\":\"JFWsyueHvXS9M9GlDlK6yEOwUzO8oPXtloalyTRxFvE\",",
    "\"applicationDescriptorSha256\":\"rOKO_7whZfy0ntMKM9RIeZNAA3x97tt3rWMAm_QshVA\"",
    "}}\n",
);

#[test]
fn signs_only_the_closed_fresh_possession_claims() {
    // Arrange
    let identity = DeviceIdentity::from_seed([7_u8; 32]);
    let request = PossessionRequest::from_frame(REQUEST.as_bytes())
        .expect("published possession request should parse");

    // Act
    let response = identity
        .prove(&request, &fixture_source_commit())
        .expect("strict possession claims should sign");

    // Assert
    assert_eq!(response.request_id(), "pos_initial_01");
    assert!(response
        .compact_jws()
        .starts_with("eyJhbGciOiJFZDI1NTE5IiwidHlwIjoiYndnLXdvcmtlci1wb3NzZXNzaW9uK2p3cyJ9."));
    let wire: Value = serde_json::from_slice(&response.to_frame().expect("response should encode"))
        .expect("response should be JSON");
    assert_eq!(
        wire.pointer("/result/claims/firmwareSourceCommit"),
        Some(&Value::String("a".repeat(40)))
    );
    let redacted = format!("{response:?}");
    assert!(!redacted.contains("07070707"));
}

#[test]
fn derives_the_browser_canonical_control_session_transcript() {
    // Arrange
    let identity = DeviceIdentity::from_seed([7_u8; 32]);
    let request = PossessionRequest::from_frame(REQUEST.as_bytes())
        .expect("published possession request should parse");
    let response = identity
        .prove(&request, &fixture_source_commit())
        .expect("identity should sign");
    let request_value: Value = serde_json::from_str(REQUEST).expect("request should be JSON");
    let response_value: Value =
        serde_json::from_slice(&response.to_frame().expect("response should encode"))
            .expect("response should be JSON");
    let transcript = serde_json::json!({
        "profile": "bwg-worker-control-session/0.1",
        "request": request_value,
        "response": response_value,
    });
    let expected = URL_SAFE_NO_PAD.encode(Sha256::digest(canonical(&transcript).as_bytes()));

    // Act
    let actual = request
        .control_session_binding(&response)
        .expect("control-session binding should derive");

    // Assert
    assert_eq!(actual, expected);
    assert_eq!(actual.len(), 43);
}

#[test]
fn rejects_a_noncanonical_firmware_source_before_signing() {
    // Arrange / Act
    let proof = FirmwareSourceCommit::parse(&"A".repeat(40));

    // Assert
    assert!(proof.is_err());
}

fn fixture_source_commit() -> FirmwareSourceCommit {
    FirmwareSourceCommit::parse(&"a".repeat(40)).expect("fixture source commit should parse")
}

fn canonical(value: &Value) -> String {
    match value {
        Value::Array(values) => format!(
            "[{}]",
            values.iter().map(canonical).collect::<Vec<_>>().join(",")
        ),
        Value::Object(record) => format!(
            "{{{}}}",
            record
                .iter()
                .map(|(key, value)| format!(
                    "{}:{}",
                    serde_json::to_string(key).expect("key should encode"),
                    canonical(value)
                ))
                .collect::<Vec<_>>()
                .join(",")
        ),
        primitive => serde_json::to_string(primitive).expect("primitive should encode"),
    }
}
