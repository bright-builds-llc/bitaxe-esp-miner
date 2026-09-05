//! Firmware safety observation and bounded actuation facade.

mod adc;
mod ds4432u;
mod emc2101;
mod i2c_bus;
mod i2c_retry;
mod ina260;
mod observation_store;
mod request_queue;
mod thermal;
mod watchdog;

pub(crate) use ds4432u::Ultra205CoreVoltage;
pub(crate) use i2c_bus::{BitaxeI2cBus, RuntimeI2cOwner};
pub(crate) use observation_store::observation_snapshot;

pub(crate) fn replace_observations_from_producer(observations: bitaxe_api::TelemetryObservations) {
    let now_ms = crate::runtime_uptime::millis();
    crate::production_mining_session::revocation::check_safety(
        observations
            .is_ultra_205_mining_safe_at(bitaxe_safety::observation::MonotonicMillis::new(now_ms)),
        observations
            .fan_rpm
            .maybe_last_good()
            .is_some_and(|sample| *sample.value() > 0),
        now_ms,
    );
    observation_store::replace_observations_from_producer(observations);
}
pub(crate) use watchdog::supervisor_checkpoint_history;

use std::{
    sync::{
        atomic::{AtomicBool, Ordering},
        mpsc::{self, Receiver, SyncSender, TryRecvError},
        OnceLock,
    },
    time::Duration,
};

use anyhow::{anyhow, Result};
use bitaxe_safety::{power::Ina260RawSample, sensor_acquisition::AcquisitionOutcome};

use request_queue::{enqueue, ActuationEnvelope, EnqueueOutcome};

