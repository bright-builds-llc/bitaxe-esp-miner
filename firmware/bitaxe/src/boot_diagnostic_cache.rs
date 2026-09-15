//! Immutable boot observations for USB replay, independent of circular-log storage.
use std::sync::atomic::{AtomicU32, Ordering};

const EMPTY: u32 = 0;
const WRITING: u32 = 1;
const READY: u32 = 2;
const INVALID: u32 = 3;
const CHECKPOINT_COUNT: usize = 7;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(usize)]
pub(crate) enum Checkpoint {
    UsbInstall,
    UsbInstalled,
    WifiDriverPrepare,
    WifiDriverPrepared,
    WorkerOwnerPrepare,
    StatisticsStart,
    StatisticsStarted,
}
impl Checkpoint {
    pub(crate) fn maybe_from_label(label: &str) -> Option<Self> {
        match label {
            "usb_install" => Some(Self::UsbInstall),
            "usb_installed" => Some(Self::UsbInstalled),
            "wifi_driver_prepare" => Some(Self::WifiDriverPrepare),
            "wifi_driver_prepared" => Some(Self::WifiDriverPrepared),
            "worker_owner_prepare" => Some(Self::WorkerOwnerPrepare),
            "statistics_start" => Some(Self::StatisticsStart),
            "statistics_started" => Some(Self::StatisticsStarted),
            _ => None,
        }
    }
    pub(crate) const fn label(self) -> &'static str {
        match self {
            Self::UsbInstall => "usb_install",
            Self::UsbInstalled => "usb_installed",
            Self::WifiDriverPrepare => "wifi_driver_prepare",
            Self::WifiDriverPrepared => "wifi_driver_prepared",
            Self::WorkerOwnerPrepare => "worker_owner_prepare",
            Self::StatisticsStart => "statistics_start",
            Self::StatisticsStarted => "statistics_started",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u32)]
pub(crate) enum StartFailure {
    OwnerSpawn = 1,
    UsbInstall,
    ControlOwner,
}
impl StartFailure {
    /// Classifies producer context without retaining or formatting its arbitrary error text.
    pub(crate) fn from_context(message: &str) -> Self {
        if message.starts_with("owner_spawn:") {
            Self::OwnerSpawn
        } else if message.starts_with("usb_install:") {
            Self::UsbInstall
        } else {
            Self::ControlOwner
        }
    }
    pub(crate) const fn label(self) -> &'static str {
        match self {
            Self::OwnerSpawn => "owner_spawn",
            Self::UsbInstall => "usb_install",
            Self::ControlOwner => "control_owner",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct MemoryObservation {
    pub(crate) free_bytes: u32,
    pub(crate) largest_block_bytes: u32,
    pub(crate) reserve_bytes: u32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Observation<T> {
    Unavailable,
    InProgress,
    Available(T),
    Corrupt,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum RecordOutcome {
    Recorded,
    AlreadyRecorded,
    InvalidObservation,
}

struct CheckpointSlot {
    state: AtomicU32,
    free: AtomicU32,
    largest: AtomicU32,
    reserve: AtomicU32,
}
impl CheckpointSlot {
    const fn new() -> Self {
        Self {
            state: AtomicU32::new(EMPTY),
            free: AtomicU32::new(0),
            largest: AtomicU32::new(0),
            reserve: AtomicU32::new(0),
        }
    }
    fn record(&self, free: usize, largest: usize, reserve: usize) -> RecordOutcome {
        if self
            .state
            .compare_exchange(EMPTY, WRITING, Ordering::AcqRel, Ordering::Acquire)
            .is_err()
        {
            return RecordOutcome::AlreadyRecorded;
        }
        let (Ok(free), Ok(largest), Ok(reserve)) = (
            u32::try_from(free),
            u32::try_from(largest),
            u32::try_from(reserve),
        ) else {
            self.state.store(INVALID, Ordering::Release);
            return RecordOutcome::InvalidObservation;
        };
        self.free.store(free, Ordering::Relaxed);
        self.largest.store(largest, Ordering::Relaxed);
        self.reserve.store(reserve, Ordering::Relaxed);
        self.state.store(READY, Ordering::Release);
        RecordOutcome::Recorded
    }
    fn read(&self) -> Observation<MemoryObservation> {
        match self.state.load(Ordering::Acquire) {
            EMPTY => Observation::Unavailable,
            WRITING => Observation::InProgress,
            READY => Observation::Available(MemoryObservation {
                free_bytes: self.free.load(Ordering::Relaxed),
                largest_block_bytes: self.largest.load(Ordering::Relaxed),
                reserve_bytes: self.reserve.load(Ordering::Relaxed),
            }),
            _ => Observation::Corrupt,
        }
    }
}

pub(crate) struct BootDiagnosticCache {
    checkpoints: [CheckpointSlot; CHECKPOINT_COUNT],
    failure: AtomicU32,
}

pub(crate) static CACHE: BootDiagnosticCache = BootDiagnosticCache::new();

impl BootDiagnosticCache {
    pub(crate) const fn new() -> Self {
        Self {
            checkpoints: [const { CheckpointSlot::new() }; CHECKPOINT_COUNT],
            failure: AtomicU32::new(EMPTY),
        }
    }
    /// Recording cannot allocate, log, wait, overwrite an observation or acquire authority.
    pub(crate) fn record_checkpoint(
        &self,
        checkpoint: Checkpoint,
        free: usize,
        largest: usize,
        reserve: usize,
    ) -> RecordOutcome {
        self.checkpoints[checkpoint as usize].record(free, largest, reserve)
    }
    pub(crate) fn checkpoint(&self, checkpoint: Checkpoint) -> Observation<MemoryObservation> {
        self.checkpoints[checkpoint as usize].read()
    }
    pub(crate) fn record_failure(&self, failure: StartFailure) -> bool {
        self.failure
            .compare_exchange(EMPTY, failure as u32, Ordering::AcqRel, Ordering::Acquire)
            .is_ok()
    }
    pub(crate) fn failure(&self) -> Observation<StartFailure> {
        match self.failure.load(Ordering::Acquire) {
            EMPTY => Observation::Unavailable,
            1 => Observation::Available(StartFailure::OwnerSpawn),
            2 => Observation::Available(StartFailure::UsbInstall),
            3 => Observation::Available(StartFailure::ControlOwner),
            _ => Observation::Corrupt,
        }
    }
    /// Formatting occurs only in the reader; missing/partial/corrupt observations are not fabricated.
    pub(crate) fn maybe_checkpoint_marker(&self, checkpoint: Checkpoint) -> Option<String> {
        let Observation::Available(value) = self.checkpoint(checkpoint) else {
            return None;
        };
        Some(format!("usb_memory_checkpoint stage={} free_bytes={} largest_block_bytes={} reserve_bytes={} redacted=true", checkpoint.label(), value.free_bytes, value.largest_block_bytes, value.reserve_bytes))
    }
    pub(crate) fn maybe_failure_marker(&self) -> Option<String> {
        let Observation::Available(failure) = self.failure() else {
            return None;
        };
        Some(format!(
            "bwg_worker_start_failure category=startup_failed detail={} redacted=true",
            failure.label()
        ))
    }
}

#[cfg(test)]
#[path = "boot_diagnostic_cache/tests.rs"]
mod tests;
