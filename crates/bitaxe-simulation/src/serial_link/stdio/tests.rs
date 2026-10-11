use super::*;
use crate::serial_link::gate_peer::GatePeer;
use crate::CheckStatus;
use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};
use bitaxe_worker_control::serial::{SerialEnvelope, SerialFrameAccumulator, SerialKind};
use serde_json::{json, value::RawValue};

const HOST_NONCE: [u8; 32] = [9; 32];

/// Encodes host records, judges device output with the Gate-rule peer.
struct Host {
    device: StdioDevice,
    peer: GatePeer,
    frames: SerialFrameAccumulator,
    records: Vec<Value>,
    sequence: u32,
    session_id: String,
    device_nonce: String,
    now: u64,
}

impl Host {
    fn new() -> Self {
        let device = StdioDevice::new(1, json!({"fixture": "capability"})).expect("device");
        Self {
            device,
            peer: GatePeer::default(),
            frames: SerialFrameAccumulator::default(),
            records: Vec::new(),
            sequence: 0,
            session_id: String::new(),
            device_nonce: String::new(),
            now: 0,
        }
    }

    fn observe(&mut self, output: &[u8]) {
        self.peer.device_bytes(output, self.now);
        for byte in output {
            if let Some(Ok(frame)) = self.frames.push_byte(*byte) {
                let envelope = SerialEnvelope::parse(&frame).expect("device record");
                self.records.push(json!({
                    "kind": envelope.kind,
                    "payload": serde_json::from_str::<Value>(envelope.payload.get()).expect("payload"),
                }));
            }
        }
    }

