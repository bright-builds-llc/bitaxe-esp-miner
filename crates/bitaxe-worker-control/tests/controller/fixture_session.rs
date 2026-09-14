use super::*;

impl WorkerSession for FakeSession {
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
        _grant: &WorkerLeaseGrant,
        _deadlines: LeaseDeadlines,
    ) -> Result<(), WorkerSessionError> {
        self.events.push("start");
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
        if self.remaining_safe_stop_failures > 0 {
            self.remaining_safe_stop_failures -= 1;
            Err(WorkerSessionError::SafeStopFailed)
        } else {
            Ok(())
        }
    }
}
