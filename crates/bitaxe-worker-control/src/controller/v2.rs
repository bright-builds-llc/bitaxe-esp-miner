use super::*;
use crate::v2::{ChannelCancel, ChannelStart, Scope, State, V2Query};

impl<V: LeaseAuthorizationVerifier, S: WorkerSession> WorkerControl<V, S> {
    pub(super) fn pending_v2_cleanup(&self) -> bool {
        self.session.v2_scope_busy() && self.maybe_cleanup_reason.is_some()
    }
    pub(super) fn prepare_v2(
        &mut self,
        request: &ControllerRequest,
        now: u64,
    ) -> Result<PreparedResponse, WorkerControlError> {
        if !self.authenticated_logical_session || self.maybe_pending_admission_token.is_some() {
            return Err(WorkerControlError::AdmissionRequired);
        }
        if request.command == "stratum_v2_channel_start" {
            return self.admit_v2(request, now);
        }
        // The retained binding authorizes only observation/cancellation, never Start.
        if self.maybe_v2_generation != Some(self.generation) {
            self.required_active_context()?;
        }
        let (scope, maybe_id, cancel) = if request.command == "stratum_v2_channel_cancel" {
            let input: ChannelCancel = request.required_payload()?;
            (Scope::Channel, Some(input.attempt_id), true)
        } else {
            let query: V2Query = request.required_payload()?;
            (query.scope, query.maybe_attempt_id, false)
        };
        if maybe_id
            .as_ref()
            .is_some_and(|id| crate::noise::canonical_bytes::<16>(id).is_none())
        {
            return Err(WorkerControlError::InvalidRequest);
        }
        let status = self
            .session
            .v2_status(scope)
            .map_err(|_| WorkerControlError::SessionFailed)?
            .ok_or(WorkerControlError::InvalidRequest)?;
        match (maybe_id.as_deref(), status.maybe_record.as_ref()) {
            (None, None) if !cancel && status.state == State::Idle => {
                self.required_start_context(now)?;
                let effect = PreparedEffect::V2Observation {
                    generation: self.generation,
                    scope,
                    observation: status.observation.clone(),
                };
                response(
                    &request.request_id,
                    serde_json::to_value(status).map_err(|_| WorkerControlError::Encoding)?,
                    Some(effect),
                )
            }
            (Some(id), Some(record)) if id == record.attempt_id => {
                self.maybe_v2_generation = Some(self.generation);
                if cancel && status.state != State::Terminal {
                    self.session
                        .v2_cancel()
                        .map_err(|_| WorkerControlError::SessionFailed)?;
                }
                let status = self
                    .session
                    .v2_status(scope)
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
    fn admit_v2(
        &mut self,
        request: &ControllerRequest,
        now: u64,
    ) -> Result<PreparedResponse, WorkerControlError> {
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
        let input: ChannelStart = request.required_payload()?;
        if !input.valid() {
            return Err(WorkerControlError::InvalidRequest);
        }
        let (scope, previous) = self
            .maybe_v2_observation
            .as_ref()
            .ok_or(WorkerControlError::AdmissionRequired)?;
        let current = self
            .session
            .v2_status(Scope::Channel)
            .map_err(|_| WorkerControlError::SessionFailed)?
            .ok_or(WorkerControlError::InvalidTransition)?
            .observation;
        if *scope != Scope::Channel
            || !current.clock_valid
            || !previous.clock_valid
            || !current.wifi_connected
            || current.maybe_station_ipv4.is_none()
            || current.maybe_station_ipv4 != previous.maybe_station_ipv4
            || current.boot_ordinal != input.expected_boot_ordinal
            || current.boot_ordinal != previous.boot_ordinal
            || current.worker_generation != previous.worker_generation
            || current.serial_transport_epoch != previous.serial_transport_epoch
            || previous.maybe_observed_at_us != Some(input.network_observed_at_us)
            || current
                .maybe_observed_at_us
                .and_then(|t| t.checked_sub(input.network_observed_at_us))
                .is_none_or(|age| age > 5_000_000)
        {
            return Err(WorkerControlError::InvalidTransition);
        }
        let status = self
            .session
            .v2_admit(input)
            .map_err(|_| WorkerControlError::SessionFailed)?;
        self.maybe_v2_generation = Some(self.generation);
        self.maybe_v2_observation = None;
        response(
            &request.request_id,
            serde_json::to_value(status).map_err(|_| WorkerControlError::Encoding)?,
            Some(PreparedEffect::V2Dispatch {
                generation: self.generation,
            }),
        )
    }
    pub(super) fn validate_v2_share_observation(&mut self) -> Result<(), WorkerControlError> {
        let (scope, previous) = self
            .maybe_v2_observation
            .as_ref()
            .ok_or(WorkerControlError::AdmissionRequired)?;
        let current = self
            .session
            .v2_status(Scope::Share)
            .map_err(|_| WorkerControlError::SessionFailed)?
            .ok_or(WorkerControlError::InvalidTransition)?;
        if *scope != Scope::Share
            || current.state != State::Idle
            || !current.observation.clock_valid
            || !current.observation.wifi_connected
            || current.observation.maybe_station_ipv4.is_none()
            || current.observation.maybe_station_ipv4 != previous.maybe_station_ipv4
            || current.observation.boot_ordinal != previous.boot_ordinal
            || current.observation.worker_generation != previous.worker_generation
            || current.observation.serial_transport_epoch != previous.serial_transport_epoch
            || current
                .observation
                .maybe_observed_at_us
                .zip(previous.maybe_observed_at_us)
                .and_then(|(a, b)| a.checked_sub(b))
                .is_none_or(|age| age > 5_000_000)
        {
            return Err(WorkerControlError::InvalidTransition);
        }
        self.maybe_v2_observation = None;
        Ok(())
    }
    pub(super) fn confirm_v2_dispatch(
        &mut self,
        mut response: PreparedResponse,
        now: u64,
    ) -> Result<(), WorkerControlError> {
        let Some(PreparedEffect::V2Dispatch { generation }) = response.maybe_effect.take() else {
            return Err(WorkerControlError::StaleResponse);
        };
        let result = (|| {
            self.enforce_clock(now)?;
            self.required_start_context(now)?;
            if generation != self.generation
                || self.maybe_v2_generation != Some(generation)
                || self.maybe_pending_admission_token.is_some()
                || self.maybe_active.is_some()
                || self.effect_cleanup_required
                || !self.authenticated_logical_session
            {
                return Err(WorkerControlError::StaleResponse);
            }
            self.session
                .v2_dispatch()
                .map_err(|_| WorkerControlError::SessionFailed)
        })();
        if result.is_err() {
            self.session
                .v2_cancel()
                .map_err(|_| WorkerControlError::SessionFailed)?;
        }
        result
    }
}
