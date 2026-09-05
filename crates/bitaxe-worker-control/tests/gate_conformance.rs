use std::collections::BTreeMap;
use std::env;
use std::fs;

use bitaxe_worker_control::{
    AcceptedSequenceStore, DeviceIdentity, LeaseAuthorizationError, LeaseDeadlines,
    SequenceStoreResult, WorkLeaseAuthorityTrust, WorkLeaseAuthorizationVerifier, WorkerControl,
    WorkerLeaseGrant, WorkerLeaseRenewal, WorkerSession, WorkerSessionError,
};
use serde_json::{json, Value};

const FIXTURE_DEVICE_SEED: [u8; 32] = [
    0x9d, 0x61, 0xb1, 0x9d, 0xef, 0xfd, 0x5a, 0x60, 0xba, 0x84, 0x4a, 0xf4, 0x92, 0xec, 0x2c, 0xc4,
    0x44, 0x49, 0xc5, 0x69, 0x7b, 0x32, 0x69, 0x19, 0x70, 0x3b, 0xac, 0x03, 0x1c, 0xae, 0x7f, 0x60,
];

#[derive(Default)]
struct MemorySequenceStore {
    accepted: BTreeMap<String, u64>,
}

impl AcceptedSequenceStore for MemorySequenceStore {
    fn mark_effect_pending(&mut self) -> Result<(), LeaseAuthorizationError> {
        Ok(())
    }

    fn clear_effect_pending(&mut self) -> Result<(), LeaseAuthorizationError> {
        Ok(())
    }

    fn load(&self, key_id: &str) -> Result<Option<u64>, LeaseAuthorizationError> {
        Ok(self.accepted.get(key_id).copied())
    }

    fn compare_and_store(
        &mut self,
        key_id: &str,
        expected: Option<u64>,
        next: u64,
    ) -> Result<SequenceStoreResult, LeaseAuthorizationError> {
        let current = self.accepted.get(key_id).copied();
        if current == Some(next) {
            return Ok(SequenceStoreResult::AlreadyCommitted);
        }
        if current != expected {
            return Ok(SequenceStoreResult::Stale);
        }
        self.accepted.insert(key_id.to_owned(), next);
        Ok(SequenceStoreResult::Committed)
    }
}

#[derive(Default)]
struct ConformanceSession;

impl WorkerSession for ConformanceSession {
    fn start(
        &mut self,
        _grant: &WorkerLeaseGrant,
        _deadlines: LeaseDeadlines,
    ) -> Result<(), WorkerSessionError> {
        Ok(())
    }

    fn renew(
        &mut self,
        _renewal: &WorkerLeaseRenewal,
        _deadlines: LeaseDeadlines,
    ) -> Result<(), WorkerSessionError> {
        Ok(())
    }

    fn safe_stop(
        &mut self,
        _reason: bitaxe_worker_control::RestorationReason,
    ) -> Result<(), WorkerSessionError> {
        Ok(())
    }
}

#[test]
fn gate_serial_and_signed_session_contracts_match_rust() {
    // Arrange: Bazel supplies the exact pinned Gate artifacts, never local live keys.
    if env::var("BWG_POSSESSION_FIXTURES").is_err() && env::var("BWG_DEPLOYMENT_FIXTURES").is_err()
    {
        assert_ne!(
            env::var("BWG_REQUIRE_PINNED_FIXTURES").ok().as_deref(),
            Some("1"),
            "required Gate fixtures are absent"
        );
        return;
    }
    let possession = required_fixture("BWG_POSSESSION_FIXTURES");
    let deployment = required_fixture("BWG_DEPLOYMENT_FIXTURES");
    let request = &possession["initialAdmission"]["request"];
    let payload = &request["payload"];
    let binding = bitaxe_worker_control::serial::SerialSessionBinding::parse(
        text(payload, "sessionId"),
        text(payload, "hostNonce"),
        text(payload, "deviceNonce"),
    )
    .expect("pinned serial session must validate");
    let trust = WorkLeaseAuthorityTrust::from_deployment_json(&deployment["trust"].to_string())
        .expect("pinned deployment trust must parse");
    let manifest_digest =
        bitaxe_worker_control::serial::serial_manifest_sha256().expect("manifest digest");
    let mut worker = WorkerControl::new(
        DeviceIdentity::from_seed(FIXTURE_DEVICE_SEED),
        WorkLeaseAuthorizationVerifier::new(trust, MemorySequenceStore::default()),
        ConformanceSession,
        None,
        bitaxe_worker_control::FirmwareIdentity::new(
            bitaxe_worker_control::FirmwareSourceCommit::parse(&"a".repeat(40))
                .expect("fixture source"),
            &"b".repeat(64),
        )
        .expect("fixture firmware identity"),
        deployment["ultra205"]["signedCapability"].clone(),
        &manifest_digest,
    )
    .expect("pinned Worker contract");
    worker
        .begin_serial_session(binding)
        .expect("fresh logical session");

    // Act: produce the exact same signed possession transcript as the browser contract.
    let proof = worker
        .prepare_frame(&json_line(request), 0)
        .expect("possession should prepare");
    let response: Value = serde_json::from_slice(proof.frame()).expect("proof JSON");

    // Assert: byte-independent JSON equality includes exact deterministic compact JWS.
    assert_eq!(manifest_digest, text(payload, "serialManifestSha256"));
    assert_eq!(response, possession["initialAdmission"]["response"]);
    worker.confirm_sent(proof).expect("proof should admit");
    let start = authorized_request(&deployment, "start");
    let response = worker
        .prepare_frame(
            &json_line(&controller_request("serial_start", "start_lease", start)),
            0,
        )
        .expect("Gate-signed Start should authenticate");
    let response: Value = serde_json::from_slice(response.frame()).expect("Start response");
    assert_eq!(response["result"]["state"], "mining");
    let renew = authorized_request(&deployment, "renew");
    worker
        .prepare_frame(
            &json_line(&controller_request("serial_renew", "renew_lease", renew)),
            10_000,
        )
        .expect("Gate-signed Renew should authenticate");
}

