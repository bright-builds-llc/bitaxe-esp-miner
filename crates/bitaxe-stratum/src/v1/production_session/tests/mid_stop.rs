//! Events that arrive while a terminal stop cools: the owner polls the cooling proof, so the
//! session handles them before the stop's confirmation (ADR-0016).
use super::*;

/// An active resumable lease, then a revocation: a terminal stop, still unconfirmed.
fn terminal_stop_in_flight(
    adapter: &mut DeterministicProductionSessionAdapter,
) -> (
    MiningCampaignLease,
    ProductionReadiness,
    Vec<ProductionSessionEffect>,
) {
    let lease = resumable_lease(9, 2_000, 10_000);
    let running = ProductionReadiness {
        maybe_campaign_lease: Some(lease),
        ..ready()
    };
    establish_active_with_readiness(adapter, running);
    let effects = adapter
        .session
        .handle(ProductionSessionEvent::CampaignLeaseRevoked)
        .expect("revocation should begin a terminal safe stop");
    (lease, running, effects)
}

fn paused_wake(running: ProductionReadiness, now_ms: u64) -> ProductionSessionEvent {
    ProductionSessionEvent::Wake {
        wakeup: Some(ProductionSessionWakeup::OperatorIntentChanged),
        readiness: ProductionReadiness {
            operator_intent: MiningOperatorIntent::Paused,
            ..running
        },
        now_ms,
    }
}

#[test]
fn a_pause_while_a_terminal_stop_cools_still_consumes_the_lease() {
    // Arrange
    let mut adapter = DeterministicProductionSessionAdapter::new(Some(pools(false)));
    let (lease, running, stop_effects) = terminal_stop_in_flight(&mut adapter);
    adapter
        .session
        .handle(paused_wake(running, 1_004))
        .expect("a pause while stopping should be handled");

    // Act
    let confirmed_effects = adapter
        .session
        .handle(ProductionSessionEvent::HardwareSafeStopConfirmed {
            lease_id: lease.id(),
            now_ms: 1_005,
        })
        .expect("terminal safe stop should be confirmed");

    // Assert
    assert!(stop_effects.iter().any(|effect| matches!(
        effect,
        ProductionSessionEffect::SafeStopHardware {
            purpose: HardwareSafeStopPurpose::Terminal,
            ..
        }
    )));
    assert!(confirmed_effects.iter().any(|effect| matches!(
        effect,
        ProductionSessionEffect::Publish(snapshot)
            if snapshot.campaign_state == MiningCampaignState::Consumed
    )));
    assert_eq!(
        adapter.session.snapshot().campaign_state,
        MiningCampaignState::Consumed
    );
}

#[test]
fn events_while_a_terminal_stop_cools_order_no_hardware_or_publication() {
    // Arrange
    let mut adapter = DeterministicProductionSessionAdapter::new(Some(pools(false)));
    let (lease, running, _) = terminal_stop_in_flight(&mut adapter);
    let events = [
        ProductionSessionEvent::CampaignLeaseRevoked,
        paused_wake(running, 1_004),
        wake(running, 1_005),
        ProductionSessionEvent::HardwarePrepared {
            lease_id: lease.id(),
            now_ms: 1_006,
        },
    ];

    // Act
    let mut effects = Vec::new();
    for event in events {
        effects.extend(
            adapter
                .session
                .handle(event)
                .expect("handled while stopping"),
        );
    }

    // Assert
    assert!(
        effects.iter().all(|effect| !matches!(
            effect,
            ProductionSessionEffect::PrepareHardware { .. }
                | ProductionSessionEffect::ConnectPool { .. }
                | ProductionSessionEffect::DispatchAsic { .. }
                | ProductionSessionEffect::SafeStopHardware { .. }
                | ProductionSessionEffect::Publish(_)
        )),
        "{effects:?}"
    );
    assert_eq!(
        adapter.session.snapshot().campaign_state,
        MiningCampaignState::SafeStopping
    );
}

#[test]
fn a_pause_while_a_terminal_stop_cools_leaves_no_pending_pause_behind() {
    // Arrange
    let mut adapter = DeterministicProductionSessionAdapter::new(Some(pools(false)));
    let (lease, running, _) = terminal_stop_in_flight(&mut adapter);
    adapter
        .session
        .handle(paused_wake(running, 1_004))
        .expect("a pause while stopping should be handled");

    // Act
    adapter
        .session
        .handle(ProductionSessionEvent::HardwareSafeStopConfirmed {
            lease_id: lease.id(),
            now_ms: 1_005,
        })
        .expect("terminal safe stop should be confirmed");

    // Assert
    assert!(!adapter.session.resumable_pause_pending);
}
