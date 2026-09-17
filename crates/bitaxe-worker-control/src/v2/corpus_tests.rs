use super::*;
use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};
fn observation() -> CurrentObservation {
    CurrentObservation {
        boot_ordinal: 2,
        worker_generation: 7,
        serial_transport_epoch: 4,
        maybe_observed_at_us: Some(1000),
        clock_valid: true,
        maybe_station_ipv4: Some("192.168.1.2".into()),
        wifi_connected: true,
        maybe_socket: None,
    }
}
fn invocation(r: &mut V2Record, op: Operation, start: u64, end: u64) {
    r.begin(op, Some(start));
    r.end(op, Some(end), false);
}
fn read(r: &mut V2Record, start: u64) {
    r.begin(Operation::FrameRead, Some(start));
    invocation(r, Operation::HeaderDecrypt, start + 1, start + 2);
    invocation(r, Operation::PayloadDecrypt, start + 3, start + 4);
    r.end(Operation::FrameRead, Some(start + 5), false);
}
fn ready(scope: Scope) -> V2Record {
    let mut r = V2Record::admit(
        scope,
        URL_SAFE_NO_PAD.encode([1; 16]),
        &observation(),
        if scope == Scope::Channel {
            Some(120001000)
        } else {
            None
        },
    )
    .expect("record");
    r.bind_pool(3, 5);
    r.event(Stage::Preparing, Some(1100), None, None, None, None);
    invocation(&mut r, Operation::InitiatorConstruction, 1100, 1110);
    invocation(&mut r, Operation::ActOneConstruction, 1110, 1120);
    invocation(&mut r, Operation::Connect, 1120, 1130);
    r.event(Stage::Connected, Some(1130), None, None, None, None);
    invocation(&mut r, Operation::ActOneWrite, 1140, 1150);
    invocation(&mut r, Operation::ActTwoRead, 1160, 1170);
    invocation(&mut r, Operation::ActTwoAuthentication, 1170, 1180);
    r.event(Stage::Authenticated, Some(1180), None, None, None, None);
    invocation(&mut r, Operation::FrameEncrypt, 1190, 1191);
    invocation(&mut r, Operation::FrameWrite, 1192, 1193);
    read(&mut r, 1200);
    r.event(Stage::Setup, Some(1205), None, None, None, None);
    invocation(&mut r, Operation::FrameEncrypt, 1210, 1211);
    invocation(&mut r, Operation::FrameWrite, 1212, 1213);
    read(&mut r, 1230);
    r.event(Stage::Channel, Some(1235), Some(9), None, None, None);
    read(&mut r, 1240);
    r.event(
        Stage::Job,
        Some(1245),
        Some(9),
        Some(7),
        None,
        Some("2".repeat(64)),
    );
    read(&mut r, 1250);
    r.event(
        Stage::Target,
        Some(1255),
        Some(9),
        Some(7),
        None,
        Some("3".repeat(64)),
    );
    read(&mut r, 1260);
    r.set_job_commitment("5".repeat(64));
    r.event(
        Stage::WorkReady,
        Some(1265),
        Some(9),
        Some(7),
        None,
        Some("4".repeat(64)),
    );
    r
}
fn close(r: &mut V2Record, start: u64) {
    r.begin(Operation::WorkerJoin, Some(start));
    invocation(r, Operation::SocketClose, start + 1, start + 2);
    r.socket_closed(Some(start + 2));
    r.end(Operation::WorkerJoin, Some(start + 10), false);
    r.joined(Some(start + 10), true);
}
#[test]
fn accepted_and_secondary_clock_corpus_uses_actual_record_serializer() {
    // Arrange: simulated clock/effect observations for parser interoperability only.
    let mut channel = ready(Scope::Channel);
    close(&mut channel, 1300);
    let mut share = ready(Scope::Share);
    share.budget_armed(2, 180_000, Some(2000));
    share.event(
        Stage::AsicDispatch,
        Some(2100),
        Some(9),
        Some(7),
        None,
        None,
    );
    share.event(Stage::Nonce, Some(2200), Some(9), Some(7), None, None);
    share.add_share(ShareFact {
        dispatch_sequence: 1,
        asic_job_id: 0,
        work_fields_sha256: "6".repeat(64),
        dispatched_at_device_us: 2100,
        nonce_at_device_us: 2200,
        maybe_write_started_at_device_us: None,
        maybe_write_completed_at_device_us: None,
        nonce: 1,
        version_bits: 0,
        asic_index: 0,
        core_id: 0,
        small_core_id: 0,
        channel_id: 9,
        job_id: 7,
        submission_sequence: 0,
        ntime: 1000,
        version: 0x20000000,
        maybe_ack_at_device_us: None,
        maybe_ack_last_sequence: None,
        maybe_ack_accepted_count: None,
        maybe_ack_shares_sum: None,
        maybe_matched_submit_count: None,
    });
    invocation(&mut share, Operation::FrameEncrypt, 2210, 2211);
    share.begin(Operation::FrameWrite, Some(2212));
    share
        .share_mut(0)
        .expect("fact")
        .maybe_write_started_at_device_us = Some(2212);
    share.event(
        Stage::Submission,
        Some(2212),
        Some(9),
        Some(7),
        Some(0),
        None,
    );
    share
        .share_mut(0)
        .expect("fact")
        .maybe_write_completed_at_device_us = Some(2213);
    share.end(Operation::FrameWrite, Some(2213), false);
    read(&mut share, 2220);
    assert!(share.observe_acknowledgement(
        ObservedAcknowledgement {
            channel_id: 9,
            last_sequence: 0,
            accepted_count: 1,
            shares_sum: 1024
        },
        "7".repeat(64),
        Some(2225),
    ));
    share.event(Stage::Revoked, Some(3000), None, None, None, None);
    share.event(Stage::Shutdown, Some(3001), None, None, None, None);
    close(&mut share, 3010);
    share.event(Stage::Cooled, Some(4000), None, None, None, None);
    let mut clock = ready(Scope::Channel);
    clock.fail(Stage::WorkReady, FailureCategory::Protocol, Some(1270));
    clock.begin(Operation::WorkerJoin, Some(1300));
    clock.begin(Operation::SocketClose, Some(1301));
    clock.end(Operation::SocketClose, None, true);
    clock.socket_closed(None);
    clock.end(Operation::WorkerJoin, None, true);
    clock.joined(None, true);
    let mut cancelled = ready(Scope::Channel);
    cancelled.fail(Stage::Job, FailureCategory::Protocol, Some(1270));
    cancelled.fail(
        Stage::WorkerQuiescent,
        FailureCategory::Authority,
        Some(1271),
    );
    close(&mut cancelled, 1300);
    // Act
    let cases:Vec<_>=[("accepted_channel_wire_only",channel.snapshot()),("accepted_share_wire_only",share.snapshot()),("protocol_then_clock",clock.snapshot()),("protocol_then_revocation",cancelled.snapshot())].into_iter().map(|(name,r)|{
        let mut obs=observation();obs.maybe_observed_at_us=r.maybe_observed_at_us;obs.clock_valid=r.maybe_observed_at_us.is_some();
        serde_json::json!({"name":name,"status":V2Status {schema:"worker-stratum-v2-status-v1",scope:r.scope,state:r.state,observation:obs,
            maybe_connection:Some(RetainedConnection {observed_at_us:1130,boot_ordinal:2,worker_generation:7,serial_transport_epoch:4,pool_session_generation:3,pool_transport_epoch:5,socket:SocketTuple {local_ipv4:"192.168.1.2".into(),local_port:12345,remote_ipv4:"192.168.1.3".into(),remote_port:54321}}),maybe_record:Some(r)}})
    }).collect();
    // Assert
    assert_eq!(cases[0]["status"]["record"]["outcome"], "accepted");
    assert_eq!(
        cases[3]["status"]["record"]["firstFailure"]["category"],
        "protocol"
    );
    assert_eq!(
        cases[3]["status"]["record"]["secondaryFailures"],
        serde_json::json!([])
    );
    assert_eq!(cases[3]["status"]["record"]["outcome"], "rejected");
    assert_eq!(cases[1]["status"]["record"]["outcome"], "accepted");
    assert_eq!(
        cases[2]["status"]["record"]["firstFailure"]["category"],
        "protocol"
    );
    if let Ok(path) = std::env::var("BWG_V2_POSITIVE_CORPUS_PATH") {
        std::fs::write(
            path,
            serde_json::to_vec_pretty(&cases).expect("producer serde"),
        )
        .expect("explicit synthetic output");
    }
}
