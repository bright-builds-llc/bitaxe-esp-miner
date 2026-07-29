//! Validated hardware profiles and bounded one-shot mining campaign leases.

use bitaxe_config::{AsicFrequencyMhz, ConfigValidationError, CoreVoltageMv, FanDutyPercent};
use thiserror::Error;

pub const MAX_MINING_CAMPAIGN_DURATION_MS: u64 = 600_000;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MiningHardwareProfile {
    frequency: AsicFrequencyMhz,
    core_voltage: CoreVoltageMv,
    fan_duty: FanDutyPercent,
}

impl MiningHardwareProfile {
    pub fn ultra_205_bm1366(
        frequency_mhz: i64,
        core_voltage_mv: i64,
        fan_duty_percent: i64,
    ) -> Result<Self, ConfigValidationError> {
        Ok(Self {
            frequency: AsicFrequencyMhz::ultra_205_bm1366(frequency_mhz)?,
            core_voltage: CoreVoltageMv::ultra_205_bm1366(core_voltage_mv)?,
            fan_duty: FanDutyPercent::parse(fan_duty_percent)?,
        })
    }

    #[must_use]
    pub const fn frequency(self) -> AsicFrequencyMhz {
        self.frequency
    }

    #[must_use]
    pub const fn core_voltage(self) -> CoreVoltageMv {
        self.core_voltage
    }

    #[must_use]
    pub const fn fan_duty(self) -> FanDutyPercent {
        self.fan_duty
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
pub enum MiningCampaignLeaseError {
    #[error("mining campaign lease id must be nonzero")]
    ZeroLeaseId,
    #[error(
        "mining campaign duration {duration_ms}ms is outside 1..={MAX_MINING_CAMPAIGN_DURATION_MS}ms"
    )]
    InvalidDuration { duration_ms: u64 },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MiningCampaignLeaseId(u64);

impl MiningCampaignLeaseId {
    pub fn new(raw: u64) -> Result<Self, MiningCampaignLeaseError> {
        if raw == 0 {
            return Err(MiningCampaignLeaseError::ZeroLeaseId);
        }
        Ok(Self(raw))
    }

    #[must_use]
    pub const fn raw(self) -> u64 {
        self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MiningCampaignDuration(u64);

impl MiningCampaignDuration {
    pub fn new(duration_ms: u64) -> Result<Self, MiningCampaignLeaseError> {
        if duration_ms == 0 || duration_ms > MAX_MINING_CAMPAIGN_DURATION_MS {
            return Err(MiningCampaignLeaseError::InvalidDuration { duration_ms });
        }
        Ok(Self(duration_ms))
    }

    #[must_use]
    pub const fn milliseconds(self) -> u64 {
        self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MiningCampaignStopCondition {
    FirstSubmitResponse { timeout: MiningCampaignDuration },
    ActiveDuration { duration: MiningCampaignDuration },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MiningCampaignLease {
    id: MiningCampaignLeaseId,
    profile: MiningHardwareProfile,
    stop_condition: MiningCampaignStopCondition,
}

impl MiningCampaignLease {
    #[must_use]
    pub const fn new(
        id: MiningCampaignLeaseId,
        profile: MiningHardwareProfile,
        stop_condition: MiningCampaignStopCondition,
    ) -> Self {
        Self {
            id,
            profile,
            stop_condition,
        }
    }

    #[must_use]
    pub const fn id(self) -> MiningCampaignLeaseId {
        self.id
    }

    #[must_use]
    pub const fn profile(self) -> MiningHardwareProfile {
        self.profile
    }

    #[must_use]
    pub const fn stop_condition(self) -> MiningCampaignStopCondition {
        self.stop_condition
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MiningHardwareState {
    Unprepared,
    Preparing,
    Ready,
    SafeStopping,
    Stopped,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MiningCampaignState {
    Unavailable,
    Preparing,
    Armed,
    Active,
    SafeStopping,
    Consumed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HardwarePreparationFailure {
    Rejected,
    TimedOut,
    DeviceFault,
}
