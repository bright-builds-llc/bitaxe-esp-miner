use super::*;

impl WorkerSession for FakeSession {
    fn v2_status(
        &self,
        scope: bitaxe_worker_control::v2::Scope,
    ) -> Result<Option<bitaxe_worker_control::v2::V2Status>, WorkerSessionError> {
        let record = self.maybe_v2.as_ref().map(|r| r.snapshot());
        Ok(Some(bitaxe_worker_control::v2::V2Status {
            schema: "worker-stratum-v2-status-v1",
            scope,
            state: record
                .as_ref()
                .map_or(bitaxe_worker_control::v2::State::Idle, |r| r.state),
            observation: v2::observation(),
            maybe_connection: None,
            maybe_record: record,
        }))
    }
    fn v2_admit(
        &mut self,
        input: bitaxe_worker_control::v2::ChannelStart,
    ) -> Result<bitaxe_worker_control::v2::V2Status, WorkerSessionError> {
        if self.maybe_v2.is_some() || self.maybe_noise.is_some() {
            return Err(WorkerSessionError::Rejected);
        }
        self.maybe_v2 = bitaxe_worker_control::v2::V2Record::admit(
            bitaxe_worker_control::v2::Scope::Channel,
            input.attempt_id,
            &v2::observation(),
            Some(121_000_000),
        );
        self.events.push("v2_admitted");
        self.v2_status(bitaxe_worker_control::v2::Scope::Channel)?
            .ok_or(WorkerSessionError::Rejected)
    }
    fn v2_dispatch(&mut self) -> Result<(), WorkerSessionError> {
        self.events.push("v2_dispatched");
        Ok(())
    }
    fn v2_cancel(&mut self) -> Result<(), WorkerSessionError> {
        if let Some(r) = self.maybe_v2.as_mut() {
            r.fail(
                bitaxe_worker_control::v2::Stage::Revoked,
                bitaxe_worker_control::v2::FailureCategory::Authority,
                Some(1_000_003),
            );
        }
        Ok(())
    }
    fn noise_observation(
        &self,
    ) -> Result<Option<bitaxe_worker_control::noise::NoiseObservation>, WorkerSessionError> {
        Ok(Some(noise::observation()))
    }
    fn noise_status(
        &self,
    ) -> Result<Option<bitaxe_worker_control::noise::NoiseStatus>, WorkerSessionError> {
        Ok(Some(self.maybe_noise.as_ref().map_or_else(
            || bitaxe_worker_control::noise::NoiseStatus {
                schema: "worker-noise-diagnostic-status-v2",
                state: bitaxe_worker_control::noise::NoiseState::Idle,
                observation: noise::observation(),
                job: None,
            },
            |record| record.status(noise::observation()),
        )))
    }
    fn noise_admit(
        &mut self,
        input: bitaxe_worker_control::noise::NoiseStart,
    ) -> Result<bitaxe_worker_control::noise::NoiseStatus, WorkerSessionError> {
        if self.maybe_noise.is_some() {
            return Err(WorkerSessionError::Rejected);
        }
        self.maybe_noise = bitaxe_worker_control::noise::NoiseRecord::admit(
            &input,
            &noise::observation(),
            input.input_sha256().ok_or(WorkerSessionError::Rejected)?,
        );
        self.events.push("noise_admitted");
        self.noise_status()?.ok_or(WorkerSessionError::Rejected)
    }
    fn noise_dispatch(&mut self) -> Result<(), WorkerSessionError> {
        if !self
            .maybe_noise
            .as_mut()
            .is_some_and(|record| record.dispatch(1_000_001))
        {
            return Err(WorkerSessionError::Rejected);
        }
        self.events.push("noise_dispatched");
        Ok(())
    }
    fn noise_cancel(
        &mut self,
        detail: bitaxe_worker_control::noise::NoiseDetail,
    ) -> Result<(), WorkerSessionError> {
        if let Some(record) = self.maybe_noise.as_mut() {
            record.fail(bitaxe_worker_control::noise::NoiseFailure::new(
                bitaxe_worker_control::noise::FailureStage::Cleanup,
                bitaxe_worker_control::noise::NoiseCategory::AuthorityLost,
                detail,
                Some(1_000_002),
            ));
        }
        Ok(())
    }
    fn v2_scope_busy(&self) -> bool {
        self.v2_cleanup_blocked
    }
    fn noise_busy(&self) -> bool {
        self.maybe_v2.as_ref().is_some_and(|r| {
            r.scope() == bitaxe_worker_control::v2::Scope::Channel
                && r.snapshot().resources.fence_retained
        }) || self
            .maybe_noise
            .as_ref()
            .is_some_and(bitaxe_worker_control::noise::NoiseRecord::active)
    }
    fn qualification_restart_context(
        &self,
    ) -> Result<Option<bitaxe_worker_control::QualificationRestartContext>, WorkerSessionError>
    {
        Ok(self.maybe_restart_context)
    }
    fn qualification_restart(
        &mut self,
        context: bitaxe_worker_control::QualificationRestartContext,
        _expires_at_ms: u64,
    ) -> Result<(), WorkerSessionError> {
        if self.maybe_restart_context != Some(context) {
            return Err(WorkerSessionError::Rejected);
        }
        self.events.push("qualification_restart");
        Ok(())
    }
    fn telemetry_cadence_probe_prepared(&self, request_bytes: usize, response_bytes: usize) {
        self.prepared_probes
            .borrow_mut()
            .push((request_bytes, response_bytes));
    }
    fn serial_trace_review(
        &self,
    ) -> Result<Option<bitaxe_worker_control::serial::trace::SerialTraceSnapshot>, WorkerSessionError>
    {
        Ok(Some(
            bitaxe_worker_control::serial::trace::SerialTrace::new().snapshot(),
        ))
    }
    fn telemetry_cadence_review(
        &self,
    ) -> Result<Option<bitaxe_worker_control::cadence::CadenceSnapshot>, WorkerSessionError> {
        Ok(Some(
            bitaxe_worker_control::cadence::CadenceRecorder::new().snapshot(),
        ))
    }
    fn telemetry_cadence_arm(
        &mut self,
        phase: bitaxe_worker_control::cadence::CadencePhase,
    ) -> Result<Option<bitaxe_worker_control::cadence::CadenceArmReceipt>, WorkerSessionError> {
        self.events.push("cadence_arm");
        Ok(Some(bitaxe_worker_control::cadence::CadenceArmReceipt {
            schema: "worker-telemetry-cadence-arm-v1",
            phase,
            armed_at_us: 100,
            generation: 7,
        }))
    }
    fn telemetry_cadence_endpoint(
        &self,
    ) -> Result<Option<bitaxe_worker_control::cadence::CadenceEndpoint>, WorkerSessionError> {
        Ok(Some(bitaxe_worker_control::cadence::CadenceEndpoint {
            schema: "worker-telemetry-endpoint-v1",
            ipv4: "192.0.2.1".to_owned(),
            http_port: 80,
            observed_at_us: 100,
            boot_ordinal: 2,
            generation: 7,
        }))
    }
    fn qualify_cooling(&mut self) -> Result<serde_json::Value, WorkerSessionError> {
        self.events.push("cooling");
        if self.fail_cooling {
            return Err(WorkerSessionError::Rejected);
        }
        Ok(
            json!({"schema":"worker-cooling-proof-v1","fan_duty_percent":100,"fan_rpm":1200,
            "post_command_fan_proven":true,"asic_effects":false,"budget_reserved":false}),
        )
    }
    fn restore_cooling(&mut self) -> Result<serde_json::Value, WorkerSessionError> {
        self.events.push("restore_cooling");
        Ok(
            json!({"schema":"worker-cooling-baseline-v1","fan_duty_percent":30,
            "cooling_proven":true,"asic_effects":false,"budget_reserved":false}),
        )
    }
    fn qualification_attempt_review(
        &self,
    ) -> Result<Option<serde_json::Value>, WorkerSessionError> {
        Ok(Some(
            json!({"schema":"worker-qualification-ledger-v1","next_ordinal":1,"total_charged_ms":0,"pending":false,"last_completed_ordinal":0}),
        ))
    }
    fn acceptance_budget_review(
        &self,
        expected: &str,
    ) -> Result<Option<serde_json::Value>, WorkerSessionError> {
        Ok(Some(
            json!({"schema":"worker-budget-review-v1", "campaign_match": expected == URL_SAFE_NO_PAD.encode([7_u8;16]),
            "reserved_mask":1,"completed_mask":1,"charged_ms":180000,"pending":false}),
        ))
    }
    fn start(
        &mut self,
        grant: &WorkerLeaseGrant,
        _deadlines: LeaseDeadlines,
    ) -> Result<(), WorkerSessionError> {
        self.events.push("start");
        if grant.maybe_v2().is_some() {
            self.maybe_v2 = bitaxe_worker_control::v2::V2Record::admit(
                bitaxe_worker_control::v2::Scope::Share,
                grant
                    .maybe_qualification_attempt()
                    .expect("validated allowance")
                    .id()
                    .to_owned(),
                &v2::observation(),
                None,
            );
        }
        if self.fail_start {
            Err(WorkerSessionError::Rejected)
        } else {
            Ok(())
        }
    }

    fn renew(
        &mut self,
        _renewal: &WorkerLeaseRenewal,
        _deadlines: LeaseDeadlines,
    ) -> Result<(), WorkerSessionError> {
        self.events.push("renew");
        Ok(())
    }

    fn safe_stop(&mut self, reason: RestorationReason) -> Result<(), WorkerSessionError> {
        self.events.push(reason.category());
        if self.v2_cleanup_blocked {
            return Err(WorkerSessionError::SafeStopFailed);
        }
        if self.remaining_safe_stop_failures > 0 {
            self.remaining_safe_stop_failures -= 1;
            Err(WorkerSessionError::SafeStopFailed)
        } else {
            Ok(())
        }
    }
}
