//! Fan-only orchestration has no ASIC, voltage, pool, or budget capability.
use bitaxe_api::TelemetryObservations;
use bitaxe_safety::observation::MonotonicMillis;

pub(crate) const FAN_PROOF_TIMEOUT_MS: u64 = 3_000;
#[derive(Clone, Copy, Debug)]
pub(crate) struct FanStamp {
    pub boot_session: u64,
    pub sequence: u64,
    pub acquired_at_ms: u64,
}

pub(crate) fn post_command_fan(
    candidate: FanStamp,
    maybe_baseline: Option<FanStamp>,
    started_at_ms: u64,
) -> bool {
    candidate.acquired_at_ms > started_at_ms
        && maybe_baseline.is_none_or(|baseline| {
            candidate.boot_session == baseline.boot_session
                && candidate.sequence > baseline.sequence
                && candidate.acquired_at_ms > baseline.acquired_at_ms
        })
}
pub(crate) fn preparation_safety(
    base_safe: bool,
    nonzero_rpm: bool,
    never_prepared_worker: bool,
) -> bool {
    base_safe && (nonzero_rpm || never_prepared_worker)
}
pub(crate) fn baseline_safe(observations: &TelemetryObservations, now_ms: u64) -> bool {
    observations.is_ultra_205_mining_safe_at(MonotonicMillis::new(now_ms))
        && observations
            .chip_temp_celsius
            .maybe_last_good()
            .is_some_and(|sample| *sample.value() <= 45.0)
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum CoolingError {
    Rejected,
    TimedOut,
    WriteFailed,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum FanCommand {
    Full,
    RestoreBaseline,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum FanReply {
    Pending,
    Applied,
}

pub(crate) trait CoolingBackend {
    fn now_ms(&self) -> u64;
    fn admitted(&self) -> bool;
    fn observations(&self) -> TelemetryObservations;
    fn command(&mut self, command: FanCommand) -> Result<(), CoolingError>;
    fn poll(&mut self) -> Result<FanReply, CoolingError>;
    fn wait(&mut self);
}
fn stamp(observations: &TelemetryObservations) -> Option<FanStamp> {
    observations
        .fan_rpm
        .maybe_last_good()
        .map(|sample| FanStamp {
            boot_session: sample.boot_session().get(),
            sequence: sample.sequence().get(),
            acquired_at_ms: sample.acquired_at().get(),
        })
}
/// Leaves the fan at full duty; only an explicit, independently checked restore lowers it.
pub(crate) fn qualify(backend: &mut impl CoolingBackend) -> Result<u16, CoolingError> {
    let started = backend.now_ms();
    let baseline = backend.observations();
    if !backend.admitted() || !baseline.is_ultra_205_mining_safe_at(MonotonicMillis::new(started)) {
        return Err(CoolingError::Rejected);
    }
    let maybe_baseline = stamp(&baseline);
    backend.command(FanCommand::Full)?;
    let mut maybe_applied_at = None;
    loop {
        if !backend.admitted() {
            return Err(CoolingError::Rejected);
        }
        let now = backend.now_ms();
        let observations = backend.observations();
        if !observations.is_ultra_205_mining_safe_at(MonotonicMillis::new(now)) {
            return Err(CoolingError::Rejected);
        }
        if maybe_applied_at.is_none() && backend.poll()? == FanReply::Applied {
            // Receipt observation is conservative: no pre-apply sample can qualify.
            maybe_applied_at = Some(backend.now_ms());
        }
        if maybe_applied_at.is_some_and(|applied_at| {
            stamp(&observations)
                .is_some_and(|candidate| post_command_fan(candidate, maybe_baseline, applied_at))
        }) {
            if let Some(sample) = observations.fan_rpm.maybe_last_good() {
                if *sample.value() > 0 {
                    return Ok(*sample.value());
                }
            }
        }
        if now.saturating_sub(started) >= FAN_PROOF_TIMEOUT_MS {
            return Err(CoolingError::TimedOut);
        }
        backend.wait();
    }
}
pub(crate) fn restore(backend: &mut impl CoolingBackend) -> Result<(), CoolingError> {
    if !backend.admitted() || !baseline_safe(&backend.observations(), backend.now_ms()) {
        return Err(CoolingError::Rejected);
    }
    let started = backend.now_ms();
    backend.command(FanCommand::RestoreBaseline)?;
    loop {
        if !backend.admitted() || !baseline_safe(&backend.observations(), backend.now_ms()) {
            return Err(CoolingError::Rejected);
        }
        if backend.poll()? == FanReply::Applied {
            return Ok(());
        }
        if backend.now_ms().saturating_sub(started) >= FAN_PROOF_TIMEOUT_MS {
            return Err(CoolingError::TimedOut);
        }
        backend.wait();
    }
}
