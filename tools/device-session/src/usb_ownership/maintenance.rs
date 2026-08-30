use std::io::Read;
use std::time::{Duration, Instant};

use anyhow::Result;

use super::{handoff_error, inspect_usb_profile, UsbProfile};
use crate::{UsbSession, UsbSessionError, UsbTerminalCategory};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NativeUsbHandoffOutcome {
    pub ready_received: bool,
    pub committed_received: bool,
    pub bus_reset_observed: bool,
    pub profile_counts: super::ProfileObservationCounts,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum MaintenanceControlStep {
    ClearDtr,
    SetBitRate(u32),
    Settle,
    AssertDtr,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum MaintenanceCommitStep {
    ClearDtr,
    AwaitCommitted,
    CloseCdc,
}

pub(super) const fn maintenance_commit_steps() -> [MaintenanceCommitStep; 3] {
    [
        MaintenanceCommitStep::ClearDtr,
        MaintenanceCommitStep::AwaitCommitted,
        MaintenanceCommitStep::CloseCdc,
    ]
}

pub(super) const fn maintenance_control_steps() -> [MaintenanceControlStep; 6] {
    [
        MaintenanceControlStep::ClearDtr,
        MaintenanceControlStep::SetBitRate(115_200),
        MaintenanceControlStep::Settle,
        MaintenanceControlStep::AssertDtr,
        MaintenanceControlStep::Settle,
        MaintenanceControlStep::SetBitRate(1_200),
    ]
}

#[cfg(target_os = "macos")]
fn apply_maintenance_control_step(
    fd: std::os::fd::RawFd,
    dtr: &mut libc::c_int,
    step: MaintenanceControlStep,
) -> Result<(), UsbSessionError> {
    const CONTROL_SETTLE_DURATION: Duration = Duration::from_millis(100);
    let accepted = match step {
        MaintenanceControlStep::ClearDtr => (unsafe { libc::ioctl(fd, libc::TIOCMBIC, dtr) }) == 0,
        MaintenanceControlStep::AssertDtr => (unsafe { libc::ioctl(fd, libc::TIOCMBIS, dtr) }) == 0,
        MaintenanceControlStep::SetBitRate(bit_rate) => {
            let speed = match bit_rate {
                1_200 => libc::B1200,
                115_200 => libc::B115200,
                _ => {
                    return Err(handoff_error(
                        UsbTerminalCategory::HandoffUnsupported,
                        "the maintenance control plan requested an unsupported bit rate",
                    ));
                }
            };
            let mut termios = unsafe { std::mem::zeroed::<libc::termios>() };
            (unsafe { libc::tcgetattr(fd, &mut termios) }) == 0
                && (unsafe { libc::cfsetspeed(&mut termios, speed) }) == 0
                && (unsafe { libc::tcsetattr(fd, libc::TCSANOW, &termios) }) == 0
        }
        MaintenanceControlStep::Settle => {
            std::thread::sleep(CONTROL_SETTLE_DURATION);
            true
        }
    };
    if accepted {
        return Ok(());
    }
    Err(handoff_error(
        UsbTerminalCategory::HandoffUnsupported,
        "the Worker CDC adapter rejected the maintenance control plan",
    ))
}

#[cfg(target_os = "macos")]
pub(super) fn wait_for_maintenance_receipt(
    file: &mut std::fs::File,
    receipt: &[u8],
    timeout: Duration,
    timeout_category: UsbTerminalCategory,
) -> Result<(), UsbSessionError> {
    const MAX_TRANSCRIPT_BYTES: usize = 4_096;
    let deadline = Instant::now() + timeout;
    let mut observed = Vec::new();
    while Instant::now() < deadline {
        let mut chunk = [0_u8; 256];
        match file.read(&mut chunk) {
            Ok(count) if observed.len().saturating_add(count) <= MAX_TRANSCRIPT_BYTES => {
                observed.extend_from_slice(&chunk[..count]);
            }
            Ok(_) => {
                return Err(handoff_error(
                    UsbTerminalCategory::HandoffRejectedUnsafeState,
                    "the maintenance transcript exceeded its fixed bound",
                ));
            }
            Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => {}
            Err(error) if error.kind() == std::io::ErrorKind::Interrupted => continue,
            Err(error) => {
                return Err(handoff_error(
                    UsbTerminalCategory::HandoffRejectedUnsafeState,
                    error,
                ));
            }
        }
        if observed
            .windows(receipt.len())
            .any(|window| window == receipt)
        {
            return Ok(());
        }
        std::thread::sleep(Duration::from_millis(20));
    }
    Err(handoff_error(
        timeout_category,
        "the Worker did not emit the required maintenance receipt",
    ))
}

#[cfg(target_os = "macos")]
fn commit_worker_maintenance(
    file: std::fs::File,
    dtr: &mut libc::c_int,
) -> Result<(), UsbSessionError> {
    use std::os::fd::AsRawFd;

    const COMMITTED_RECEIPT: &[u8] = b"usb_maintenance={\"status\":\"committed\"}";
    let fd = file.as_raw_fd();
    let mut maybe_file = Some(file);
    for step in maintenance_commit_steps() {
        match step {
            MaintenanceCommitStep::ClearDtr => {
                if unsafe { libc::ioctl(fd, libc::TIOCMBIC, std::ptr::from_mut(dtr)) } != 0 {
                    return Err(handoff_error(
                        UsbTerminalCategory::HandoffUnsupported,
                        "the Worker CDC adapter rejected the commit edge",
                    ));
                }
            }
            MaintenanceCommitStep::AwaitCommitted => {
                let Some(file) = maybe_file.as_mut() else {
                    return Err(handoff_error(
                        UsbTerminalCategory::HandoffRejectedUnsafeState,
                        "the Worker CDC adapter closed before commit acknowledgment",
                    ));
                };
                wait_for_maintenance_receipt(
                    file,
                    COMMITTED_RECEIPT,
                    Duration::from_secs(2),
                    UsbTerminalCategory::HandoffCommitTimeout,
                )?;
            }
            MaintenanceCommitStep::CloseCdc => drop(maybe_file.take()),
        }
    }
    Ok(())
}

pub fn handoff_worker_to_rom(
    session: &mut UsbSession,
) -> Result<NativeUsbHandoffOutcome, UsbSessionError> {
    if !cfg!(target_os = "macos") {
        return Err(handoff_error(
            UsbTerminalCategory::HandoffUnsupported,
            "native USB maintenance handoff is qualified only on macOS",
        ));
    }
    let inspection = inspect_usb_profile(session.port())
        .map_err(|error| handoff_error(UsbTerminalCategory::RuntimeProfileUnknown, error))?;
    if inspection.profile != UsbProfile::WorkerRuntime {
        return Err(handoff_error(
            UsbTerminalCategory::HandoffUnsupported,
            "maintenance handoff requires the admitted Worker runtime profile",
        ));
    }
    if inspection.physical_identity_digest != session.physical_identity_digest() {
        return Err(handoff_error(
            UsbTerminalCategory::PhysicalIdentityDrift,
            "the Worker profile does not match the retained USB lease",
        ));
    }
    #[cfg(target_os = "macos")]
    {
        use std::fs::OpenOptions;
        use std::os::fd::AsRawFd;
        use std::os::unix::fs::OpenOptionsExt;

        let mut file = OpenOptions::new()
            .read(true)
            .write(true)
            .custom_flags(libc::O_NOCTTY | libc::O_NONBLOCK)
            .open(session.port())
            .map_err(|error| handoff_error(UsbTerminalCategory::ForeignHolder, error))?;
        let fd = file.as_raw_fd();
        let mut dtr = libc::TIOCM_DTR;
        for step in maintenance_control_steps() {
            if let Err(error) = apply_maintenance_control_step(fd, &mut dtr, step) {
                clear_dtr(fd, &mut dtr);
                return Err(error);
            }
        }
        if let Err(error) = wait_for_maintenance_receipt(
            &mut file,
            b"usb_maintenance={\"status\":\"ready\"}",
            Duration::from_secs(6),
            UsbTerminalCategory::HandoffReadyTimeout,
        ) {
            clear_dtr(fd, &mut dtr);
            return Err(error);
        }
        commit_worker_maintenance(file, &mut dtr)?;
        let profile_counts = session.reacquire_profile(UsbProfile::SerialJtagRuntime)?;
        Ok(NativeUsbHandoffOutcome {
            ready_received: true,
            committed_received: true,
            bus_reset_observed: true,
            profile_counts,
        })
    }
    #[cfg(not(target_os = "macos"))]
    unreachable!()
}

#[cfg(target_os = "macos")]
fn clear_dtr(fd: std::os::fd::RawFd, dtr: &mut libc::c_int) {
    let _result = unsafe { libc::ioctl(fd, libc::TIOCMBIC, dtr) };
}