    fn hello(&mut self) {
        let nonce = URL_SAFE_NO_PAD.encode(HOST_NONCE);
        let payload = format!(r#"{{"op":"hello","hostNonce":"{nonce}"}}"#);
        let record = encode(SerialKind::Session, None, 0, &payload);
        self.peer.hello_sent(&nonce, self.now);
        let output = self.device.step(&record, self.now);
        self.observe(&output);
        let ack = &self.records[0]["payload"];
        self.device_nonce = ack["deviceNonce"]
            .as_str()
            .expect("device nonce")
            .to_owned();
        let envelope = SerialEnvelope::parse(
            &output[..=output.iter().position(|b| *b == b'\n').expect("ack line")],
        )
        .expect("ack");
        self.session_id = envelope.session_id.clone().expect("session");
    }

    fn send(&mut self, kind: SerialKind, payload: &Value) {
        self.sequence += 1;
        let record = encode(
            kind,
            Some(&self.session_id),
            self.sequence,
            &payload.to_string(),
        );
        let request_id = payload["requestId"].as_str().map(str::to_owned);
        let close = payload["op"] == "close";
        self.peer
            .host_record(record.len(), request_id, close, self.now);
        let output = self.device.step(&record, self.now);
        self.observe(&output);
    }

    fn command(&mut self, command: &str) {
        self.send(
            SerialKind::Control,
            &json!({"protocolVersion":"bwg-worker-controller/0.4","requestId":format!("serial_{command}"),"command":command}),
        );
    }

    fn possession(&mut self) {
        let payload = json!({"profile":"bwg-worker-possession/0.2","requestId":"pos_test","command":"prove_possession",
            "payload":{"purpose":"initial_admission","possessionNonce":URL_SAFE_NO_PAD.encode([4;32]),
            "challengeBindingSha256":"A".repeat(43),"controllerCapabilitySha256":self.device.capability_sha256(),
            "serialManifestSha256":serial_manifest_sha256().expect("manifest"),"sessionId":self.session_id,
            "hostNonce":URL_SAFE_NO_PAD.encode(HOST_NONCE),"deviceNonce":self.device_nonce}});
        self.send(SerialKind::Control, &payload);
    }

    fn heartbeat(&mut self) {
        self.send(SerialKind::Heartbeat, &json!({}));
    }

    fn idle_until(&mut self, until: u64) {
        while self.now < until {
            self.now += 100;
            let output = self.device.step(&[], self.now);
            self.observe(&output);
        }
    }

    fn admit(&mut self) {
        self.hello();
        self.command("discover");
        self.possession();
        self.heartbeat();
        self.command("status");
    }

    fn close(&mut self) {
        self.send(
            SerialKind::Session,
            &json!({"op":"close","reason":"tab_closed"}),
        );
    }

    fn reply(&self, request_id: &str) -> Option<&Value> {
        self.records
            .iter()
            .map(|record| &record["payload"])
            .find(|payload| payload["requestId"] == request_id)
    }
}

fn encode(kind: SerialKind, maybe_session: Option<&str>, sequence: u32, payload: &str) -> Vec<u8> {
    let raw = RawValue::from_string(payload.to_owned()).expect("payload");
    SerialEnvelope::encode(kind, maybe_session, sequence, &raw).expect("record")
}

#[test]
fn admission_then_close_is_clean() {
    // Arrange
    let mut host = Host::new();
    // Act
    host.admit();
    host.close();
    // Assert
    assert!(host.device.outcome().clean(), "{:?}", host.device.outcome());
}

#[test]
fn device_output_meets_gate_rules_through_admission_and_close() {
    // Arrange
    let mut host = Host::new();
    // Act
    host.admit();
    host.close();
    // Assert
    let check = host.peer.check();
    assert_eq!(check.status, CheckStatus::Passed, "{}", check.detail);
}

#[test]
fn possession_is_accepted() {
    // Arrange
    let mut host = Host::new();
    // Act
    host.hello();
    host.command("discover");
    host.possession();
    // Assert
    assert_eq!(
        host.reply("pos_test").expect("possession reply")["ok"],
        true
    );
}

#[test]
fn baseline_status_carries_no_simulation_evidence() {
    // Arrange
    let mut host = Host::new();
    // Act
    host.admit();
    // Assert
    let status = host.reply("serial_status").expect("status reply");
    assert_eq!(status["result"]["state"], "baseline");
    assert!(status["result"].get("qualification").is_none());
}

#[test]
fn device_heartbeats_keep_the_host_peer_live() {
    // Arrange
    let mut host = Host::new();
    host.admit();
    // Act
    for second in 1..=4 {
        host.idle_until(second * 1000);
        host.heartbeat();
    }
    host.peer.settle(host.now);
    // Assert
    assert_eq!(
        host.peer.check().status,
        CheckStatus::Passed,
        "{}",
        host.peer.check().detail
    );
}

#[test]
fn a_rejected_request_is_answered_then_revokes_the_session() {
    // Arrange
    let mut host = Host::new();
    host.admit();
    // Act
    host.command("start_lease");
    host.command("status");
    // Assert
    assert_eq!(
        host.reply("serial_start_lease").expect("rejection")["ok"],
        false
    );
    assert_eq!(host.device.outcome().frames_rejected, 1);
    assert_eq!(
        host.records
            .iter()
            .filter(|record| record["payload"]["requestId"] == "serial_status")
            .count(),
        1
    );
}

#[test]
fn input_closed_without_close_is_not_clean() {
    // Arrange
    let mut host = Host::new();
    host.admit();
    // Act
    host.device.input_closed(host.now);
    // Assert
    assert!(!host.device.outcome().clean());
}

#[test]
fn a_second_hello_is_unsupported() {
    // Arrange
    let mut host = Host::new();
    host.admit();
    let nonce = URL_SAFE_NO_PAD.encode(HOST_NONCE);
    let hello = encode(
        SerialKind::Session,
        None,
        0,
        &format!(r#"{{"op":"hello","hostNonce":"{nonce}"}}"#),
    );
    // Act
    host.device.step(&hello, host.now);
    // Assert
    assert_eq!(
        host.device.outcome().maybe_owner_failure.as_deref(),
        Some("second_session_unsupported")
    );
}
