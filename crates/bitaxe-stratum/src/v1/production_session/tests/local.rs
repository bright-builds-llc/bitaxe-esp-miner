//! Probe before power for local mining runs (ADR-0038).
use super::*;

fn local_lease(id: u64) -> MiningCampaignLease {
    MiningCampaignLease::new(
        MiningCampaignLeaseId::new(id).expect("test lease id should be valid"),
        profile(),
        MiningCampaignStopCondition::Local,
    )
}

fn local_ready() -> ProductionReadiness {
    ProductionReadiness {
        maybe_campaign_lease: Some(local_lease(1)),
        ..ready()
    }
}

fn primary_only() -> ProductionPoolSet {
    ProductionPoolSet {
        primary: Some(pool("primary.invalid")),
        fallback: None,
        prefer_fallback: false,
    }
}

fn prepared(adapter: &DeterministicProductionSessionAdapter) -> usize {
    adapter
        .effects
        .iter()
        .filter(|effect| matches!(effect, ProductionSessionEffect::PrepareHardware { .. }))
        .count()
}

fn safe_stops(adapter: &DeterministicProductionSessionAdapter) -> Vec<HardwareSafeStopPurpose> {
    adapter
        .effects
        .iter()
        .filter_map(|effect| match effect {
            ProductionSessionEffect::SafeStopHardware { purpose, .. } => Some(*purpose),
            _ => None,
        })
        .collect()
}

/// Fails the only pool until the recovery policy gives up on it.
fn exhaust_primary(adapter: &mut DeterministicProductionSessionAdapter, from_ms: u64) -> u64 {
    let mut now_ms = from_ms;
    for attempt in 0..CONNECTION_ATTEMPTS_PER_POOL {
        adapter.fail_connect(ProductionPool::Primary, now_ms);
        if attempt + 1 < CONNECTION_ATTEMPTS_PER_POOL {
            now_ms += CONNECTION_RETRY_DELAY_MS;
            adapter.drive(wake(local_ready(), now_ms));
        }
    }
    now_ms
}

#[test]
fn an_authorized_probe_closes_and_powers_up_the_asic() {
    // Arrange
    let mut adapter = DeterministicProductionSessionAdapter::new(Some(primary_only()));
    adapter.drive(wake(local_ready(), 0));
    adapter.connect(ProductionPool::Primary, 1);
    // Act
    authorize_pool(&mut adapter, ProductionPool::Primary, 2);
    // Assert: the probe closes before preparation, and the run reconnects once ready.
    let close = adapter
        .effects
        .iter()
        .position(|effect| matches!(effect, ProductionSessionEffect::ClosePoolConnection { .. }));
    let prepare = adapter
        .effects
        .iter()
        .position(|effect| matches!(effect, ProductionSessionEffect::PrepareHardware { .. }));
    assert!(close.is_some() && prepare.is_some() && close < prepare);
    assert_eq!(
        adapter.connections,
        vec![ProductionPool::Primary, ProductionPool::Primary]
    );
}

#[test]
fn exhausted_pools_before_power_keep_the_asic_off() {
    // Arrange
    let mut adapter = DeterministicProductionSessionAdapter::new(Some(primary_only()));
    adapter.drive(wake(local_ready(), 0));
    // Act
    exhaust_primary(&mut adapter, 0);
    // Assert
    assert_eq!(
        (prepared(&adapter), adapter.session.snapshot().phase),
        (0, ProductionSessionPhase::RecoveryPaused)
    );
}

#[test]
fn an_exhausted_probe_tries_again_after_thirty_seconds_with_the_asic_off() {
    // Arrange
    let mut adapter = DeterministicProductionSessionAdapter::new(Some(primary_only()));
    adapter.drive(wake(local_ready(), 0));
    let exhausted_at = exhaust_primary(&mut adapter, 0);
    let attempts = adapter.connections.len();
    // Act
    adapter.drive(wake(local_ready(), exhausted_at + RECOVERY_PROBE_DELAY_MS));
    // Assert
    assert_eq!(
        (adapter.connections.len(), prepared(&adapter)),
        (attempts + 1, 0)
    );
}

