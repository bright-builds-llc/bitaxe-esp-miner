//! Firmware safety observation and bounded actuation facade.

mod ds4432u;
mod emc2101;
mod i2c_bus;
mod ina260;
mod observation_store;
mod thermal;
mod watchdog;

pub(crate) use ds4432u::Ultra205CoreVoltage;
pub(crate) use i2c_bus::{BitaxeI2cBus, RuntimeI2cOwner};
pub(crate) use observation_store::{observation_snapshot, replace_observations_from_producer};
pub(crate) use watchdog::supervisor_checkpoint_history;

use std::{
    sync::{
        atomic::{AtomicBool, Ordering},
        mpsc::{self, Receiver, SyncSender, TrySendError},
        OnceLock,
    },
    time::Duration,
};

use anyhow::{anyhow, Result};
use bitaxe_safety::{power::Ina260RawSample, sensor_acquisition::AcquisitionOutcome};

const ACTUATION_REQUEST_CAPACITY: usize = 4;
const ACTUATION_REPLY_CAPACITY: usize = 1;
const ACTUATION_REPLY_TIMEOUT: Duration = Duration::from_millis(1_500);

static ACTUATION_REQUEST_SENDER: OnceLock<SyncSender<SafetyActuationEnvelope>> = OnceLock::new();
static ACTUATION_OWNER_AVAILABLE: AtomicBool = AtomicBool::new(false);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct FanDutyPercent(u8);

impl FanDutyPercent {
    pub(crate) const FULL: Self = Self(100);
    pub(crate) const PAUSED: Self = Self(30);

