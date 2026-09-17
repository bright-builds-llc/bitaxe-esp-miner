use super::*;
mod acknowledgement;
pub use acknowledgement::ObservedAcknowledgement;

/// Captures bounded facts. Native owners provide clocks and actual effect completion.
pub struct V2Record {
    record: DeviceRecord,
    last_time: u64,
}
impl V2Record {
    pub fn admit(
        scope: Scope,
        attempt_id: String,
        observation: &CurrentObservation,
        maybe_authority_deadline: Option<u64>,
    ) -> Option<Self> {
        let now = observation.maybe_observed_at_us?;
        if !observation.clock_valid
            || (scope == Scope::Channel
                && maybe_authority_deadline != now.checked_add(AUTHORITY_US))
            || (scope == Scope::Share && maybe_authority_deadline.is_some())
        {
            return None;
        }
        let mut events = Vec::new();
        events.try_reserve_exact(MAX_EVENTS).ok()?;
        let mut share_facts = Vec::new();
        share_facts.try_reserve_exact(MAX_SHARES).ok()?;
        let mut secondary_failures = Vec::new();
        secondary_failures.try_reserve_exact(MAX_FAILURES).ok()?;
        let mut timings = Vec::new();
        timings.try_reserve_exact(13).ok()?;
        for operation in Operation::ALL {
            timings.push(Timing {
                operation,
                count: 0,
                failed_count: 0,
                maybe_max_duration_us: None,
                maybe_total_duration_us: None,
                maybe_first_started_at_device_us: None,
                maybe_last_finished_at_device_us: None,
                maybe_in_flight_started_at_device_us: None,
            });
        }
        let mut result = Self {
            last_time: now,
            record: DeviceRecord {
                schema: "worker-v2-serial-evidence-v1",
                scope,
                attempt_id,
                boot_ordinal: observation.boot_ordinal,
                worker_generation: observation.worker_generation,
                maybe_pool_session_generation: None,
                maybe_pool_transport_epoch: None,
                serial_transport_epoch: observation.serial_transport_epoch,
                maybe_job_commitment: None,
                maybe_observed_at_us: Some(now),
                state: State::Admitted,
                admitted_at_device_us: now,
                maybe_authority_deadline_device_us: maybe_authority_deadline,
                maybe_observation_deadline_device_us: if scope == Scope::Channel {
                    Some(now.checked_add(OBSERVATION_US)?)
                } else {
                    None
                },
                maybe_terminal_at_device_us: None,
                maybe_outcome: None,
                events,
                timings,
                share_facts,
                maybe_first_failure: None,
                secondary_failures,
                resources: Resources {
                    socket_closed: false,
                    worker_quiescent: false,
                    fence_retained: true,
                    maybe_socket_closed_at_us: None,
                    maybe_worker_quiescent_at_us: None,
                },
            },
        };
        result.event(Stage::Admitted, Some(now), None, None, None, None);
        Some(result)
    }
    /// Observes the unchanged native guarded-dispatch arming epoch, never a host clock.
    pub fn budget_armed(
        &mut self,
        arming_epoch_ms: u64,
        limit_ms: u32,
        observed_at_us: Option<u64>,
    ) -> bool {
        let observed = self.time(observed_at_us);
        let deadline = arming_epoch_ms
            .checked_add(u64::from(limit_ms))
            .and_then(|v| v.checked_mul(1000));
        if self.record.scope != Scope::Share
            || observed.is_none_or(|t| {
                arming_epoch_ms > t / 1000
                    || arming_epoch_ms < self.record.admitted_at_device_us / 1000
            })
            || limit_ms != 180_000
            || deadline.is_none_or(|d| {
                d > crate::noise::MAX_SAFE_INTEGER || d <= self.record.admitted_at_device_us
            })
        {
            self.fail(Stage::AsicDispatch, FailureCategory::Clock, None);
            return false;
        }
        match self.record.maybe_authority_deadline_device_us {
            None => {
                self.record.maybe_authority_deadline_device_us = deadline;
                true
            }
            Some(previous) if Some(previous) == deadline => true,
            _ => {
                self.fail(
                    Stage::AsicDispatch,
                    FailureCategory::Evidence,
                    Some(self.last_time),
                );
                false
            }
        }
    }
    pub fn maybe_failure(&self) -> Option<Failure> {
        self.record.maybe_first_failure
    }
    pub fn has_stage(&self, stage: Stage) -> bool {
        self.record.events.iter().any(|e| e.kind == stage)
    }
    pub fn snapshot(&self) -> DeviceRecord {
        self.record.clone()
    }
    pub fn binding(&self) -> (u64, u64, u64) {
        (
            self.record.boot_ordinal,
            self.record.worker_generation,
            self.record.serial_transport_epoch,
        )
    }
    pub fn pool_binding(&self) -> Option<(u64, u64)> {
        self.record
            .maybe_pool_session_generation
            .zip(self.record.maybe_pool_transport_epoch)
    }
    pub fn scope(&self) -> Scope {
        self.record.scope
    }
    pub fn bind_pool(&mut self, generation: u64, epoch: u64) -> bool {
        if self.record.maybe_pool_session_generation.is_some()
            || self.record.maybe_outcome.is_some()
        {
            return false;
        }
        self.record.maybe_pool_session_generation = Some(generation);
        self.record.maybe_pool_transport_epoch = Some(epoch);
        true
    }
    pub fn fail(&mut self, stage: Stage, category: FailureCategory, time: Option<u64>) {
        let failure = Failure {
            stage,
            category,
            maybe_at_device_us: time,
        };
        // Revocation is a consequence of the original failure. Repeated
        // protocol/authority errors cannot replace or expand that first cause;
        // only independently observed cleanup/clock problems are secondary.
        if self.record.maybe_first_failure.is_none() {
            self.record.maybe_first_failure = Some(failure);
        } else if matches!(category, FailureCategory::Clock | FailureCategory::Cleanup)
            && self.record.maybe_first_failure != Some(failure)
            && !self
                .record
                .secondary_failures
                .iter()
                .any(|f| f.stage == stage && f.category == category)
            && self.record.secondary_failures.len() < MAX_FAILURES
        {
            self.record.secondary_failures.push(failure);
        }
    }
    fn time(&mut self, now: Option<u64>) -> Option<u64> {
        match now {
            Some(value) if value >= self.last_time && value <= crate::noise::MAX_SAFE_INTEGER => {
                self.last_time = value;
                self.record.maybe_observed_at_us = Some(value);
                Some(value)
            }
            _ => {
                self.record.maybe_observed_at_us = None;
                self.fail(Stage::WorkerQuiescent, FailureCategory::Clock, None);
                None
            }
        }
    }
    pub fn event(
        &mut self,
        kind: Stage,
        now: Option<u64>,
        channel: Option<u32>,
        job: Option<u32>,
        submission: Option<u32>,
        digest: Option<String>,
    ) {
        let time = self.time(now);
        if self.record.events.len() >= MAX_EVENTS {
            self.fail(kind, FailureCategory::Evidence, time);
            return;
        }
        if self.record.maybe_outcome.is_some()
            && !matches!(
                kind,
                Stage::SocketClosed
                    | Stage::WorkerQuiescent
                    | Stage::Revoked
                    | Stage::Shutdown
                    | Stage::Cooled
            )
        {
            return;
        }
        self.record.events.push(Event {
            sequence: self.record.events.len() as u64 + 1,
            maybe_at_device_us: time,
            kind,
            maybe_channel_id: channel,
            maybe_job_id: job,
            maybe_submission_sequence: submission,
            maybe_payload_sha256: if matches!(
                kind,
                Stage::Setup
                    | Stage::Channel
                    | Stage::Admitted
                    | Stage::Preparing
                    | Stage::Connected
                    | Stage::Authenticated
            ) {
                None
            } else {
                digest
            },
        });
        if kind == Stage::Preparing {
            self.record.state = State::Running;
        }
    }
    pub fn begin(&mut self, operation: Operation, now: Option<u64>) {
        let time = self.time(now);
        let timing = &mut self.record.timings[operation as usize];
        if timing.maybe_in_flight_started_at_device_us.is_some() {
            self.fail(Stage::WorkerQuiescent, FailureCategory::Evidence, time);
            return;
        }
        timing.maybe_in_flight_started_at_device_us = time;
    }
    pub fn end(&mut self, operation: Operation, now: Option<u64>, failed: bool) {
        let time = self.time(now);
        let timing = &mut self.record.timings[operation as usize];
        let start = timing.maybe_in_flight_started_at_device_us.take();
        if timing.count == 0 {
            timing.maybe_first_started_at_device_us = start;
        }
        let duration = start.zip(time).and_then(|(a, b)| b.checked_sub(a));
        timing.count += 1;
        timing.failed_count += u64::from(failed);
        timing.maybe_last_finished_at_device_us = time;
        let aggregates = duration.and_then(|d| {
            if timing.count > 1 && timing.maybe_total_duration_us.is_none() {
                return None;
            }
            let total = timing.maybe_total_duration_us.unwrap_or(0).checked_add(d)?;
            (total <= crate::noise::MAX_SAFE_INTEGER)
                .then_some((timing.maybe_max_duration_us.unwrap_or(0).max(d), total))
        });
        timing.maybe_max_duration_us = aggregates.map(|v| v.0);
        timing.maybe_total_duration_us = aggregates.map(|v| v.1);
        if aggregates.is_none() {
            self.fail(Stage::WorkerQuiescent, FailureCategory::Clock, None);
        }
    }
    pub fn set_job_commitment(&mut self, digest: String) {
        if self.record.maybe_job_commitment.is_some() {
            self.fail(Stage::Job, FailureCategory::Evidence, Some(self.last_time));
        } else {
            self.record.maybe_job_commitment = Some(digest);
        }
    }
    pub fn add_share(&mut self, fact: ShareFact) -> bool {
        if self.record.maybe_authority_deadline_device_us.is_none()
            || self.record.scope != Scope::Share
            || self.record.share_facts.len() >= MAX_SHARES
            || self
                .record
                .share_facts
                .iter()
                .any(|f| f.submission_sequence == fact.submission_sequence)
        {
            self.fail(
                Stage::Nonce,
                FailureCategory::Evidence,
                Some(self.last_time),
            );
            return false;
        }
        self.record.share_facts.push(fact);
        true
    }
    pub fn has_accepted_share(&self) -> bool {
        self.record
            .share_facts
            .iter()
            .any(|f| f.maybe_ack_accepted_count == Some(1))
    }
    pub fn has_candidate(&self, asic_job_id: u8, nonce: u32, version_bits: u32) -> bool {
        self.record.share_facts.iter().any(|f| {
            f.asic_job_id == asic_job_id && f.nonce == nonce && f.version_bits == version_bits
        })
    }
    pub fn share_mut(&mut self, sequence: u32) -> Option<&mut ShareFact> {
        self.record
            .share_facts
            .iter_mut()
            .find(|f| f.submission_sequence == sequence)
    }
    pub fn socket_closed(&mut self, now: Option<u64>) {
        let time = self.time(now);
        self.record.resources.socket_closed = true;
        self.record.resources.maybe_socket_closed_at_us = time;
        self.event(Stage::SocketClosed, time, None, None, None, None);
    }
    pub fn joined(&mut self, now: Option<u64>, fence_released: bool) {
        let time = self.time(now);
        self.record.resources.worker_quiescent = true;
        self.record.resources.maybe_worker_quiescent_at_us = time;
        self.record.resources.fence_retained = !fence_released;
        self.event(Stage::WorkerQuiescent, time, None, None, None, None);
        if !fence_released {
            return;
        }
        self.finish(time);
    }
    pub fn release_fence(&mut self, now: Option<u64>) -> bool {
        if !self.record.resources.worker_quiescent {
            return false;
        }
        self.record.resources.fence_retained = false;
        let time = self.time(now);
        self.finish(time);
        true
    }
    fn finish(&mut self, time: Option<u64>) {
        if self.record.maybe_outcome.is_some() {
            return;
        }
        let complete = self.record.resources.socket_closed
            && time.is_some()
            && !self.record.resources.fence_retained;
        let success = self.record.maybe_first_failure.is_none()
            && complete
            && self.record.maybe_job_commitment.is_some()
            && (if self.record.scope == Scope::Share {
                self.record.maybe_authority_deadline_device_us.is_some()
                    && self.record.share_facts.iter().any(|f| {
                        f.maybe_write_completed_at_device_us.is_some()
                            && f.maybe_ack_at_device_us.is_some()
                            && f.maybe_ack_accepted_count == Some(1)
                    })
            } else {
                time.zip(self.record.maybe_authority_deadline_device_us)
                    .is_some_and(|(t, d)| t <= d)
            });
        if !success && self.record.maybe_first_failure.is_none() {
            self.fail(Stage::WorkerQuiescent, FailureCategory::Evidence, time);
        }
        self.terminal(
            if success {
                Outcome::Accepted
            } else if complete {
                Outcome::Rejected
            } else {
                Outcome::Incomplete
            },
            time,
        );
    }
    pub fn tick(&mut self, now: Option<u64>) {
        let time = self.time(now);
        if self.record.scope == Scope::Channel
            && time
                .zip(self.record.maybe_authority_deadline_device_us)
                .is_some_and(|(t, d)| t >= d)
            && self.record.maybe_outcome.is_none()
        {
            self.fail(Stage::WorkerQuiescent, FailureCategory::Timeout, time);
        }
        if self
            .record
            .maybe_observation_deadline_device_us
            .zip(time)
            .is_some_and(|(d, t)| t >= d)
            && self.record.maybe_outcome.is_none()
        {
            self.fail(Stage::WorkerQuiescent, FailureCategory::Cleanup, time);
            self.terminal(Outcome::Incomplete, time);
        }
    }
    fn terminal(&mut self, outcome: Outcome, now: Option<u64>) {
        self.record.state = State::Terminal;
        self.record.maybe_outcome = Some(outcome);
        self.record.maybe_terminal_at_device_us = now;
    }
}
