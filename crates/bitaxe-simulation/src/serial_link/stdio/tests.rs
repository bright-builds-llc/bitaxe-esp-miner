use super::*;
use crate::serial_link::gate_peer::GatePeer;
use crate::CheckStatus;
use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};
use bitaxe_worker_control::serial::{SerialEnvelope, SerialFrameAccumulator, SerialKind};
use serde_json::{json, value::RawValue};
use std::collections::VecDeque;

mod lease;

const HOST_NONCE: [u8; 32] = [9; 32];

/// The host's side of the transport, shared with the device's port: a virtual wall
/// clock, the Gate-rule peer judging device output, and, while enabled, the 1 s link
/// heartbeats the Gate keeps sending whatever the device is doing.
#[derive(Default)]
struct Wire {
    clock: u64,
    peer: GatePeer,
    frames: SerialFrameAccumulator,
    records: Vec<Value>,
    session_id: String,
    sequence: u32,
    auto_heartbeats: bool,
    last_heartbeat: u64,
    queued: VecDeque<Vec<u8>>,
}

impl Wire {
    fn observe(&mut self, output: &[u8]) {
        self.peer.device_bytes(output, self.clock);
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

    /// The next host record in this session, as the Gate numbers them.
    fn record(&mut self, kind: SerialKind, payload: &Value) -> Vec<u8> {
        self.sequence += 1;
        let request_id = payload["requestId"].as_str().map(str::to_owned);
        let close = payload["op"] == "close";
        let record = encode(
            kind,
            Some(&self.session_id),
            self.sequence,
            &payload.to_string(),
        );
        self.peer
            .host_record(record.len(), request_id, close, self.clock);
        record
    }
}

struct ScriptedPort(Rc<RefCell<Wire>>);

impl StdioPort for ScriptedPort {
    fn input_until(&mut self, wall_ms: u64) -> PortInput {
        let mut wire = self.0.borrow_mut();
        wire.clock = wire.clock.max(wall_ms);
        let mut bytes: Vec<u8> = wire.queued.drain(..).flatten().collect();
        if wire.auto_heartbeats && wire.clock - wire.last_heartbeat >= 1000 {
            wire.last_heartbeat = wire.clock;
            bytes.extend(wire.record(SerialKind::Heartbeat, &json!({})));
        }
        PortInput {
            bytes,
            closed: false,
        }
    }

    fn output(&mut self, bytes: &[u8]) -> bool {
        self.0.borrow_mut().observe(bytes);
        true
    }
}

/// Encodes host records and judges device output with the Gate-rule peer.
struct Host {
    device: StdioDevice,
    wire: Rc<RefCell<Wire>>,
    device_nonce: String,
    now: u64,
    binding: String,
}

impl Host {
    fn new() -> Self {
        Self::with_trust(crate::authorization::synthetic_trust_json().expect("trust"))
    }

    fn with_trust(trust_json: String) -> Self {
        let wire = Rc::new(RefCell::new(Wire::default()));
        let config = StdioConfig {
            seed: 1,
            capability: json!({"fixture": "capability"}),
            trust_json,
        };
        let device =
            StdioDevice::new(config, Box::new(ScriptedPort(Rc::clone(&wire)))).expect("device");
        Self {
            device,
            wire,
            device_nonce: String::new(),
            now: 0,
            binding: String::new(),
        }
    }

    /// One device step at the host's clock; output written while the owner actuated
    /// precedes what the step returns, and the host clock follows the board.
    fn step(&mut self, input: &[u8]) -> Vec<u8> {
        let output = self.device.step(input, self.now);
        let mut wire = self.wire.borrow_mut();
        self.now = self.now.max(wire.clock);
        wire.clock = self.now;
        wire.observe(&output);
        output
    }

