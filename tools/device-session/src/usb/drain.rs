//! Receive-only stale-frame disposal. Payload bytes never leave the stack buffer.
use super::*;
use std::io;
use std::time::{Duration, Instant};

const MAX_BYTES: usize = 65 * 1024;
const MAX_DURATION: Duration = Duration::from_secs(2);

#[derive(Debug, Clone, Copy, Serialize)]
pub struct SerialDrainMetadata {
    pub discarded_bytes: usize,
    pub elapsed_milliseconds: u128,
}
struct DiscardBuffer([u8; 1024]);
impl DiscardBuffer {
    fn clear(&mut self) {
        for byte in &mut self.0 {
            // Volatile stores prevent the compiler from eliding secret-byte disposal.
            unsafe { std::ptr::write_volatile(byte, 0) };
        }
    }
}
impl Drop for DiscardBuffer {
    fn drop(&mut self) {
        self.clear();
    }
}
pub(crate) fn discard(
    mut read: impl FnMut(&mut [u8]) -> io::Result<usize>,
    duration: Duration,
) -> io::Result<SerialDrainMetadata> {
    let started = Instant::now();
    let mut count = 0;
    let mut buffer = DiscardBuffer([0; 1024]);
    while started.elapsed() < duration && count < MAX_BYTES {
        let capacity = buffer.0.len().min(MAX_BYTES - count);
        let result = read(&mut buffer.0[..capacity]);
        buffer.clear();
        match result {
            Ok(0) => return Err(io::Error::from(io::ErrorKind::UnexpectedEof)),
            Ok(bytes) if bytes <= capacity => count += bytes,
            Ok(_) => return Err(io::Error::from(io::ErrorKind::InvalidData)),
            Err(error) if error.kind() == io::ErrorKind::WouldBlock => {
                std::thread::sleep(
                    Duration::from_millis(1).min(duration.saturating_sub(started.elapsed())),
                );
            }
            Err(error) if error.kind() == io::ErrorKind::Interrupted => {}
            Err(error) => return Err(error),
        }
    }
    Ok(SerialDrainMetadata {
        discarded_bytes: count,
        elapsed_milliseconds: started.elapsed().as_millis(),
    })
}
fn admit_profile(profile: crate::UsbProfile, same_identity: bool) -> Result<(), UsbSessionError> {
    if profile != crate::UsbProfile::SerialJtagRuntime || !same_identity {
        return Err(session_error(
            UsbTerminalCategory::RuntimeProfileUnknown,
            "drain requires the leased Serial/JTAG profile",
        ));
    }
    Ok(())
}
impl UsbSession {
    /// Discards at most 65 KiB for at most two seconds without TX, reset or a transcript.
    pub fn drain_worker_serial(&mut self) -> Result<SerialDrainMetadata, UsbSessionError> {
        let result = self.drain_worker_serial_inner();
        if let Err(error) = &result {
            self.fail_once(error.category);
        }
        result
    }
    fn drain_worker_serial_inner(&mut self) -> Result<SerialDrainMetadata, UsbSessionError> {
        let _signal_supervisor = process::SignalSupervisor::acquire()?;
        let snapshot = self.reacquire(RecoveryPhase::MonitorAdmission)?;
        let profile = crate::inspect_usb_profile(&snapshot.port).map_err(|_| {
            session_error(
                UsbTerminalCategory::RuntimeProfileUnknown,
                "drain profile inspection failed",
            )
        })?;
        admit_profile(
            profile.profile,
            profile.physical_identity_digest == self.physical_identity_digest,
        )?;
        self.transition(UsbLifecycleEvent::BeginObservation)?;
        let mut reader = crate::macos::ReceiveOnlyReader::open(&snapshot.port).map_err(|_| {
            session_error(
                UsbTerminalCategory::MonitorFailed,
                "drain reader open failed",
            )
        })?;
        let result = discard(
            |buffer| {
                if process::maybe_pending_signal().is_some() {
                    return Err(io::Error::other("drain interrupted"));
                }
                reader.read_into(buffer)
            },
            MAX_DURATION,
        );
        drop(reader);
        let completion = self.complete_drain_observation();
        match result {
            Err(_) => Err(session_error(
                UsbTerminalCategory::MonitorFailed,
                "drain read failed",
            )),
            Ok(metadata) => completion.map(|()| metadata),
        }
    }
    fn complete_drain_observation(&mut self) -> Result<(), UsbSessionError> {
        self.transition(UsbLifecycleEvent::ObservationComplete)?;
        let after = crate::inspect_usb_profile(self.port()).map_err(|_| {
            session_error(
                UsbTerminalCategory::RuntimeProfileUnknown,
                "drain final profile unavailable",
            )
        })?;
        if after.profile != crate::UsbProfile::SerialJtagRuntime
            || after.physical_identity_digest != self.physical_identity_digest
        {
            return Err(session_error(
                UsbTerminalCategory::PhysicalIdentityDrift,
                "drain final identity changed",
            ));
        }
        Ok(())
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn continuous_bytes_stop_at_exact_wire_bound_and_buffer_is_wiped() {
        let mut reads = 0;
        let result = discard(
            |buffer| {
                assert!(buffer.iter().all(|byte| *byte == 0));
                buffer.fill(0xa5);
                reads += 1;
                Ok(buffer.len())
            },
            MAX_DURATION,
        )
        .expect("bounded drain");
        assert_eq!(result.discarded_bytes, 66560);
        assert_eq!(reads, 65);
    }
    #[test]
    fn silence_is_time_bounded_and_errors_are_not_success() {
        let started = Instant::now();
        let result = discard(
            |_| Err(io::ErrorKind::WouldBlock.into()),
            Duration::from_millis(10),
        )
        .expect("silent drain");
        assert_eq!(result.discarded_bytes, 0);
        assert!(started.elapsed() < Duration::from_secs(1));
        assert!(discard(
            |_| Err(io::ErrorKind::PermissionDenied.into()),
            MAX_DURATION
        )
        .is_err());
        assert!(discard(|_| Ok(0), MAX_DURATION).is_err());
    }
    #[test]
    fn rom_unknown_and_changed_identity_cannot_admit_a_reader() {
        for profile in [
            crate::UsbProfile::RomDownloader,
            crate::UsbProfile::Unknown,
            crate::UsbProfile::WorkerRuntime,
        ] {
            assert!(admit_profile(profile, true).is_err());
        }
        assert!(admit_profile(crate::UsbProfile::SerialJtagRuntime, false).is_err());
        assert!(admit_profile(crate::UsbProfile::SerialJtagRuntime, true).is_ok());
    }
    #[test]
    fn interruption_wipes_before_retry_and_read_error_stays_visible() {
        let mut calls = 0;
        let result = discard(
            |buffer| {
                assert!(buffer.iter().all(|byte| *byte == 0));
                buffer.fill(0xa5);
                calls += 1;
                Err(if calls == 1 {
                    io::ErrorKind::Interrupted
                } else {
                    io::ErrorKind::PermissionDenied
                }
                .into())
            },
            MAX_DURATION,
        );
        assert_eq!(
            result.expect_err("read failure").kind(),
            io::ErrorKind::PermissionDenied
        );
    }
}
