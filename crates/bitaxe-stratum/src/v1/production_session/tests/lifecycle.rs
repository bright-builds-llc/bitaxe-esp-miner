use super::*;

#[test]
fn every_readiness_blocker_prevents_secret_network_and_asic_effects() {
    // Arrange
    let cases = [
        ProductionReadiness {
            operator_intent: MiningOperatorIntent::Paused,
            ..ready()
        },
        ProductionReadiness {
            network_ready: false,
            ..ready()
        },
        ProductionReadiness {
            stratum_v1_supported: false,
            ..ready()
        },
        ProductionReadiness {
            safety_prerequisites_fresh: false,
            ..ready()
        },
        ProductionReadiness {
            production_asic_ready: false,
            ..ready()
        },
        ProductionReadiness {
            actuation_qualified: false,
            ..ready()
        },
    ];

    // Act / Assert
    for readiness in cases {
        let mut adapter = DeterministicProductionSessionAdapter::new(Some(pools(false)));
        adapter.drive(wake(readiness, 0));
        assert_eq!(adapter.pool_reads, 0);
        assert!(adapter.connections.is_empty());
        assert!(adapter.writes.is_empty());
        assert!(adapter.asic_commands.is_empty());
    }
}

#[test]
fn admitted_lifecycle_frames_protocol_dispatches_and_accepts_share() {
    // Arrange
    let mut adapter = DeterministicProductionSessionAdapter::new(Some(pools(false)));
    establish_active(&mut adapter);
    let observation = dispatched_observation(&adapter);

    // Act
    adapter.drive(ProductionSessionEvent::AsicResult {
        observation,
        now_ms: 4,
    });
    adapter.bytes(
        ProductionPool::Primary,
        b"{\"id\":4,\"result\":true,\"error\":null}\n",
        5,
    );

    // Assert
    assert_eq!(adapter.pool_reads, 1);
    assert_eq!(adapter.connections, [ProductionPool::Primary]);
    assert!(adapter
        .writes
        .iter()
        .any(|(_, line)| line.contains("mining.configure")));
    assert!(adapter
        .writes
        .iter()
        .any(|(_, line)| line.contains("mining.subscribe")));
    assert!(adapter
        .writes
        .iter()
        .any(|(_, line)| line.contains("mining.authorize")));
    assert!(adapter
        .writes
        .iter()
        .any(|(_, line)| line.contains("mining.submit")));
    assert_eq!(adapter.asic_commands.len(), 1);
    let snapshot = adapter.session.snapshot();
    assert_eq!(snapshot.phase, ProductionSessionPhase::RunningPrimary);
    assert_eq!(snapshot.mining.counters.accepted, 1);
    assert_eq!(snapshot.mining.counters.rejected, 0);
}

#[test]
fn work_received_before_authorization_remains_safe_blocked() {
    // Arrange
    let mut adapter = DeterministicProductionSessionAdapter::new(Some(pools(false)));
    adapter.drive(wake(ready(), 0));
    adapter.connect(ProductionPool::Primary, 1);
    adapter.asic_commands.clear();

    // Act
    adapter.bytes(
        ProductionPool::Primary,
        concat!(
            "{\"id\":null,\"method\":\"mining.set_difficulty\",\"params\":[42]}\n",
            "{\"id\":null,\"method\":\"mining.notify\",\"params\":[\"early-job\",",
            "\"0000000000000000000000000000000000000000000000000000000000000000\",",
            "\"ffffffff\",\"ffffffff\",[],\"20000004\",\"1705ae3a\",",
            "\"647025b5\",true]}\n"
        ),
        2,
    );

    // Assert
    assert_eq!(
        adapter.session.snapshot().phase,
        ProductionSessionPhase::ConnectingPrimary
    );
    assert_eq!(
        adapter.session.snapshot().mining.work_submission,
        WorkSubmissionGate::Blocked
    );
    assert!(adapter.asic_commands.is_empty());
}

#[test]
fn rejected_submit_is_counted_with_redacted_reason() {
    // Arrange
    let mut adapter = DeterministicProductionSessionAdapter::new(Some(pools(false)));
    establish_active(&mut adapter);
    let observation = dispatched_observation(&adapter);
    adapter.drive(ProductionSessionEvent::AsicResult {
        observation,
        now_ms: 4,
    });

    // Act
    adapter.bytes(
        ProductionPool::Primary,
        b"{\"id\":4,\"result\":false,\"error\":[21,\"raw reject\",null]}\n",
        5,
    );

    // Assert
    let snapshot = adapter.session.snapshot();
    assert_eq!(snapshot.mining.counters.accepted, 0);
    assert_eq!(snapshot.mining.counters.rejected, 1);
    assert_eq!(
        snapshot.mining.counters.rejected_reasons,
        ["pool_rejected_share"]
    );
    assert!(!format!("{:?}", adapter.effects).contains("raw reject"));
}

#[test]
fn mismatched_and_duplicate_response_ids_never_accept_share() {
    // Arrange
    let mut adapter = DeterministicProductionSessionAdapter::new(Some(pools(false)));
    establish_active(&mut adapter);
    let observation = dispatched_observation(&adapter);
    adapter.drive(ProductionSessionEvent::AsicResult {
        observation,
        now_ms: 4,
    });

    // Act
    adapter.bytes(
        ProductionPool::Primary,
        concat!(
            "{\"id\":99,\"result\":true,\"error\":null}\n",
            "{\"id\":4,\"result\":true,\"error\":null}\n",
            "{\"id\":4,\"result\":true,\"error\":null}\n"
        ),
        5,
    );

    // Assert
    assert_eq!(adapter.session.snapshot().mining.counters.accepted, 1);
}

#[test]
fn fragmented_coalesced_and_crlf_transport_input_reaches_one_session() {
    // Arrange
    let mut adapter = DeterministicProductionSessionAdapter::new(Some(pools(false)));
    adapter.drive(wake(ready(), 0));
    adapter.connect(ProductionPool::Primary, 1);

    // Act
    adapter.bytes(ProductionPool::Primary, b"{\"id\":1", 2);
    adapter.bytes(
        ProductionPool::Primary,
        concat!(
            ",\"result\":{\"version-rolling\":true,",
            "\"version-rolling.mask\":\"1fffe000\"},\"error\":null}\r\n",
            "{\"id\":2,\"result\":[[],\"4de05269\",8],\"error\":null}\n",
            "{\"id\":3,\"result\":true,\"error\":null}\n"
        ),
        3,
    );

    // Assert
    assert_eq!(
        adapter.session.snapshot().phase,
        ProductionSessionPhase::RunningPrimary
    );
}

#[test]
fn malformed_invalid_utf8_and_oversized_input_recover_without_acceptance() {
    // Arrange
    let invalid_inputs = [
        b"{not-json}\n".to_vec(),
        vec![0xff, b'\n'],
        vec![b'x'; crate::v1::line_framer::MAX_STRATUM_JSON_LINE_BYTES + 1],
    ];

    // Act / Assert
    for bytes in invalid_inputs {
        let mut adapter = DeterministicProductionSessionAdapter::new(Some(pools(false)));
        adapter.drive(wake(ready(), 0));
        adapter.connect(ProductionPool::Primary, 1);
        adapter.bytes(ProductionPool::Primary, bytes, 2);
        assert_eq!(adapter.session.snapshot().mining.counters.accepted, 0);
        assert!(adapter.effects.iter().any(|effect| matches!(
            effect,
            ProductionSessionEffect::ClosePoolConnection(ProductionPool::Primary)
        )));
    }
}
