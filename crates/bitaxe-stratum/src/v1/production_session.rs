//! Deep, software-complete Production Mining Session.
//!
//! Callers feed typed events through one interface. The implementation owns
//! recovery, Stratum V1 progression, framing, work correlation, submit
//! classification, bridge cadence, and ordered safe stop.

pub(super) mod campaign;
mod orchestration;
mod runtime;
mod types;

pub use crate::v1::live_runtime::{LivePoolCredentials, LiveRuntimeConfig};
pub use crate::v1::recovery_policy::{
    ProductionPool, ProductionPoolAvailability, ProductionReadiness, ProductionSessionBlocker,
    ProductionSessionNotificationOutcome, ProductionSessionPhase, ProductionSessionWakeup,
    CONNECTION_ATTEMPTS_PER_POOL, CONNECTION_RETRY_DELAY_MS, PRIMARY_INITIAL_PROBE_DELAY_MS,
    PRIMARY_RECURRING_PROBE_DELAY_MS, RECOVERY_PROBE_DELAY_MS,
};
pub use campaign::{
    HardwarePreparationFailure, MiningCampaignDuration, MiningCampaignLease,
    MiningCampaignLeaseError, MiningCampaignLeaseId, MiningCampaignState,
    MiningCampaignStopCondition, MiningHardwareProfile, MiningHardwareProfilePreset,
    MiningHardwareState, MAX_MINING_CAMPAIGN_DURATION_MS,
};
pub use runtime::ProductionMiningSession;
pub use types::{
    ProductionAsicFailure, ProductionPoolConfiguration, ProductionPoolEndpoint, ProductionPoolSet,
    ProductionSessionEffect, ProductionSessionEvent, ProductionSessionSnapshot,
    ProductionTransportEpoch, ProductionTransportFailure,
};

#[cfg(test)]
mod tests;
