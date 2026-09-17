use super::*;
use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};
const MAX: u64 = crate::noise::MAX_SAFE_INTEGER;
fn fact() -> ShareFact {
    ShareFact {
        dispatch_sequence: MAX,
        asic_job_id: 120,
        work_fields_sha256: "f".repeat(64),
        dispatched_at_device_us: MAX,
        nonce_at_device_us: MAX,
        maybe_write_started_at_device_us: Some(MAX),
        maybe_write_completed_at_device_us: Some(MAX),
        nonce: u32::MAX,
        version_bits: u32::MAX,
        asic_index: 0,
        core_id: 111,
        small_core_id: 7,
        channel_id: u32::MAX,
        job_id: u32::MAX,
        submission_sequence: u32::MAX,
        ntime: u32::MAX,
        version: u32::MAX,
        maybe_ack_at_device_us: Some(MAX),
        maybe_ack_last_sequence: Some(u32::MAX),
        maybe_ack_accepted_count: Some(u32::MAX),
        maybe_ack_shares_sum: Some(MAX),
        maybe_matched_submit_count: Some(u32::MAX),
    }
}
#[test]
fn worst_bounded_reply_including_controller_and_private_runtime_metadata_fits_65536() {
    // Arrange: conservative encoding maxima, not a fabricated successful lifecycle.
    let socket = SocketTuple {
        local_ipv4: "192.168.255.255".into(),
        local_port: u16::MAX,
        remote_ipv4: "192.168.255.255".into(),
        remote_port: u16::MAX,
    };
    let observation = CurrentObservation {
        boot_ordinal: MAX,
        worker_generation: MAX,
        serial_transport_epoch: MAX,
        maybe_observed_at_us: Some(MAX),
        clock_valid: false,
        maybe_station_ipv4: Some("192.168.255.255".into()),
        wifi_connected: false,
        maybe_socket: Some(socket.clone()),
    };
    let base = CurrentObservation {
        maybe_observed_at_us: Some(1),
        clock_valid: true,
        ..observation.clone()
    };
    let mut record = V2Record::admit(Scope::Share, URL_SAFE_NO_PAD.encode([255; 16]), &base, None)
        .expect("record")
        .snapshot();
    record.maybe_pool_session_generation = Some(MAX);
    record.maybe_pool_transport_epoch = Some(MAX);
    record.maybe_job_commitment = Some("f".repeat(64));
    record.maybe_observed_at_us = Some(MAX);
    record.state = State::Terminal;
    record.admitted_at_device_us = MAX;
    record.maybe_authority_deadline_device_us = Some(MAX);
    record.maybe_observation_deadline_device_us = Some(MAX);
    record.maybe_terminal_at_device_us = Some(MAX);
    record.maybe_outcome = Some(Outcome::Incomplete);
    record.events = (0..MAX_EVENTS)
        .map(|_| Event {
            sequence: MAX,
            maybe_at_device_us: Some(MAX),
            kind: Stage::WorkerQuiescent,
            maybe_channel_id: Some(u32::MAX),
            maybe_job_id: Some(u32::MAX),
            maybe_submission_sequence: Some(u32::MAX),
            maybe_payload_sha256: Some("f".repeat(64)),
        })
        .collect();
    for timing in &mut record.timings {
        timing.count = MAX;
        timing.failed_count = MAX;
        timing.maybe_max_duration_us = Some(MAX);
        timing.maybe_total_duration_us = Some(MAX);
        timing.maybe_first_started_at_device_us = Some(MAX);
        timing.maybe_last_finished_at_device_us = Some(MAX);
        timing.maybe_in_flight_started_at_device_us = Some(MAX);
    }
    record.share_facts = (0..MAX_SHARES).map(|_| fact()).collect();
    let failure = Failure {
        stage: Stage::WorkerQuiescent,
        category: FailureCategory::ChannelMismatch,
        maybe_at_device_us: Some(MAX),
    };
    record.maybe_first_failure = Some(failure);
    record.secondary_failures = vec![failure; MAX_FAILURES];
    record.resources = Resources {
        socket_closed: false,
        worker_quiescent: false,
        fence_retained: false,
        maybe_socket_closed_at_us: Some(MAX),
        maybe_worker_quiescent_at_us: Some(MAX),
    };
    let status = V2Status {
        schema: "worker-stratum-v2-status-v1",
        scope: Scope::Channel,
        state: State::Terminal,
        observation,
        maybe_connection: Some(RetainedConnection {
            observed_at_us: MAX,
            boot_ordinal: MAX,
            worker_generation: MAX,
            serial_transport_epoch: MAX,
            pool_session_generation: MAX,
            pool_transport_epoch: MAX,
            socket,
        }),
        maybe_record: Some(record),
    };
    // Act
    let response=serde_json::to_vec(&serde_json::json!({"protocolVersion":"bwg-worker-controller/0.4","requestId":format!("serial_{}","x".repeat(128)),"ok":true,"result":status})).expect("actual serde reply");
    // Assert: one final controller LF is included in the physical bound.
    assert!(response.len() < 65536, "{} bytes", response.len() + 1);
}
#[test]
fn runtime_debug_never_formats_pool_tuples() {
    // Arrange
    let socket = SocketTuple {
        local_ipv4: "192.168.1.2".into(),
        local_port: 12345,
        remote_ipv4: "192.168.1.3".into(),
        remote_port: 54321,
    };
    // Act / Assert
    let printed = format!("{socket:?}");
    assert!(printed.contains("redacted"));
    assert!(!printed.contains("192.168"));
    assert!(!printed.contains("54321"));
}
