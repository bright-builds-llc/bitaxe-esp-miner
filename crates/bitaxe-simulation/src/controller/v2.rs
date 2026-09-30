//! Controller-owned encrypted functional share with exact native-gate boundaries.
use super::Session;
use crate::{v2::ProfileIoEvent, SimulationError};
use bitaxe_worker_control::v2::{CurrentObservation, Scope, Stage, V2Record};
use bitaxe_worker_control::{WorkerLeaseGrant, WorkerSessionError};

impl Session {
    pub(super) fn v2_observation(&self) -> CurrentObservation {
        CurrentObservation {
            boot_ordinal: self.actuation.board.borrow().storage.boot_ordinal,
            worker_generation: u64::from(self.actuation.generation.raw()),
            serial_transport_epoch: 1,
            maybe_observed_at_us: Some(self.actuation.now_ms() * 1000),
            clock_valid: true,
            maybe_station_ipv4: Some("192.168.0.1".into()),
            wifi_connected: self.actuation.board.borrow().network.connected,
            maybe_socket: None,
        }
    }
    pub(super) fn admit_v2_record(
        &mut self,
        grant: &WorkerLeaseGrant,
    ) -> Result<(), WorkerSessionError> {
        let attempt = grant
            .maybe_qualification_attempt()
            .ok_or(WorkerSessionError::Rejected)?;
        let record = V2Record::admit(
            Scope::Share,
            attempt.id().into(),
            &self.v2_observation(),
            None,
        )
        .ok_or(WorkerSessionError::Rejected)?;
        let mut record = record;
        record.event(
            Stage::Preparing,
            Some(self.actuation.now_ms() * 1000),
            None,
            None,
            None,
            None,
        );
        *self.maybe_v2_record.borrow_mut() = Some(record);
        Ok(())
    }
    pub(super) fn run_encrypted_share(
        &mut self,
        grant: &WorkerLeaseGrant,
    ) -> Result<(), WorkerSessionError> {
        let terms = grant.maybe_v2().ok_or(WorkerSessionError::Rejected)?;
        let endpoint = format!(
            "stratum+tcp://{}:{}/",
            crate::v2::SYNTHETIC_ENDPOINT_HOST,
            crate::v2::SYNTHETIC_ENDPOINT_PORT
        );
        let authority = base64::Engine::encode(
            &base64::engine::general_purpose::URL_SAFE_NO_PAD,
            crate::v2::synthetic_authority_public_key(),
        );
        if terms.endpoint.as_str() != endpoint
            || terms.user_identity.as_str() != crate::v2::SYNTHETIC_USER_IDENTITY
            || terms.authority_public_key.as_str() != authority
        {
            return Err(WorkerSessionError::Rejected);
        }
        let gate = &self.actuation.gate;
        let generation = self.actuation.generation;
        let record = self.maybe_v2_record.clone();
        let facts = crate::v2::run_encrypted_share_on_board_with_io(
            self.seed,
            &mut self.actuation.board.borrow_mut(),
            |now| {
                gate.check_deadline(now);
                if gate.permits(Some(generation)) {
                    Ok(())
                } else {
                    Err(SimulationError::Boundary("generation_revoked"))
                }
            },
            |now| {
                let (admitted, maybe_arming) =
                    gate.begin_dispatch_observed(gate.stamp(Some(generation)), now);
                if !admitted {
                    return Err(SimulationError::Boundary("dispatch_revoked"));
                }
                if let Some((_, epoch, limit)) = maybe_arming {
                    if !record
                        .borrow_mut()
                        .as_mut()
                        .is_some_and(|value| value.budget_armed(epoch, limit, Some(now * 1000)))
                    {
                        return Err(SimulationError::Boundary("budget_arming_record"));
                    }
                }
                Ok(())
            },
            |event, now| {
                let stage = match event {
                    ProfileIoEvent::AsicWriteCompleted => {
                        gate.note_io(Some(generation), false);
                        Stage::AsicDispatch
                    }
                    ProfileIoEvent::ShareWriteCompleted => {
                        gate.note_io(Some(generation), true);
                        Stage::Submission
                    }
                    ProfileIoEvent::AcknowledgementValidated => {
                        gate.publish_counts(generation, 1, 0, 1);
                        Stage::Accepted
                    }
                };
                if let Some(value) = record.borrow_mut().as_mut() {
                    value.event(stage, Some(now * 1000), None, None, None, None);
                }
            },
        )
        .map_err(|_| WorkerSessionError::Rejected)?;
        if facts.acknowledged_shares != 1
            || !facts.independently_target_valid
            || !facts.resources_released
        {
            return Err(WorkerSessionError::Rejected);
        }
        if let Some(value) = self.maybe_v2_record.borrow_mut().as_mut() {
            value.set_job_commitment(facts.job_commitment.clone());
            if !value.add_share(facts.share_fact.clone()) {
                return Err(WorkerSessionError::Rejected);
            }
            value.socket_closed(Some(self.actuation.now_ms() * 1000));
            value.joined(Some(self.actuation.now_ms() * 1000), false);
        }
        self.maybe_encrypted_share = Some(facts);
        Ok(())
    }
}
