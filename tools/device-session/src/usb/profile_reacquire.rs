use super::*;
use crate::macos::PhysicalSnapshotObservation;
use crate::usb_ownership::{
    profile_observation_category, ProfileObservationCategory, ProfileObservationTrace,
    ProfileTransition, ProfileTransitionDecision, ProfileTransitionSample,
};
use crate::{inspect_usb_profile, UsbProfile};

impl UsbSession {
    pub(crate) fn reacquire_application_transport(
        &mut self,
    ) -> Result<(UsbProfile, bool), UsbSessionError> {
        let previous_enumeration = self.current_enumeration_token.clone();
        let snapshot = self.reacquire(RecoveryPhase::Handoff)?;
        let profile = inspect_usb_profile(&snapshot.port)
            .map(|inspection| inspection.profile)
            .map_err(|error| UsbSessionError {
                category: UsbTerminalCategory::RuntimeProfileUnknown,
                detail: error.to_string(),
            })?;
        if !matches!(
            profile,
            UsbProfile::WorkerRuntime | UsbProfile::SerialJtagRuntime
        ) {
            return Err(UsbSessionError {
                category: UsbTerminalCategory::RuntimeProfileUnknown,
                detail: "ROM exit did not expose an application-capable transport".to_owned(),
            });
        }
        Ok((profile, snapshot.enumeration_token != previous_enumeration))
    }

    pub(crate) fn reacquire_profile(
        &mut self,
        expected_profile: UsbProfile,
    ) -> Result<crate::usb_ownership::ProfileObservationCounts, UsbSessionError> {
        let phase = RecoveryPhase::Handoff;
        let timeout = phase.timeout();
        let deadline = Instant::now() + timeout;
        let mut tracker = RecoveryTracker::new(phase, timeout);
        let mut transition = ProfileTransition::new(expected_profile);
        let mut observation_trace = ProfileObservationTrace::new(expected_profile);

        while Instant::now() < deadline {
            let observation = match MacOsDeviceAdapter::profile_transition_snapshot(
                &self.physical_identity_digest,
                self.port(),
            ) {
                Ok(observation) => observation,
                Err(error) => {
                    let decision = transition.observe(ProfileTransitionSample::Ambiguous);
                    let ProfileTransitionDecision::Failed(category) = decision else {
                        unreachable!("ambiguous profile transition must fail");
                    };
                    self.fail_once(category);
                    let detail = error.to_string();
                    return Err(self.profile_transition_error(
                        &tracker,
                        &observation_trace,
                        category,
                        &detail,
                    ));
                }
            };
            let snapshot = match observation {
                PhysicalSnapshotObservation::Absent => {
                    observation_trace.observe(ProfileObservationCategory::Absent);
                    tracker.observe(RecoverySample::absent());
                    let _decision = transition.observe(ProfileTransitionSample::Absent);
                    thread::sleep(SAMPLE_INTERVAL);
                    continue;
                }
                PhysicalSnapshotObservation::PhysicalMismatch => {
                    observation_trace.observe(ProfileObservationCategory::PhysicalMismatch);
                    let category = UsbTerminalCategory::PhysicalIdentityDrift;
                    self.fail_once(category);
                    return Err(self.profile_transition_error(
                        &tracker,
                        &observation_trace,
                        category,
                        "the selected device node changed physical identity",
                    ));
                }
                PhysicalSnapshotObservation::Match(snapshot) => snapshot,
            };
            let profile = inspect_usb_profile(&snapshot.port)
                .map(|inspection| inspection.profile)
                .unwrap_or(UsbProfile::Unknown);
            observation_trace.observe(profile_observation_category(Some(profile), true));
            let stability_key = format!("{}\n{}", snapshot.port, snapshot.enumeration_token);
            let physical_identity_matches =
                snapshot.physical_identity_digest == self.physical_identity_digest;
            tracker.observe(RecoverySample {
                same_device: physical_identity_matches,
                accessible: snapshot.accessible,
                holder_free: snapshot.holder_count == 0,
                enumeration_changed: snapshot.enumeration_token != self.current_enumeration_token,
                maybe_stability_key: physical_identity_matches.then(|| stability_key.clone()),
            });
            let decision = transition.observe(ProfileTransitionSample::Candidate {
                profile,
                physical_identity_matches,
                accessible: snapshot.accessible,
                holder_count: snapshot.holder_count,
                stability_key: stability_key.clone(),
            });
            if let ProfileTransitionDecision::Failed(category) = decision {
                self.fail_once(category);
                return Err(self.profile_transition_error(
                    &tracker,
                    &observation_trace,
                    category,
                    "the USB handoff profile transition was rejected",
                ));
            }
            if decision == ProfileTransitionDecision::Pending {
                thread::sleep(SAMPLE_INTERVAL);
                continue;
            }
            self.write_recovery_summary(&tracker.summary())?;
            self.write_profile_observation_trace(&observation_trace)?;
            let counts = observation_trace.counts();
            self.record_profile_observation_counts(counts);
            self.current_port = snapshot.port;
            self.current_enumeration_token = snapshot.enumeration_token;
            return Ok(counts);
        }
        let ProfileTransitionDecision::Failed(category) = transition.timeout() else {
            unreachable!("profile transition timeout must fail");
        };
        let category = profile_timeout_category(expected_profile, &observation_trace, category);
        self.fail_once(category);
        Err(self.profile_transition_error(
            &tracker,
            &observation_trace,
            category,
            "the admitted physical device did not reappear in the ROM USB profile",
        ))
    }

