//! Stack-local, exclusive live-publication stage observations; no shared recorder access.
use std::cell::Cell;

pub const LIVE_STAGE_COUNT: usize = 11;
const COMPLETE_MASK: u16 = (1 << LIVE_STAGE_COUNT) - 1;

/// Stable wire-array order. Never reorder stages within the v2 diagnostic schema.
#[derive(Clone, Copy)]
#[repr(usize)]
pub enum LiveStage {
    VisibleState,
    Platform,
    HealthSafety,
    ConfirmedSettings,
    SettingsTransactionWait,
    SettingsNvsRead,
    Wifi,
    PublicationOrderWait,
    ProjectionComplete,
    Retention,
    SerializationQueue,
}

#[derive(Clone, Copy, Debug)]
pub struct LiveStageMeasurements {
    pub durations_us: [u64; LIVE_STAGE_COUNT],
    pub complete: bool,
}

#[derive(Clone, Copy)]
struct Observation {
    durations_us: [u64; LIVE_STAGE_COUNT],
    seen: u16,
    valid: bool,
}

/// Own one instance on the actual live-loop stack. Disabled instances preserve other callers.
pub struct LiveStageProfiler {
    maybe_clock: Option<fn() -> u64>,
    observation: Cell<Observation>,
}

impl LiveStageProfiler {
    pub const fn new(clock: fn() -> u64) -> Self {
        Self {
            maybe_clock: Some(clock),
            observation: Cell::new(Observation {
                durations_us: [0; LIVE_STAGE_COUNT],
                seen: 0,
                valid: true,
            }),
        }
    }

    pub const fn disabled() -> Self {
        Self {
            maybe_clock: None,
            observation: Cell::new(Observation {
                durations_us: [0; LIVE_STAGE_COUNT],
                seen: 0,
                valid: false,
            }),
        }
    }

    /// Starts an explicit timing boundary without forwarding its operation's value.
    #[inline(always)]
    pub fn maybe_start_stage(&self) -> Option<u64> {
        self.maybe_clock.map(|clock| clock())
    }

    /// Completes an explicit boundary after the caller has materialized its own result in place.
    #[inline(always)]
    pub fn finish_stage(&self, stage: LiveStage, maybe_started: Option<u64>) {
        let (Some(started), Some(clock)) = (maybe_started, self.maybe_clock) else {
            return;
        };
        self.finish_record(stage, started, clock());
    }

    /// Measures one exclusive operation. Missing/duplicate stages and invalid clocks remain explicit.
    #[inline(always)]
    pub fn measure<T>(&self, stage: LiveStage, operation: impl FnOnce() -> T) -> T {
        let Some(clock) = self.maybe_clock else {
            return operation();
        };
        let started = clock();
        let result = operation();
        let ended = clock();
        self.finish_record(stage, started, ended);
        result
    }

    // Keep the fixed-array bookkeeping shared across every operation/return-type instantiation.
    #[inline(never)]
    fn finish_record(&self, stage: LiveStage, started: u64, ended: u64) {
        let mut observation = self.observation.get();
        let bit = 1 << stage as usize;
        observation.valid &= started > 0 && ended >= started && observation.seen & bit == 0;
        observation.seen |= bit;
        observation.durations_us[stage as usize] = ended.saturating_sub(started);
        self.observation.set(observation);
    }

    pub fn measurements(&self) -> LiveStageMeasurements {
        let observation = self.observation.get();
        LiveStageMeasurements {
            durations_us: observation.durations_us,
            complete: observation.valid && observation.seen == COMPLETE_MASK,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    thread_local! { static NOW: Cell<u64> = const { Cell::new(100) }; }
    fn now() -> u64 {
        NOW.with(Cell::get)
    }
    const STAGES: [LiveStage; LIVE_STAGE_COUNT] = [
        LiveStage::VisibleState,
        LiveStage::Platform,
        LiveStage::HealthSafety,
        LiveStage::ConfirmedSettings,
        LiveStage::SettingsTransactionWait,
        LiveStage::SettingsNvsRead,
        LiveStage::Wifi,
        LiveStage::PublicationOrderWait,
        LiveStage::ProjectionComplete,
        LiveStage::Retention,
        LiveStage::SerializationQueue,
    ];

    #[test]
    fn measures_exclusive_actual_operations_with_a_fake_clock() {
        // Arrange
        NOW.with(|clock| clock.set(100));
        let profiler = LiveStageProfiler::new(now);
        // Act
        for (index, stage) in STAGES.into_iter().enumerate() {
            profiler.measure(stage, || {
                NOW.with(|clock| clock.set(clock.get() + index as u64 + 1))
            });
        }
        // Assert
        assert!(profiler.measurements().complete);
        assert_eq!(
            profiler.measurements().durations_us,
            [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11]
        );
    }

    #[test]
    fn missing_measurement_is_not_a_valid_zero_duration() {
        // Arrange
        let profiler = LiveStageProfiler::new(now);
        // Act
        profiler.measure(LiveStage::VisibleState, || ());
        // Assert
        assert!(!profiler.measurements().complete);
    }

    #[test]
    fn reversed_clock_is_retained_as_invalid() {
        // Arrange
        NOW.with(|clock| clock.set(100));
        let profiler = LiveStageProfiler::new(now);
        // Act
        for (index, stage) in STAGES.into_iter().enumerate() {
            profiler.measure(stage, || {
                if index == 9 {
                    NOW.with(|clock| clock.set(1));
                }
            });
        }
        // Assert
        assert!(!profiler.measurements().complete);
    }
}