#[test]
fn a_pause_while_probing_ends_the_run_without_a_hardware_stop() {
    // Arrange
    let mut adapter = DeterministicProductionSessionAdapter::new(Some(primary_only()));
    adapter.drive(wake(local_ready(), 0));
    adapter.connect(ProductionPool::Primary, 1);
    let paused = ProductionReadiness {
        operator_intent: MiningOperatorIntent::Paused,
        ..local_ready()
    };
    // Act
    adapter.drive(wake(paused, 2));
    // Assert
    assert_eq!(
        (
            safe_stops(&adapter).len(),
            adapter.session.snapshot().campaign_state
        ),
        (0, MiningCampaignState::Consumed)
    );
}

const NOTIFY: &str = concat!(
    "{\"id\":null,\"method\":\"mining.set_difficulty\",\"params\":[1e-30]}\n",
    "{\"id\":null,\"method\":\"mining.notify\",\"params\":[\"job\",",
    "\"0000000000000000000000000000000000000000000000000000000000000000\",",
    "\"ffffffff\",\"ffffffff\",[],\"20000004\",\"1705ae3a\",",
    "\"647025b5\",true]}\n"
);

/// Probes, powers up, reconnects and hashes; returns the time it began hashing.
fn power_and_hash(adapter: &mut DeterministicProductionSessionAdapter) -> u64 {
    adapter.drive(wake(local_ready(), 0));
    adapter.connect(ProductionPool::Primary, 1);
    authorize_pool(adapter, ProductionPool::Primary, 2);
    adapter.connect(ProductionPool::Primary, 3);
    authorize_pool(adapter, ProductionPool::Primary, 4);
    adapter.bytes(ProductionPool::Primary, NOTIFY, 5);
    5
}

/// Drops the powered run's pool and fails every reconnect; returns when it gave up.
fn drop_and_exhaust(adapter: &mut DeterministicProductionSessionAdapter, at_ms: u64) -> u64 {
    adapter.drive(ProductionSessionEvent::TransportClosed {
        pool: ProductionPool::Primary,
        transport_epoch: adapter.latest_transport_epoch(ProductionPool::Primary),
        now_ms: at_ms,
    });
    let retry_at = at_ms + CONNECTION_RETRY_DELAY_MS;
    adapter.drive(wake(local_ready(), retry_at));
    exhaust_primary(adapter, retry_at)
}

fn hardware_effects(adapter: &DeterministicProductionSessionAdapter) -> usize {
    adapter
        .effects
        .iter()
        .filter(|effect| {
            matches!(
                effect,
                ProductionSessionEffect::PrepareHardware { .. }
                    | ProductionSessionEffect::SafeStopHardware { .. }
                    | ProductionSessionEffect::DispatchAsic { .. }
                    | ProductionSessionEffect::PollAsic { .. }
                    | ProductionSessionEffect::ApplyVersionMask { .. }
            )
        })
        .count()
}

fn closed(adapter: &DeterministicProductionSessionAdapter) -> bool {
    adapter
        .effects
        .iter()
        .any(|effect| matches!(effect, ProductionSessionEffect::ClosePoolConnection { .. }))
}

fn consumed(adapter: &DeterministicProductionSessionAdapter) -> bool {
    adapter.session.snapshot().campaign_state == MiningCampaignState::Consumed
}

#[test]
fn a_local_run_connects_its_pool_before_any_hardware_effect() {
    // Arrange
    let mut adapter = DeterministicProductionSessionAdapter::new(Some(primary_only()));
    // Act
    adapter.drive(wake(local_ready(), 0));
    adapter.connect(ProductionPool::Primary, 1);
    // Assert
    assert_eq!(
        (adapter.connections.clone(), hardware_effects(&adapter)),
        (vec![ProductionPool::Primary], 0)
    );
}

