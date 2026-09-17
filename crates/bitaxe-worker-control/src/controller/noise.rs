use super::*;
use crate::noise::{NoiseDetail, NoiseQuery, NoiseStart, NoiseState};

impl<V: LeaseAuthorizationVerifier, S: WorkerSession> WorkerControl<V, S> {
    pub(super) fn prepare_noise(
        &mut self,
        request: &ControllerRequest,
        now: u64,
    ) -> Result<PreparedResponse, WorkerControlError> {
        if !self.authenticated_logical_session || self.maybe_pending_admission_token.is_some() {
            return Err(WorkerControlError::AdmissionRequired);
        }
        if request.command == "noise_diagnostic_start" {
            self.required_start_context(now)?;
            if self.maybe_active.is_some()
                || self.effect_cleanup_required
                || self.boot_restoration_clear_required
                || self.session.noise_busy()
                || self.session.v2_scope_busy()
                || matches!(self.restoration, RestorationState::Pending)
            {
                return Err(WorkerControlError::InvalidTransition);
            }
            let input: NoiseStart = request.required_payload()?;
            if !input.valid() {
                return Err(WorkerControlError::InvalidRequest);
            }
            let previous = self
                .maybe_noise_observation
                .as_ref()
                .ok_or(WorkerControlError::AdmissionRequired)?;
            let current = self
                .session
                .noise_observation()
                .map_err(|_| WorkerControlError::SessionFailed)?
                .ok_or(WorkerControlError::InvalidTransition)?;
            if input.network_observed_at_us != previous.observed_at_us
                || input.expected_boot_ordinal != current.boot_ordinal
                || current.boot_ordinal != previous.boot_ordinal
                || current.worker_generation != previous.worker_generation
                || current.transport_epoch != previous.transport_epoch
                || !current.wifi_connected
                || current.station_ipv4.is_none()
                || current.station_ipv4 != previous.station_ipv4
                || current
                    .observed_at_us
                    .checked_sub(previous.observed_at_us)
                    .is_none_or(|age| age > 5_000_000)
            {
                return Err(WorkerControlError::InvalidTransition);
            }
            let status = self
                .session
                .noise_admit(input)
                .map_err(|_| WorkerControlError::SessionFailed)?;
            self.maybe_noise_generation = Some(self.generation);
            self.maybe_noise_observation = None;
            return response(
                &request.request_id,
                serde_json::to_value(status).map_err(|_| WorkerControlError::Encoding)?,
                Some(PreparedEffect::NoiseDispatch {
                    generation: self.generation,
                }),
            );
        }
        let query: NoiseQuery = request.required_payload()?;
        let retained_binding = self.maybe_noise_generation == Some(self.generation);
        if retained_binding {
            self.required_active_context()?;
        } else {
            self.required_start_context(now)?;
        }
        if query
            .attempt_id
            .as_ref()
            .is_some_and(|id| crate::noise::canonical_bytes::<16>(id).is_none())
        {
            return Err(WorkerControlError::InvalidRequest);
        }
        let status = self
            .session
            .noise_status()
            .map_err(|_| WorkerControlError::SessionFailed)?
            .ok_or(WorkerControlError::InvalidRequest)?;
        match (query.attempt_id.as_deref(), status.job.as_ref()) {
            (None, None)
                if request.command == "noise_diagnostic_status"
                    && status.state == NoiseState::Idle =>
            {
                self.required_start_context(now)?;
                let effect = PreparedEffect::NoiseObservation {
                    generation: self.generation,
                    observation: status.observation.clone(),
                };
                response(
                    &request.request_id,
                    serde_json::to_value(status).map_err(|_| WorkerControlError::Encoding)?,
                    Some(effect),
                )
            }
            (Some(id), Some(job)) if id == job.attempt_id => {
                if request.command == "noise_diagnostic_cancel" {
                    self.session
                        .noise_cancel(NoiseDetail::CancelRequested)
                        .map_err(|_| WorkerControlError::SessionFailed)?;
                }
                let status = self
                    .session
                    .noise_status()
                    .map_err(|_| WorkerControlError::SessionFailed)?
                    .ok_or(WorkerControlError::SessionFailed)?;
                response(
                    &request.request_id,
                    serde_json::to_value(status).map_err(|_| WorkerControlError::Encoding)?,
                    None,
                )
            }
            _ => Err(WorkerControlError::InvalidTransition),
        }
    }

    pub(super) fn confirm_noise_dispatch(
        &mut self,
        mut response: PreparedResponse,
        now: u64,
    ) -> Result<(), WorkerControlError> {
        let Some(PreparedEffect::NoiseDispatch { generation }) = response.maybe_effect.take()
        else {
            return Err(WorkerControlError::StaleResponse);
        };
        let result = (|| {
            self.enforce_clock(now)?;
            self.required_start_context(now)?;
            if generation != self.generation
                || self.maybe_noise_generation != Some(generation)
                || self.maybe_pending_admission_token.is_some()
                || self.maybe_active.is_some()
                || self.effect_cleanup_required
                || !self.authenticated_logical_session
            {
                return Err(WorkerControlError::StaleResponse);
            }
            self.session
                .noise_dispatch()
                .map_err(|_| WorkerControlError::SessionFailed)
        })();
        if result.is_err() {
            self.session
                .noise_cancel(NoiseDetail::DeliveryAmbiguous)
                .map_err(|_| WorkerControlError::SessionFailed)?;
        }
        result
    }
}
