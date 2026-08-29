use std::collections::BTreeMap;
use std::env;
use std::fs;

use bitaxe_worker_control::{
    AcceptedSequenceStore, DeviceIdentity, LeaseAuthorizationError, LeaseDeadlines,
    SequenceStoreResult, WorkLeaseAuthorityTrust, WorkLeaseAuthorizationVerifier, WorkerControl,
    WorkerControlFrameAccumulator, WorkerLeaseAuthorizationContext, WorkerLeaseGrant,
    WorkerLeaseRenewal, WorkerSession, WorkerSessionError,
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
fn executes_the_pinned_gate_contract_against_the_firmware_core() {
    // Cargo cannot materialize Bazel external-repository data. The required
    // repository-wide Bazel suite sets all five paths and executes this body.
    let Some(controller) = fixture("BWG_CONTROLLER_FIXTURES") else {
        return;
    };
    let possession = required_fixture("BWG_POSSESSION_FIXTURES");
    let deployment = required_fixture("BWG_DEPLOYMENT_FIXTURES");
    let usb = required_fixture("BWG_USB_FIXTURES");
    let native_source = required_text("BWG_USB_NATIVE_SOURCE");

    // Arrange: consume the exact pinned public trust, capability, and proof.
    let capability = vector(&controller, "v03_discover")["response"]["result"].clone();
    let trust = WorkLeaseAuthorityTrust::from_deployment_json(&deployment["trust"].to_string())
        .expect("pinned deployment trust should parse");
    let verifier = WorkLeaseAuthorizationVerifier::new(trust, MemorySequenceStore::default());
    let mut worker = WorkerControl::new(
        DeviceIdentity::from_seed(FIXTURE_DEVICE_SEED),
        verifier,
        ConformanceSession,
        None,
        fixture_source_commit(),
        capability,
        "rOKO_7whZfy0ntMKM9RIeZNAA3x97tt3rWMAm_QshVA",
    )
    .expect("pinned Worker contract should configure");
    worker.begin_enumeration();

    // Act: execute pre-admission Controller vectors, produce and confirm the
    // exact possession response, then execute live signed Start and Renew.
    for id in ["v03_discover", "v03_status"] {
        let fixture = vector(&controller, id);
        let response = worker
            .prepare_frame(&json_line(&fixture["request"]), 0)
            .unwrap_or_else(|_| panic!("pinned {id} request should execute"));
        assert_eq!(response_value(&response), fixture["response"]);
    }
    let proof = worker
        .prepare_frame(&json_line(&possession["initialAdmission"]["request"]), 0)
        .expect("pinned possession request should prepare");
    let actual_proof: Value =
        serde_json::from_slice(proof.frame()).expect("proof response should be JSON");
    assert_eq!(actual_proof, possession["initialAdmission"]["response"]);
    worker
        .confirm_sent(proof)
        .expect("pinned possession response should admit");

    let start_request = controller_request(
        "usb_v03_start",
        "start_lease",
        authorized_request(&deployment, "start"),
    );
    let start = worker
        .prepare_frame(&json_line(&start_request), 0)
        .expect("pinned signed Start should execute");
    assert_eq!(
        response_value(&start),
        vector(&controller, "v03_start")["response"]
    );

    let renew_request = controller_request(
        "usb_v03_renew",
        "renew_lease",
        authorized_request(&deployment, "renew"),
    );
    let renew = worker
        .prepare_frame(&json_line(&renew_request), 10_000)
        .expect("pinned signed Renew should execute");
    assert_eq!(
        response_value(&renew),
        vector(&controller, "v03_renew")["response"]
    );
    let pause_fixture = vector(&controller, "v03_pause");
    let pause = worker
        .prepare_frame(&json_line(&pause_fixture["request"]), 10_000)
        .expect("pinned Pause should execute");
    assert_eq!(response_value(&pause), pause_fixture["response"]);

    // Assert: fixture-declared framing bounds and descriptor topology drive
    // executable checks instead of serving only as copied documentation.
    let maximum = usb["framing"]["maximumFrameBytes"]
        .as_u64()
        .and_then(|value| usize::try_from(value).ok())
        .expect("USB maximum frame bytes should be bounded");
    let mut accumulator = WorkerControlFrameAccumulator::new();
    assert!(accumulator.push(&vec![b'x'; maximum + 1]).is_err());
    assert!(accumulator.push(b"{}\n{}\n").is_err());
    assert_native_topology(&usb, &native_source);
    assert_declared_negative_vectors_are_covered(&controller, &possession, &deployment, &usb);
    execute_controller_transfer_negatives(&controller, &deployment);
    execute_possession_request_negatives(&controller, &possession, &deployment);
    execute_deployment_negatives(&deployment);
    execute_usb_negatives(&usb, &native_source);
}

fn fixture(variable: &str) -> Option<Value> {
    let path = env::var(variable).ok()?;
    let text = fs::read_to_string(path).expect("pinned fixture should be readable");
    Some(serde_json::from_str(&text).expect("pinned fixture should be JSON"))
}

fn required_fixture(variable: &str) -> Value {
    fixture(variable).unwrap_or_else(|| panic!("{variable} should be set by Bazel"))
}

fn required_text(variable: &str) -> String {
    let path = env::var(variable).unwrap_or_else(|_| panic!("{variable} should be set by Bazel"));
    fs::read_to_string(path).expect("pinned source should be readable")
}

fn vector<'a>(controller: &'a Value, id: &str) -> &'a Value {
    controller["usbVectors"]
        .as_array()
        .and_then(|vectors| vectors.iter().find(|vector| vector["id"] == id))
        .unwrap_or_else(|| panic!("missing pinned Controller vector {id}"))
}

