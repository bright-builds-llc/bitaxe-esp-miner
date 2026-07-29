use std::fmt;

use bitaxe_asic::bm1366::{
    command::VersionMask, production::Bm1366ProductionCommand, result::Bm1366ValidJobIds,
};

use super::campaign::{
    HardwarePreparationFailure, MiningCampaignLeaseId, MiningCampaignState, MiningHardwareProfile,
    MiningHardwareState,
};
use crate::v1::live_runtime::LiveRuntimeConfig;
use crate::v1::production_work::{PoolSessionGeneration, ProductionNonceObservation};
use crate::v1::recovery_policy::{
    ProductionPool, ProductionPoolAvailability, ProductionReadiness, ProductionSessionBlocker,
    ProductionSessionPhase, ProductionSessionWakeup,
};
use crate::v1::state::MiningRuntimeState;

#[derive(Clone, PartialEq, Eq)]
pub struct ProductionPoolEndpoint {
    pub host: String,
    pub port: u16,
}

impl fmt::Debug for ProductionPoolEndpoint {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("ProductionPoolEndpoint")
            .field("endpoint", &"redacted")
            .finish()
    }
}

#[derive(Clone, PartialEq, Eq)]
pub struct ProductionPoolConfiguration {
    pub endpoint: ProductionPoolEndpoint,
    pub runtime: LiveRuntimeConfig,
}

impl fmt::Debug for ProductionPoolConfiguration {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("ProductionPoolConfiguration")
            .field("endpoint", &"redacted")
            .field("runtime", &"redacted")
            .finish()
    }
}

#[derive(Clone, PartialEq, Eq)]
pub struct ProductionPoolSet {
    pub primary: Option<ProductionPoolConfiguration>,
    pub fallback: Option<ProductionPoolConfiguration>,
    pub prefer_fallback: bool,
}

impl fmt::Debug for ProductionPoolSet {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("ProductionPoolSet")
            .field("primary_configured", &self.primary.is_some())
            .field("fallback_configured", &self.fallback.is_some())
            .field("prefer_fallback", &self.prefer_fallback)
            .finish()
    }
}

impl ProductionPoolSet {
    pub(super) fn availability(&self) -> ProductionPoolAvailability {
        ProductionPoolAvailability {
            primary_configured: self.primary.is_some(),
            fallback_configured: self.fallback.is_some(),
            prefer_fallback: self.prefer_fallback,
        }
    }