pub(crate) use adc::Ultra205CoreVoltageAdc;
pub(crate) use i2c_retry::{RuntimeI2cBudget, RuntimeI2cBudgetOutcome};

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
    SetFanDutyAfterCoolingProof,
    SetCoreVoltage(Ultra205CoreVoltage),
    SetCoreVoltageForGeneration {
        voltage: Ultra205CoreVoltage,
        permit: crate::production_mining_session::revocation::WorkPermit,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum SafetyActuationRequestOutcome {
    Applied,
    QueueFull,
    OwnerUnavailable,
    ReplyTimedOut,
    HardwareWriteFailed,
}

pub(crate) enum SafetyActuationQueueOutcome {
    Queued(PendingSafetyActuation),
    QueueFull,
    OwnerUnavailable,
}

pub(crate) struct PendingSafetyActuation {
    reply_receiver: Receiver<SafetyActuationReply>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum SafetyActuationPollOutcome {
    Pending,
    Applied,
    OwnerUnavailable,
    HardwareWriteFailed,
}

impl PendingSafetyActuation {
    pub(crate) fn poll(&self) -> SafetyActuationPollOutcome {
        match self.reply_receiver.try_recv() {
            Ok(SafetyActuationReply::Applied) => SafetyActuationPollOutcome::Applied,
            Ok(SafetyActuationReply::HardwareWriteFailed) => {
                SafetyActuationPollOutcome::HardwareWriteFailed
            }
            Err(TryRecvError::Empty) => SafetyActuationPollOutcome::Pending,
            Err(TryRecvError::Disconnected) => {
                ACTUATION_OWNER_AVAILABLE.store(false, Ordering::Release);
                SafetyActuationPollOutcome::OwnerUnavailable
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SafetyActuationReply {
    Applied,
    HardwareWriteFailed,
}

type SafetyActuationEnvelope = ActuationEnvelope<SafetyActuationCommand, SafetyActuationReply>;

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
    match enqueue(request_sender, stamp_safety_command(command), reply_sender) {
        EnqueueOutcome::Queued => {}
        EnqueueOutcome::Full => return SafetyActuationRequestOutcome::QueueFull,
        EnqueueOutcome::Disconnected => {
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

/// Queues an effect whose result is proven by a subsequent fresh observation.
pub(crate) fn queue_safety_actuation(
    command: SafetyActuationCommand,
) -> SafetyActuationQueueOutcome {
    if !safety_actuation_available() {
        return SafetyActuationQueueOutcome::OwnerUnavailable;
    }
    let Some(request_sender) = ACTUATION_REQUEST_SENDER.get() else {
        return SafetyActuationQueueOutcome::OwnerUnavailable;
    };

    let (reply_sender, reply_receiver) = mpsc::sync_channel(ACTUATION_REPLY_CAPACITY);
    match enqueue(request_sender, stamp_safety_command(command), reply_sender) {
        EnqueueOutcome::Queued => {
            SafetyActuationQueueOutcome::Queued(PendingSafetyActuation { reply_receiver })
        }
        EnqueueOutcome::Full => SafetyActuationQueueOutcome::QueueFull,
        EnqueueOutcome::Disconnected => {
            ACTUATION_OWNER_AVAILABLE.store(false, Ordering::Release);
            SafetyActuationQueueOutcome::OwnerUnavailable
        }
    }
}

pub(crate) fn service_next_safety_actuation_request(
    owner: &mut RuntimeI2cOwner<'_>,
    inbox: &SafetyActuationOwnerInbox,
    timeout: Duration,
    sensor_publish_deadline_ms: u64,
) -> SafetyActuationOwnerWait {
    let envelope = match inbox.request_receiver.recv_timeout(timeout) {
        Ok(envelope) => envelope,
        Err(mpsc::RecvTimeoutError::Timeout) => return SafetyActuationOwnerWait::TimedOut,
        Err(mpsc::RecvTimeoutError::Disconnected) => {
            ACTUATION_OWNER_AVAILABLE.store(false, Ordering::Release);
            return SafetyActuationOwnerWait::Disconnected;
        }
    };

    let (command, reply_sender) = envelope.into_parts();
    let started_at_ms = crate::runtime_uptime::millis();
    let mut budget = RuntimeI2cBudget::new(sensor_publish_deadline_ms);
    let reply = apply_safety_actuation(owner, &mut budget, command);
    if let Some(diagnostic) = crate::operator_sensor_diagnostics::record_stage(
        crate::operator_sensor_diagnostics::OperatorSensorStage::Actuation,
        started_at_ms,
        crate::runtime_uptime::millis(),
        operator_sensor_outcome(budget.outcome()),
    ) {
        crate::info_retained(&diagnostic.marker());
    }
    if reply == SafetyActuationReply::HardwareWriteFailed {
        log::warn!("safety_actuation=fault category=hardware_write_failed");
    }
    if reply_sender.try_send(reply).is_err() {
        log::warn!("safety_actuation=fault category=reply_receiver_unavailable");
    }
    SafetyActuationOwnerWait::Serviced
}

fn operator_sensor_outcome(
    outcome: RuntimeI2cBudgetOutcome,
) -> crate::operator_sensor_diagnostics::OperatorSensorOutcome {
    match outcome {
        RuntimeI2cBudgetOutcome::Ready => {
            crate::operator_sensor_diagnostics::OperatorSensorOutcome::Ready
        }
        RuntimeI2cBudgetOutcome::Recovered => {
            crate::operator_sensor_diagnostics::OperatorSensorOutcome::Recovered
        }
        RuntimeI2cBudgetOutcome::DriverFailed => {
            crate::operator_sensor_diagnostics::OperatorSensorOutcome::DriverFailed
        }
        RuntimeI2cBudgetOutcome::BudgetExhausted => {
            crate::operator_sensor_diagnostics::OperatorSensorOutcome::BudgetExhausted
        }
    }
}

fn apply_safety_actuation(
    owner: &mut RuntimeI2cOwner<'_>,
    budget: &mut RuntimeI2cBudget,
    command: SafetyActuationCommand,
) -> SafetyActuationReply {
    let mut bus = owner.actuators(budget);
    let result = match command {
        SafetyActuationCommand::SetFanDuty(percent) => {
            if percent.get() < 100 && !crate::production_mining_session::revocation::permits(None) {
                return SafetyActuationReply::HardwareWriteFailed;
            }
            emc2101::write_fan_duty_percent(&mut bus, percent.get())
        }
        SafetyActuationCommand::SetFanDutyAfterCoolingProof => {
            emc2101::write_fan_duty_percent(&mut bus, FanDutyPercent::PAUSED.get())
        }
        SafetyActuationCommand::SetCoreVoltage(voltage) => {
            if !crate::production_mining_session::revocation::permits(None) {
                return SafetyActuationReply::HardwareWriteFailed;
            }
            ds4432u::write_core_voltage(&mut bus, voltage)
        }
        SafetyActuationCommand::SetCoreVoltageForGeneration { voltage, permit } => {
            if !crate::production_mining_session::revocation::permits_work(permit) {
                return SafetyActuationReply::HardwareWriteFailed;
            }
            ds4432u::write_core_voltage(&mut bus, voltage)
        }
    };
    match result {
        Ok(()) => SafetyActuationReply::Applied,
        Err(_) => SafetyActuationReply::HardwareWriteFailed,
    }
}

fn stamp_safety_command(command: SafetyActuationCommand) -> SafetyActuationCommand {
    match command {
        SafetyActuationCommand::SetCoreVoltage(voltage) => {
            SafetyActuationCommand::SetCoreVoltageForGeneration {
                voltage,
                permit: crate::production_mining_session::revocation::stamp(None),
            }
        }
        other => other,
    }
}

pub(crate) fn read_power_acquisition(
    owner: &mut RuntimeI2cOwner<'_>,
    budget: &mut RuntimeI2cBudget,
) -> AcquisitionOutcome<Ina260RawSample> {
    ina260::read_acquisition(&mut owner.sensors(budget))
}

pub(crate) fn read_asic_temperature_acquisition(
    owner: &mut RuntimeI2cOwner<'_>,
    budget: &mut RuntimeI2cBudget,
) -> AcquisitionOutcome<f64> {
    emc2101::read_ultra205_asic_temperature_acquisition(&mut owner.sensors(budget))
}

pub(crate) fn read_tachometer_acquisition(
    owner: &mut RuntimeI2cOwner<'_>,
    budget: &mut RuntimeI2cBudget,
) -> AcquisitionOutcome<u16> {
    emc2101::read_tachometer_acquisition(&mut owner.sensors(budget))
}

pub(crate) fn read_core_voltage_acquisition(
    adc: &mut Ultra205CoreVoltageAdc,
) -> AcquisitionOutcome<u16> {
    match adc.read_millivolts() {
        Ok(millivolts) => AcquisitionOutcome::Success(millivolts),
        Err(_) => {
            log::warn!("core_voltage_adc=fault category=read_failed");
            AcquisitionOutcome::ReadFailed
        }
    }
}

pub fn start_safety_supervisor() -> std::io::Result<()> {
    watchdog::start_safety_supervisor_thread()
}
