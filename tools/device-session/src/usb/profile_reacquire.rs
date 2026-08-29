use super::*;
use crate::usb_ownership::{ProfileTransition, ProfileTransitionDecision, ProfileTransitionSample};
use crate::{inspect_usb_profile, UsbProfile};

impl UsbSession {
    pub(crate) fn reacquire_profile(
        &mut self,
        expected_profile: UsbProfile,
    ) -> Result<(), UsbSessionError> {
        let phase = RecoveryPhase::Handoff;
        let timeout = phase.timeout();
        let deadline = Instant::now() + timeout;
        let mut tracker = RecoveryTracker::new(phase, timeout);
        let mut transition = ProfileTransition::new(expected_profile);

        while Instant::now() < deadline {
            let maybe_snapshot =
                match MacOsDeviceAdapter::maybe_physical_snapshot(&self.physical_identity_digest) {
                    Ok(snapshot) => snapshot,
                    Err(error) => {
                        let decision = transition.observe(ProfileTransitionSample::Ambiguous);
                        let ProfileTransitionDecision::Failed(category) = decision else {
                            unreachable!("ambiguous profile transition must fail");
                        };
                        self.fail_once(category);
                        let detail = error.to_string();
                        return Err(self.recovery_error_with_summary(&tracker, category, &detail));
                    }
                };
            let Some(snapshot) = maybe_snapshot else {
                tracker.observe(RecoverySample::absent());
                let _decision = transition.observe(ProfileTransitionSample::Absent);
                thread::sleep(SAMPLE_INTERVAL);
                continue;
            };
            let profile = inspect_usb_profile(&snapshot.port)
                .map(|inspection| inspection.profile)
                .unwrap_or(UsbProfile::Unknown);
            let stability_key = format!("{}\n{}", snapshot.port, snapshot.enumeration_token);
            let physical_identity_matches =
                snapshot.physical_identity_digest == self.physical_identity_digest;
            let transition_candidate = profile == expected_profile
                && physical_identity_matches
                && snapshot.accessible
                && snapshot.holder_count == 0;
            if transition_candidate {
                tracker.observe(RecoverySample {
                    same_device: true,
                    accessible: true,
                    holder_free: true,
                    enumeration_changed: snapshot.enumeration_token
                        != self.current_enumeration_token,
                    maybe_stability_key: Some(stability_key.clone()),
                });
            } else {
                tracker.observe(RecoverySample::absent());
            }
            let decision = transition.observe(ProfileTransitionSample::Candidate {
                profile,
                physical_identity_matches,
                accessible: snapshot.accessible,
                holder_count: snapshot.holder_count,
                stability_key: stability_key.clone(),
            });
            if let ProfileTransitionDecision::Failed(category) = decision {
                self.fail_once(category);
                return Err(self.recovery_error_with_summary(
                    &tracker,
                    category,
                    "the USB handoff profile transition was rejected",
                ));
            }
            if decision == ProfileTransitionDecision::Pending {
                thread::sleep(SAMPLE_INTERVAL);
                continue;
            }
            self.write_recovery_summary(&tracker.summary())?;
            self.current_port = snapshot.port;
            self.current_enumeration_token = snapshot.enumeration_token;
            return Ok(());
        }
        let ProfileTransitionDecision::Failed(category) = transition.timeout() else {
            unreachable!("profile transition timeout must fail");
        };
        self.fail_once(category);
        Err(self.recovery_error_with_summary(
            &tracker,
            category,
            "the admitted physical device did not reappear in the ROM USB profile",
        ))
    }
}
