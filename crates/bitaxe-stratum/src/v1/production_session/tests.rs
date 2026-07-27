use std::collections::VecDeque;

use bitaxe_asic::bm1366::{production::Bm1366ProductionCommand, result::Bm1366NonceResult};

use super::*;
use crate::v1::state::{MiningOperatorIntent, WorkSubmissionGate};

struct DeterministicProductionSessionAdapter {
    session: ProductionMiningSession,
    pools: Option<ProductionPoolSet>,
    pool_reads: usize,
    connections: Vec<ProductionPool>,
    writes: Vec<(ProductionPool, String)>,
    asic_commands: Vec<Bm1366ProductionCommand>,
    effects: Vec<ProductionSessionEffect>,
    snapshots: Vec<ProductionSessionSnapshot>,
}

impl DeterministicProductionSessionAdapter {
    fn new(pools: Option<ProductionPoolSet>) -> Self {
        Self {
            session: ProductionMiningSession::new(),
            pools,
            pool_reads: 0,
            connections: Vec::new(),
            writes: Vec::new(),
            asic_commands: Vec::new(),
            effects: Vec::new(),
            snapshots: Vec::new(),
        }
    }

    fn drive(&mut self, event: ProductionSessionEvent) {
        let mut events = VecDeque::from([event]);
        while let Some(event) = events.pop_front() {
            let effects = self
                .session
                .handle(event)
                .expect("deterministic event should be handled");
            for effect in effects {
                match &effect {
                    ProductionSessionEffect::ReadPoolConfiguration => {
                        self.pool_reads += 1;
                        events.push_back(ProductionSessionEvent::PoolConfigurationLoaded(
                            self.pools.clone().map(Box::new),
                        ));
                    }
                    ProductionSessionEffect::ConnectPool(pool) => self.connections.push(*pool),
                    ProductionSessionEffect::WritePoolLine { pool, line } => {
                        self.writes.push((*pool, line.clone()));
                    }
                    ProductionSessionEffect::DispatchAsic(command) => {
                        self.asic_commands.push(*command);
                    }
                    ProductionSessionEffect::Publish(snapshot) => {
                        self.snapshots.push(snapshot.clone());
                    }
                    ProductionSessionEffect::ApplyVersionMask(_)
                    | ProductionSessionEffect::PollAsic { .. }
                    | ProductionSessionEffect::BlockSubmissions
                    | ProductionSessionEffect::InvalidateWorkAndSubmissions
                    | ProductionSessionEffect::StopAsicInteraction
                    | ProductionSessionEffect::ClosePoolConnection(_) => {}
                }
                self.effects.push(effect);
            }
        }
    }

    fn connect(&mut self, pool: ProductionPool, now_ms: u64) {
        self.drive(ProductionSessionEvent::TransportConnected { pool, now_ms });
    }

    fn bytes(&mut self, pool: ProductionPool, bytes: impl AsRef<[u8]>, now_ms: u64) {
        self.drive(ProductionSessionEvent::TransportBytes {
            pool,
            bytes: bytes.as_ref().to_vec(),
            now_ms,
        });
    }
}

fn ready() -> ProductionReadiness {
    ProductionReadiness {
        operator_intent: MiningOperatorIntent::Run,
        network_ready: true,
        stratum_v1_supported: true,
        safety_prerequisites_fresh: true,
        production_asic_ready: true,
        actuation_qualified: true,
    }
}

fn pools(prefer_fallback: bool) -> ProductionPoolSet {
    ProductionPoolSet {
        primary: Some(pool("primary.invalid")),
        fallback: Some(pool("fallback.invalid")),
        prefer_fallback,
    }
}

fn pool(host: &str) -> ProductionPoolConfiguration {
    ProductionPoolConfiguration {
        endpoint: ProductionPoolEndpoint {
            host: host.to_owned(),
            port: 3333,
        },
        runtime: LiveRuntimeConfig {
            model: "ultra".to_owned(),
            version: "205".to_owned(),
            credentials: LivePoolCredentials {
                username: "synthetic-user".to_owned(),
                password: "synthetic-secret".to_owned(),
            },
        },
    }
}

fn wake(readiness: ProductionReadiness, now_ms: u64) -> ProductionSessionEvent {
    ProductionSessionEvent::Wake {
        wakeup: None,
        readiness,
        now_ms,
    }
}

