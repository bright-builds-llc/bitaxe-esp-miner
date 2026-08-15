//! Privacy-safe timing facts for the shared operator sensor owner.
#![cfg_attr(test, allow(dead_code))]

use std::sync::{Mutex, OnceLock};

const PRESSURE_THRESHOLD_MS: u64 = 100;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum OperatorSensorStage {
    Power,
    AsicTemperature,
    Tachometer,
    CoreVoltage,
    Display,
    Actuation,
}

impl OperatorSensorStage {
    pub(crate) const fn label(self) -> &'static str {
        match self {
            Self::Power => "power",
            Self::AsicTemperature => "asic_temperature",
            Self::Tachometer => "tachometer",
            Self::CoreVoltage => "core_voltage",
            Self::Display => "display",
            Self::Actuation => "actuation",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum OperatorSensorOutcome {
    Ready,
    Recovered,
    DriverFailed,
    BudgetExhausted,
    SampleInvalid,
    Unavailable,
}

impl OperatorSensorOutcome {
    pub(crate) const fn label(self) -> &'static str {
        match self {
            Self::Ready => "ready",
            Self::Recovered => "recovered",
            Self::DriverFailed => "driver_failed",
            Self::BudgetExhausted => "budget_exhausted",
            Self::SampleInvalid => "sample_invalid",
            Self::Unavailable => "unavailable",
        }
    }

    const fn is_pressure(self) -> bool {
        !matches!(self, Self::Ready)
    }

    const fn severity(self) -> u8 {
        match self {
            Self::Ready => 0,
            Self::Recovered => 1,
            Self::Unavailable => 2,
            Self::SampleInvalid => 3,
            Self::DriverFailed => 4,
            Self::BudgetExhausted => 5,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum OperatorSensorDurationBucket {
    Under100Ms,
    Under250Ms,
    Under500Ms,
    Under1000Ms,
    AtLeast1000Ms,
}

impl OperatorSensorDurationBucket {
    const fn from_duration_ms(duration_ms: u64) -> Self {
        match duration_ms {
            0..=99 => Self::Under100Ms,
            100..=249 => Self::Under250Ms,
            250..=499 => Self::Under500Ms,
            500..=999 => Self::Under1000Ms,
            _ => Self::AtLeast1000Ms,
        }
    }

    pub(crate) const fn label(self) -> &'static str {
        match self {
            Self::Under100Ms => "under_100_ms",
            Self::Under250Ms => "under_250_ms",
            Self::Under500Ms => "under_500_ms",
            Self::Under1000Ms => "under_1000_ms",
            Self::AtLeast1000Ms => "at_least_1000_ms",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct OperatorSensorDiagnostic {
    boot_session: u64,
    revision: u64,
    stage: OperatorSensorStage,
    outcome: OperatorSensorOutcome,
    duration_bucket: OperatorSensorDurationBucket,
}

impl OperatorSensorDiagnostic {
    pub(crate) const fn boot_session(self) -> u64 {
        self.boot_session
    }

    pub(crate) const fn revision(self) -> u64 {
        self.revision
    }

    pub(crate) const fn stage(self) -> OperatorSensorStage {
        self.stage
    }

    pub(crate) const fn outcome(self) -> OperatorSensorOutcome {
        self.outcome
    }

    pub(crate) const fn duration_bucket(self) -> OperatorSensorDurationBucket {
        self.duration_bucket
    }

    pub(crate) fn marker(self) -> String {
        format!(
            "operator_sensor_diagnostic schema=operator-sensor-diagnostic-v1 boot_session={} revision={} stage={} outcome={} duration_bucket={} redacted=true",
            self.boot_session,
            self.revision,
            self.stage.label(),
            self.outcome.label(),
            self.duration_bucket.label(),
        )
    }
}

#[derive(Debug)]
pub(crate) struct OperatorSensorDiagnosticTracker {
    boot_session: u64,
    next_revision: u64,
    maybe_latest_pressure: Option<OperatorSensorDiagnostic>,
}

impl OperatorSensorDiagnosticTracker {
    pub(crate) const fn new(boot_session: u64) -> Self {
        Self {
            boot_session,
            next_revision: 1,
            maybe_latest_pressure: None,
        }
    }

    pub(crate) fn observe(
        &mut self,
        stage: OperatorSensorStage,
        started_at_ms: u64,
        completed_at_ms: u64,
        outcome: OperatorSensorOutcome,
    ) -> Option<OperatorSensorDiagnostic> {
        let duration_ms = completed_at_ms.saturating_sub(started_at_ms);
        if duration_ms < PRESSURE_THRESHOLD_MS && !outcome.is_pressure() {
            return None;
        }
        let diagnostic = OperatorSensorDiagnostic {
            boot_session: self.boot_session,
            revision: self.next_revision,
            stage,
            outcome,
            duration_bucket: OperatorSensorDurationBucket::from_duration_ms(duration_ms),
        };
        self.next_revision = self.next_revision.saturating_add(1);
        let replace_retained = self
            .maybe_latest_pressure
            .map(|current| diagnostic.outcome.severity() > current.outcome.severity())
            .unwrap_or(true);
        if replace_retained {
            // Keep the first event at the highest severity so a later display
            // flush cannot hide the stage that made the sweep unactionable.
            self.maybe_latest_pressure = Some(diagnostic);
        }
        Some(diagnostic)
    }

    pub(crate) const fn maybe_latest_pressure(&self) -> Option<OperatorSensorDiagnostic> {
        self.maybe_latest_pressure
    }
}

static TRACKER: OnceLock<Mutex<OperatorSensorDiagnosticTracker>> = OnceLock::new();

pub(crate) fn initialize(boot_session: u64) -> bool {
    TRACKER
        .set(Mutex::new(OperatorSensorDiagnosticTracker::new(
            boot_session,
        )))
        .is_ok()
}

pub(crate) fn record_stage(
    stage: OperatorSensorStage,
    started_at_ms: u64,
    completed_at_ms: u64,
    outcome: OperatorSensorOutcome,
) -> Option<OperatorSensorDiagnostic> {
    let tracker = TRACKER.get()?;
    let Ok(mut tracker) = tracker.lock() else {
        return None;
    };
    tracker.observe(stage, started_at_ms, completed_at_ms, outcome)
}

pub(crate) fn maybe_latest_pressure() -> Option<OperatorSensorDiagnostic> {
    let tracker = TRACKER.get()?;
    let Ok(tracker) = tracker.lock() else {
        return None;
    };
    tracker.maybe_latest_pressure()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fast_ready_stage_does_not_replace_pressure_evidence() {
        // Arrange
        let mut tracker = OperatorSensorDiagnosticTracker::new(7);
        let pressure = tracker
            .observe(
                OperatorSensorStage::Display,
                100,
                400,
                OperatorSensorOutcome::Ready,
            )
            .expect("slow stage should be retained");

        // Act
        let fast = tracker.observe(
            OperatorSensorStage::Power,
            500,
            510,
            OperatorSensorOutcome::Ready,
        );

        // Assert
        assert_eq!(fast, None);
        assert_eq!(tracker.maybe_latest_pressure(), Some(pressure));
    }

    #[test]
    fn failed_stage_is_retained_even_when_fast() {
        // Arrange
        let mut tracker = OperatorSensorDiagnosticTracker::new(9);

        // Act
        let diagnostic = tracker
            .observe(
                OperatorSensorStage::Power,
                200,
                210,
                OperatorSensorOutcome::BudgetExhausted,
            )
            .expect("failed stage should be retained");

        // Assert
        assert_eq!(diagnostic.boot_session(), 9);
        assert_eq!(diagnostic.revision(), 1);
        assert_eq!(diagnostic.stage(), OperatorSensorStage::Power);
        assert_eq!(
            diagnostic.duration_bucket(),
            OperatorSensorDurationBucket::Under100Ms
        );
    }

    #[test]
    fn lower_severity_pressure_cannot_replace_an_actionable_failure() {
        // Arrange
        let mut tracker = OperatorSensorDiagnosticTracker::new(10);
        let failure = tracker
            .observe(
                OperatorSensorStage::Power,
                100,
                500,
                OperatorSensorOutcome::BudgetExhausted,
            )
            .expect("budget exhaustion should be emitted");

        // Act
        let later = tracker
            .observe(
                OperatorSensorStage::Display,
                600,
                750,
                OperatorSensorOutcome::Ready,
            )
            .expect("slow display pressure should still be emitted");

        // Assert
        assert_eq!(later.revision(), 2);
        assert_eq!(tracker.maybe_latest_pressure(), Some(failure));
    }

    #[test]
    fn higher_severity_pressure_replaces_the_retained_event() {
        // Arrange
        let mut tracker = OperatorSensorDiagnosticTracker::new(12);
        tracker
            .observe(
                OperatorSensorStage::Display,
                100,
                250,
                OperatorSensorOutcome::Ready,
            )
            .expect("slow display pressure should be emitted");

        // Act
        let failure = tracker
            .observe(
                OperatorSensorStage::Tachometer,
                300,
                310,
                OperatorSensorOutcome::DriverFailed,
            )
            .expect("driver failure should be emitted");

        // Assert
        assert_eq!(tracker.maybe_latest_pressure(), Some(failure));
    }

    #[test]
    fn equal_severity_keeps_the_earliest_stage() {
        // Arrange
        let mut tracker = OperatorSensorDiagnosticTracker::new(13);
        let first = tracker
            .observe(
                OperatorSensorStage::Power,
                100,
                500,
                OperatorSensorOutcome::BudgetExhausted,
            )
            .expect("first exhaustion should be emitted");

        // Act
        let second = tracker
            .observe(
                OperatorSensorStage::AsicTemperature,
                600,
                700,
                OperatorSensorOutcome::BudgetExhausted,
            )
            .expect("second exhaustion should still be emitted");

        // Assert
        assert_eq!(second.revision(), 2);
        assert_eq!(tracker.maybe_latest_pressure(), Some(first));
    }

    #[test]
    fn marker_contains_only_closed_redaction_safe_fields() {
        // Arrange
        let mut tracker = OperatorSensorDiagnosticTracker::new(11);
        let diagnostic = tracker
            .observe(
                OperatorSensorStage::Actuation,
                0,
                1_200,
                OperatorSensorOutcome::DriverFailed,
            )
            .expect("failed stage should be retained");

        // Act
        let marker = diagnostic.marker();

        // Assert
        assert_eq!(
            marker,
            "operator_sensor_diagnostic schema=operator-sensor-diagnostic-v1 boot_session=11 revision=1 stage=actuation outcome=driver_failed duration_bucket=at_least_1000_ms redacted=true"
        );
        assert!(!marker.contains("temperature="));
        assert!(!marker.contains("voltage="));
        assert!(!marker.contains("port="));
    }
}
