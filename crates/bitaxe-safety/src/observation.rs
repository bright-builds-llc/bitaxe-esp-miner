//! Producer-owned observation truth and sample provenance.
//!
//! This module is intentionally pure: producers supply session, sequence, and
//! monotonic acquisition values, while consumers only inspect or copy them.

use core::cmp::Ordering;

use serde::{Deserialize, Serialize};

/// Identifies one boot or producer session.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct BootSessionId(u64);

impl BootSessionId {
    #[must_use]
    pub const fn new(value: u64) -> Self {
        Self(value)
    }

    #[must_use]
    pub const fn get(self) -> u64 {
        self.0
    }
}

/// Source-local sequence for successful observations within one boot session.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct ObservationSequence(u64);

impl ObservationSequence {
    pub const ZERO: Self = Self(0);

    #[must_use]
    pub const fn new(value: u64) -> Self {
        Self(value)
    }

    #[must_use]
    pub const fn get(self) -> u64 {
        self.0
    }

    pub const fn advance(self) -> Result<Self, SequenceOverflow> {
        let Some(next) = self.0.checked_add(1) else {
            return Err(SequenceOverflow);
        };

        Ok(Self(next))
    }
}

/// Monotonic acquisition time supplied by the producer, in milliseconds.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct MonotonicMillis(u64);

impl MonotonicMillis {
    #[must_use]
    pub const fn new(value: u64) -> Self {
        Self(value)
    }

    #[must_use]
    pub const fn get(self) -> u64 {
        self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SequenceOverflow;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MissingLastGood;

/// A validated successful value and its producer-owned provenance.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct StampedSample<T> {
    value: T,
    boot_session: BootSessionId,
    sequence: ObservationSequence,
    acquired_at: MonotonicMillis,
}

impl<T> StampedSample<T> {
    #[must_use]
    pub const fn new(
        value: T,
        boot_session: BootSessionId,
        sequence: ObservationSequence,
        acquired_at: MonotonicMillis,
    ) -> Self {
        Self {
            value,
            boot_session,
            sequence,
            acquired_at,
        }
    }

    #[must_use]
    pub const fn value(&self) -> &T {
        &self.value
    }

    #[must_use]
    pub const fn boot_session(&self) -> BootSessionId {
        self.boot_session
    }

    #[must_use]
    pub const fn sequence(&self) -> ObservationSequence {
        self.sequence
    }

    #[must_use]
    pub const fn acquired_at(&self) -> MonotonicMillis {
        self.acquired_at
    }