fn establish_active(adapter: &mut DeterministicProductionSessionAdapter) {
    adapter.drive(wake(ready(), 0));
    adapter.connect(ProductionPool::Primary, 1);
    authorize_pool(adapter, ProductionPool::Primary, 2);
    adapter.bytes(
        ProductionPool::Primary,
        concat!(
            "{\"id\":null,\"method\":\"mining.set_difficulty\",\"params\":[42]}\n",
            "{\"id\":null,\"method\":\"mining.notify\",\"params\":[\"job\",",
            "\"0000000000000000000000000000000000000000000000000000000000000000\",",
            "\"ffffffff\",\"ffffffff\",[],\"20000004\",\"1705ae3a\",",
            "\"647025b5\",true]}\n"
        ),
        3,
    );
}

fn establish_automatic_fallback(adapter: &mut DeterministicProductionSessionAdapter) {
    adapter.drive(wake(ready(), 0));
    for attempt in 0..CONNECTION_ATTEMPTS_PER_POOL {
        let now_ms = u64::from(attempt) * CONNECTION_RETRY_DELAY_MS;
        adapter.drive(ProductionSessionEvent::TransportConnectFailed {
            pool: ProductionPool::Primary,
            now_ms,
        });
        if attempt + 1 < CONNECTION_ATTEMPTS_PER_POOL {
            adapter.drive(wake(ready(), now_ms + CONNECTION_RETRY_DELAY_MS));
        }
    }
    adapter.connect(ProductionPool::Fallback, 11_000);
    authorize_pool(adapter, ProductionPool::Fallback, 11_001);
}

fn authorize_pool(
    adapter: &mut DeterministicProductionSessionAdapter,
    pool: ProductionPool,
    now_ms: u64,
) {
    adapter.bytes(
        pool,
        concat!(
            "{\"id\":1,\"result\":{\"version-rolling\":true,",
            "\"version-rolling.mask\":\"1fffe000\"},\"error\":null}\n",
            "{\"id\":2,\"result\":[[],\"4de05269\",8],\"error\":null}\n",
            "{\"id\":3,\"result\":true,\"error\":null}\n"
        ),
        now_ms,
    );
}

fn dispatched_observation(
    adapter: &DeterministicProductionSessionAdapter,
) -> ProductionNonceObservation {
    let command = adapter
        .asic_commands
        .last()
        .expect("work should have been dispatched");
    let Bm1366ProductionCommand::SendProductionWork(payload) = command else {
        panic!("expected production work command");
    };
    ProductionNonceObservation {
        observed_generation: adapter.session.snapshot().generation,
        result: Bm1366NonceResult {
            job_id: payload.job_id(),
            nonce: 0x1234_5678,
            asic_index: 0,
            core_id: 1,
            small_core_id: 0,
            version_bits: 0x0000_2000,
        },
    }
}

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

#[test]
fn explicit_fallback_preference_does_not_schedule_primary_probe() {
    // Arrange
    let mut adapter = DeterministicProductionSessionAdapter::new(Some(pools(true)));
    adapter.drive(wake(ready(), 0));
    adapter.connect(ProductionPool::Fallback, 1);
    authorize_pool(&mut adapter, ProductionPool::Fallback, 2);
    adapter.connections.clear();

    // Act
    adapter.drive(wake(ready(), PRIMARY_INITIAL_PROBE_DELAY_MS + 2));

    // Assert
    assert!(adapter.connections.is_empty());
    assert_eq!(
        adapter.session.snapshot().maybe_active_pool,
        Some(ProductionPool::Fallback)
    );
}

#[test]
fn retry_budgets_exhaust_primary_then_fallback_and_recovery_probe() {
    // Arrange
    let mut adapter = DeterministicProductionSessionAdapter::new(Some(pools(false)));
    adapter.drive(wake(ready(), 0));

    // Act
    for attempt in 0..CONNECTION_ATTEMPTS_PER_POOL {
        let now_ms = u64::from(attempt) * CONNECTION_RETRY_DELAY_MS;
        adapter.drive(ProductionSessionEvent::TransportConnectFailed {
            pool: ProductionPool::Primary,
            now_ms,
        });
        if attempt + 1 < CONNECTION_ATTEMPTS_PER_POOL {
            adapter.drive(wake(ready(), now_ms + CONNECTION_RETRY_DELAY_MS));
        }
    }
    assert_eq!(adapter.connections.last(), Some(&ProductionPool::Fallback));
    for attempt in 0..CONNECTION_ATTEMPTS_PER_POOL {
        let now_ms = 20_000 + u64::from(attempt) * CONNECTION_RETRY_DELAY_MS;
        adapter.drive(ProductionSessionEvent::TransportConnectFailed {
            pool: ProductionPool::Fallback,
            now_ms,
        });
        if attempt + 1 < CONNECTION_ATTEMPTS_PER_POOL {
            adapter.drive(wake(ready(), now_ms + CONNECTION_RETRY_DELAY_MS));
        }
    }
    let paused_at = 30_000;
    let before = adapter.connections.len();
    adapter.drive(wake(ready(), paused_at + RECOVERY_PROBE_DELAY_MS - 1));
    let before_due = adapter.connections.len();
    adapter.drive(wake(ready(), paused_at + RECOVERY_PROBE_DELAY_MS));

    // Assert
    assert_eq!(
        adapter.session.snapshot().phase,
        ProductionSessionPhase::ConnectingPrimary
    );
    assert_eq!(before, before_due);
    assert_eq!(adapter.connections.last(), Some(&ProductionPool::Primary));
}