    const fn get(self) -> u8 {
        self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct FanDutyPercentOutOfRange;

impl TryFrom<u8> for FanDutyPercent {
    type Error = FanDutyPercentOutOfRange;

    fn try_from(percent: u8) -> Result<Self, Self::Error> {
        if percent > 100 {
            return Err(FanDutyPercentOutOfRange);
        }
        Ok(Self(percent))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum SafetyActuationCommand {
    SetFanDuty(FanDutyPercent),
    SetCoreVoltage(Ultra205CoreVoltage),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum SafetyActuationRequestOutcome {
    Applied,
    QueueFull,
    OwnerUnavailable,
    ReplyTimedOut,
    HardwareWriteFailed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SafetyActuationReply {
    Applied,
    HardwareWriteFailed,
}

struct SafetyActuationEnvelope {
    command: SafetyActuationCommand,
    reply_sender: SyncSender<SafetyActuationReply>,
}

pub(crate) struct SafetyActuationOwnerRegistration {
    request_sender: SyncSender<SafetyActuationEnvelope>,
}

pub(crate) struct SafetyActuationOwnerInbox {
    request_receiver: Receiver<SafetyActuationEnvelope>,
}

impl Drop for SafetyActuationOwnerInbox {
    fn drop(&mut self) {
        ACTUATION_OWNER_AVAILABLE.store(false, Ordering::Release);
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum SafetyActuationOwnerWait {
    Serviced,
    TimedOut,
    Disconnected,
}

pub(crate) fn prepare_safety_actuation_owner(
) -> (SafetyActuationOwnerRegistration, SafetyActuationOwnerInbox) {
    let (request_sender, request_receiver) = mpsc::sync_channel(ACTUATION_REQUEST_CAPACITY);
    (
        SafetyActuationOwnerRegistration { request_sender },
        SafetyActuationOwnerInbox { request_receiver },
    )
}

pub(crate) fn publish_safety_actuation_owner(
    registration: SafetyActuationOwnerRegistration,
) -> Result<()> {
    ACTUATION_REQUEST_SENDER
        .set(registration.request_sender)
        .map_err(|_| anyhow!("safety actuation owner already published"))?;
    ACTUATION_OWNER_AVAILABLE.store(true, Ordering::Release);
    Ok(())
}

pub(crate) fn safety_actuation_available() -> bool {
    ACTUATION_OWNER_AVAILABLE.load(Ordering::Acquire) && ACTUATION_REQUEST_SENDER.get().is_some()
}

pub(crate) fn request_safety_actuation(
    command: SafetyActuationCommand,
) -> SafetyActuationRequestOutcome {
    if !safety_actuation_available() {
        return SafetyActuationRequestOutcome::OwnerUnavailable;
    }
    let Some(request_sender) = ACTUATION_REQUEST_SENDER.get() else {
        return SafetyActuationRequestOutcome::OwnerUnavailable;
    };
    let (reply_sender, reply_receiver) = mpsc::sync_channel(ACTUATION_REPLY_CAPACITY);
    let envelope = SafetyActuationEnvelope {
        command,
        reply_sender,
    };

    match request_sender.try_send(envelope) {
        Ok(()) => {}
        Err(TrySendError::Full(_)) => return SafetyActuationRequestOutcome::QueueFull,
        Err(TrySendError::Disconnected(_)) => {
            ACTUATION_OWNER_AVAILABLE.store(false, Ordering::Release);
            return SafetyActuationRequestOutcome::OwnerUnavailable;
        }
    }

    match reply_receiver.recv_timeout(ACTUATION_REPLY_TIMEOUT) {
        Ok(SafetyActuationReply::Applied) => SafetyActuationRequestOutcome::Applied,
        Ok(SafetyActuationReply::HardwareWriteFailed) => {
            SafetyActuationRequestOutcome::HardwareWriteFailed
        }
        Err(mpsc::RecvTimeoutError::Timeout) => SafetyActuationRequestOutcome::ReplyTimedOut,
        Err(mpsc::RecvTimeoutError::Disconnected) => {
            ACTUATION_OWNER_AVAILABLE.store(false, Ordering::Release);
            SafetyActuationRequestOutcome::OwnerUnavailable
        }
    }
}

pub(crate) fn service_next_safety_actuation_request(
    owner: &mut RuntimeI2cOwner<'_>,
    inbox: &SafetyActuationOwnerInbox,
    timeout: Duration,
) -> SafetyActuationOwnerWait {
    let envelope = match inbox.request_receiver.recv_timeout(timeout) {
        Ok(envelope) => envelope,
        Err(mpsc::RecvTimeoutError::Timeout) => return SafetyActuationOwnerWait::TimedOut,
        Err(mpsc::RecvTimeoutError::Disconnected) => {
            ACTUATION_OWNER_AVAILABLE.store(false, Ordering::Release);
            return SafetyActuationOwnerWait::Disconnected;
        }
    };

    let reply = apply_safety_actuation(owner, envelope.command);
    if reply == SafetyActuationReply::HardwareWriteFailed {
        log::warn!("safety_actuation=fault category=hardware_write_failed");
    }
    if envelope.reply_sender.try_send(reply).is_err() {
        log::warn!("safety_actuation=fault category=reply_receiver_unavailable");
    }
    SafetyActuationOwnerWait::Serviced
}

fn apply_safety_actuation(
    owner: &mut RuntimeI2cOwner<'_>,
    command: SafetyActuationCommand,
) -> SafetyActuationReply {
    let mut bus = owner.actuators();
    let result = match command {
        SafetyActuationCommand::SetFanDuty(percent) => {
            emc2101::write_fan_duty_percent(&mut bus, percent.get())
        }
        SafetyActuationCommand::SetCoreVoltage(voltage) => {
            ds4432u::write_core_voltage(&mut bus, voltage)
        }
    };
    match result {
        Ok(()) => SafetyActuationReply::Applied,
        Err(_) => SafetyActuationReply::HardwareWriteFailed,
    }
}

pub(crate) fn read_power_acquisition(
    owner: &mut RuntimeI2cOwner<'_>,
) -> AcquisitionOutcome<Ina260RawSample> {
    ina260::read_acquisition(&mut owner.sensors())
}

pub(crate) fn read_asic_temperature_acquisition(
    owner: &mut RuntimeI2cOwner<'_>,
) -> AcquisitionOutcome<f64> {
    emc2101::read_ultra205_asic_temperature_acquisition(&mut owner.sensors())
}

pub(crate) fn read_tachometer_acquisition(
    owner: &mut RuntimeI2cOwner<'_>,
) -> AcquisitionOutcome<u16> {
    emc2101::read_tachometer_acquisition(&mut owner.sensors())
}

pub fn start_safety_supervisor() {
    if let Err(error) = watchdog::start_safety_supervisor_thread() {
        log::warn!("safety_supervisor=unavailable reason=spawn_failed error={error}");
    }
}
