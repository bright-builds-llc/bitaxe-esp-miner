use super::*;
use crate::v2::messages::*;
fn session() -> StandardSession {
    StandardSession::new(SessionConfig {
        endpoint_host: "192.168.1.3".into(),
        endpoint_port: 12345,
        vendor: "test".into(),
        hardware_version: "205".into(),
        firmware: "test".into(),
        device_id: String::new(),
        user_identity: "synthetic".into(),
        nominal_hashrate: 1.0,
        channel_kind: ChannelKind::Standard,
        minimum_extranonce_size: 0,
    })
    .expect("configuration")
}
fn transcript() -> [Frame; 5] {
    [
        SetupConnectionSuccess {
            used_version: 2,
            flags: 0,
        }
        .encode()
        .expect("setup"),
        OpenStandardMiningChannelSuccess {
            request_id: 1,
            channel_id: 9,
            target: TARGET,
            extranonce_prefix: Vec::new(),
            group_channel_id: 0,
        }
        .encode()
        .expect("channel"),
        NewMiningJob {
            channel_id: 9,
            job_id: 7,
            maybe_min_ntime: None,
            version: BASE_VERSION,
            merkle_root: [3; 32],
        }
        .encode()
        .expect("job"),
        SetTarget {
            channel_id: 9,
            maximum_target: TARGET,
        }
        .encode()
        .expect("target"),
        SetNewPrevHash {
            channel_id: 9,
            job_id: 7,
            prev_hash: [4; 32],
            min_ntime: 1000,
            nbits: NBITS,
        }
        .encode()
        .expect("previous hash"),
    ]
}
#[test]
fn standard_handshake_requires_explicit_rolling_and_coherent_single_job() {
    // Arrange
    let mut s = session();
    let start = s.start().expect("start");
    // Act
    let events: Vec<_> = transcript()
        .iter()
        .map(|f| s.receive(f).expect("exact message order"))
        .collect();
    // Assert
    assert_eq!(&start.payload()[5..9], &5u32.to_le_bytes());
    assert_eq!(s.phase, 5);
    assert!(matches!(
        events[4].first(),
        Some(StandardEvent::Work { .. })
    ));
    assert_eq!(s.maybe_work().expect("work").pool_target, TARGET);
    assert!(
        s.receive(&transcript()[2]).is_err(),
        "never replace or refeed the frozen job"
    );
}
#[test]
fn every_wrong_extension_and_wrong_order_fails_before_work() {
    for index in 0..5 {
        // Arrange
        let mut s = session();
        s.start().expect("start");
        let mut frames = transcript();
        for f in &frames[..index] {
            s.receive(f).expect("prefix");
        }
        frames[index].header.extension_type ^= 0x8000;
        // Act / Assert
        assert!(s.receive(&frames[index]).is_err());
    }
    let mut s = session();
    s.start().expect("start");
    assert!(s.receive(&transcript()[1]).is_err());
}
#[test]
fn uncompleted_write_and_forged_batch_ack_cannot_credit_a_share() {
    // Arrange: isolate acknowledgement correlation after independent qualification.
    let mut s = session();
    s.phase = 5;
    s.maybe_channel = Some(9);
    s.pending.insert(
        0,
        SubmitSharesStandard {
            channel_id: 9,
            sequence_number: 0,
            job_id: 7,
            nonce: 1,
            ntime: 1000,
            version: BASE_VERSION,
        },
    );
    let ack = SubmitSharesSuccess {
        channel_id: 9,
        last_sequence_number: 0,
        accepted_count: 1,
        shares_sum: 1024,
    }
    .encode()
    .expect("ack");
    // Act / Assert
    assert!(s.receive(&ack).is_err(), "queueing is not writing");
    s.written(0).expect("actual completed write");
    let inflated = SubmitSharesSuccess {
        channel_id: 9,
        last_sequence_number: 0,
        accepted_count: 2,
        shares_sum: 2048,
    }
    .encode()
    .expect("forged ack");
    assert!(s.receive(&inflated).is_err());
}
#[test]
fn nominal_pool_target_does_not_change_the_hardware_ticket_filter() {
    // Arrange / Act / Assert
    assert_eq!(super::super::work::target_to_pdiff(TARGET), 1024);
    assert_eq!(
        bitaxe_asic::bm1366::mining_ready::difficulty_mask_value(256.0),
        [0, 0, 0, 255]
    );
}
