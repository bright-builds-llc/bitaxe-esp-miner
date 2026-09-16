use super::*;

impl<V: LeaseAuthorizationVerifier, S: WorkerSession> WorkerControl<V, S> {
    pub fn confirm_sent(
        &mut self,
        mut response: PreparedResponse,
    ) -> Result<(), WorkerControlError> {
        let Some(effect) = response.maybe_effect.take() else {
            return Ok(());
        };
        match effect {
            PreparedEffect::NoiseObservation {
                generation,
                observation,
            } => {
                if generation != self.generation {
                    return Err(WorkerControlError::StaleResponse);
                }
                self.maybe_noise_observation = Some(observation);
            }
            PreparedEffect::NoiseDispatch { .. } => return Err(WorkerControlError::StaleResponse),
            PreparedEffect::QualificationRestart { .. } => {
                return Err(WorkerControlError::StaleResponse)
            }
            PreparedEffect::Admit {
                generation,
                token,
                established_at_monotonic_milliseconds,
                control_session_binding_sha256,
            } => {
                if generation != self.generation
                    || self.maybe_pending_admission_token != Some(token)
                {
                    return Err(WorkerControlError::StaleResponse);
                }
                self.maybe_pending_admission_token = None;
                self.authenticated_logical_session = true;
                self.maybe_admission = Some(LogicalSessionAdmission {
                    generation,
                    established_at_monotonic_milliseconds,
                    context: WorkerLeaseAuthorizationContext::parse(
                        &control_session_binding_sha256,
                    )
                    .map_err(|_| WorkerControlError::InvalidProof)?,
                });
            }
            PreparedEffect::BootRestorationReported { generation } => {
                if generation != self.generation || !self.boot_restoration_clear_required {
                    return Err(WorkerControlError::StaleResponse);
                }
                self.maybe_boot_restoration_report_generation = Some(generation);
            }
        }
        Ok(())
    }
}