    fn hello(&mut self) {
        let nonce = URL_SAFE_NO_PAD.encode(HOST_NONCE);
        let payload = format!(r#"{{"op":"hello","hostNonce":"{nonce}"}}"#);
        let record = encode(SerialKind::Session, None, 0, &payload);
        self.wire.borrow_mut().peer.hello_sent(&nonce, self.now);
        let output = self.step(&record);
        let mut wire = self.wire.borrow_mut();
        let ack = &wire.records[0]["payload"];
        self.device_nonce = ack["deviceNonce"]
            .as_str()
            .expect("device nonce")
            .to_owned();
        let envelope = SerialEnvelope::parse(
            &output[..=output.iter().position(|b| *b == b'\n').expect("ack line")],
        )
        .expect("ack");
        wire.session_id = envelope.session_id.clone().expect("session");
        wire.last_heartbeat = self.now;
    }

    fn send(&mut self, kind: SerialKind, payload: &Value) {
        let record = {
            let mut wire = self.wire.borrow_mut();
            wire.clock = self.now;
            wire.record(kind, payload)
        };
        self.step(&record);
    }

    fn request(&mut self, command: &str, maybe_payload: Option<Value>) {
        let mut request = json!({"protocolVersion":"bwg-worker-controller/0.4","requestId":format!("serial_{command}"),"command":command});
        if let Some(payload) = maybe_payload {
            request["payload"] = payload;
        }
        self.send(SerialKind::Control, &request);
    }

    fn command(&mut self, command: &str) {
        self.request(command, None);
    }

    fn possession(&mut self) {
        let payload = json!({"profile":"bwg-worker-possession/0.2","requestId":"pos_test","command":"prove_possession",
            "payload":{"purpose":"initial_admission","possessionNonce":URL_SAFE_NO_PAD.encode([4;32]),
            "challengeBindingSha256":"A".repeat(43),"controllerCapabilitySha256":self.device.capability_sha256(),
            "serialManifestSha256":serial_manifest_sha256().expect("manifest"),"sessionId":self.wire.borrow().session_id,
            "hostNonce":URL_SAFE_NO_PAD.encode(HOST_NONCE),"deviceNonce":self.device_nonce}});
        self.send(SerialKind::Control, &payload);
        let response = self.reply("pos_test").expect("possession reply").clone();
        let transcript = json!({"profile":"bwg-worker-control-session/0.2","request":payload,"response":response});
        self.binding = URL_SAFE_NO_PAD.encode(<sha2::Sha256 as sha2::Digest>::digest(
            serde_json::to_vec(&transcript).expect("transcript"),
        ));
    }

    fn heartbeat(&mut self) {
        self.send(SerialKind::Heartbeat, &json!({}));
        self.wire.borrow_mut().last_heartbeat = self.now;
    }

    fn idle_until(&mut self, until: u64) {
        while self.now < until {
            self.now += 100;
            self.step(&[]);
        }
    }

    fn admit(&mut self) {
        self.hello();
        self.command("discover");
        self.possession();
        self.heartbeat();
        self.command("status");
    }

    /// The Gate stops heartbeating once it sends Close.
    fn close(&mut self) {
        self.wire.borrow_mut().auto_heartbeats = false;
        self.send(
            SerialKind::Session,
            &json!({"op":"close","reason":"tab_closed"}),
        );
    }

    fn reply(&self, request_id: &str) -> Option<Value> {
        self.replies(request_id).last().cloned()
    }

    fn replies(&self, request_id: &str) -> Vec<Value> {
        self.wire
            .borrow()
            .records
            .iter()
            .map(|record| record["payload"].clone())
            .filter(|payload| payload["requestId"] == request_id)
            .collect()
    }

    fn peer_check(&self) -> crate::ProfileCheck {
        let mut wire = self.wire.borrow_mut();
        wire.peer.settle(self.now);
        wire.peer.check()
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
    let check = host.peer_check();
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
fn baseline_status_carries_idle_qualification_evidence() {
    // Arrange
    let mut host = Host::new();
    // Act
    host.admit();
    // Assert
    let status = host.reply("serial_status").expect("status reply");
    let evidence = &status["result"]["qualification"];
    assert_eq!(status["result"]["state"], "baseline");
    assert_eq!(evidence["schema"], "worker-qualification-v1");
    assert_eq!(evidence["generation"], 0);
    assert!(evidence.get("owner_resources").is_none());
}

#[test]
fn admission_long_after_boot_begins_a_live_generation() {
    // Arrange
    let mut host = Host::new();
    host.idle_until(5_000);
    // Act
    host.admit();
    host.idle_until(6_000);
    host.heartbeat();
    // Assert
    let outcome = host.device.outcome();
    assert!(outcome.authenticated, "{outcome:?}");
    assert_eq!(outcome.maybe_link_failure, None);
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
    // Assert
    let check = host.peer_check();
    assert_eq!(check.status, CheckStatus::Passed, "{}", check.detail);
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
    assert_eq!(host.replies("serial_status").len(), 1);
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
    host.step(&hello);
    // Assert
    assert_eq!(
        host.device.outcome().maybe_owner_failure.as_deref(),
        Some("second_session_unsupported")
    );
}
