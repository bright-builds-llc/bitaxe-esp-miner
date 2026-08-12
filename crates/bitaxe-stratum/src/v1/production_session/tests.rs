use std::collections::VecDeque;

use bitaxe_asic::bm1366::{production::Bm1366ProductionCommand, result::Bm1366NonceResult};

use super::*;
use crate::v1::production_work::ProductionNonceObservation;
use crate::v1::state::{MiningActivityStatus, MiningOperatorIntent, WorkSubmissionGate};

struct DeterministicProductionSessionAdapter {
    session: ProductionMiningSession,
    pools: Option<ProductionPoolSet>,
    pool_reads: usize,
    connections: Vec<ProductionPool>,
    maybe_primary_transport_epoch: Option<ProductionTransportEpoch>,
    maybe_fallback_transport_epoch: Option<ProductionTransportEpoch>,
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
            maybe_primary_transport_epoch: None,
            maybe_fallback_transport_epoch: None,
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
                    ProductionSessionEffect::PrepareHardware { lease_id, .. } => {
                        events.push_back(ProductionSessionEvent::HardwarePrepared {
                            lease_id: *lease_id,
                            now_ms: 0,
                        });
                    }
                    ProductionSessionEffect::ReadPoolConfiguration => {
                        self.pool_reads += 1;
                        events.push_back(ProductionSessionEvent::PoolConfigurationLoaded(
                            self.pools.clone().map(Box::new),
                        ));
                    }
                    ProductionSessionEffect::ConnectPool {
                        pool,
                        transport_epoch,
                        ..
                    } => {
                        self.connections.push(*pool);
                        match pool {
                            ProductionPool::Primary => {
                                self.maybe_primary_transport_epoch = Some(*transport_epoch);
                            }
                            ProductionPool::Fallback => {
                                self.maybe_fallback_transport_epoch = Some(*transport_epoch);
                            }
                        }
                    }
                    ProductionSessionEffect::WritePoolLine { pool, line, .. } => {
                        self.writes.push((*pool, line.clone()));
                    }
                    ProductionSessionEffect::DispatchAsic { command, .. } => {
                        self.asic_commands.push(*command);
                    }
                    ProductionSessionEffect::Publish(snapshot) => {
                        self.snapshots.push(snapshot.as_ref().clone());
                    }
                    ProductionSessionEffect::ApplyVersionMask { .. }
                    | ProductionSessionEffect::PollAsic { .. }
                    | ProductionSessionEffect::RecordScoreboard { .. }
                    | ProductionSessionEffect::RecordBlockFound
                    | ProductionSessionEffect::BlockSubmissions
                    | ProductionSessionEffect::InvalidateWorkAndSubmissions
                    | ProductionSessionEffect::StopAsicInteraction
                    | ProductionSessionEffect::ClosePoolConnection { .. } => {}
                    ProductionSessionEffect::SafeStopHardware { lease_id } => {
                        events.push_back(ProductionSessionEvent::HardwareSafeStopConfirmed {
                            lease_id: *lease_id,
                            now_ms: 0,
                        });
                    }
                }
                self.effects.push(effect);
            }
        }
    }

    fn connect(&mut self, pool: ProductionPool, now_ms: u64) {
        let transport_epoch = self.latest_transport_epoch(pool);
        self.drive(ProductionSessionEvent::TransportConnected {
            pool,
            transport_epoch,
            now_ms,
        });
    }

    fn fail_connect(&mut self, pool: ProductionPool, now_ms: u64) {
        let transport_epoch = self.latest_transport_epoch(pool);
        self.drive(ProductionSessionEvent::TransportFailed {
            pool,
            transport_epoch,
            failure: ProductionTransportFailure::Connect,
            now_ms,
        });
    }

    fn bytes(&mut self, pool: ProductionPool, bytes: impl AsRef<[u8]>, now_ms: u64) {
        let transport_epoch = self.latest_transport_epoch(pool);
        self.drive(ProductionSessionEvent::TransportBytes {
            pool,
            transport_epoch,
            bytes: bytes.as_ref().to_vec(),
            now_ms,
        });
    }

    fn latest_transport_epoch(&self, pool: ProductionPool) -> ProductionTransportEpoch {
        match pool {
            ProductionPool::Primary => self.maybe_primary_transport_epoch,
            ProductionPool::Fallback => self.maybe_fallback_transport_epoch,
        }
        .expect("pool should have a requested transport epoch")
    }
}

