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
            maybe_campaign_lease: None,
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
fn transport_effect_debug_redacts_endpoint_credentials_and_lines() {
    // Arrange
    let mut adapter = DeterministicProductionSessionAdapter::new(Some(pools(false)));
    adapter.drive(wake(ready(), 0));
    adapter.connect(ProductionPool::Primary, 1);

    // Act
    authorize_pool(&mut adapter, ProductionPool::Primary, 2);
    let rendered = format!("{:?}", adapter.effects);

    // Assert
    for secret in [
        "primary.invalid",
        "synthetic-user",
        "synthetic-secret",
        "mining.authorize",
    ] {
        assert!(!rendered.contains(secret));
    }
    assert!(rendered.contains("transport_epoch"));
    assert!(rendered.contains("redacted"));
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
            "{\"id\":null,\"method\":\"mining.set_difficulty\",\"params\":[1e-30]}\n",
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
            ProductionSessionEffect::ClosePoolConnection {
                pool: ProductionPool::Primary,
                ..
            }
        )));
    }
}

#[test]
fn pool_configuration_is_read_only_after_hardware_preparation_succeeds() {
    // Arrange
    let mut session = ProductionMiningSession::new();

    // Act
    let effects = session
        .handle(wake(ready(), 0))
        .expect("readiness should request preparation");
    let prepared_effects = session
        .handle(ProductionSessionEvent::HardwarePrepared {
            lease_id: active_duration_lease(1, 600_000).id(),
            now_ms: 1,
        })
        .expect("preparation confirmation should advance the lifecycle");

    // Assert
    assert!(matches!(
        effects.as_slice(),
        [
            ProductionSessionEffect::PrepareHardware { .. },
            ProductionSessionEffect::Publish(_)
        ]
    ));
    assert!(!effects
        .iter()
        .any(|effect| matches!(effect, ProductionSessionEffect::ReadPoolConfiguration)));
    assert!(prepared_effects
        .iter()
        .any(|effect| matches!(effect, ProductionSessionEffect::ReadPoolConfiguration)));
}

#[test]
fn preparation_failure_rolls_back_before_terminal_publication() {
    // Arrange
    let mut session = ProductionMiningSession::new();
    let lease_id = active_duration_lease(1, 600_000).id();
    let _preparing = session
        .handle(wake(ready(), 0))
        .expect("readiness should request preparation");

    // Act
    let failed = session
        .handle(ProductionSessionEvent::HardwarePreparationFailed {
            lease_id,
            failure: HardwarePreparationFailure::DeviceFault,
            now_ms: 1,
        })
        .expect("preparation failure should enter safe stop");
    let confirmed = session
        .handle(ProductionSessionEvent::HardwareSafeStopConfirmed {
            lease_id,
            now_ms: 2,
        })
        .expect("safe-stop confirmation should publish terminal state");

    // Assert
    assert!(matches!(
        failed.as_slice(),
        [
            ProductionSessionEffect::BlockSubmissions,
            ProductionSessionEffect::InvalidateWorkAndSubmissions,
            ProductionSessionEffect::StopAsicInteraction,
            ProductionSessionEffect::SafeStopHardware { .. }
        ]
    ));
    assert!(!failed
        .iter()
        .any(|effect| matches!(effect, ProductionSessionEffect::Publish(_))));
    assert!(matches!(
        confirmed.as_slice(),
        [ProductionSessionEffect::Publish(snapshot)]
            if snapshot.hardware_state == MiningHardwareState::Stopped
                && snapshot.campaign_state == MiningCampaignState::Consumed
    ));
}

#[test]
fn unavailable_pool_configuration_safe_stops_prepared_hardware() {
    // Arrange
    let mut adapter = DeterministicProductionSessionAdapter::new(None);

    // Act
    adapter.drive(wake(ready(), 0));

    // Assert
    assert_eq!(adapter.pool_reads, 1);
    assert_eq!(
        adapter.session.snapshot().maybe_blocker,
        Some(ProductionSessionBlocker::PoolConfigurationUnavailable)
    );
    assert_eq!(
        adapter.session.snapshot().hardware_state,
        MiningHardwareState::Stopped
    );
    assert_eq!(
        adapter.session.snapshot().campaign_state,
        MiningCampaignState::Consumed
    );
}