#[test]
fn automatic_fallback_probe_keeps_fallback_until_primary_authorizes() {
    // Arrange
    let mut adapter = DeterministicProductionSessionAdapter::new(Some(pools(false)));
    establish_automatic_fallback(&mut adapter);
    let fallback_generation = adapter.session.snapshot().generation;
    adapter.connections.clear();

    // Act
    adapter.drive(wake(ready(), 11_001 + PRIMARY_INITIAL_PROBE_DELAY_MS));
    assert_eq!(
        adapter.session.snapshot().maybe_active_pool,
        Some(ProductionPool::Fallback)
    );
    assert_eq!(
        adapter.session.snapshot().generation,
        fallback_generation,
        "background probes must not replace the active fallback generation"
    );
    adapter.connect(ProductionPool::Primary, 21_002);
    assert_eq!(adapter.session.snapshot().generation, fallback_generation);
    authorize_pool(&mut adapter, ProductionPool::Primary, 21_003);

    // Assert
    assert_eq!(adapter.connections, [ProductionPool::Primary]);
    assert_eq!(
        adapter.session.snapshot().maybe_active_pool,
        Some(ProductionPool::Primary)
    );
    assert_ne!(adapter.session.snapshot().generation, fallback_generation);
    let close_fallback = adapter.effects.iter().position(|effect| {
        matches!(
            effect,
            ProductionSessionEffect::ClosePoolConnection(ProductionPool::Fallback)
        )
    });
    let primary_publish = adapter.effects.iter().rposition(|effect| {
        matches!(
            effect,
            ProductionSessionEffect::Publish(snapshot)
                if snapshot.maybe_active_pool == Some(ProductionPool::Primary)
        )
    });
    assert!(matches!(
        (close_fallback, primary_publish),
        (Some(close), Some(publish)) if close < publish
    ));
}

#[test]
fn failed_primary_probe_does_not_disrupt_automatic_fallback() {
    // Arrange
    let mut adapter = DeterministicProductionSessionAdapter::new(Some(pools(false)));
    establish_automatic_fallback(&mut adapter);
    let fallback_generation = adapter.session.snapshot().generation;
    adapter.effects.clear();
    adapter.drive(wake(ready(), 11_001 + PRIMARY_INITIAL_PROBE_DELAY_MS));
    adapter.connect(ProductionPool::Primary, 21_002);

    // Act
    adapter.bytes(ProductionPool::Primary, b"{malformed}\n", 21_003);

    // Assert
    assert_eq!(
        adapter.session.snapshot().phase,
        ProductionSessionPhase::RunningFallback
    );
    assert_eq!(
        adapter.session.snapshot().maybe_active_pool,
        Some(ProductionPool::Fallback)
    );
    assert_eq!(adapter.session.snapshot().generation, fallback_generation);
    assert!(adapter.effects.iter().any(|effect| {
        matches!(
            effect,
            ProductionSessionEffect::ClosePoolConnection(ProductionPool::Primary)
        )
    }));
    assert!(!adapter.effects.iter().any(|effect| {
        matches!(
            effect,
            ProductionSessionEffect::ClosePoolConnection(ProductionPool::Fallback)
                | ProductionSessionEffect::StopAsicInteraction
        )
    }));
}

#[test]
fn clean_jobs_and_reconnect_invalidate_stale_nonce_results() {
    // Arrange
    let mut adapter = DeterministicProductionSessionAdapter::new(Some(pools(false)));
    establish_active(&mut adapter);
    let stale = dispatched_observation(&adapter);
    let submit_count = adapter
        .writes
        .iter()
        .filter(|(_, line)| line.contains("mining.submit"))
        .count();

    // Act
    adapter.bytes(
        ProductionPool::Primary,
        concat!(
            "{\"id\":null,\"method\":\"mining.notify\",\"params\":[\"job-2\",",
            "\"0000000000000000000000000000000000000000000000000000000000000000\",",
            "\"ffffffff\",\"ffffffff\",[],\"20000004\",\"1705ae3a\",",
            "\"647025b6\",true]}\n"
        ),
        4,
    );
    adapter.drive(ProductionSessionEvent::AsicResult {
        observation: stale,
        now_ms: 5,
    });

    // Assert
    assert_eq!(
        adapter
            .writes
            .iter()
            .filter(|(_, line)| line.contains("mining.submit"))
            .count(),
        submit_count
    );
    assert_eq!(adapter.session.snapshot().mining.counters.accepted, 0);
}