fn required_fixture(variable: &str) -> Value {
    let path = env::var(variable).unwrap_or_else(|_| panic!("{variable} must be set by Bazel"));
    serde_json::from_str(&fs::read_to_string(path).expect("pinned fixture readable"))
        .expect("fixture JSON")
}

fn text<'a>(value: &'a Value, key: &str) -> &'a str {
    value[key]
        .as_str()
        .unwrap_or_else(|| panic!("missing fixture field {key}"))
}

fn authorized_request(deployment: &Value, operation: &str) -> Value {
    let mut request = deployment[operation]["input"]["request"].clone();
    request["authorization"] = deployment[operation]["artifact"]["authorization"].clone();
    request
}

fn controller_request(request_id: &str, command: &str, payload: Value) -> Value {
    json!({ "protocolVersion": "bwg-worker-controller/0.4", "requestId": request_id,
        "command": command, "payload": payload })
}

fn json_line(value: &Value) -> Vec<u8> {
    let mut line = serde_json::to_vec(value).expect("fixture value should encode");
    line.push(b'\n');
    line
}

#[test]
fn gate_serial_vectors_drive_fragmentation_and_negative_boundary_checks() {
    use bitaxe_worker_control::serial::{
        serial_manifest, SerialEnvelope, SerialError, SerialFrameAccumulator,
        MAXIMUM_CONTROL_PAYLOAD_BYTES, MAXIMUM_WIRE_FRAME_BYTES,
    };
    // Arrange
    if env::var("BWG_SERIAL_FIXTURES").is_err() {
        assert_ne!(
            env::var("BWG_REQUIRE_PINNED_FIXTURES").ok().as_deref(),
            Some("1"),
            "required serial fixtures are absent"
        );
        return;
    }
    let fixture = required_fixture("BWG_SERIAL_FIXTURES");
    assert_eq!(fixture["manifest"], serial_manifest());
    let frames = fixture["frames"]
        .as_array()
        .expect("published serial frames");
    assert!(!frames.is_empty());
    let stream: Vec<_> = frames
        .iter()
        .flat_map(|frame| json_line(&frame["frame"]))
        .collect();
    let mut accumulator = SerialFrameAccumulator::default();
    // Act: exercise arbitrary splitting and multiple complete records in one read.
    let mut decoded = Vec::new();
    for chunk in stream.chunks(7) {
        for byte in chunk {
            if let Some(record) = accumulator.push_byte(*byte) {
                let envelope = SerialEnvelope::parse(&record.expect("bounded shared record"))
                    .expect("shared frame parses");
                decoded.push(serde_json::to_value(&envelope).expect("frame value"));
            }
        }
    }
    // Assert
    assert_eq!(
        decoded,
        frames
            .iter()
            .map(|frame| frame["frame"].clone())
            .collect::<Vec<_>>()
    );
    for frame in frames {
        let mut crlf = json_line(&frame["frame"]);
        crlf.insert(crlf.len() - 1, b'\r');
        assert!(matches!(
            SerialEnvelope::parse(&crlf),
            Err(SerialError::Invalid)
        ));
        let mut unknown = frame["frame"].clone();
        unknown["unexpected"] = true.into();
        assert!(matches!(
            SerialEnvelope::parse(&json_line(&unknown)),
            Err(SerialError::Invalid)
        ));
        let wire = String::from_utf8(json_line(&frame["frame"])).expect("UTF8 fixture");
        let sequence = format!("\"sequence\":{}", frame["frame"]["sequence"]);
        let duplicate = wire.replacen(&sequence, &format!("{sequence},{sequence}"), 1);
        assert!(matches!(
            SerialEnvelope::parse(duplicate.as_bytes()),
            Err(SerialError::Invalid)
        ));
    }
    let mut control = frames
        .iter()
        .find(|frame| frame["frame"]["kind"] == "control")
        .expect("published control frame")["frame"]
        .clone();
    control["payload"] = json!({"padding":""});
    let overhead = serde_json::to_vec(&control["payload"])
        .expect("payload")
        .len();
    control["payload"]["padding"] = "x"
        .repeat(MAXIMUM_CONTROL_PAYLOAD_BYTES + 1 - overhead)
        .into();
    let oversized = json_line(&control);
    assert!(oversized.len() <= MAXIMUM_WIRE_FRAME_BYTES);
    assert!(matches!(
        SerialEnvelope::parse(&oversized),
        Err(SerialError::Oversized)
    ));
}
