use super::*;
use crate::v2::messages::{
    OpenStandardMiningChannelSuccess, SetupConnectionSuccess, SubmitSharesSuccess,
};

#[test]
fn standard_lifecycle_orders_setup_channel_future_job_and_prev_hash_work() {
    // Arrange
    let mut session = standard_session();

    // Act
    let setup = session.start().expect("setup");
    let open = session
        .handle(ServerMessage::SetupConnectionSuccess(
            SetupConnectionSuccess {
                used_version: 2,
                flags: 1,
            },
        ))
        .expect("open");
    let ready = session
        .handle(ServerMessage::OpenStandardMiningChannelSuccess(
            OpenStandardMiningChannelSuccess {
                request_id: 1,
                channel_id: 7,
                target: [0xff; 32],
                extranonce_prefix: vec![],
                group_channel_id: 0,
            },
        ))
        .expect("ready");
    let pending = session
        .handle(ServerMessage::NewMiningJob(standard_job(7, 9, None)))
        .expect("pending");
    let work = session
        .handle(ServerMessage::SetNewPrevHash(previous_hash(7, 9)))
        .expect("work");

    // Assert
    assert!(matches!(setup, SessionEvent::Outbound(_)));
    assert!(matches!(open.as_slice(), [SessionEvent::Outbound(_)]));
    assert!(matches!(
        ready.as_slice(),
        [SessionEvent::ChannelReady { .. }]
    ));
    assert!(pending.is_empty());
    assert!(matches!(work.as_slice(), [SessionEvent::Work(_)]));
    assert_eq!(session.phase(), SessionPhase::Active);
}

#[test]
fn standard_share_is_qualified_deduplicated_and_correlated_to_success() {
    // Arrange
    let mut session = active_standard_session([0xff; 32]);
    session
        .handle(ServerMessage::NewMiningJob(standard_job(7, 9, Some(3))))
        .expect("work");
    let result = nonce_result(0);

    // Act
    let submit = session.observe_nonce(result).expect("submit");
    let duplicate = session.observe_nonce(result).expect("duplicate");
    let accepted = session
        .handle(ServerMessage::SubmitSharesSuccess(SubmitSharesSuccess {
            channel_id: 7,
            last_sequence_number: 0,
            accepted_count: 1,
            shares_sum: 1,
        }))
        .expect("accepted");

    // Assert
    assert!(matches!(submit, Some(SessionEvent::Outbound(_))));
    assert_eq!(duplicate, None);
    assert_eq!(
        accepted,
        vec![SessionEvent::ShareAccepted { accepted_count: 1 }]
    );
}

#[test]
fn wrong_channel_fails_closed_and_clears_work_before_publication() {
    // Arrange
    let mut session = active_standard_session([0xff; 32]);
    session
        .handle(ServerMessage::NewMiningJob(standard_job(7, 9, Some(3))))
        .expect("work");

    // Act
    let events = session
        .handle(ServerMessage::SetNewPrevHash(previous_hash(8, 9)))
        .expect("failure");
    let stale_result = session.observe_nonce(nonce_result(0));

    // Assert
    assert_eq!(
        events,
        vec![SessionEvent::Failed(SessionFailure::ChannelMismatch)]
    );
    assert_eq!(session.phase(), SessionPhase::Failed);
    assert!(stale_result.is_err());
}

#[test]
fn session_debug_never_renders_endpoint_user_or_job_values() {
    // Arrange
    let mut config = config(ChannelKind::Standard);
    config.endpoint_host = "private-endpoint-canary".to_owned();
    config.user_identity = "private-user-canary".to_owned();
    let session = V2Session::new(config).expect("session");

    // Act
    let rendered = format!("{session:?}");

    // Assert
    assert!(!rendered.contains("private-endpoint-canary"));
    assert!(!rendered.contains("private-user-canary"));
}

fn standard_session() -> V2Session {
    V2Session::new(config(ChannelKind::Standard)).expect("session")
}

fn active_standard_session(target: [u8; 32]) -> V2Session {
    let mut session = standard_session();
    session.start().expect("setup");
    session
        .handle(ServerMessage::SetupConnectionSuccess(
            SetupConnectionSuccess {
                used_version: 2,
                flags: 1,
            },
        ))
        .expect("open");
    session
        .handle(ServerMessage::OpenStandardMiningChannelSuccess(
            OpenStandardMiningChannelSuccess {
                request_id: 1,
                channel_id: 7,
                target,
                extranonce_prefix: vec![],
                group_channel_id: 0,
            },
        ))
        .expect("ready");
    session
        .handle(ServerMessage::SetNewPrevHash(previous_hash(7, 8)))
        .expect("prev hash");
    session
}

fn config(channel_kind: ChannelKind) -> SessionConfig {
    SessionConfig {
        endpoint_host: "pool".to_owned(),
        endpoint_port: 3333,
        vendor: "bitaxe".to_owned(),
        hardware_version: "BM1366".to_owned(),
        firmware: String::new(),
        device_id: String::new(),
        user_identity: "worker".to_owned(),
        nominal_hashrate: 1.0e12,
        channel_kind,
        minimum_extranonce_size: 6,
    }
}

fn standard_job(channel_id: u32, job_id: u32, maybe_min_ntime: Option<u32>) -> NewMiningJob {
    NewMiningJob {
        channel_id,
        job_id,
        maybe_min_ntime,
        version: 4,
        merkle_root: [0x11; 32],
    }
}

fn previous_hash(channel_id: u32, job_id: u32) -> SetNewPrevHash {
    SetNewPrevHash {
        channel_id,
        job_id,
        prev_hash: [0x22; 32],
        min_ntime: 3,
        nbits: 4,
    }
}

fn nonce_result(job_id: u8) -> Bm1366NonceResult {
    Bm1366NonceResult {
        job_id: Bm1366JobId::new(job_id),
        nonce: 1,
        asic_index: 0,
        core_id: 0,
        small_core_id: 0,
        version_bits: 0,
    }
}