#[test]
fn work_sent_during_the_probe_never_reaches_the_asic() {
    // Arrange: the pool sends a job before it answers authorization.
    let mut adapter = DeterministicProductionSessionAdapter::new(Some(primary_only()));
    adapter.drive(wake(local_ready(), 0));
    adapter.connect(ProductionPool::Primary, 1);
    adapter.bytes(ProductionPool::Primary, NOTIFY, 2);
    // Act
    authorize_pool(&mut adapter, ProductionPool::Primary, 3);
    // Assert
    assert_eq!((adapter.asic_commands.len(), prepared(&adapter)), (0, 1));
}

#[test]
fn a_powered_local_run_mines_after_it_reconnects() {
    // Arrange
    let mut adapter = DeterministicProductionSessionAdapter::new(Some(primary_only()));
    // Act
    power_and_hash(&mut adapter);
    // Assert
    assert!(!adapter.asic_commands.is_empty());
}

#[test]
fn a_run_that_hashed_pauses_its_hardware_when_the_pools_are_exhausted() {
    // Arrange
    let mut adapter = DeterministicProductionSessionAdapter::new(Some(primary_only()));
    let hashed_at = power_and_hash(&mut adapter);
    // Act
    drop_and_exhaust(&mut adapter, hashed_at + 1);
    // Assert
    assert_eq!(
        safe_stops(&adapter),
        vec![HardwareSafeStopPurpose::ResumablePause]
    );
    assert!(!consumed(&adapter));
}

#[test]
fn a_paused_run_powers_up_again_once_a_later_probe_authorizes() {
    // Arrange
    let mut adapter = DeterministicProductionSessionAdapter::new(Some(primary_only()));
    let hashed_at = power_and_hash(&mut adapter);
    let exhausted_at = drop_and_exhaust(&mut adapter, hashed_at + 1);
    // Act
    adapter.drive(wake(local_ready(), exhausted_at + RECOVERY_PROBE_DELAY_MS));
    adapter.connect(
        ProductionPool::Primary,
        exhausted_at + RECOVERY_PROBE_DELAY_MS + 1,
    );
    authorize_pool(
        &mut adapter,
        ProductionPool::Primary,
        exhausted_at + RECOVERY_PROBE_DELAY_MS + 2,
    );
    // Assert
    assert_eq!(prepared(&adapter), 2);
}

#[test]
fn a_run_that_never_hashed_ends_when_the_pools_are_exhausted() {
    // Arrange: the probe authorized, then the pool dropped before any job.
    let mut adapter = DeterministicProductionSessionAdapter::new(Some(primary_only()));
    adapter.drive(wake(local_ready(), 0));
    adapter.connect(ProductionPool::Primary, 1);
    authorize_pool(&mut adapter, ProductionPool::Primary, 2);
    adapter.connect(ProductionPool::Primary, 3);
    // Act
    drop_and_exhaust(&mut adapter, 4);
    // Assert: the run ends, so the owner's restart guard paces the next one.
    assert_eq!(
        safe_stops(&adapter),
        vec![HardwareSafeStopPurpose::Terminal]
    );
    assert!(consumed(&adapter));
}

#[test]
fn a_revoked_run_ends_while_still_probing() {
    // Arrange
    let mut adapter = DeterministicProductionSessionAdapter::new(Some(primary_only()));
    adapter.drive(wake(local_ready(), 0));
    adapter.connect(ProductionPool::Primary, 1);
    // Act
    adapter.drive(ProductionSessionEvent::CampaignLeaseRevoked);
    // Assert
    assert!(consumed(&adapter) && closed(&adapter) && hardware_effects(&adapter) == 0);
}

#[test]
fn a_shutdown_ends_a_run_still_probing() {
    // Arrange
    let mut adapter = DeterministicProductionSessionAdapter::new(Some(primary_only()));
    adapter.drive(wake(local_ready(), 0));
    adapter.connect(ProductionPool::Primary, 1);
    // Act
    adapter.drive(ProductionSessionEvent::Wake {
        wakeup: Some(ProductionSessionWakeup::ShutdownRequested),
        readiness: local_ready(),
        now_ms: 1,
    });
    // Assert
    assert!(consumed(&adapter) && closed(&adapter) && hardware_effects(&adapter) == 0);
}