fn ready() -> ProductionReadiness {
    ProductionReadiness {
        operator_intent: MiningOperatorIntent::Run,
        network_ready: true,
        stratum_v1_supported: true,
        safety_prerequisites_fresh: true,
        maybe_campaign_lease: Some(active_duration_lease(1, 600_000)),
        actuation_qualified: true,
    }
}

fn profile() -> MiningHardwareProfile {
    MiningHardwareProfile::ultra_205_bm1366(400, 1_100, 100).expect("test profile should be valid")
}

fn active_duration_lease(id: u64, duration_ms: u64) -> MiningCampaignLease {
    MiningCampaignLease::new(
        MiningCampaignLeaseId::new(id).expect("test lease id should be valid"),
        profile(),
        MiningCampaignStopCondition::ActiveDuration {
            duration: MiningCampaignDuration::new(duration_ms)
                .expect("test duration should be valid"),
        },
    )
}

fn first_submit_lease(id: u64, timeout_ms: u64) -> MiningCampaignLease {
    MiningCampaignLease::new(
        MiningCampaignLeaseId::new(id).expect("test lease id should be valid"),
        profile(),
        MiningCampaignStopCondition::FirstSubmitResponse {
            timeout: MiningCampaignDuration::new(timeout_ms).expect("test timeout should be valid"),
        },
    )
}

fn resumable_lease(id: u64, duration_ms: u64) -> MiningCampaignLease {
    MiningCampaignLease::new(
        MiningCampaignLeaseId::new(id).expect("test lease id should be valid"),
        profile(),
        MiningCampaignStopCondition::ResumableWallClockDuration {
            duration: MiningCampaignDuration::new(duration_ms)
                .expect("test duration should be valid"),
        },
    )
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
    establish_active_with_readiness_and_nbits(adapter, ready(), "1705ae3a");
}

fn establish_active_with_readiness(
    adapter: &mut DeterministicProductionSessionAdapter,
    readiness: ProductionReadiness,
) {
    establish_active_with_readiness_and_nbits(adapter, readiness, "1705ae3a");
}

fn establish_active_with_nbits(
    adapter: &mut DeterministicProductionSessionAdapter,
    compact_nbits: &str,
) {
    establish_active_with_readiness_and_nbits(adapter, ready(), compact_nbits);
}

fn establish_active_with_readiness_and_nbits(
    adapter: &mut DeterministicProductionSessionAdapter,
    readiness: ProductionReadiness,
    compact_nbits: &str,
) {
    adapter.drive(wake(readiness, 0));
    adapter.connect(ProductionPool::Primary, 1);
    authorize_pool(adapter, ProductionPool::Primary, 2);
    let notification = concat!(
        "{\"id\":null,\"method\":\"mining.set_difficulty\",\"params\":[1e-30]}\n",
        "{\"id\":null,\"method\":\"mining.notify\",\"params\":[\"job\",",
        "\"0000000000000000000000000000000000000000000000000000000000000000\",",
        "\"ffffffff\",\"ffffffff\",[],\"20000004\",\"1705ae3a\",",
        "\"647025b5\",true]}\n"
    )
    .replace("1705ae3a", compact_nbits);
    adapter.bytes(ProductionPool::Primary, notification, 3);
}

fn establish_automatic_fallback(adapter: &mut DeterministicProductionSessionAdapter) {
    adapter.drive(wake(ready(), 0));
    for attempt in 0..CONNECTION_ATTEMPTS_PER_POOL {
        let now_ms = u64::from(attempt) * CONNECTION_RETRY_DELAY_MS;
        adapter.fail_connect(ProductionPool::Primary, now_ms);
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

mod block_found;
mod job_transition;
mod lifecycle;
mod recovery;
mod scoreboard;
