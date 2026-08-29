//! Validated hardware profiles and bounded one-shot mining campaign leases.

use bitaxe_config::{AsicFrequencyMhz, ConfigValidationError, CoreVoltageMv, FanDutyPercent};
use thiserror::Error;

pub const MAX_MINING_CAMPAIGN_DURATION_MS: u64 = 1_800_000;

/// Closed Ultra 205 BM1366 profiles admitted by production mining campaigns.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MiningHardwareProfilePreset {
    Conservative,
    UpstreamDefault,
}

impl MiningHardwareProfilePreset {
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::Conservative => "conservative",
            Self::UpstreamDefault => "upstream-default",
        }
    }

    #[must_use]
    pub fn profile(self) -> MiningHardwareProfile {
        let (frequency_mhz, core_voltage_mv, fan_duty_percent) = match self {
            Self::Conservative => (400, 1_100, 100),
            Self::UpstreamDefault => (485, 1_200, 100),
        };

        MiningHardwareProfile::ultra_205_bm1366(frequency_mhz, core_voltage_mv, fan_duty_percent)
            .expect("closed Ultra 205 production profiles must remain catalog-valid")
    }
}

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

    /// Returns whether this profile is one of the two closed production pairs.
    #[must_use]
    pub const fn is_closed_ultra_205_production_profile(self) -> bool {
        matches!(
            (
                self.frequency.mhz(),
                self.core_voltage.millivolts(),
                self.fan_duty.percent(),
            ),
            (400, 1_100, 100) | (485, 1_200, 100)
        )
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
    #[error("mining campaign monotonic deadline must be nonzero")]
    ZeroMonotonicDeadline,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MiningCampaignMonotonicDeadline(u64);

impl MiningCampaignMonotonicDeadline {
    pub fn new(expires_at_ms: u64) -> Result<Self, MiningCampaignLeaseError> {
        if expires_at_ms == 0 {
            return Err(MiningCampaignLeaseError::ZeroMonotonicDeadline);
        }
        Ok(Self(expires_at_ms))
    }

    #[must_use]
    pub const fn milliseconds(self) -> u64 {
        self.0
    }
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
    MonotonicDeadline {
        deadline: MiningCampaignMonotonicDeadline,
    },
    FirstSubmitResponse {
        timeout: MiningCampaignDuration,
    },
    ActiveDuration {
        duration: MiningCampaignDuration,
    },
    ResumableActiveEpoch {
        activation_timeout: MiningCampaignDuration,
        duration: MiningCampaignDuration,
    },
}

impl MiningCampaignStopCondition {
    #[must_use]
    pub const fn allows_operator_resume(self) -> bool {
        matches!(self, Self::ResumableActiveEpoch { .. })
    }

    pub(super) fn maybe_expiration(
        self,
        now_ms: u64,
        timing: MiningCampaignTiming,
    ) -> Option<CampaignExpiration> {
        match self {
            Self::MonotonicDeadline { deadline } => {
                (now_ms >= deadline.milliseconds()).then_some(CampaignExpiration::LeaseConsumed)
            }
            Self::FirstSubmitResponse { timeout } => timing
                .maybe_prepared_at_ms
                .is_some_and(|started| now_ms.saturating_sub(started) >= timeout.milliseconds())
                .then_some(CampaignExpiration::LeaseConsumed),
            Self::ActiveDuration { duration } => timing
                .maybe_active_since_ms
                .is_some_and(|started| now_ms.saturating_sub(started) >= duration.milliseconds())
                .then_some(CampaignExpiration::LeaseConsumed),
            Self::ResumableActiveEpoch {
                activation_timeout,
                duration,
            } => {
                if timing.maybe_resumable_epoch_started_at_ms.is_some() {
                    let current_segment_ms = timing
                        .maybe_active_since_ms
                        .map_or(0, |started| now_ms.saturating_sub(started));
                    let active_ms = timing
                        .resumable_active_ms
                        .saturating_add(current_segment_ms);
                    return (active_ms >= duration.milliseconds())
                        .then_some(CampaignExpiration::LeaseConsumed);
                }
                timing
                    .maybe_activation_started_at_ms
                    .is_some_and(|started| {
                        now_ms.saturating_sub(started) >= activation_timeout.milliseconds()
                    })
                    .then_some(CampaignExpiration::ActivationTimedOut)
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum CampaignExpiration {
    ActivationTimedOut,
    LeaseConsumed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct MiningCampaignTiming {
    pub(super) maybe_prepared_at_ms: Option<u64>,
    pub(super) maybe_activation_started_at_ms: Option<u64>,
    pub(super) maybe_resumable_epoch_started_at_ms: Option<u64>,
    pub(super) resumable_active_ms: u64,
    pub(super) maybe_active_since_ms: Option<u64>,
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

#[cfg(test)]
mod tests {
    use super::{
        MiningCampaignDuration, MiningCampaignLeaseError, MiningHardwareProfile,
        MiningHardwareProfilePreset,
    };

    #[test]
    fn conservative_profile_is_the_validated_low_power_ultra_205_profile() {
        // Arrange
        let preset = MiningHardwareProfilePreset::Conservative;

        // Act
        let profile = preset.profile();

        // Assert
        assert_eq!(preset.label(), "conservative");
        assert_eq!(profile.frequency().mhz(), 400);
        assert_eq!(profile.core_voltage().millivolts(), 1_100);
        assert_eq!(profile.fan_duty().percent(), 100);
        assert!(profile.is_closed_ultra_205_production_profile());
    }

    #[test]
    fn upstream_default_profile_matches_the_ultra_205_catalog_defaults() {
        // Arrange
        let preset = MiningHardwareProfilePreset::UpstreamDefault;

        // Act
        let profile = preset.profile();

        // Assert
        assert_eq!(preset.label(), "upstream-default");
        assert_eq!(profile.frequency().mhz(), 485);
        assert_eq!(profile.core_voltage().millivolts(), 1_200);
        assert_eq!(profile.fan_duty().percent(), 100);
        assert!(profile.is_closed_ultra_205_production_profile());
    }

    #[test]
    fn individually_valid_but_mismatched_profile_values_are_not_closed() {
        // Arrange
        let profile = MiningHardwareProfile::ultra_205_bm1366(400, 1_200, 100)
            .expect("catalog-valid values should construct a general hardware profile");

        // Act
        let closed = profile.is_closed_ultra_205_production_profile();

        // Assert
        assert!(!closed);
    }

    #[test]
    fn thirty_minute_duration_is_the_closed_lease_ceiling() {
        // Arrange
        let exact_duration_ms = 1_800_000;

        // Act
        let exact = MiningCampaignDuration::new(exact_duration_ms);
        let over = MiningCampaignDuration::new(exact_duration_ms + 1);

        // Assert
        assert_eq!(
            exact
                .expect("thirty-minute lease should be admitted")
                .milliseconds(),
            exact_duration_ms
        );
        assert_eq!(
            over,
            Err(MiningCampaignLeaseError::InvalidDuration {
                duration_ms: exact_duration_ms + 1,
            })
        );
    }
}
