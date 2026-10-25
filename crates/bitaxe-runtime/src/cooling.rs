//! Fan-only orchestration has no ASIC, voltage, pool, or budget capability.
use bitaxe_api::{ChipTemperatureSource, MiningSafetyLimits, TelemetryObservations};
use bitaxe_safety::observation::MonotonicMillis;

pub const FAN_PROOF_TIMEOUT_MS: u64 = 3_000;
/// An ASIC-diode board's stop proof waits at least this long after its window opens, as
/// upstream's 600-series recovery waits a fixed 30 s with the diode unreadable.
pub const ASIC_DIODE_STOP_COOLING_MS: u64 = crate::local_mining::OVERHEAT_COOLING_MS;
#[derive(Clone, Copy, Debug)]
pub struct FanStamp {
    pub boot_session: u64,
    pub sequence: u64,
    pub acquired_at_ms: u64,
}

pub fn post_command_fan(
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
pub fn preparation_safety(base_safe: bool, nonzero_rpm: bool, never_prepared_worker: bool) -> bool {
    base_safe && (nonzero_rpm || never_prepared_worker)
}
/// Mining-safe observations and a cool restart sample, without a time rule: the chip at or
/// below 45 C, or on an ASIC-diode board the chip reported unpowered and the regulator at or
/// below 95 C.
pub fn baseline_safe(
    limits: &MiningSafetyLimits,
    observations: &TelemetryObservations,
    now_ms: u64,
) -> bool {
    observations.is_mining_safe_at(limits, MonotonicMillis::new(now_ms))
        && cool_sample(limits, observations, |_| true)
}
/// Whether a run may qualify its fan. A board-sensor board needs mining-safe observations.
/// An ASIC-diode board also needs [`baseline_safe`]'s sample, so a reboot that left its
/// regulator on, with the chip not reported unpowered, is not admitted.
pub fn admission_safe(
    limits: &MiningSafetyLimits,
    observations: &TelemetryObservations,
    now_ms: u64,
) -> bool {
    match limits.chip_temperature_source() {
        ChipTemperatureSource::BoardSensor => {
            observations.is_mining_safe_at(limits, MonotonicMillis::new(now_ms))
        }
        ChipTemperatureSource::AsicDiode => baseline_safe(limits, observations, now_ms),
    }
}
/// The terminal stop's cooling proof: mining-safe observations and a cool restart sample taken
/// after the window opened. An ASIC-diode board also needs [`ASIC_DIODE_STOP_COOLING_MS`]
/// since the window opened.
pub fn stop_cooling_proven(
    limits: &MiningSafetyLimits,
    observations: &TelemetryObservations,
    window_start_ms: u64,
    now_ms: u64,
) -> bool {
    let waited = match limits.chip_temperature_source() {
        ChipTemperatureSource::BoardSensor => true,
        ChipTemperatureSource::AsicDiode => {
            now_ms.saturating_sub(window_start_ms) >= ASIC_DIODE_STOP_COOLING_MS
        }
    };
    waited
        && observations.is_mining_safe_at(limits, MonotonicMillis::new(now_ms))
        && cool_sample(limits, observations, |acquired_at_ms| {
            acquired_at_ms > window_start_ms
        })
}
fn cool_sample(
    limits: &MiningSafetyLimits,
    observations: &TelemetryObservations,
    taken_in_window: impl Fn(u64) -> bool,
) -> bool {
    observations
        .restart_cooling_sample(limits)
        .is_some_and(|sample| sample.cool && taken_in_window(sample.acquired_at_ms))
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CoolingError {
    Rejected,
    TimedOut,
    WriteFailed,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FanCommand {
    Full,
    RestoreBaseline,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FanReply {
    Pending,
    Applied,
}

pub trait CoolingBackend {
    fn now_ms(&self) -> u64;
    fn admitted(&self) -> bool;
    /// The ranges the board's mining-safety facts must hold while cooling.
    fn safety_limits(&self) -> MiningSafetyLimits;
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
pub fn qualify(backend: &mut impl CoolingBackend) -> Result<u16, CoolingError> {
    let started = backend.now_ms();
    let baseline = backend.observations();
    let limits = backend.safety_limits();
    if !backend.admitted() || !admission_safe(&limits, &baseline, started) {
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
        if !admission_safe(&limits, &observations, now) {
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
pub fn restore(backend: &mut impl CoolingBackend) -> Result<(), CoolingError> {
    if !backend.admitted()
        || !baseline_safe(
            &backend.safety_limits(),
            &backend.observations(),
            backend.now_ms(),
        )
    {
        return Err(CoolingError::Rejected);
    }
    let started = backend.now_ms();
    backend.command(FanCommand::RestoreBaseline)?;
    loop {
        if !backend.admitted()
            || !baseline_safe(
                &backend.safety_limits(),
                &backend.observations(),
                backend.now_ms(),
            )
        {
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
#[cfg(test)]
#[path = "cooling/tests.rs"]
mod tests;
