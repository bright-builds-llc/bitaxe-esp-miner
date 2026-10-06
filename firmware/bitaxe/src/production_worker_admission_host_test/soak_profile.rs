//! ADR-0033: the signed profile reaches hardware preparation and survives renewal.
use super::*;

/// Owner Start with an explicit preset, returning the core effects it produced.
pub(super) fn start_with_profile(
    scope: &mut TestScope,
    adapter: &mut OrdinaryEspProductionSessionAdapter,
    core: &mut ProductionMiningSession,
    preset: MiningHardwareProfilePreset,
) -> (
    revocation::WorkerGeneration,
    bitaxe_runtime::reply::ReplyReceiver<Result<(), bwg::Error>>,
    Vec<bitaxe_stratum::v1::production_session::ProductionSessionEffect>,
) {
    let generation = scope.link();
    assert!(revocation::admit_budget(generation, 180_000));
    let (reply, receiver) = bitaxe_runtime::reply::reply();
    let event = adapter.event(
        bwg::OwnerCommand::Start {
            generation,
            worker_lease_id: "synthetic-lease".to_owned(),
            profile: preset.profile(),
            deadline: MiningCampaignMonotonicDeadline::new(61_000).expect("valid deadline"),
            pools: ProductionPoolSet {
                primary: None,
                fallback: None,
                prefer_fallback: false,
            },
            reply,
        },
        1_000,
        &core.snapshot(),
        core.next_campaign_lease_id(),
    );
    let effects = core.handle(event).expect("synthetic readiness event");
    for effect in &effects {
        if matches!(
            effect,
            bitaxe_stratum::v1::production_session::ProductionSessionEffect::PrepareHardware { .. }
        ) {
            adapter.note_worker_preparation_started();
        }
    }
    adapter.complete_reply(&core.snapshot());
    (generation, receiver, effects)
}

fn ready_adapter() -> OrdinaryEspProductionSessionAdapter {
    OrdinaryEspProductionSessionAdapter {
        readiness: readiness(),
        ..blocked_adapter()
    }
}

#[test]
fn a_signed_upstream_default_start_prepares_hardware_at_upstream_defaults() {
    // Arrange
    let mut scope = TestScope::new();
    let mut adapter = ready_adapter();
    let mut core = ProductionMiningSession::new();
    // Act
    let (_, _reply, effects) = start_with_profile(
        &mut scope,
        &mut adapter,
        &mut core,
        MiningHardwareProfilePreset::UpstreamDefault,
    );
    // Assert
    let prepared: Vec<_> = effects
        .iter()
        .filter_map(|effect| match effect {
            bitaxe_stratum::v1::production_session::ProductionSessionEffect::PrepareHardware {
                profile,
                ..
            } => Some(*profile),
            _ => None,
        })
        .collect();
    assert_eq!(
        prepared,
        vec![MiningHardwareProfilePreset::UpstreamDefault.profile()]
    );
}

#[test]
fn a_renewal_keeps_the_profile_its_start_chose() {
    // Arrange
    let mut scope = TestScope::new();
    let mut adapter = ready_adapter();
    let mut core = ProductionMiningSession::new();
    let (generation, _reply, _) = start_with_profile(
        &mut scope,
        &mut adapter,
        &mut core,
        MiningHardwareProfilePreset::UpstreamDefault,
    );
    let mut snapshot = core.snapshot();
    snapshot.campaign_state = MiningCampaignState::Active;
    let (reply, _receiver) = bitaxe_runtime::reply::reply();
    // Act
    let event = adapter.event(
        bwg::OwnerCommand::Renew {
            generation,
            worker_lease_id: "synthetic-lease".to_owned(),
            deadline: MiningCampaignMonotonicDeadline::new(81_000).expect("valid deadline"),
            reply,
        },
        21_000,
        &snapshot,
        None,
    );
    // Assert
    let ProductionSessionEvent::CampaignLeaseRenewed { lease, .. } = event else {
        panic!("renewal of the active soak lease");
    };
    assert_eq!(
        lease.profile(),
        MiningHardwareProfilePreset::UpstreamDefault.profile()
    );
}
