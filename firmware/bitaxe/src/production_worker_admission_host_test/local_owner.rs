//! The local mining owner's run, as `bwg.rs` sees it (ADR-0038); local mining
//! itself is outside this owner-admission boundary.
use super::{bwg, revocation, ProductionMiningSession};

pub(super) struct LocalRun {
    pub(super) preparation_started: bool,
}

#[derive(Default)]
pub(super) struct LocalOwner {
    pub(super) maybe_run: Option<LocalRun>,
}

#[test]
fn a_start_is_refused_beside_a_local_run_without_activating_its_generation() {
    // Arrange
    let mut scope = super::TestScope::new();
    let mut adapter = super::OrdinaryEspProductionSessionAdapter {
        readiness: super::readiness(),
        ..super::blocked_adapter()
    };
    adapter.local.maybe_run = Some(LocalRun {
        preparation_started: false,
    });
    let mut core = ProductionMiningSession::new(bitaxe_asic::asic::AsicFamily::Bm1366, 1);

    // Act
    let (generation, reply) = super::start(&mut scope, &mut adapter, &mut core);

    // Assert
    assert_eq!(reply.try_recv(), Ok(Err(bwg::Error::Rejected)));
    assert!(!revocation::permits(Some(generation)));
}
