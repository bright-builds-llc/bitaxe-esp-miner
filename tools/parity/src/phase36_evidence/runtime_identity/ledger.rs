use bitaxe_device_session::{DevicePhase, SerialPhase, SessionEvent};

use super::RuntimeIdentityEvidenceError;

pub(super) fn parse_and_validate(
    document: &str,
) -> Result<Vec<SessionEvent>, RuntimeIdentityEvidenceError> {
    let events = document
        .lines()
        .filter(|line| !line.trim().is_empty())
        .map(|line| {
            serde_json::from_str(line).map_err(|_| RuntimeIdentityEvidenceError::DocumentInvalid)
        })
        .collect::<Result<Vec<_>, _>>()?;
    if events.is_empty() {
        return Err(RuntimeIdentityEvidenceError::MissingLedgerStep);
    }
    let mut coverage = LedgerCoverage::default();
    for event in &events {
        coverage.observe(event)?;
    }
    if !coverage.complete() {
        return Err(RuntimeIdentityEvidenceError::MissingLedgerStep);
    }
    Ok(events)
}

#[derive(Debug, Default)]
struct LedgerCoverage {
    maybe_rank: Option<u8>,
    platform: bool,
    initial_samples: u8,
    reader_armed: bool,
    pre_serial: bool,
    baseline: bool,
    request_started: bool,
    request_written: bool,
    response: bool,
    service_loss: bool,
    absence: bool,
    recovery_samples: u8,
    reader_reacquired: bool,
    post_serial: bool,
    boot_b: bool,
    expired: bool,
    cleanup: bool,
}

impl LedgerCoverage {
    fn observe(&mut self, event: &SessionEvent) -> Result<(), RuntimeIdentityEvidenceError> {
        let maybe_rank = match event {
            SessionEvent::PlatformObserved { .. } => {
                self.platform = true;
                Some(0)
            }
            SessionEvent::DeviceSample {
                phase: DevicePhase::Initial,
                ..
            } => {
                self.initial_samples = self.initial_samples.saturating_add(1);
                Some(1)
            }
            SessionEvent::ReaderArmed => {
                self.reader_armed = true;
                Some(2)
            }
            SessionEvent::SerialBytes {
                phase: SerialPhase::PreRestart,
                count,
            } if *count > 0 => {
                self.pre_serial = true;
                Some(3)
            }
            SessionEvent::BaselineConfirmed => {
                self.baseline = true;
                Some(4)
            }
            SessionEvent::RestartRequestStarted => {
                self.request_started = true;
                Some(5)
            }
            SessionEvent::RestartRequestBytesWritten { count } if *count > 0 => {
                self.request_written = true;
                Some(6)
            }
            SessionEvent::RestartRequestWriteComplete => Some(7),
            SessionEvent::RestartResponseReceived => {
                self.response = true;
                Some(8)
            }
            SessionEvent::ServiceLossObserved => {
                self.service_loss = true;
                Some(9)
            }
            SessionEvent::DeviceAbsent => {
                self.absence = true;
                Some(10)
            }
            SessionEvent::DeviceSample {
                phase: DevicePhase::Recovery,
                ..
            } => {
                self.recovery_samples = self.recovery_samples.saturating_add(1);
                Some(11)
            }
            SessionEvent::ReaderReacquired => {
                self.reader_reacquired = true;
                Some(12)
            }
            SessionEvent::SerialBytes {
                phase: SerialPhase::PostRestart,
                count,
            } if *count > 0 => {
                self.post_serial = true;
                Some(13)
            }
            SessionEvent::BootBObserved { .. } => {
                self.boot_b = true;
                Some(14)
            }
            SessionEvent::ObservationWindowExpired { duration_millis } if *duration_millis > 0 => {
                self.expired = true;
                Some(15)
            }
            SessionEvent::CleanupComplete => {
                self.cleanup = true;
                Some(16)
            }
            SessionEvent::ReaderLost => None,
            _ => return Err(RuntimeIdentityEvidenceError::MissingLedgerStep),
        };
        if let Some(rank) = maybe_rank {
            if self.maybe_rank.is_some_and(|prior| rank < prior) {
                return Err(RuntimeIdentityEvidenceError::MissingLedgerStep);
            }
            self.maybe_rank = Some(rank);
        }
        Ok(())
    }

    fn complete(&self) -> bool {
        self.platform
            && self.initial_samples >= 3
            && self.reader_armed
            && self.pre_serial
            && self.baseline
            && self.request_started
            && self.request_written
            && self.response
            && self.service_loss
            && self.absence
            && self.recovery_samples >= 3
            && self.reader_reacquired
            && self.post_serial
            && self.boot_b
            && self.expired
            && self.cleanup
    }
}