    fn profile_transition_error(
        &mut self,
        tracker: &RecoveryTracker,
        trace: &ProfileObservationTrace,
        category: UsbTerminalCategory,
        detail: &str,
    ) -> UsbSessionError {
        self.record_profile_observation_counts(trace.counts());
        let profile_trace_recorded = self.write_profile_observation_trace(trace).is_ok();
        self.recovery_error_with_summary(
            tracker,
            category,
            &format!("{detail} profile_trace_recorded={profile_trace_recorded}"),
        )
    }
}

fn profile_timeout_category(
    expected_profile: UsbProfile,
    trace: &ProfileObservationTrace,
    fallback: UsbTerminalCategory,
) -> UsbTerminalCategory {
    if expected_profile == UsbProfile::WorkerRuntime {
        return UsbTerminalCategory::ApplicationReappearanceTimeout;
    }
    let counts = trace.counts();
    if counts.same_worker > 0 {
        UsbTerminalCategory::SameWorkerAfterCommit
    } else if counts.absent > 0 {
        UsbTerminalCategory::BusResetTimeout
    } else {
        fallback
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::usb_ownership::ProfileObservationCategory;

    #[test]
    fn worker_observed_after_commit_has_a_specific_terminal_category() {
        // Arrange
        let mut trace = ProfileObservationTrace::new(UsbProfile::SerialJtagRuntime);
        trace.observe(ProfileObservationCategory::SameWorker);

        // Act
        let category = profile_timeout_category(
            UsbProfile::SerialJtagRuntime,
            &trace,
            UsbTerminalCategory::HandoffTransitionTimeout,
        );

        // Assert
        assert_eq!(category, UsbTerminalCategory::SameWorkerAfterCommit);
    }

    #[test]
    fn missing_serial_jtag_after_detach_has_a_bus_reset_terminal_category() {
        // Arrange
        let mut trace = ProfileObservationTrace::new(UsbProfile::SerialJtagRuntime);
        trace.observe(ProfileObservationCategory::Absent);

        // Act
        let category = profile_timeout_category(
            UsbProfile::SerialJtagRuntime,
            &trace,
            UsbTerminalCategory::HandoffTransitionTimeout,
        );

        // Assert
        assert_eq!(category, UsbTerminalCategory::BusResetTimeout);
    }

    #[test]
    fn missing_worker_after_rom_admission_has_an_application_terminal_category() {
        // Arrange
        let trace = ProfileObservationTrace::new(UsbProfile::WorkerRuntime);

        // Act
        let category = profile_timeout_category(
            UsbProfile::WorkerRuntime,
            &trace,
            UsbTerminalCategory::HandoffTransitionTimeout,
        );

        // Assert
        assert_eq!(
            category,
            UsbTerminalCategory::ApplicationReappearanceTimeout
        );
    }
}