#[test]
fn cadence_regenerates_work_and_poll_timeout_is_non_terminal() {
    // Arrange
    let mut adapter = DeterministicProductionSessionAdapter::new(Some(pools(false)));
    establish_active(&mut adapter);
    let initial_dispatches = adapter.asic_commands.len();

    // Act
    adapter.drive(ProductionSessionEvent::AsicPollTimedOut { now_ms: 100 });
    adapter.drive(wake(ready(), 2_003));

    // Assert
    assert_eq!(
        adapter.session.snapshot().phase,
        ProductionSessionPhase::RunningPrimary
    );
    assert!(adapter.asic_commands.len() > initial_dispatches);
    assert!(!adapter
        .effects
        .iter()
        .any(|effect| matches!(effect, ProductionSessionEffect::ClosePoolConnection(_))));
}

#[test]
fn pause_settings_change_and_shutdown_reread_authoritative_state() {
    // Arrange
    let mut adapter = DeterministicProductionSessionAdapter::new(Some(pools(false)));
    establish_active(&mut adapter);
    let paused = ProductionReadiness {
        operator_intent: MiningOperatorIntent::Paused,
        ..ready()
    };

    // Act
    adapter.drive(ProductionSessionEvent::Wake {
        wakeup: Some(ProductionSessionWakeup::OperatorIntentChanged),
        readiness: paused,
        now_ms: 10,
    });
    let paused_snapshot = adapter.session.snapshot();
    adapter.drive(ProductionSessionEvent::Wake {
        wakeup: Some(ProductionSessionWakeup::SettingsChanged),
        readiness: ready(),
        now_ms: 11,
    });
    adapter.drive(ProductionSessionEvent::Wake {
        wakeup: Some(ProductionSessionWakeup::ShutdownRequested),
        readiness: ready(),
        now_ms: 12,
    });

    // Assert
    assert_eq!(
        paused_snapshot.mining.mining_activity,
        MiningActivityStatus::Paused
    );
    assert!(adapter.pool_reads >= 2);
    assert_eq!(
        adapter.session.snapshot().phase,
        ProductionSessionPhase::Shutdown
    );
}

#[test]
fn safe_stop_effect_order_and_final_snapshot_are_idempotent() {
    // Arrange
    let mut adapter = DeterministicProductionSessionAdapter::new(Some(pools(false)));
    establish_active(&mut adapter);
    adapter.effects.clear();
    let blocked = ProductionReadiness {
        network_ready: false,
        ..ready()
    };

    // Act
    adapter.drive(ProductionSessionEvent::Wake {
        wakeup: Some(ProductionSessionWakeup::NetworkChanged),
        readiness: blocked,
        now_ms: 10,
    });
    let first = adapter.effects.clone();
    adapter.effects.clear();
    adapter.drive(ProductionSessionEvent::Wake {
        wakeup: Some(ProductionSessionWakeup::NetworkChanged),
        readiness: blocked,
        now_ms: 11,
    });

    // Assert
    let ordered: Vec<&ProductionSessionEffect> = first
        .iter()
        .filter(|effect| {
            matches!(
                effect,
                ProductionSessionEffect::BlockSubmissions
                    | ProductionSessionEffect::InvalidateWorkAndSubmissions
                    | ProductionSessionEffect::StopAsicInteraction
                    | ProductionSessionEffect::ClosePoolConnection(_)
                    | ProductionSessionEffect::Publish(_)
            )
        })
        .collect();
    assert!(matches!(
        ordered[0],
        ProductionSessionEffect::BlockSubmissions
    ));
    assert!(matches!(
        ordered[1],
        ProductionSessionEffect::InvalidateWorkAndSubmissions
    ));
    assert!(matches!(
        ordered[2],
        ProductionSessionEffect::StopAsicInteraction
    ));
    assert!(matches!(
        ordered[3],
        ProductionSessionEffect::ClosePoolConnection(_)
    ));
    assert!(matches!(
        ordered.last(),
        Some(ProductionSessionEffect::Publish(_))
    ));
    assert_eq!(
        adapter.session.snapshot().mining.work_submission,
        WorkSubmissionGate::Blocked
    );
    assert!(!adapter.effects.iter().any(|effect| matches!(
        effect,
        ProductionSessionEffect::StopAsicInteraction
            | ProductionSessionEffect::ClosePoolConnection(_)
    )));
}