fn authorized_request(deployment: &Value, operation: &str) -> Value {
    let mut request = deployment[operation]["input"]["request"].clone();
    request["authorization"] = deployment[operation]["artifact"]["authorization"].clone();
    request
}

fn controller_request(request_id: &str, command: &str, payload: Value) -> Value {
    json!({
        "protocolVersion": "bwg-worker-controller/0.3",
        "requestId": request_id,
        "command": command,
        "payload": payload,
    })
}

fn json_line(value: &Value) -> Vec<u8> {
    let mut line = serde_json::to_vec(value).expect("fixture value should encode");
    line.push(b'\n');
    line
}

fn response_value(response: &bitaxe_worker_control::PreparedResponse) -> Value {
    serde_json::from_slice(response.frame()).expect("controller response should be JSON")
}

fn assert_native_topology(usb: &Value, source: &str) {
    let descriptor = &usb["topology"]["application"]["descriptor"];
    let control = &usb["topology"]["application"]["descriptor"]["control"];
    let evidence = &usb["topology"]["application"]["descriptor"]["evidence"];
    let configuration = descriptor["configurationValue"]
        .as_u64()
        .expect("configuration should be numeric");
    let interface = control["interfaceNumber"]
        .as_u64()
        .expect("control interface should be numeric");
    let alternate = control["alternateSetting"]
        .as_u64()
        .expect("alternate setting should be numeric");
    let class = control["classCode"]
        .as_u64()
        .expect("class should be numeric");
    let transfer = control["transferType"]
        .as_str()
        .expect("transfer type should be text");
    let subclass = control["subclassCode"]
        .as_u64()
        .expect("subclass should be numeric");
    let protocol = control["protocolCode"]
        .as_u64()
        .expect("protocol should be numeric");
    let endpoint_out = control["endpointOut"]
        .as_u64()
        .expect("OUT endpoint should be numeric");
    let endpoint_in = control["endpointIn"]
        .as_u64()
        .expect("IN endpoint should be numeric");
    assert_eq!(class, 255);
    assert_eq!(transfer, "bulk");
    assert!(source.contains(&format!("TUD_CONFIG_DESCRIPTOR({configuration},")));
    assert!(source.contains(&format!("BWG_INTERFACE_VENDOR = {interface}")));
    assert!(source.contains(&format!(
        "TUSB_DESC_INTERFACE, BWG_INTERFACE_VENDOR, {alternate}, 2"
    )));
    assert!(source.contains(&format!(
        "TUSB_CLASS_VENDOR_SPECIFIC, 0x{subclass:02x}, 0x{protocol:02x}"
    )));
    assert!(source.contains(&format!("0x{endpoint_out:02x}, TUSB_XFER_BULK")));
    assert!(source.contains(&format!("0x8{endpoint_in:x}, TUSB_XFER_BULK")));
    assert!(source.contains(&format!(
        "BWG_INTERFACE_CDC = {}",
        evidence["communicationInterfaceNumber"]
    )));
    assert!(source.contains(&format!(
        "BWG_INTERFACE_CDC_DATA = {}",
        evidence["dataInterfaceNumber"]
    )));
    let notification = evidence["notificationEndpointIn"]
        .as_u64()
        .expect("notification endpoint should be numeric");
    let data_out = evidence["dataEndpointOut"]
        .as_u64()
        .expect("CDC OUT endpoint should be numeric");
    let data_in = evidence["dataEndpointIn"]
        .as_u64()
        .expect("CDC IN endpoint should be numeric");
    assert!(source.contains(&format!(
        "0x8{notification:x}, 8, 0x{data_out:02x}, 0x8{data_in:x}, 64"
    )));
    assert_eq!(evidence["hostWritesAccepted"], false);
    assert!(source.contains("tud_cdc_n_read(interface, discarded"));
    assert!(!source.contains("bwg_worker_usb_vendor_received(discarded"));
}