    /// Compares source order only when both samples belong to the same session.
    #[must_use]
    pub fn source_order(&self, other: &Self) -> SampleOrder {
        if self.boot_session != other.boot_session {
            return SampleOrder::DifferentSession;
        }

        match self.sequence.0.cmp(&other.sequence.0) {
            Ordering::Less => SampleOrder::Earlier,
            Ordering::Equal => SampleOrder::Same,
            Ordering::Greater => SampleOrder::Later,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SampleOrder {
    Earlier,
    Same,
    Later,
    DifferentSession,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StaleReason {
    ProducerCadenceExpired,
    ProducerTimeout,
    PowerSampleStale,
    ThermalSampleStale,
    TachometerStale,
}

impl StaleReason {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::ProducerCadenceExpired => "producer_cadence_expired",
            Self::ProducerTimeout => "producer_timeout",
            Self::PowerSampleStale => "power_sample_stale",
            Self::ThermalSampleStale => "thermal_sample_stale",
            Self::TachometerStale => "tachometer_stale",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UnavailableReason {
    NotYetObserved,
    ProducerUnavailable,
    PowerSampleUnavailable,
    ThermalReadingUnavailable,
    TachometerUnavailable,
}

impl UnavailableReason {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::NotYetObserved => "not_yet_observed",
            Self::ProducerUnavailable => "producer_unavailable",
            Self::PowerSampleUnavailable => "power_sample_unavailable",
            Self::ThermalReadingUnavailable => "thermal_reading_unavailable",
            Self::TachometerUnavailable => "tachometer_unavailable",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FaultReason {
    ReadFailed,
    InvalidSample,
    UnsafeReading,
    Ina260ReadFailed,
    InputVoltageUnsafe,
    PowerLimitExceeded,
    PowerReadingInvalid,
    ThermalReadingInvalid,
}

impl FaultReason {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::ReadFailed => "read_failed",
            Self::InvalidSample => "invalid_sample",
            Self::UnsafeReading => "unsafe_reading",
            Self::Ina260ReadFailed => "ina260_read_failed",
            Self::InputVoltageUnsafe => "input_voltage_unsafe",
            Self::PowerLimitExceeded => "power_limit_exceeded",
            Self::PowerReadingInvalid => "power_reading_invalid",
            Self::ThermalReadingInvalid => "thermal_reading_invalid",
        }
    }
}

/// Mutually exclusive observation truth with variant-owned data.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Observation<T> {
    Fresh {
        sample: StampedSample<T>,
    },
    Stale {
        last_good: StampedSample<T>,
        reason: StaleReason,
    },
    Unavailable {
        reason: UnavailableReason,
    },
    Fault {
        reason: FaultReason,
        maybe_last_good: Option<StampedSample<T>>,
    },
}

impl<T> Observation<T> {
    /// Records one validated producer success and advances its sequence once.
    pub fn record_success(
        value: T,
        boot_session: BootSessionId,
        prior_sequence: ObservationSequence,
        acquired_at: MonotonicMillis,
    ) -> Result<(Self, ObservationSequence), SequenceOverflow> {
        let sequence = prior_sequence.advance()?;
        let sample = StampedSample::new(value, boot_session, sequence, acquired_at);

        Ok((Self::Fresh { sample }, sequence))
    }

    #[must_use]
    pub const fn unavailable(reason: UnavailableReason) -> Self {
        Self::Unavailable { reason }
    }

    #[must_use]
    pub const fn maybe_last_good(&self) -> Option<&StampedSample<T>> {
        match self {
            Self::Fresh { sample } => Some(sample),
            Self::Stale { last_good, .. } => Some(last_good),
            Self::Unavailable { .. } => None,
            Self::Fault {
                maybe_last_good, ..
            } => maybe_last_good.as_ref(),
        }
    }

    #[must_use]
    pub const fn state_label(&self) -> &'static str {
        match self {
            Self::Fresh { .. } => "fresh",
            Self::Stale { .. } => "stale",
            Self::Unavailable { .. } => "unavailable",
            Self::Fault { .. } => "fault",
        }
    }

    #[must_use]
    pub const fn is_fresh(&self) -> bool {
        matches!(self, Self::Fresh { .. })
    }

    #[must_use]
    pub const fn maybe_reason(&self) -> Option<&'static str> {
        match self {
            Self::Fresh { .. } => None,
            Self::Stale { reason, .. } => Some(reason.as_str()),
            Self::Unavailable { reason } => Some(reason.as_str()),
            Self::Fault { reason, .. } => Some(reason.as_str()),
        }
    }
}

impl<T: Clone> Observation<T> {
    /// Marks an existing successful sample stale without changing its stamp.
    pub fn mark_stale(&self, reason: StaleReason) -> Result<Self, MissingLastGood> {
        let Some(last_good) = self.maybe_last_good() else {
            return Err(MissingLastGood);
        };

        Ok(Self::Stale {
            last_good: last_good.clone(),
            reason,
        })
    }