    pub(super) fn maybe_configuration(
        &self,
        pool: ProductionPool,
    ) -> Option<&ProductionPoolConfiguration> {
        match pool {
            ProductionPool::Primary => self.primary.as_ref(),
            ProductionPool::Fallback => self.fallback.as_ref(),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ProductionSessionSnapshot {
    pub phase: ProductionSessionPhase,
    pub maybe_blocker: Option<ProductionSessionBlocker>,
    pub maybe_active_pool: Option<ProductionPool>,
    pub generation: PoolSessionGeneration,
    pub hardware_state: MiningHardwareState,
    pub campaign_state: MiningCampaignState,
    pub mining: MiningRuntimeState,
}

#[derive(Clone, PartialEq)]
pub enum ProductionSessionEvent {
    Wake {
        wakeup: Option<ProductionSessionWakeup>,
        readiness: ProductionReadiness,
        now_ms: u64,
    },
    PoolConfigurationLoaded(Option<Box<ProductionPoolSet>>),
    TransportConnected {
        pool: ProductionPool,
        now_ms: u64,
    },
    TransportConnectFailed {
        pool: ProductionPool,
        now_ms: u64,
    },
    TransportBytes {
        pool: ProductionPool,
        bytes: Vec<u8>,
        now_ms: u64,
    },
    TransportClosed {
        pool: ProductionPool,
        now_ms: u64,
    },
    AsicResult {
        observation: ProductionNonceObservation,
        now_ms: u64,
    },
    AsicPollTimedOut {
        now_ms: u64,
    },
    HardwarePrepared {
        lease_id: MiningCampaignLeaseId,
        now_ms: u64,
    },
    HardwarePreparationFailed {
        lease_id: MiningCampaignLeaseId,
        failure: HardwarePreparationFailure,
        now_ms: u64,
    },
    HardwareSafeStopConfirmed {
        lease_id: MiningCampaignLeaseId,
        now_ms: u64,
    },
    EffectFailed {
        maybe_pool: Option<ProductionPool>,
        reason: &'static str,
        now_ms: u64,
    },
}

impl fmt::Debug for ProductionSessionEvent {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::TransportBytes {
                pool,
                bytes,
                now_ms,
            } => formatter
                .debug_struct("ProductionSessionEvent::TransportBytes")
                .field("pool", pool)
                .field("byte_count", &bytes.len())
                .field("now_ms", now_ms)
                .finish(),
            Self::PoolConfigurationLoaded(maybe_pools) => formatter
                .debug_tuple("ProductionSessionEvent::PoolConfigurationLoaded")
                .field(
                    &maybe_pools
                        .as_ref()
                        .map(|_| "configured_redacted")
                        .unwrap_or("unavailable"),
                )
                .finish(),
            other => formatter.write_str(match other {
                Self::Wake { .. } => "ProductionSessionEvent::Wake",
                Self::TransportConnected { .. } => "ProductionSessionEvent::TransportConnected",
                Self::TransportConnectFailed { .. } => {
                    "ProductionSessionEvent::TransportConnectFailed"
                }
                Self::TransportClosed { .. } => "ProductionSessionEvent::TransportClosed",
                Self::AsicResult { .. } => "ProductionSessionEvent::AsicResult(redacted)",
                Self::AsicPollTimedOut { .. } => "ProductionSessionEvent::AsicPollTimedOut",
                Self::HardwarePrepared { .. } => "ProductionSessionEvent::HardwarePrepared",
                Self::HardwarePreparationFailed { .. } => {
                    "ProductionSessionEvent::HardwarePreparationFailed"
                }
                Self::HardwareSafeStopConfirmed { .. } => {
                    "ProductionSessionEvent::HardwareSafeStopConfirmed"
                }
                Self::EffectFailed { .. } => "ProductionSessionEvent::EffectFailed",
                Self::PoolConfigurationLoaded(_) | Self::TransportBytes { .. } => unreachable!(),
            }),
        }
    }
}

#[derive(Clone, PartialEq)]
pub enum ProductionSessionEffect {
    PrepareHardware {
        lease_id: MiningCampaignLeaseId,
        profile: MiningHardwareProfile,
    },
    ReadPoolConfiguration,
    ConnectPool(ProductionPool),
    WritePoolLine {
        pool: ProductionPool,
        line: String,
    },
    ApplyVersionMask(VersionMask),
    DispatchAsic {
        generation: PoolSessionGeneration,
        valid_jobs: Bm1366ValidJobIds,
        command: Bm1366ProductionCommand,
    },
    PollAsic {
        generation: PoolSessionGeneration,
        valid_jobs: Bm1366ValidJobIds,
        slice_ms: u32,
    },
    BlockSubmissions,
    InvalidateWorkAndSubmissions,
    StopAsicInteraction,
    ClosePoolConnection(ProductionPool),
    SafeStopHardware {
        lease_id: MiningCampaignLeaseId,
    },
    Publish(ProductionSessionSnapshot),
}

impl fmt::Debug for ProductionSessionEffect {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::WritePoolLine { pool, .. } => formatter
                .debug_struct("ProductionSessionEffect::WritePoolLine")
                .field("pool", pool)
                .field("line", &"redacted")
                .finish(),
            Self::DispatchAsic { .. } => {
                formatter.write_str("ProductionSessionEffect::DispatchAsic(redacted)")
            }
            other => match other {
                Self::PrepareHardware { lease_id, .. } => formatter
                    .debug_struct("ProductionSessionEffect::PrepareHardware")
                    .field("lease_id", lease_id)
                    .field("profile", &"validated_redacted")
                    .finish(),
                Self::ReadPoolConfiguration => {
                    formatter.write_str("ProductionSessionEffect::ReadPoolConfiguration")
                }
                Self::ConnectPool(pool) => formatter
                    .debug_tuple("ProductionSessionEffect::ConnectPool")
                    .field(pool)
                    .finish(),
                Self::ApplyVersionMask(_) => {
                    formatter.write_str("ProductionSessionEffect::ApplyVersionMask(redacted)")
                }
                Self::PollAsic {
                    generation,
                    slice_ms,
                    ..
                } => formatter
                    .debug_struct("ProductionSessionEffect::PollAsic")
                    .field("generation", generation)
                    .field("valid_jobs", &"redacted")
                    .field("slice_ms", slice_ms)
                    .finish(),
                Self::BlockSubmissions => {
                    formatter.write_str("ProductionSessionEffect::BlockSubmissions")
                }
                Self::InvalidateWorkAndSubmissions => {
                    formatter.write_str("ProductionSessionEffect::InvalidateWorkAndSubmissions")
                }
                Self::StopAsicInteraction => {
                    formatter.write_str("ProductionSessionEffect::StopAsicInteraction")
                }
                Self::ClosePoolConnection(pool) => formatter
                    .debug_tuple("ProductionSessionEffect::ClosePoolConnection")
                    .field(pool)
                    .finish(),
                Self::SafeStopHardware { lease_id } => formatter
                    .debug_struct("ProductionSessionEffect::SafeStopHardware")
                    .field("lease_id", lease_id)
                    .finish(),
                Self::Publish(snapshot) => formatter
                    .debug_tuple("ProductionSessionEffect::Publish")
                    .field(snapshot)
                    .finish(),
                Self::WritePoolLine { .. } | Self::DispatchAsic { .. } => unreachable!(),
            },
        }
    }
}