#[test]
fn a_settings_change_ends_a_run_still_probing() {
    // Arrange: the next run reads the changed profile.
    let mut adapter = DeterministicProductionSessionAdapter::new(Some(primary_only()));
    adapter.drive(wake(local_ready(), 0));
    adapter.connect(ProductionPool::Primary, 1);
    // Act
    adapter.drive(ProductionSessionEvent::Wake {
        wakeup: Some(ProductionSessionWakeup::SettingsChanged),
        readiness: local_ready(),
        now_ms: 2,
    });
    // Assert
    assert!(consumed(&adapter) && closed(&adapter) && hardware_effects(&adapter) == 0);
}

#[test]
fn another_lease_kind_under_the_same_id_never_skips_the_probe() {
    // Arrange
    let mut adapter = DeterministicProductionSessionAdapter::new(Some(primary_only()));
    adapter.drive(wake(local_ready(), 0));
    let same_id_lease = ProductionReadiness {
        maybe_campaign_lease: Some(active_duration_lease(1, 600_000)),
        ..ready()
    };
    // Act
    adapter.drive(wake(same_id_lease, 1));
    // Assert
    assert!(consumed(&adapter) && prepared(&adapter) == 0);
}

#[test]
fn a_power_up_that_never_dispatches_ends_the_run_after_an_earlier_one_hashed() {
    // Arrange: hash, pause, probe and power up again, then authorize without a job.
    let mut adapter = DeterministicProductionSessionAdapter::new(Some(primary_only()));
    let hashed_at = power_and_hash(&mut adapter);
    let probe_at = drop_and_exhaust(&mut adapter, hashed_at + 1) + RECOVERY_PROBE_DELAY_MS;
    adapter.drive(wake(local_ready(), probe_at));
    adapter.connect(ProductionPool::Primary, probe_at + 1);
    authorize_pool(&mut adapter, ProductionPool::Primary, probe_at + 2);
    adapter.connect(ProductionPool::Primary, probe_at + 3);
    authorize_pool(&mut adapter, ProductionPool::Primary, probe_at + 4);
    // Act
    drop_and_exhaust(&mut adapter, probe_at + 5);
    // Assert: the restart guard sees this run end.
    assert_eq!(
        safe_stops(&adapter),
        vec![
            HardwareSafeStopPurpose::ResumablePause,
            HardwareSafeStopPurpose::Terminal
        ]
    );
    assert!(consumed(&adapter));
}

#[test]
fn a_revoke_after_power_up_stops_the_hardware() {
    // Arrange
    let mut adapter = DeterministicProductionSessionAdapter::new(Some(primary_only()));
    power_and_hash(&mut adapter);
    // Act
    adapter.drive(ProductionSessionEvent::CampaignLeaseRevoked);
    // Assert
    assert_eq!(
        safe_stops(&adapter),
        vec![HardwareSafeStopPurpose::Terminal]
    );
}

#[test]
fn a_local_run_is_never_renewed() {
    // Arrange
    let mut adapter = DeterministicProductionSessionAdapter::new(Some(primary_only()));
    let hashed_at = power_and_hash(&mut adapter);
    // Act
    adapter.drive(ProductionSessionEvent::CampaignLeaseRenewed {
        lease: local_lease(1),
        now_ms: hashed_at + 1,
    });
    // Assert
    assert_eq!(
        safe_stops(&adapter),
        vec![HardwareSafeStopPurpose::Terminal]
    );
}

#[test]
fn a_local_run_under_a_gate_leases_id_ends_the_lease() {
    // Arrange: a Gate lease is running.
    let mut adapter = DeterministicProductionSessionAdapter::new(Some(primary_only()));
    adapter.drive(wake(ready(), 0));
    // Act
    adapter.drive(wake(local_ready(), 1));
    // Assert
    assert!(consumed(&adapter));
}
