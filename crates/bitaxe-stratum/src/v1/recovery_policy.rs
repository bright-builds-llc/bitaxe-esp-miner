//! Private production mining lifecycle and recovery policy.
//!
//! The deep Production Mining Session owns this policy and translates its
//! decisions into the session event/effect interface.

use crate::v1::production_session::campaign::MiningCampaignLease;
use crate::v1::state::{MiningActivityStatus, MiningOperatorIntent, PoolLifecycleStatus};

pub const CONNECTION_ATTEMPTS_PER_POOL: u8 = 3;
pub const CONNECTION_RETRY_DELAY_MS: u64 = 5_000;
pub const RECOVERY_PROBE_DELAY_MS: u64 = 30_000;
pub const PRIMARY_INITIAL_PROBE_DELAY_MS: u64 = 10_000;
pub const PRIMARY_RECURRING_PROBE_DELAY_MS: u64 = 60_000;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProductionSessionPhase {
    WaitingForReadiness,
    ConnectingPrimary,
    RunningPrimary,
    ConnectingFallback,
    RunningFallback,
    RecoveryPaused,
    SafeStopping,
    Shutdown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProductionSessionBlocker {
    OperatorPaused,
    NetworkUnavailable,
    StratumV1Unsupported,
    SafetyPrerequisitesStale,
    CampaignLeaseUnavailable,
    CampaignLeaseConsumed,
    ProductionAsicUnavailable,
    ProductionAsicVersionMaskUnavailable,
    ProductionAsicDispatchUnavailable,
    ProductionAsicPollUnavailable,
    ProductionAsicQueueFull,
    ProductionAsicWorkerUnavailable,
    ActuationUnqualified,
    PoolConfigurationUnavailable,
    PoolsExhausted,
}

impl ProductionSessionBlocker {
    pub const fn label(self) -> &'static str {
        match self {
            Self::OperatorPaused => "operator_paused",
            Self::NetworkUnavailable => "network_unavailable",
            Self::StratumV1Unsupported => "stratum_v1_unsupported",
            Self::SafetyPrerequisitesStale => "safety_prerequisites_stale",
            Self::CampaignLeaseUnavailable => "campaign_lease_unavailable",
            Self::CampaignLeaseConsumed => "campaign_lease_consumed",
            Self::ProductionAsicUnavailable => "production_asic_unavailable",
            Self::ProductionAsicVersionMaskUnavailable => {
                "production_asic_version_mask_unavailable"
            }
            Self::ProductionAsicDispatchUnavailable => "production_asic_dispatch_unavailable",
            Self::ProductionAsicPollUnavailable => "production_asic_poll_unavailable",
            Self::ProductionAsicQueueFull => "production_asic_queue_full",
            Self::ProductionAsicWorkerUnavailable => "production_asic_worker_unavailable",
            Self::ActuationUnqualified => "actuation_unqualified",
            Self::PoolConfigurationUnavailable => "pool_configuration_unavailable",
            Self::PoolsExhausted => "pools_exhausted",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProductionSessionWakeup {
    NetworkChanged,
    SettingsChanged,
    ObservationsChanged,
    OperatorIntentChanged,
    ShutdownRequested,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProductionSessionNotificationOutcome {
    Queued,
    Coalesced,
    OwnerUnavailable,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProductionPool {
    Primary,
    Fallback,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ProductionReadiness {
    pub operator_intent: MiningOperatorIntent,
    pub network_ready: bool,
    pub stratum_v1_supported: bool,
    pub safety_prerequisites_fresh: bool,
    pub maybe_campaign_lease: Option<MiningCampaignLease>,
    pub actuation_qualified: bool,
}

impl ProductionReadiness {
    pub const fn maybe_blocker(self) -> Option<ProductionSessionBlocker> {
        if matches!(self.operator_intent, MiningOperatorIntent::Paused) {
            return Some(ProductionSessionBlocker::OperatorPaused);
        }
        if !self.network_ready {
            return Some(ProductionSessionBlocker::NetworkUnavailable);
        }
        if !self.stratum_v1_supported {
            return Some(ProductionSessionBlocker::StratumV1Unsupported);
        }
        if !self.safety_prerequisites_fresh {
            return Some(ProductionSessionBlocker::SafetyPrerequisitesStale);
        }
        if self.maybe_campaign_lease.is_none() {
            return Some(ProductionSessionBlocker::CampaignLeaseUnavailable);
        }
        if !self.actuation_qualified {
            return Some(ProductionSessionBlocker::ActuationUnqualified);
        }
        None
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ProductionPoolAvailability {
    pub primary_configured: bool,
    pub fallback_configured: bool,
    pub prefer_fallback: bool,
}

impl ProductionPoolAvailability {
    const fn maybe_preferred(self) -> Option<ProductionPool> {
        if self.prefer_fallback && self.fallback_configured {
            return Some(ProductionPool::Fallback);
        }
        if self.primary_configured {
            return Some(ProductionPool::Primary);
        }
        if self.fallback_configured {
            return Some(ProductionPool::Fallback);
        }
        None
    }

    const fn maybe_alternate(self, pool: ProductionPool) -> Option<ProductionPool> {
        match pool {
            ProductionPool::Primary if self.fallback_configured => Some(ProductionPool::Fallback),
            ProductionPool::Fallback if self.primary_configured => Some(ProductionPool::Primary),
            _ => None,
        }
    }

    const fn configured(self, pool: ProductionPool) -> bool {
        match pool {
            ProductionPool::Primary => self.primary_configured,
            ProductionPool::Fallback => self.fallback_configured,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RecoveryProjection {
    pub phase: ProductionSessionPhase,
    pub maybe_blocker: Option<ProductionSessionBlocker>,
    pub maybe_active_pool: Option<ProductionPool>,
    pub mining_activity: MiningActivityStatus,
    pub pool_lifecycle: PoolLifecycleStatus,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RecoveryAction {
    ReadPoolConfiguration,
    ConnectPool(ProductionPool),
    BlockSubmissions,
    InvalidateWorkAndSubmissions,
    StopAsicInteraction,
    ClosePoolConnection,
    Publish(RecoveryProjection),
}

#[derive(Debug, Clone)]
pub struct RecoveryPolicy {
    phase: ProductionSessionPhase,
    maybe_blocker: Option<ProductionSessionBlocker>,
    maybe_pool_availability: Option<ProductionPoolAvailability>,
    maybe_active_pool: Option<ProductionPool>,
    attempts: [u8; 2],
    maybe_next_action_at_ms: Option<u64>,
    maybe_primary_probe_at_ms: Option<u64>,
    primary_probe_started: bool,
    fallback_automatic: bool,
    resources_live: bool,
    maybe_last_projection: Option<RecoveryProjection>,
}

impl Default for RecoveryPolicy {
    fn default() -> Self {
        Self::new()
    }
}

impl RecoveryPolicy {
    pub const fn new() -> Self {
        Self {
            phase: ProductionSessionPhase::WaitingForReadiness,
            maybe_blocker: None,
            maybe_pool_availability: None,
            maybe_active_pool: None,
            attempts: [0; 2],
            maybe_next_action_at_ms: None,
            maybe_primary_probe_at_ms: None,
            primary_probe_started: false,
            fallback_automatic: false,
            resources_live: false,
            maybe_last_projection: None,
        }
    }

    pub const fn projection(&self) -> RecoveryProjection {
        let (mining_activity, pool_lifecycle) = match self.phase {
            ProductionSessionPhase::RunningPrimary => {
                (MiningActivityStatus::Active, PoolLifecycleStatus::Active)
            }
            ProductionSessionPhase::RunningFallback => (
                MiningActivityStatus::Active,
                PoolLifecycleStatus::FallbackActive,
            ),
            ProductionSessionPhase::ConnectingPrimary
            | ProductionSessionPhase::ConnectingFallback => (
                MiningActivityStatus::SafeBlocked,
                PoolLifecycleStatus::Connecting,
            ),
            ProductionSessionPhase::RecoveryPaused => (
                MiningActivityStatus::SafeBlocked,
                PoolLifecycleStatus::RecoveryPaused,
            ),
            ProductionSessionPhase::Shutdown => (
                MiningActivityStatus::SafeBlocked,
                PoolLifecycleStatus::Disconnected,
            ),
            ProductionSessionPhase::WaitingForReadiness | ProductionSessionPhase::SafeStopping => {
                let activity = if matches!(
                    self.maybe_blocker,
                    Some(ProductionSessionBlocker::OperatorPaused)
                ) {
                    MiningActivityStatus::Paused
                } else {
                    MiningActivityStatus::SafeBlocked
                };
                (activity, PoolLifecycleStatus::Disconnected)
            }
        };
        RecoveryProjection {
            phase: self.phase,
            maybe_blocker: self.maybe_blocker,
            maybe_active_pool: self.maybe_active_pool,
            mining_activity,
            pool_lifecycle,
        }
    }

    pub fn on_wakeup(
        &mut self,
        wakeup: Option<ProductionSessionWakeup>,
        readiness: ProductionReadiness,
        now_ms: u64,
    ) -> Vec<RecoveryAction> {
        if self.phase == ProductionSessionPhase::Shutdown {
            return Vec::new();
        }
        if matches!(wakeup, Some(ProductionSessionWakeup::ShutdownRequested)) {
            return self.shutdown();
        }

        let settings_changed = matches!(wakeup, Some(ProductionSessionWakeup::SettingsChanged));
        if settings_changed {
            self.reset_pool_policy();
        }

        if let Some(blocker) = readiness.maybe_blocker() {
            return self.safe_stop(ProductionSessionPhase::WaitingForReadiness, Some(blocker));
        }

        if settings_changed {
            let mut actions = self.safe_stop(ProductionSessionPhase::WaitingForReadiness, None);
            actions.push(RecoveryAction::ReadPoolConfiguration);
            return actions;
        }

        let Some(pool_availability) = self.maybe_pool_availability else {
            self.maybe_blocker = None;
            self.phase = ProductionSessionPhase::WaitingForReadiness;
            let mut actions = Vec::new();
            self.publish_if_changed(&mut actions);
            actions.push(RecoveryAction::ReadPoolConfiguration);
            return actions;
        };

        if self.phase == ProductionSessionPhase::WaitingForReadiness {
            self.attempts = [0; 2];
            let Some(pool) = pool_availability.maybe_preferred() else {
                return self.safe_stop(
                    ProductionSessionPhase::WaitingForReadiness,
                    Some(ProductionSessionBlocker::PoolConfigurationUnavailable),
                );
            };
            return self.connect(pool);
        }

        if self.phase == ProductionSessionPhase::RunningFallback
            && pool_availability.primary_configured
            && self
                .maybe_primary_probe_at_ms
                .is_some_and(|deadline| now_ms >= deadline)
        {
            self.primary_probe_started = true;
            self.phase = ProductionSessionPhase::ConnectingPrimary;
            self.resources_live = true;
            self.maybe_primary_probe_at_ms = None;
            return vec![RecoveryAction::ConnectPool(ProductionPool::Primary)];
        }

        if matches!(
            self.phase,
            ProductionSessionPhase::ConnectingPrimary
                | ProductionSessionPhase::ConnectingFallback
                | ProductionSessionPhase::RecoveryPaused
        ) && self
            .maybe_next_action_at_ms
            .is_some_and(|deadline| now_ms >= deadline)
        {
            let pool = if self.phase == ProductionSessionPhase::RecoveryPaused {
                self.attempts = [0; 2];
                pool_availability.maybe_preferred()
            } else {
                self.maybe_connecting_pool()
            };
            if let Some(pool) = pool {
                return self.connect(pool);
            }
        }

        let mut actions = Vec::new();
        self.publish_if_changed(&mut actions);
        actions
    }

    pub fn on_pool_configuration(
        &mut self,
        availability: ProductionPoolAvailability,
    ) -> Vec<RecoveryAction> {
        self.maybe_pool_availability = Some(availability);
        self.attempts = [0; 2];
        let Some(pool) = availability.maybe_preferred() else {
            return self.safe_stop(
                ProductionSessionPhase::WaitingForReadiness,
                Some(ProductionSessionBlocker::PoolConfigurationUnavailable),
            );
        };
        self.connect(pool)
    }

    pub fn on_session_blocker(&mut self, blocker: ProductionSessionBlocker) -> Vec<RecoveryAction> {
        self.safe_stop(ProductionSessionPhase::WaitingForReadiness, Some(blocker))
    }

    pub fn on_connection_result(
        &mut self,
        pool: ProductionPool,
        connected: bool,
        now_ms: u64,
    ) -> Vec<RecoveryAction> {
        if connected {
            self.maybe_active_pool = Some(pool);
            self.maybe_blocker = None;
            self.maybe_next_action_at_ms = None;
            self.resources_live = true;
            self.phase = match pool {
                ProductionPool::Primary => ProductionSessionPhase::RunningPrimary,
                ProductionPool::Fallback => ProductionSessionPhase::RunningFallback,
            };
            if pool == ProductionPool::Fallback && self.fallback_automatic {
                let delay = if self.primary_probe_started {
                    PRIMARY_RECURRING_PROBE_DELAY_MS
                } else {
                    PRIMARY_INITIAL_PROBE_DELAY_MS
                };
                self.maybe_primary_probe_at_ms = Some(now_ms.saturating_add(delay));
            } else {
                self.primary_probe_started = false;
                self.maybe_primary_probe_at_ms = None;
                if pool == ProductionPool::Primary {
                    self.fallback_automatic = false;
                }
            }
            let mut actions = Vec::new();
            self.publish_if_changed(&mut actions);
            return actions;
        }

        if pool == ProductionPool::Primary
            && self.primary_probe_started
            && self.maybe_active_pool == Some(ProductionPool::Fallback)
        {
            self.phase = ProductionSessionPhase::RunningFallback;
            self.maybe_primary_probe_at_ms =
                Some(now_ms.saturating_add(PRIMARY_RECURRING_PROBE_DELAY_MS));
            let mut actions = Vec::new();
            self.publish_if_changed(&mut actions);
            return actions;
        }

        let index = pool_index(pool);
        self.attempts[index] = self.attempts[index].saturating_add(1);
        if self.attempts[index] < CONNECTION_ATTEMPTS_PER_POOL {
            self.phase = connecting_phase(pool);
            self.maybe_next_action_at_ms = Some(now_ms.saturating_add(CONNECTION_RETRY_DELAY_MS));
            let mut actions = Vec::new();
            self.publish_if_changed(&mut actions);
            return actions;
        }

        let Some(availability) = self.maybe_pool_availability else {
            return self.enter_recovery_pause(now_ms);
        };
        if let Some(alternate) = availability.maybe_alternate(pool) {
            if availability.configured(alternate)
                && self.attempts[pool_index(alternate)] < CONNECTION_ATTEMPTS_PER_POOL
            {
                self.fallback_automatic =
                    pool == ProductionPool::Primary && alternate == ProductionPool::Fallback;
                return self.connect(alternate);
            }
        }
        self.enter_recovery_pause(now_ms)
    }

    pub fn on_connection_lost(&mut self, now_ms: u64) -> Vec<RecoveryAction> {
        let pool = self
            .maybe_active_pool
            .or_else(|| self.maybe_connecting_pool())
            .unwrap_or(ProductionPool::Primary);
        let actions = self.safe_stop(connecting_phase(pool), None);
        self.phase = connecting_phase(pool);
        self.maybe_next_action_at_ms = Some(now_ms.saturating_add(CONNECTION_RETRY_DELAY_MS));
        actions
    }

    fn connect(&mut self, pool: ProductionPool) -> Vec<RecoveryAction> {
        self.phase = connecting_phase(pool);
        self.maybe_active_pool = None;
        self.maybe_blocker = None;
        self.maybe_next_action_at_ms = None;
        self.resources_live = true;
        vec![RecoveryAction::ConnectPool(pool)]
    }

    fn enter_recovery_pause(&mut self, now_ms: u64) -> Vec<RecoveryAction> {
        let mut actions = self.safe_stop(
            ProductionSessionPhase::RecoveryPaused,
            Some(ProductionSessionBlocker::PoolsExhausted),
        );
        self.phase = ProductionSessionPhase::RecoveryPaused;
        self.maybe_next_action_at_ms = Some(now_ms.saturating_add(RECOVERY_PROBE_DELAY_MS));
        self.publish_if_changed(&mut actions);
        actions
    }

    fn shutdown(&mut self) -> Vec<RecoveryAction> {
        self.safe_stop(ProductionSessionPhase::Shutdown, None)
    }

    fn safe_stop(
        &mut self,
        final_phase: ProductionSessionPhase,
        maybe_blocker: Option<ProductionSessionBlocker>,
    ) -> Vec<RecoveryAction> {
        let next_projection = RecoveryProjection {
            phase: final_phase,
            maybe_blocker,
            maybe_active_pool: None,
            mining_activity: if matches!(
                maybe_blocker,
                Some(ProductionSessionBlocker::OperatorPaused)
            ) {
                MiningActivityStatus::Paused
            } else {
                MiningActivityStatus::SafeBlocked
            },
            pool_lifecycle: if final_phase == ProductionSessionPhase::RecoveryPaused {
                PoolLifecycleStatus::RecoveryPaused
            } else {
                PoolLifecycleStatus::Disconnected
            },
        };
        if !self.resources_live && self.maybe_last_projection == Some(next_projection) {
            return Vec::new();
        }

        let mut actions = Vec::new();
        if self.resources_live {
            self.phase = ProductionSessionPhase::SafeStopping;
            actions.extend([
                RecoveryAction::BlockSubmissions,
                RecoveryAction::InvalidateWorkAndSubmissions,
                RecoveryAction::StopAsicInteraction,
                RecoveryAction::ClosePoolConnection,
            ]);
        }
        self.resources_live = false;
        self.phase = final_phase;
        self.maybe_blocker = maybe_blocker;
        self.maybe_active_pool = None;
        self.maybe_next_action_at_ms = None;
        self.maybe_primary_probe_at_ms = None;
        self.publish_if_changed(&mut actions);
        actions
    }

    fn reset_pool_policy(&mut self) {
        self.maybe_pool_availability = None;
        self.attempts = [0; 2];
        self.maybe_next_action_at_ms = None;
        self.maybe_primary_probe_at_ms = None;
        self.primary_probe_started = false;
        self.fallback_automatic = false;
    }

    fn maybe_connecting_pool(&self) -> Option<ProductionPool> {
        match self.phase {
            ProductionSessionPhase::ConnectingPrimary => Some(ProductionPool::Primary),
            ProductionSessionPhase::ConnectingFallback => Some(ProductionPool::Fallback),
            _ => None,
        }
    }

    fn publish_if_changed(&mut self, actions: &mut Vec<RecoveryAction>) {
        let projection = self.projection();
        if self.maybe_last_projection != Some(projection) {
            self.maybe_last_projection = Some(projection);
            actions.push(RecoveryAction::Publish(projection));
        }
    }
}

const fn pool_index(pool: ProductionPool) -> usize {
    match pool {
        ProductionPool::Primary => 0,
        ProductionPool::Fallback => 1,
    }
}

const fn connecting_phase(pool: ProductionPool) -> ProductionSessionPhase {
    match pool {
        ProductionPool::Primary => ProductionSessionPhase::ConnectingPrimary,
        ProductionPool::Fallback => ProductionSessionPhase::ConnectingFallback,
    }
}