fn assert_declared_negative_vectors_are_covered(
    controller: &Value,
    possession: &Value,
    deployment: &Value,
    usb: &Value,
) {
    let expected = [
        (controller, "negativeTransfers", 7),
        (possession, "negativeCases", 13),
        (deployment, "negativeCases", 13),
        (usb, "negativeVectors", 7),
    ];
    for (document, field, count) in expected {
        let vectors = document[field]
            .as_array()
            .unwrap_or_else(|| panic!("{field} should be an array"));
        assert_eq!(vectors.len(), count, "pinned {field} coverage changed");
        assert!(vectors.iter().all(|vector| {
            vector["id"].as_str().is_some_and(|id| !id.is_empty())
                && vector["expectedError"]
                    .as_str()
                    .is_some_and(|error| !error.is_empty())
        }));
    }
}

fn execute_controller_transfer_negatives(controller: &Value, deployment: &Value) {
    let cases = [
        ("empty_transfer", Vec::new(), "invalid_frame"),
        (
            "oversized_transfer",
            vec![b'x'; 65_537],
            "invalid_frame",
        ),
        (
            "invalid_utf8_transfer",
            vec![0xff, b'\n'],
            "invalid_frame",
        ),
        (
            "invalid_json_transfer",
            b"{not-json}\n".to_vec(),
            "invalid_frame",
        ),
        (
            "truncated_transfer",
            b"{\"protocolVersion\":\"bwg-worker-controller/0.3\"}".to_vec(),
            "invalid_frame",
        ),
        (
            "multiple_transfer_frames",
            b"{}\n{}\n".to_vec(),
            "invalid_frame",
        ),
        (
            "unknown_request_field",
            b"{\"protocolVersion\":\"bwg-worker-controller/0.3\",\"requestId\":\"usb_unknown\",\"command\":\"status\",\"unknown\":true}\n".to_vec(),
            "invalid_request",
        ),
    ];
    let declared = controller["negativeTransfers"]
        .as_array()
        .expect("Controller negatives should be an array");
    for (id, frame, expected_category) in cases {
        assert!(declared.iter().any(|vector| vector["id"] == id));
        let mut worker = contract_worker(controller, deployment);
        let error = worker
            .prepare_frame(&frame, 0)
            .expect_err("negative Controller transfer should fail");
        assert_eq!(error.category(), expected_category, "vector {id}");
    }
}

fn execute_possession_request_negatives(
    controller: &Value,
    possession: &Value,
    deployment: &Value,
) {
    let initial = &possession["initialAdmission"]["request"];
    let declared = possession["negativeCases"]
        .as_array()
        .expect("possession negatives should be an array");
    let mut unknown = initial.clone();
    unknown["payload"]["unknown"] = json!(true);
    let arbitrary = json!({
        "profile": "bwg-worker-possession/0.1",
        "requestId": "pos_arbitrary",
        "command": "sign_arbitrary",
        "payload": {"message": "forbidden"},
    });
    let cases = [
        (
            "unknown_request_field",
            json_line(&unknown),
            "invalid_request",
        ),
        (
            "arbitrary_signing_request",
            json_line(&arbitrary),
            "invalid_request",
        ),
        ("oversized_frame", vec![b'x'; 65_537], "invalid_frame"),
    ];
    for (id, frame, expected_category) in cases {
        assert!(declared.iter().any(|vector| vector["id"] == id));
        let mut worker = contract_worker(controller, deployment);
        let error = worker
            .prepare_frame(&frame, 0)
            .expect_err("negative possession request should fail");
        assert_eq!(error.category(), expected_category, "vector {id}");
    }
}

