use super::*;
use crate::v1::production_work::PoolSessionGeneration;
use crate::v2::{
    messages::*,
    standard::{BASE_VERSION, NBITS, TARGET},
};
fn configured() -> DeterministicProductionSessionAdapter {
    let config = ProductionPoolConfiguration {
        endpoint: ProductionPoolEndpoint {
            host: "192.168.1.3".into(),
            port: 12345,
        },
        runtime: ProductionProtocolConfig::V2(V2PoolConfig {
            authority: [2; 32],
            user_identity: "synthetic".into(),
            firmware: "test".into(),
        }),
    };
    let mut a = DeterministicProductionSessionAdapter::new(Some(ProductionPoolSet {
        primary: Some(config),
        fallback: None,
        prefer_fallback: false,
    }));
    a.drive(wake(ready(), 0));
    a.connect(ProductionPool::Primary, 1);
    a
}
fn feed(a: &mut DeterministicProductionSessionAdapter, frame: crate::v2::frame::Frame, time: u64) {
    a.drive(ProductionSessionEvent::TransportFrame {
        pool: ProductionPool::Primary,
        transport_epoch: a.maybe_primary_transport_epoch.expect("transport"),
        frame,
        now_ms: time,
    });
}
fn running() -> DeterministicProductionSessionAdapter {
    let mut a = configured();
    for (i, frame) in [
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
        .expect("previous"),
    ]
    .into_iter()
    .enumerate()
    {
        feed(&mut a, frame, 10 + i as u64);
    }
    a
}
#[test]
fn standard_single_job_keeps_polling_after_two_seconds_without_refeeding() {
    // Arrange
    let mut a = running();
    let generation = a.session.snapshot().generation;
    assert_eq!(a.asic_commands.len(), 1);
    let Bm1366ProductionCommand::SendProductionWork(payload) = a.asic_commands[0] else {
        panic!("actual dispatch effect")
    };
    // Act: actual driver completion, followed by delayed result-poll completion.
    a.drive(ProductionSessionEvent::AsicDispatched {
        generation,
        job_id: payload.job_id(),
        now_ms: 20,
    });
    let boundary = a.effects.len();
    a.drive(wake(ready(), 5000));
    a.drive(ProductionSessionEvent::AsicPollTimedOut {
        generation,
        now_ms: 5001,
    });
    // Assert
    assert_eq!(
        a.asic_commands.len(),
        1,
        "same standard header must not restart nonce search"
    );
    assert!(a.effects[boundary..]
        .iter()
        .any(|e| matches!(e, ProductionSessionEffect::PollAsic { slice_ms: 50, .. })));
}
#[test]
fn stale_nonce_cannot_queue_a_share_in_the_v2_owner() {
    // Arrange
    let mut a = running();
    let before = a.effects.len();
    // Act
    a.drive(ProductionSessionEvent::AsicResult {
        observation: ProductionNonceObservation {
            observed_generation: PoolSessionGeneration::initial(),
            result: bitaxe_asic::bm1366::result::Bm1366NonceResult {
                job_id: bitaxe_asic::bm1366::work::Bm1366JobId::new(0),
                nonce: 0,
                version_bits: 0,
                asic_index: 0,
                core_id: 0,
                small_core_id: 0,
            },
        },
        now_ms: 30,
    });
    // Assert
    assert!(!a.effects[before..]
        .iter()
        .any(|e| matches!(e, ProductionSessionEffect::WritePoolFrame { .. })));
}
#[test]
fn protocol_failure_uses_ordered_safe_stop_without_v1_retry() {
    // Arrange
    let mut a = configured();
    let connections = a.connections.len();
    // Act
    feed(
        &mut a,
        SetupConnectionSuccess {
            used_version: 2,
            flags: 1,
        }
        .encode()
        .expect("unsupported rolling policy"),
        10,
    );
    a.drive(wake(ready(), 30_000));
    // Assert
    assert_eq!(a.connections.len(), connections);
    assert_eq!(
        a.session.snapshot().hardware_state,
        MiningHardwareState::Stopped
    );
    assert!(a
        .effects
        .iter()
        .any(|e| matches!(e, ProductionSessionEffect::RecordV2Failure { .. })));
    assert!(a
        .effects
        .iter()
        .any(|e| matches!(e, ProductionSessionEffect::InvalidateWorkAndSubmissions)));
}
