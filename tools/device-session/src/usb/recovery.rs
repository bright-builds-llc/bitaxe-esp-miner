use std::time::Duration;

use serde::Serialize;

pub(super) const REQUIRED_STABLE_SAMPLES: u8 = 3;
pub(super) const STANDARD_RECOVERY_TIMEOUT: Duration = Duration::from_secs(30);
pub(super) const EXTENDED_RECOVERY_TIMEOUT: Duration = Duration::from_secs(60);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub(super) enum RecoveryPhase {
    PostFlash,
    PostProbe,
    RetryAdmission,
    MonitorAdmission,
    FinalCleanup,
}

impl RecoveryPhase {
    pub(super) const fn timeout(self) -> Duration {
        match self {
            Self::PostFlash | Self::MonitorAdmission | Self::FinalCleanup => {
                EXTENDED_RECOVERY_TIMEOUT
            }
            Self::PostProbe | Self::RetryAdmission => STANDARD_RECOVERY_TIMEOUT,
        }
    }

    pub(super) const fn as_str(self) -> &'static str {
        match self {
            Self::PostFlash => "post_flash",
            Self::PostProbe => "post_probe",
            Self::RetryAdmission => "retry_admission",
            Self::MonitorAdmission => "monitor_admission",
            Self::FinalCleanup => "final_cleanup",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub(super) enum RecoveryFinalState {
    Absent,
    Inaccessible,
    Stabilizing,
}

impl RecoveryFinalState {
    const fn as_str(self) -> &'static str {
        match self {
            Self::Absent => "absent",
            Self::Inaccessible => "inaccessible",
            Self::Stabilizing => "stabilizing",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct RecoverySample {
    pub(super) same_device: bool,
    pub(super) accessible: bool,
    pub(super) holder_free: bool,
    pub(super) enumeration_changed: bool,
    pub(super) maybe_stability_key: Option<String>,
}

impl RecoverySample {
    pub(super) fn absent() -> Self {
        Self {
            same_device: false,
            accessible: false,
            holder_free: false,
            enumeration_changed: false,
            maybe_stability_key: None,
        }
    }

    #[cfg(test)]
    fn accessible(stability_key: &str, enumeration_changed: bool) -> Self {
        Self {
            same_device: true,
            accessible: true,
            holder_free: true,
            enumeration_changed,
            maybe_stability_key: Some(stability_key.to_owned()),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(super) struct RecoverySummary {
    pub(super) phase: RecoveryPhase,
    pub(super) deadline_seconds: u64,
    pub(super) same_device_seen: bool,
    pub(super) accessible_seen: bool,
    pub(super) holder_free_seen: bool,
    pub(super) stable_samples_max: u8,
    pub(super) enumeration_changed: bool,
    pub(super) final_state: RecoveryFinalState,
}

impl RecoverySummary {
    pub(super) fn safe_signature(&self) -> String {
        format!(
            "phase={},deadline_seconds={},same_device_seen={},accessible_seen={},\
             holder_free_seen={},stable_samples_max={},enumeration_changed={},final_state={}",
            self.phase.as_str(),
            self.deadline_seconds,
            self.same_device_seen,
            self.accessible_seen,
            self.holder_free_seen,
            self.stable_samples_max,
            self.enumeration_changed,
            self.final_state.as_str(),
        )
    }
}

#[derive(Debug)]
pub(super) struct RecoveryTracker {
    phase: RecoveryPhase,
    deadline: Duration,
    stable_samples: u8,
    maybe_previous_stability_key: Option<String>,
    same_device_seen: bool,
    accessible_seen: bool,
    holder_free_seen: bool,
    stable_samples_max: u8,
    enumeration_changed: bool,
    final_state: RecoveryFinalState,
}

impl RecoveryTracker {
    pub(super) fn new(phase: RecoveryPhase, deadline: Duration) -> Self {
        Self {
            phase,
            deadline,
            stable_samples: 0,
            maybe_previous_stability_key: None,
            same_device_seen: false,
            accessible_seen: false,
            holder_free_seen: false,
            stable_samples_max: 0,
            enumeration_changed: false,
            final_state: RecoveryFinalState::Absent,
        }
    }

    pub(super) fn observe(&mut self, sample: RecoverySample) -> bool {
        self.same_device_seen |= sample.same_device;
        self.accessible_seen |= sample.same_device && sample.accessible;
        self.holder_free_seen |= sample.same_device && sample.holder_free;
        self.enumeration_changed |= sample.enumeration_changed;

        if !sample.same_device {
            self.reset_stability();
            self.final_state = RecoveryFinalState::Absent;
            return false;
        }
        if !sample.accessible || !sample.holder_free {
            self.reset_stability();
            self.final_state = RecoveryFinalState::Inaccessible;
            return false;
        }

        let same_sample = self
            .maybe_previous_stability_key
            .as_ref()
            .zip(sample.maybe_stability_key.as_ref())
            .is_some_and(|(previous, current)| previous == current);
        self.stable_samples = if same_sample {
            self.stable_samples.saturating_add(1)
        } else {
            1
        };
        self.maybe_previous_stability_key = sample.maybe_stability_key;
        self.stable_samples_max = self.stable_samples_max.max(self.stable_samples);
        self.final_state = RecoveryFinalState::Stabilizing;
        self.stable_samples >= REQUIRED_STABLE_SAMPLES
    }

    pub(super) fn summary(&self) -> RecoverySummary {
        RecoverySummary {
            phase: self.phase,
            deadline_seconds: self.deadline.as_secs(),
            same_device_seen: self.same_device_seen,
            accessible_seen: self.accessible_seen,
            holder_free_seen: self.holder_free_seen,
            stable_samples_max: self.stable_samples_max.min(REQUIRED_STABLE_SAMPLES),
            enumeration_changed: self.enumeration_changed,
            final_state: self.final_state,
        }
    }

    fn reset_stability(&mut self) {
        self.stable_samples = 0;
        self.maybe_previous_stability_key = None;
    }
}

#[cfg(test)]
fn reduce_virtual_timeline(
    phase: RecoveryPhase,
    deadline: Duration,
    samples: Vec<(Duration, RecoverySample)>,
) -> (bool, RecoverySummary) {
    let mut tracker = RecoveryTracker::new(phase, deadline);
    for (observed_at, sample) in samples {
        if observed_at >= deadline {
            break;
        }
        if tracker.observe(sample) {
            return (true, tracker.summary());
        }
    }
    (false, tracker.summary())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn available_sample(stability_key: &str, enumeration_changed: bool) -> RecoverySample {
        RecoverySample::accessible(stability_key, enumeration_changed)
    }

    #[test]
    fn attempt_002_delayed_recovery_requires_the_post_flash_window() {
        // Arrange
        let samples = vec![
            (Duration::from_secs(29), RecoverySample::absent()),
            (
                Duration::from_millis(30_150),
                RecoverySample::accessible("epoch-b", true),
            ),
            (
                Duration::from_millis(30_300),
                RecoverySample::accessible("epoch-b", true),
            ),
            (
                Duration::from_millis(30_450),
                RecoverySample::accessible("epoch-b", true),
            ),
        ];

        // Act
        let old_policy = reduce_virtual_timeline(
            RecoveryPhase::PostFlash,
            STANDARD_RECOVERY_TIMEOUT,
            samples.clone(),
        );
        let production_policy = reduce_virtual_timeline(
            RecoveryPhase::PostFlash,
            RecoveryPhase::PostFlash.timeout(),
            samples,
        );

        // Assert
        assert!(!old_policy.0);
        assert!(production_policy.0);
    }

    #[test]
    fn slow_sampler_requires_the_extended_final_cleanup_window() {
        // Arrange
        let samples = vec![
            (
                Duration::from_secs(12),
                RecoverySample::accessible("epoch-a", false),
            ),
            (
                Duration::from_secs(24),
                RecoverySample::accessible("epoch-a", false),
            ),
            (
                Duration::from_secs(36),
                RecoverySample::accessible("epoch-a", false),
            ),
        ];

        // Act
        let standard_policy = reduce_virtual_timeline(
            RecoveryPhase::FinalCleanup,
            STANDARD_RECOVERY_TIMEOUT,
            samples.clone(),
        );
        let final_cleanup_policy = reduce_virtual_timeline(
            RecoveryPhase::FinalCleanup,
            RecoveryPhase::FinalCleanup.timeout(),
            samples,
        );

        // Assert
        assert!(!standard_policy.0);
        assert_eq!(standard_policy.1.stable_samples_max, 2);
        assert!(final_cleanup_policy.0);
        assert_eq!(final_cleanup_policy.1.stable_samples_max, 3);
    }

    #[test]
    fn slow_sampler_requires_the_extended_monitor_admission_window() {
        // Arrange
        let samples = vec![
            (
                Duration::from_secs(12),
                RecoverySample::accessible("epoch-a", false),
            ),
            (
                Duration::from_secs(24),
                RecoverySample::accessible("epoch-a", false),
            ),
            (
                Duration::from_secs(36),
                RecoverySample::accessible("epoch-a", false),
            ),
        ];

        // Act
        let standard_policy = reduce_virtual_timeline(
            RecoveryPhase::MonitorAdmission,
            STANDARD_RECOVERY_TIMEOUT,
            samples.clone(),
        );
        let production_policy = reduce_virtual_timeline(
            RecoveryPhase::MonitorAdmission,
            RecoveryPhase::MonitorAdmission.timeout(),
            samples,
        );

        // Assert
        assert!(!standard_policy.0);
        assert_eq!(standard_policy.1.stable_samples_max, 2);
        assert!(production_policy.0);
        assert_eq!(production_policy.1.stable_samples_max, 3);
    }

    #[test]
    fn every_recovery_phase_has_the_intended_timeout() {
        // Arrange
        let cases = [
            (RecoveryPhase::PostFlash, EXTENDED_RECOVERY_TIMEOUT),
            (RecoveryPhase::PostProbe, STANDARD_RECOVERY_TIMEOUT),
            (RecoveryPhase::RetryAdmission, STANDARD_RECOVERY_TIMEOUT),
            (RecoveryPhase::MonitorAdmission, EXTENDED_RECOVERY_TIMEOUT),
            (RecoveryPhase::FinalCleanup, EXTENDED_RECOVERY_TIMEOUT),
        ];

        for (phase, expected_timeout) in cases {
            // Act
            let timeout = phase.timeout();

            // Assert
            assert_eq!(timeout, expected_timeout, "{phase:?}");
        }
    }

    #[test]
    fn immediate_recovery_requires_exactly_three_stable_samples() {
        // Arrange
        let mut tracker =
            RecoveryTracker::new(RecoveryPhase::PostFlash, RecoveryPhase::PostFlash.timeout());

        // Act
        let first = tracker.observe(available_sample("epoch-a", false));
        let second = tracker.observe(available_sample("epoch-a", false));
        let third = tracker.observe(available_sample("epoch-a", false));

        // Assert
        assert!(!first);
        assert!(!second);
        assert!(third);
        assert_eq!(tracker.summary().stable_samples_max, 3);
    }

    #[test]
    fn absent_recovery_produces_a_bounded_summary() {
        // Arrange
        let mut tracker =
            RecoveryTracker::new(RecoveryPhase::PostFlash, RecoveryPhase::PostFlash.timeout());

        // Act
        let ready = tracker.observe(RecoverySample::absent());
        let summary = tracker.summary();

        // Assert
        assert!(!ready);
        assert!(!summary.same_device_seen);
        assert_eq!(summary.final_state, RecoveryFinalState::Absent);
    }

    #[test]
    fn inaccessible_sample_resets_stability() {
        // Arrange
        let mut tracker =
            RecoveryTracker::new(RecoveryPhase::FinalCleanup, STANDARD_RECOVERY_TIMEOUT);
        tracker.observe(available_sample("epoch-a", false));
        tracker.observe(available_sample("epoch-a", false));
        let inaccessible = RecoverySample {
            same_device: true,
            accessible: false,
            holder_free: true,
            enumeration_changed: false,
            maybe_stability_key: Some("epoch-a".to_owned()),
        };

        // Act
        let ready = tracker.observe(inaccessible);
        let summary = tracker.summary();

        // Assert
        assert!(!ready);
        assert!(summary.same_device_seen);
        assert!(summary.accessible_seen);
        assert_eq!(summary.stable_samples_max, 2);
        assert_eq!(summary.final_state, RecoveryFinalState::Inaccessible);
    }

    #[test]
    fn changing_enumeration_never_accumulates_stable_samples() {
        // Arrange
        let mut tracker =
            RecoveryTracker::new(RecoveryPhase::PostFlash, RecoveryPhase::PostFlash.timeout());

        // Act
        let observations = ["epoch-a", "epoch-b", "epoch-c"]
            .into_iter()
            .map(|key| tracker.observe(available_sample(key, true)))
            .collect::<Vec<_>>();
        let summary = tracker.summary();

        // Assert
        assert!(observations.into_iter().all(|ready| !ready));
        assert_eq!(summary.stable_samples_max, 1);
        assert!(summary.enumeration_changed);
        assert_eq!(summary.final_state, RecoveryFinalState::Stabilizing);
    }

    #[test]
    fn safe_signature_contains_only_bounded_recovery_fields() {
        // Arrange
        let mut tracker =
            RecoveryTracker::new(RecoveryPhase::PostFlash, RecoveryPhase::PostFlash.timeout());
        tracker.observe(available_sample("private-stability-key", true));
        let summary = tracker.summary();

        // Act
        let signature = summary.safe_signature();

        // Assert
        assert_eq!(
            signature,
            "phase=post_flash,deadline_seconds=60,same_device_seen=true,\
             accessible_seen=true,holder_free_seen=true,stable_samples_max=1,\
             enumeration_changed=true,final_state=stabilizing"
        );
        assert!(!signature.contains("private-stability-key"));
        assert!(!signature.contains("/dev/"));
    }

    #[test]
    fn sample_at_the_deadline_is_not_admitted() {
        // Arrange
        let samples = vec![
            (
                Duration::from_millis(59_700),
                available_sample("epoch-b", true),
            ),
            (
                Duration::from_millis(59_850),
                available_sample("epoch-b", true),
            ),
            (
                RecoveryPhase::PostFlash.timeout(),
                available_sample("epoch-b", true),
            ),
        ];

        // Act
        let (ready, summary) = reduce_virtual_timeline(
            RecoveryPhase::PostFlash,
            RecoveryPhase::PostFlash.timeout(),
            samples,
        );

        // Assert
        assert!(!ready);
        assert_eq!(summary.stable_samples_max, 2);
    }
}