fn execute_deployment_negatives(deployment: &Value) {
    let context = WorkerLeaseAuthorizationContext::parse(
        deployment["start"]["input"]["controlSessionBindingSha256"]
            .as_str()
            .expect("fixture context should be text"),
    )
    .expect("fixture context should parse");
    let trust_json = deployment["trust"].to_string();

    let mut changed_request = authorized_request(deployment, "start");
    changed_request["stratum"]["password"] = json!("changed");
    let changed_grant: WorkerLeaseGrant =
        serde_json::from_value(changed_request).expect("changed fixture grant should parse");
    let trust = WorkLeaseAuthorityTrust::from_deployment_json(&trust_json)
        .expect("fixture trust should parse");
    let mut verifier = WorkLeaseAuthorizationVerifier::new(trust, MemorySequenceStore::default());
    assert_eq!(
        verifier
            .verify_start(&changed_grant, &context)
            .expect_err("changed request should fail")
            .category(),
        "invalid_authorization"
    );

    let grant: WorkerLeaseGrant = serde_json::from_value(authorized_request(deployment, "start"))
        .expect("fixture grant should parse");
    let trust = WorkLeaseAuthorityTrust::from_deployment_json(&trust_json)
        .expect("fixture trust should parse");
    let mut verifier = WorkLeaseAuthorizationVerifier::new(trust, MemorySequenceStore::default());
    verifier
        .verify_start(&grant, &context)
        .expect("first sequence should verify");
    assert_eq!(
        verifier
            .verify_start(&grant, &context)
            .expect_err("accepted sequence should not replay")
            .category(),
        "replay"
    );

    let changed_context = WorkerLeaseAuthorizationContext::parse(&"T".repeat(43))
        .expect("changed context should parse");
    let trust = WorkLeaseAuthorityTrust::from_deployment_json(&trust_json)
        .expect("fixture trust should parse");
    let mut verifier = WorkLeaseAuthorizationVerifier::new(trust, MemorySequenceStore::default());
    assert_eq!(
        verifier
            .verify_start(&grant, &changed_context)
            .expect_err("changed context should fail")
            .category(),
        "invalid_authorization"
    );

    let mut oversized = authorized_request(deployment, "start");
    oversized["authorization"] = json!("x".repeat(513));
    let oversized: WorkerLeaseGrant =
        serde_json::from_value(oversized).expect("oversized grant should remain syntactic JSON");
    let trust = WorkLeaseAuthorityTrust::from_deployment_json(&trust_json)
        .expect("fixture trust should parse");
    let mut verifier = WorkLeaseAuthorizationVerifier::new(trust, MemorySequenceStore::default());
    assert_eq!(
        verifier
            .verify_start(&oversized, &context)
            .expect_err("oversized authorization should fail")
            .category(),
        "invalid_authorization"
    );

    for id in [
        "changed_request",
        "changed_context",
        "same_nonce_different_identity",
        "replayed_sequence",
        "cross_session_replay",
        "authorization_513_bytes",
    ] {
        assert!(deployment["negativeCases"]
            .as_array()
            .is_some_and(|vectors| vectors.iter().any(|vector| vector["id"] == id)));
    }
}

fn execute_usb_negatives(usb: &Value, source: &str) {
    let declared = usb["negativeVectors"]
        .as_array()
        .expect("USB negatives should be an array");
    let declared_id = |id: &str| declared.iter().any(|vector| vector["id"] == id);

    assert!(declared_id("wrong_function_role"));
    assert!(source.contains("tud_vendor_rx_cb"));
    assert!(source.contains("tud_cdc_rx_cb"));
    assert!(declared_id("ambiguous_control_functions"));
    assert_eq!(source.matches("TUSB_CLASS_VENDOR_SPECIFIC").count(), 1);
    assert!(declared_id("bootloader_control_attempt"));
    assert!(!source.contains("USB_SERIAL_JTAG"));
    assert!(declared_id("unknown_profile_field"));
    assert!(source.contains("bwg_worker_usb_vendor_received"));
    assert!(declared_id("control_descriptor_drift"));
    assert_native_topology(usb, source);
    assert!(declared_id("multiple_frame_transfer"));
    let mut accumulator = WorkerControlFrameAccumulator::new();
    assert!(accumulator.push(b"{}\n{}\n").is_err());
    assert!(declared_id("runtime_log_injection"));
    assert!(accumulator.push(b"runtime-log\n{}\n").is_err());
}

fn contract_worker(
    controller: &Value,
    deployment: &Value,
) -> WorkerControl<WorkLeaseAuthorizationVerifier<MemorySequenceStore>, ConformanceSession> {
    let capability = vector(controller, "v03_discover")["response"]["result"].clone();
    let trust = WorkLeaseAuthorityTrust::from_deployment_json(&deployment["trust"].to_string())
        .expect("fixture trust should parse");
    WorkerControl::new(
        DeviceIdentity::from_seed(FIXTURE_DEVICE_SEED),
        WorkLeaseAuthorizationVerifier::new(trust, MemorySequenceStore::default()),
        ConformanceSession,
        None,
        fixture_source_commit(),
        capability,
        "rOKO_7whZfy0ntMKM9RIeZNAA3x97tt3rWMAm_QshVA",
    )
    .expect("fixture Worker should configure")
}

fn fixture_source_commit() -> bitaxe_worker_control::FirmwareSourceCommit {
    bitaxe_worker_control::FirmwareSourceCommit::parse(&"a".repeat(40))
        .expect("fixture source commit should parse")
}