    /// Records a failed attempt without storing its attempted value.
    #[must_use]
    pub fn record_fault(&self, reason: FaultReason) -> Self {
        Self::Fault {
            reason,
            maybe_last_good: self.maybe_last_good().cloned(),
        }
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    fn fresh(value: u16, session: u64, prior_sequence: u64) -> Observation<u16> {
        Observation::record_success(
            value,
            BootSessionId::new(session),
            ObservationSequence::new(prior_sequence),
            MonotonicMillis::new(250),
        )
        .expect("fixture sequence should advance")
        .0
    }

    #[test]
    fn observation_variants_expose_exact_state_labels() {
        // Arrange
        let fresh = fresh(42, 7, 0);
        let stale = fresh
            .mark_stale(StaleReason::ProducerTimeout)
            .expect("fresh sample can become stale");
        let unavailable = Observation::<u16>::unavailable(UnavailableReason::NotYetObserved);
        let fault = unavailable.record_fault(FaultReason::ReadFailed);

        // Act
        let labels = [
            fresh.state_label(),
            stale.state_label(),
            unavailable.state_label(),
            fault.state_label(),
        ];

        // Assert
        assert_eq!(labels, ["fresh", "stale", "unavailable", "fault"]);
        assert!(fresh.is_fresh());
        assert!(!stale.is_fresh());
    }

    #[test]
    fn successful_production_advances_sequence_exactly_once() {
        // Arrange
        let prior_sequence = ObservationSequence::new(9);

        // Act
        let (observation, sequence) = Observation::record_success(
            42_u16,
            BootSessionId::new(7),
            prior_sequence,
            MonotonicMillis::new(250),
        )
        .expect("fixture sequence should advance");

        // Assert
        assert_eq!(sequence, ObservationSequence::new(10));
        assert_eq!(
            observation
                .maybe_last_good()
                .expect("fresh observation has a sample")
                .sequence(),
            sequence
        );
    }

    #[test]
    fn stale_transition_preserves_the_exact_last_good_sample() {
        // Arrange
        let fresh = fresh(42, 7, 9);
        let expected = *fresh
            .maybe_last_good()
            .expect("fresh observation has a sample");

        // Act
        let stale = fresh
            .mark_stale(StaleReason::ProducerCadenceExpired)
            .expect("fresh sample can become stale");

        // Assert
        assert_eq!(stale.maybe_last_good(), Some(&expected));
        assert_eq!(stale.maybe_last_good().map(StampedSample::value), Some(&42));
    }

    #[test]
    fn fault_retains_last_good_without_storing_failed_attempt() {
        // Arrange
        let fresh = fresh(42, 7, 9);
        let expected = *fresh
            .maybe_last_good()
            .expect("fresh observation has a sample");

        // Act
        let fault = fresh.record_fault(FaultReason::InvalidSample);

        // Assert
        assert_eq!(fault.maybe_last_good(), Some(&expected));
        assert_eq!(fault.maybe_last_good().map(StampedSample::value), Some(&42));
    }

    #[test]
    fn fault_without_last_good_and_unavailable_have_no_sample() {
        // Arrange
        let unavailable = Observation::<u16>::unavailable(UnavailableReason::ProducerUnavailable);

        // Act
        let fault = unavailable.record_fault(FaultReason::ReadFailed);

        // Assert
        assert!(unavailable.maybe_last_good().is_none());
        assert!(fault.maybe_last_good().is_none());
        assert!(unavailable
            .mark_stale(StaleReason::ProducerTimeout)
            .is_err());
    }

    #[test]
    fn sample_order_is_scoped_to_one_boot_session() {
        // Arrange
        let earlier = *fresh(10, 7, 0)
            .maybe_last_good()
            .expect("fresh observation has a sample");
        let later = *fresh(11, 7, 1)
            .maybe_last_good()
            .expect("fresh observation has a sample");
        let other_session = *fresh(12, 8, 99)
            .maybe_last_good()
            .expect("fresh observation has a sample");

        // Act / Assert
        assert_eq!(earlier.source_order(&later), SampleOrder::Earlier);
        assert_eq!(later.source_order(&earlier), SampleOrder::Later);
        assert_eq!(earlier.source_order(&earlier), SampleOrder::Same);
        assert_eq!(
            earlier.source_order(&other_session),
            SampleOrder::DifferentSession
        );
    }

    #[test]
    fn observation_reason_enums_use_stable_snake_case_labels() {
        // Arrange
        let stale = StaleReason::ProducerCadenceExpired;
        let unavailable = UnavailableReason::NotYetObserved;
        let fault = FaultReason::ReadFailed;

        // Act
        let serialized = json!({
            "stale": stale,
            "unavailable": unavailable,
            "fault": fault,
        });

        // Assert
        assert_eq!(serialized["stale"], "producer_cadence_expired");
        assert_eq!(serialized["unavailable"], "not_yet_observed");
        assert_eq!(serialized["fault"], "read_failed");
    }
}
