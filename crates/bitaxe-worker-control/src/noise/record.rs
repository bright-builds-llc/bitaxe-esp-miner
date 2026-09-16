use super::*;

/// One consumed boot slot; only the supervisor may acknowledge worker quiescence.
pub struct NoiseRecord {
    state: NoiseState,
    job: NoiseJob,
    last_us: u64,
    clock_valid: bool,
}
impl NoiseRecord {
    pub fn admit(
        input: &NoiseStart,
        observation: &NoiseObservation,
        input_sha256: String,
    ) -> Option<Self> {
        let now = observation.observed_at_us;
        let authority_deadline_us = now.checked_add(AUTHORITY_US)?;
        if authority_deadline_us.checked_add(OBSERVATION_TAIL_US)? > MAX_SAFE_INTEGER {
            return None;
        }
        Some(Self {
            state: NoiseState::Admitted,
            last_us: now,
            clock_valid: true,
            job: NoiseJob {
                attempt_id: input.attempt_id.clone(),
                input_sha256,
                boot_ordinal: observation.boot_ordinal,
                worker_generation: observation.worker_generation,
                transport_epoch: observation.transport_epoch,
                admitted_at_us: now,
                authority_deadline_us,
                local_socket_port: None,
                stages: Vec::with_capacity(8),
                first_failure: None,
                terminal: None,
                resources: NoiseRelease {
                    socket_state: SocketState::NotCreated,
                    // The reserved ordinary owner already exists, even before dispatch.
                    worker_state: WorkerState::Running,
                    volatile_inputs_disposed: false,
                    started_at_us: None,
                    deadline_at_us: None,
                    released_at_us: None,
                    deadline_met: None,
                    failure: None,
                },
            },
        })
    }
    pub fn status(&self, observation: NoiseObservation) -> NoiseStatus {
        NoiseStatus {
            schema: "worker-noise-diagnostic-status-v2",
            state: self.state,
            observation,
            job: Some(self.job.clone()),
        }
    }
    pub fn job(&self) -> &NoiseJob {
        &self.job
    }
    pub fn active(&self) -> bool {
        self.job.resources.worker_state != WorkerState::Quiescent
            || self.job.resources.socket_state == SocketState::Open
    }
    pub fn permitted(&self) -> bool {
        self.clock_valid && self.job.first_failure.is_none() && self.job.terminal.is_none()
    }
    /// A third clock fault after a frozen failure cannot be invented into its schema.
    pub fn reportable(&self) -> bool {
        self.clock_valid
            || [self.job.first_failure, self.job.resources.failure]
                .iter()
                .flatten()
                .any(|failure| failure.category == NoiseCategory::ClockInvalid)
    }
    pub fn dispatch(&mut self, now: u64) -> bool {
        self.tick(now);
        if self.state != NoiseState::Admitted || !self.permitted() {
            return false;
        }
        self.state = NoiseState::Running;
        true
    }
    pub fn fail(&mut self, failure: NoiseFailure) {
        if failure.detail == NoiseDetail::ClockDiscontinuity {
            self.clock_valid = false;
            if self.job.terminal.is_none() && self.job.resources.failure.is_none() {
                self.job.resources.failure = Some(NoiseFailure::new(
                    FailureStage::Cleanup,
                    NoiseCategory::ClockInvalid,
                    NoiseDetail::ClockDiscontinuity,
                    None,
                ));
            }
        }
        if self.job.terminal.is_some() {
            return;
        }
        if self.job.first_failure.is_none() {
            self.job.first_failure = Some(failure);
        }
        self.state = NoiseState::Cancelling;
        self.cleanup(failure.at_us);
    }
    pub fn cleanup(&mut self, now: Option<u64>) {
        if self.job.resources.deadline_at_us.is_some() {
            return;
        }
        self.job.resources.started_at_us = now;
        self.job.resources.deadline_at_us =
            Some(self.job.authority_deadline_us + OBSERVATION_TAIL_US);
        if self.job.terminal.is_none() && self.job.first_failure.is_some() {
            self.state = NoiseState::Cancelling;
        }
    }
    pub fn tick(&mut self, now: u64) {
        if now < self.last_us || now > MAX_SAFE_INTEGER {
            self.clock_valid = false;
            self.fail(NoiseFailure::new(
                FailureStage::Evidence,
                NoiseCategory::ClockInvalid,
                NoiseDetail::ClockDiscontinuity,
                None,
            ));
        } else {
            self.last_us = now;
        }
        if now >= self.job.authority_deadline_us && self.job.terminal.is_none() {
            self.fail(NoiseFailure::new(
                FailureStage::Cleanup,
                NoiseCategory::AuthorityLost,
                NoiseDetail::Timeout,
                self.clock_valid.then_some(now),
            ));
        }
        if now >= self.job.authority_deadline_us + OBSERVATION_TAIL_US
            && self.job.terminal.is_none()
        {
            let at = self.clock_valid.then_some(now);
            self.job.resources.deadline_met = Some(false);
            self.job.resources.failure.get_or_insert(NoiseFailure::new(
                FailureStage::Cleanup,
                NoiseCategory::Cleanup,
                NoiseDetail::ResourceUnreleased,
                at,
            ));
            self.job.terminal = Some(NoiseTerminal {
                outcome: NoiseOutcome::Incomplete,
                decided_at_us: at,
            });
            self.state = NoiseState::Terminal;
        }
    }
    /// Missing clock evidence closes permission immediately, even during opaque work.
    pub fn observe_clock(&mut self, maybe_now: Option<u64>) -> bool {
        match maybe_now {
            Some(now) => self.tick(now),
            None => self.fail(NoiseFailure::new(
                FailureStage::Evidence,
                NoiseCategory::ClockInvalid,
                NoiseDetail::ClockDiscontinuity,
                None,
            )),
        }
        self.permitted()
    }
    pub fn stage(
        &mut self,
        stage: NoiseStage,
        now: u64,
        duration: u64,
        bytes: Option<u16>,
    ) -> bool {
        // Completion timestamps can precede a concurrent supervisor poll. Compare
        // event order, not the later poll's clock sample, to detect discontinuity.
        let previous = self
            .job
            .stages
            .last()
            .and_then(|event| event.at_us)
            .unwrap_or(self.job.admitted_at_us);
        if now < previous || now > MAX_SAFE_INTEGER {
            self.clock_valid = false;
            self.fail(NoiseFailure::new(
                FailureStage::Evidence,
                NoiseCategory::ClockInvalid,
                NoiseDetail::ClockDiscontinuity,
                None,
            ));
        }
        let cleanup_stage = matches!(
            stage,
            NoiseStage::SocketClosed | NoiseStage::WorkerQuiescent
        );
        if !cleanup_stage && !self.permitted() {
            return false;
        }
        let ordered = [
            NoiseStage::NoisePrepared,
            NoiseStage::TcpConnected,
            NoiseStage::ActOneWritten,
            NoiseStage::ActTwoReceived,
            NoiseStage::AuthorityVerified,
            NoiseStage::ProofWritten,
        ];
        if !cleanup_stage && ordered.get(self.job.stages.len()) != Some(&stage) {
            self.fail(NoiseFailure::new(
                FailureStage::Evidence,
                NoiseCategory::EvidenceIncomplete,
                NoiseDetail::Malformed,
                self.clock_valid.then_some(now),
            ));
            return false;
        }
        if self.job.stages.len() == 8 || self.job.stages.iter().any(|seen| seen.stage == stage) {
            self.fail(NoiseFailure::new(
                FailureStage::Evidence,
                NoiseCategory::EvidenceIncomplete,
                NoiseDetail::Duplicate,
                self.clock_valid.then_some(now),
            ));
            return false;
        }
        self.job.stages.push(NoiseStageObservation {
            stage,
            sequence: (self.job.stages.len() + 1) as u8,
            at_us: self.clock_valid.then_some(now),
            duration_us: self.clock_valid.then_some(duration),
            bytes,
        });
        true
    }
    pub fn socket_opened(&mut self, port: u16) {
        self.job.local_socket_port = Some(port);
        self.job.resources.socket_state = SocketState::Open;
    }
    pub fn socket_closed(&mut self, now: u64, duration: u64) {
        self.job.resources.socket_state = SocketState::Closed;
        self.stage(NoiseStage::SocketClosed, now, duration, None);
    }
    pub fn socket_closed_without_clock(&mut self) {
        self.job.resources.socket_state = SocketState::Closed;
        self.fail(NoiseFailure::new(
            FailureStage::SocketClosed,
            NoiseCategory::ClockInvalid,
            NoiseDetail::ClockDiscontinuity,
            None,
        ));
        if self.reportable() {
            self.stage(NoiseStage::SocketClosed, self.last_us, 0, None);
        }
    }
    pub fn joined(&mut self, now: u64, panicked: bool) {
        self.tick(now);
        if panicked {
            self.fail(NoiseFailure::new(
                FailureStage::Cleanup,
                NoiseCategory::Cleanup,
                NoiseDetail::Io,
                self.clock_valid.then_some(now),
            ));
        }
        if self.job.resources.started_at_us.is_none() {
            self.cleanup(self.clock_valid.then_some(now));
        }
        let duration = self
            .job
            .resources
            .started_at_us
            .map_or(0, |start| now.saturating_sub(start));
        if self.reportable() {
            self.stage(NoiseStage::WorkerQuiescent, now, duration, None);
        }
        self.job.resources.worker_state = WorkerState::Quiescent;
        self.job.resources.volatile_inputs_disposed = true;
        self.job.resources.released_at_us = self.clock_valid.then_some(now);
        if self.job.resources.deadline_met != Some(false) {
            self.job.resources.deadline_met = Some(
                self.clock_valid && now <= self.job.authority_deadline_us + OBSERVATION_TAIL_US,
            );
        }
        if self.job.terminal.is_some() {
            return;
        }
        if !self.clock_valid || self.job.resources.socket_state == SocketState::Open {
            self.job.resources.deadline_met = Some(false);
            self.job.resources.failure.get_or_insert(NoiseFailure::new(
                FailureStage::Cleanup,
                NoiseCategory::Cleanup,
                NoiseDetail::ResourceUnreleased,
                self.clock_valid.then_some(now),
            ));
            self.job.terminal = Some(NoiseTerminal {
                outcome: NoiseOutcome::Incomplete,
                decided_at_us: self.clock_valid.then_some(now),
            });
            self.state = NoiseState::Terminal;
            return;
        }
        let complete =
            self.job.stages.len() == 8 && self.job.resources.socket_state == SocketState::Closed;
        let outcome = match self.job.first_failure {
            None if complete && now < self.job.authority_deadline_us && self.clock_valid => {
                NoiseOutcome::Accepted
            }
            Some(failure)
                if matches!(
                    failure.detail,
                    NoiseDetail::Timeout | NoiseDetail::HeartbeatExpired
                ) =>
            {
                NoiseOutcome::Expired
            }
            Some(failure)
                if matches!(
                    failure.detail,
                    NoiseDetail::CancelRequested | NoiseDetail::SessionReplaced
                ) =>
            {
                NoiseOutcome::Cancelled
            }
            Some(_) => NoiseOutcome::Rejected,
            None => {
                self.job.first_failure = Some(NoiseFailure::new(
                    FailureStage::Evidence,
                    NoiseCategory::EvidenceIncomplete,
                    NoiseDetail::Missing,
                    self.clock_valid.then_some(now),
                ));
                NoiseOutcome::Rejected
            }
        };
        self.job.terminal = Some(NoiseTerminal {
            outcome,
            decided_at_us: self.clock_valid.then_some(now),
        });
        self.state = NoiseState::Terminal;
    }
}