#[test]
fn first_submit_response_consumes_lease_and_safe_stops() {
    // Arrange
    let mut readiness = ready();
    readiness.maybe_campaign_lease = Some(first_submit_lease(7, 600_000));
    let mut adapter = DeterministicProductionSessionAdapter::new(Some(pools(false)));
    adapter.drive(wake(readiness, 0));
    adapter.connect(ProductionPool::Primary, 1);
    authorize_pool(&mut adapter, ProductionPool::Primary, 2);
    adapter.bytes(
        ProductionPool::Primary,
        concat!(
            "{\"id\":null,\"method\":\"mining.set_difficulty\",\"params\":[1e-30]}\n",
            "{\"id\":null,\"method\":\"mining.notify\",\"params\":[\"job\",",
            "\"0000000000000000000000000000000000000000000000000000000000000000\",",
            "\"ffffffff\",\"ffffffff\",[],\"20000004\",\"1705ae3a\",",
            "\"647025b5\",true]}\n"
        ),
        3,
    );
    let observation = dispatched_observation(&adapter);
    adapter.drive(ProductionSessionEvent::AsicResult {
        observation,
        now_ms: 4,
    });
    adapter.effects.clear();

    // Act
    adapter.bytes(
        ProductionPool::Primary,
        b"{\"id\":4,\"result\":false,\"error\":[21,\"raw reject\",null]}\n",
        5,
    );

    // Assert
    assert_eq!(
        adapter.session.snapshot().campaign_state,
        MiningCampaignState::Consumed
    );
    assert_eq!(
        adapter.session.snapshot().hardware_state,
        MiningHardwareState::Stopped
    );
    assert_eq!(adapter.session.snapshot().mining.counters.rejected, 1);
    assert!(adapter
        .effects
        .iter()
        .any(|effect| matches!(effect, ProductionSessionEffect::SafeStopHardware { .. })));
}

#[test]
fn accepted_first_submit_response_consumes_lease_and_safe_stops() {
    // Arrange
    let mut readiness = ready();
    readiness.maybe_campaign_lease = Some(first_submit_lease(8, 600_000));
    let mut adapter = DeterministicProductionSessionAdapter::new(Some(pools(false)));
    establish_active_with_readiness(&mut adapter, readiness);
    let observation = dispatched_observation(&adapter);
    adapter.drive(ProductionSessionEvent::AsicResult {
        observation,
        now_ms: 4,
    });
    adapter.effects.clear();

    // Act
    adapter.bytes(
        ProductionPool::Primary,
        b"{\"id\":4,\"result\":true,\"error\":null}\n",
        5,
    );

    // Assert
    assert_eq!(
        adapter.session.snapshot().campaign_state,
        MiningCampaignState::Consumed
    );
    assert_eq!(
        adapter.session.snapshot().hardware_state,
        MiningHardwareState::Stopped
    );
    assert_eq!(adapter.session.snapshot().mining.counters.accepted, 1);
    assert!(adapter
        .effects
        .iter()
        .any(|effect| matches!(effect, ProductionSessionEffect::SafeStopHardware { .. })));
}

#[test]
fn first_submit_timeout_expires_at_the_exact_prepared_deadline() {
    // Arrange
    let mut readiness = ready();
    readiness.maybe_campaign_lease = Some(first_submit_lease(9, 10));
    let mut adapter = DeterministicProductionSessionAdapter::new(Some(pools(false)));
    adapter.drive(wake(readiness, 0));

    // Act
    adapter.drive(wake(readiness, 9));
    let before_deadline = adapter.session.snapshot().campaign_state;
    adapter.drive(wake(readiness, 10));

    // Assert
    assert_eq!(before_deadline, MiningCampaignState::Armed);
    assert_eq!(
        adapter.session.snapshot().campaign_state,
        MiningCampaignState::Consumed
    );
    assert_eq!(
        adapter.session.snapshot().hardware_state,
        MiningHardwareState::Stopped
    );
}

#[test]
fn consumed_lease_cannot_rearm_but_a_higher_lease_can() {
    // Arrange
    let mut readiness = ready();
    readiness.maybe_campaign_lease = Some(first_submit_lease(10, 1));
    let mut adapter = DeterministicProductionSessionAdapter::new(Some(pools(false)));
    adapter.drive(wake(readiness, 0));
    adapter.drive(wake(readiness, 1));
    adapter.effects.clear();

    // Act
    adapter.drive(wake(readiness, 2));
    let effects_after_replay = adapter.effects.len();
    readiness.maybe_campaign_lease = Some(first_submit_lease(11, 10));
    adapter.drive(wake(readiness, 3));

    // Assert
    assert_eq!(effects_after_replay, 0);
    assert!(adapter.effects.iter().any(|effect| matches!(
        effect,
        ProductionSessionEffect::PrepareHardware { lease_id, .. }
            if lease_id.raw() == 11
    )));
}

