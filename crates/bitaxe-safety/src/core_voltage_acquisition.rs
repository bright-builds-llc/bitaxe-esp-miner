//! Pure reduction for calibrated ADC core-voltage observations.
//!
//! Reference breadcrumbs:
//! - `reference/esp-miner/main/adc.c`
//! - `reference/esp-miner/main/power/vcore.c:VCORE_get_voltage_mv`
//! - `reference/esp-miner/main/tasks/power_management_task.c`

use crate::{
    observation::{
        BootSessionId, FaultReason, MonotonicMillis, Observation, ObservationSequence,
        SequenceOverflow, StaleReason, UnavailableReason,
    },
    sensor_acquisition::AcquisitionOutcome,
};

pub const MODULE_NAME: &str = "core-voltage-acquisition";

/// Producer-owned ADC observation and its next successful sequence lineage.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CoreVoltageProducerState {
    observation: Observation<u16>,
    sequence: ObservationSequence,
}

impl Default for CoreVoltageProducerState {
    fn default() -> Self {
        Self {
            observation: Observation::unavailable(UnavailableReason::NotYetObserved),
            sequence: ObservationSequence::ZERO,
        }
    }
}

impl CoreVoltageProducerState {
    /// Returns calibrated core voltage in millivolts with producer truth.
    #[must_use]
    pub const fn observation(&self) -> &Observation<u16> {
        &self.observation
    }

    /// Reduces one completed ADC acquisition without reading a clock or hardware.
    pub fn record(
        self,
        outcome: AcquisitionOutcome<u16>,
        boot_session: BootSessionId,
        acquired_at: MonotonicMillis,
    ) -> Result<Self, SequenceOverflow> {
        let (observation, sequence) = match outcome {
            AcquisitionOutcome::Success(millivolts) => {
                Observation::record_success(millivolts, boot_session, self.sequence, acquired_at)?
            }
            AcquisitionOutcome::Unavailable(reason) => {
                (Observation::unavailable(reason), self.sequence)
            }
            AcquisitionOutcome::ReadFailed => (
                self.observation.record_fault(FaultReason::AdcReadFailed),
                self.sequence,
            ),
            AcquisitionOutcome::InvalidSample => (
                self.observation
                    .record_fault(FaultReason::CoreVoltageReadingInvalid),
                self.sequence,
            ),
        };

        Ok(Self {
            observation,
            sequence,
        })
    }

    /// Marks an expired last-good ADC sample stale without changing its stamp.
    #[must_use]
    pub fn mark_stale_at(self, now: MonotonicMillis, stale_after_ms: u64) -> Self {
        let expired = self.observation.maybe_last_good().is_some_and(|sample| {
            now.get().saturating_sub(sample.acquired_at().get()) > stale_after_ms
        });
        if !expired {
            return self;
        }

        let observation = self
            .observation
            .mark_stale(StaleReason::ProducerCadenceExpired)
            .unwrap_or(self.observation);
        Self {
            observation,
            ..self
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const SESSION: BootSessionId = BootSessionId::new(7);

    #[test]
    fn successful_zero_is_fresh_truth_and_advances_once() {
        // Arrange
        let state = CoreVoltageProducerState::default();

        // Act
        let state = state
            .record(
                AcquisitionOutcome::Success(0),
                SESSION,
                MonotonicMillis::new(250),
            )
            .expect("first ADC sequence should advance");

        // Assert
        let sample = state
            .observation()
            .maybe_last_good()
            .expect("successful zero is a real sample");
        assert!(state.observation().is_fresh());
        assert_eq!(*sample.value(), 0);
        assert_eq!(sample.sequence(), ObservationSequence::new(1));
    }

    #[test]
    fn read_failure_preserves_last_good_and_sequence() {
        // Arrange
        let fresh = CoreVoltageProducerState::default()
            .record(
                AcquisitionOutcome::Success(1_198),
                SESSION,
                MonotonicMillis::new(250),
            )
            .expect("first ADC sequence should advance");
        let expected = *fresh
            .observation()
            .maybe_last_good()
            .expect("fresh ADC sample");

        // Act
        let faulted = fresh
            .record(
                AcquisitionOutcome::ReadFailed,
                SESSION,
                MonotonicMillis::new(750),
            )
            .expect("failure does not advance a sequence");
        let recovered = faulted
            .record(
                AcquisitionOutcome::Success(1_201),
                SESSION,
                MonotonicMillis::new(1_250),
            )
            .expect("next success should advance");

        // Assert
        assert_eq!(
            faulted.observation().maybe_reason(),
            Some("adc_read_failed")
        );
        assert_eq!(faulted.observation().maybe_last_good(), Some(&expected));
        assert_eq!(
            recovered
                .observation()
                .maybe_last_good()
                .map(|sample| sample.sequence()),
            Some(ObservationSequence::new(2))
        );
    }

    #[test]
    fn unavailable_and_invalid_outcomes_remain_distinct() {
        // Arrange
        let state = CoreVoltageProducerState::default();

        // Act
        let unavailable = state
            .record(
                AcquisitionOutcome::Unavailable(UnavailableReason::CoreVoltageUnavailable),
                SESSION,
                MonotonicMillis::new(250),
            )
            .expect("unavailable does not advance a sequence");
        let invalid = state
            .record(
                AcquisitionOutcome::InvalidSample,
                SESSION,
                MonotonicMillis::new(250),
            )
            .expect("invalid does not advance a sequence");

        // Assert
        assert_eq!(
            unavailable.observation().maybe_reason(),
            Some("core_voltage_unavailable")
        );
        assert_eq!(
            invalid.observation().maybe_reason(),
            Some("core_voltage_reading_invalid")
        );
    }

    #[test]
    fn stale_transition_preserves_stamp_and_exact_boundary() {
        // Arrange
        let fresh = CoreVoltageProducerState::default()
            .record(
                AcquisitionOutcome::Success(1_198),
                SESSION,
                MonotonicMillis::new(250),
            )
            .expect("first ADC sequence should advance");
        let expected = *fresh
            .observation()
            .maybe_last_good()
            .expect("fresh ADC sample");

        // Act
        let boundary = fresh.mark_stale_at(MonotonicMillis::new(1_250), 1_000);
        let stale = fresh.mark_stale_at(MonotonicMillis::new(1_251), 1_000);

        // Assert
        assert!(boundary.observation().is_fresh());
        assert_eq!(
            stale.observation().maybe_reason(),
            Some("producer_cadence_expired")
        );
        assert_eq!(stale.observation().maybe_last_good(), Some(&expected));
    }
}