#[test]
fn active_duration_counts_from_authorized_mining() {
    // Arrange
    let mut readiness = ready();
    readiness.maybe_campaign_lease = Some(active_duration_lease(9, 10));
    let mut adapter = DeterministicProductionSessionAdapter::new(Some(pools(false)));
    adapter.drive(wake(readiness, 0));
    adapter.drive(wake(readiness, 100));
    assert_eq!(
        adapter.session.snapshot().campaign_state,
        MiningCampaignState::Armed
    );
    adapter.connect(ProductionPool::Primary, 101);
    authorize_pool(&mut adapter, ProductionPool::Primary, 102);

    // Act
    adapter.drive(wake(readiness, 111));
    let before_expiry = adapter.session.snapshot().campaign_state;
    adapter.drive(wake(readiness, 112));

    // Assert
    assert_eq!(before_expiry, MiningCampaignState::Active);
    assert_eq!(
        adapter.session.snapshot().campaign_state,
        MiningCampaignState::Consumed
    );
}

#[test]
fn campaign_becomes_active_before_work_submission_marks_mining_active() {
    // Arrange
    let mut adapter = DeterministicProductionSessionAdapter::new(Some(pools(false)));
    adapter.drive(wake(ready(), 0));
    adapter.connect(ProductionPool::Primary, 1);

    // Act
    authorize_pool(&mut adapter, ProductionPool::Primary, 2);
    let before_work = adapter.session.snapshot();
    adapter.bytes(
        ProductionPool::Primary,
        concat!(
            "{\"id\":null,\"method\":\"mining.set_difficulty\",\"params\":[1e-30]}\n",
            "{\"id\":null,\"method\":\"mining.notify\",\"params\":[\"job\",",
            "\"0000000000000000000000000000000000000000000000000000000000000000\",",
            "\"ffffffff\",\"ffffffff\",[],\"20000004\",\"1705ae3a\",",
            "\"647025b5\",true]}\n"
        ),
        3,
    );
    let after_work = adapter.session.snapshot();

    // Assert
    assert_eq!(before_work.campaign_state, MiningCampaignState::Active);
    assert_eq!(
        before_work.mining.mining_activity,
        MiningActivityStatus::SafeBlocked
    );
    assert_eq!(after_work.campaign_state, MiningCampaignState::Active);
    assert_eq!(
        after_work.mining.mining_activity,
        MiningActivityStatus::Active
    );
}

#[test]
fn asic_effects_bind_generation_and_valid_job_context() {
    // Arrange
    let mut adapter = DeterministicProductionSessionAdapter::new(Some(pools(false)));

    // Act
    establish_active(&mut adapter);
    adapter.drive(wake(ready(), 4));

    // Assert
    let maybe_dispatch = adapter.effects.iter().find_map(|effect| {
        let ProductionSessionEffect::DispatchAsic {
            generation,
            valid_jobs,
            command,
        } = effect
        else {
            return None;
        };
        Some((*generation, valid_jobs, command))
    });
    let Some((generation, valid_jobs, Bm1366ProductionCommand::SendProductionWork(payload))) =
        maybe_dispatch
    else {
        panic!("expected a correlated ASIC dispatch");
    };
    assert_eq!(generation, adapter.session.snapshot().generation);
    assert!(valid_jobs.contains(payload.job_id()));
    assert!(adapter.effects.iter().any(|effect| {
        matches!(
            effect,
            ProductionSessionEffect::PollAsic {
                generation: poll_generation,
                ..
            } if *poll_generation == generation
        )
    }));
}

#[test]
fn profile_and_campaign_bounds_reject_invalid_values() {
    // Arrange / Act
    let invalid_frequency = MiningHardwareProfile::ultra_205_bm1366(401, 1_100, 100);
    let zero_id = MiningCampaignLeaseId::new(0);
    let zero_duration = MiningCampaignDuration::new(0);
    let overlong_duration =
        MiningCampaignDuration::new(MAX_MINING_CAMPAIGN_DURATION_MS.saturating_add(1));

    // Assert
    assert!(invalid_frequency.is_err());
    assert_eq!(zero_id, Err(MiningCampaignLeaseError::ZeroLeaseId));
    assert!(matches!(
        zero_duration,
        Err(MiningCampaignLeaseError::InvalidDuration { duration_ms: 0 })
    ));
    assert!(matches!(
        overlong_duration,
        Err(MiningCampaignLeaseError::InvalidDuration { .. })
    ));
}
