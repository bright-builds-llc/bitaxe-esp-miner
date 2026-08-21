//! Sole boot-time hardware self-test owner for SELF-001.

use std::ptr;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Mutex, OnceLock,
};
use std::thread;
use std::time::Duration;

use bitaxe_asic::bm1366::{
    command::Bm1366Command,
    mining_ready::difficulty_mask_value,
    result::Bm1366ValidJobIds,
    work::{Bm1366JobId, Bm1366WorkFields, Bm1366WorkPayload},
};
use bitaxe_core::runtime_health::{
    PassiveSelfTestState, TaskWatchdogObservation, TaskWatchdogOwnerPhase,
    TaskWatchdogOwnerSubphase,
};
use bitaxe_safety::{
    self_test::{
        evaluate_hardware_self_test_metrics, HardwareSelfTestCase, HardwareSelfTestFailure,
        HardwareSelfTestMetrics, HardwareSelfTestSchedule, HardwareSelfTestStage,
        HARDWARE_SELF_TEST_DIFFICULTY, HARDWARE_SELF_TEST_DOMAIN_COUNT,
        HARDWARE_SELF_TEST_FAN_MIN_PERCENT, HARDWARE_SELF_TEST_MAX_C,
        HARDWARE_SELF_TEST_MEASUREMENT_MS, HARDWARE_SELF_TEST_PLANNED_FAILURE_LOAD_MS,
        HARDWARE_SELF_TEST_RESTART_DELAY_MS, HARDWARE_SELF_TEST_TARGET_C,
        HARDWARE_SELF_TEST_WARMUP_C, HARDWARE_SELF_TEST_WARMUP_TIMEOUT_MS,
    },
    thermal::{PidController, PidState, PID_SAMPLE_TIME_MS},
};
use bitaxe_stratum::v1::production_session::{
    HardwareSafeStopPurpose, MiningHardwareProfilePreset,
};
use esp_idf_svc::sys;

use crate::asic_adapter::production::ProductionReadOutcome;
use crate::settings_adapter::{SelfTestAdmission, SelfTestReceipt};

const OWNER_THREAD_NAME: &str = "self-test";
const OWNER_STACK_BYTES: usize = 24 * 1_024;
const RESULT_POLL_MS: u32 = 100;
const WORK_DISPATCH_MS: u64 = 2_000;
const SAFETY_PREFLIGHT_TIMEOUT_MS: u64 = 10_000;

static ACTIVE: AtomicBool = AtomicBool::new(false);
static CANCEL_REQUESTED: AtomicBool = AtomicBool::new(false);
static SNAPSHOT: OnceLock<Mutex<SelfTestRuntimeSnapshot>> = OnceLock::new();

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct SelfTestRuntimeSnapshot {
    pub(crate) lease_present: bool,
    pub(crate) case: Option<HardwareSelfTestCase>,
    pub(crate) stage: Option<HardwareSelfTestStage>,
    pub(crate) maybe_failure: Option<HardwareSelfTestFailure>,
    pub(crate) safe_stop_complete: bool,
    pub(crate) cancel_requested: bool,
}

impl Default for SelfTestRuntimeSnapshot {
    fn default() -> Self {
        Self {
            lease_present: false,
            case: None,
            stage: None,
            maybe_failure: None,
            safe_stop_complete: false,
            cancel_requested: false,
        }
    }
}

pub(crate) fn start(admission: SelfTestAdmission) -> anyhow::Result<()> {
    if ACTIVE.swap(true, Ordering::AcqRel) {
        anyhow::bail!("self-test owner already active");
    }
    publish(SelfTestRuntimeSnapshot {
        lease_present: admission.lease() != 0,
        case: Some(admission.case()),
        stage: Some(HardwareSelfTestStage::Admitted),
        ..SelfTestRuntimeSnapshot::default()
    });
    thread::Builder::new()
        .name(OWNER_THREAD_NAME.to_owned())
        .stack_size(OWNER_STACK_BYTES)
        .spawn(move || run(admission))?;
    Ok(())
}

#[must_use]
pub(crate) fn is_active() -> bool {
    ACTIVE.load(Ordering::Acquire)
}

#[must_use]
pub(crate) fn snapshot() -> SelfTestRuntimeSnapshot {
    let snapshot = SNAPSHOT.get_or_init(|| Mutex::new(SelfTestRuntimeSnapshot::default()));
    snapshot
        .lock()
        .map_or_else(|_| SelfTestRuntimeSnapshot::default(), |snapshot| *snapshot)
}

#[must_use]
pub(crate) fn passive_state() -> PassiveSelfTestState {
    let snapshot = snapshot();
    if !is_active() {
        return PassiveSelfTestState::Idle;
    }
    match snapshot.stage {
        Some(HardwareSelfTestStage::AwaitingCancel) => PassiveSelfTestState::Failed,
        Some(HardwareSelfTestStage::Restarting | HardwareSelfTestStage::Complete)
            if snapshot.maybe_failure.is_none() =>
        {
            PassiveSelfTestState::Passed
        }
        Some(_) => PassiveSelfTestState::Running,
        None => PassiveSelfTestState::Unavailable,
    }
}

pub(crate) fn request_cancel() -> bool {
    let snapshot = snapshot();
    if snapshot.stage != Some(HardwareSelfTestStage::AwaitingCancel) || !snapshot.safe_stop_complete
    {
        return false;
    }
    CANCEL_REQUESTED.store(true, Ordering::Release);
    publish(SelfTestRuntimeSnapshot {
        cancel_requested: true,
        ..snapshot
    });
    true
}

fn run(admission: SelfTestAdmission) {
    let mut watchdog = SelfTestTaskWatchdog::subscribe();
    let mut schedule = HardwareSelfTestSchedule::admitted(crate::runtime_uptime::millis());
    let mut actuation = crate::mining_actuation_adapter::Ultra205MiningActuationAdapter::new();
    let outcome = run_inner(admission, &mut schedule, &mut actuation, &mut watchdog);
    let maybe_failure = outcome.err();
    let safe_stop = safe_stop(&mut actuation, &mut watchdog).is_ok();
    if admission.case() == HardwareSelfTestCase::PlannedFailure || maybe_failure.is_some() {
        await_cancel(admission, maybe_failure, safe_stop, &mut watchdog);
    }
    if !safe_stop {
        await_cancel(
            admission,
            Some(HardwareSelfTestFailure::SafeStopFailed),
            false,
            &mut watchdog,
        );
    }
    publish_stage(admission, HardwareSelfTestStage::Restarting, None, true);
    log::info!("self_test_stage={{\"schema\":\"self-test-runtime-v1\",\"stage\":\"restarting\",\"case\":\"pass\",\"lease_present\":true}}");
    if crate::settings_adapter::clear_self_test_flag_and_record_receipt(
        admission.lease(),
        SelfTestReceipt::Passed,
    )
    .is_err()
    {
        await_cancel(
            admission,
            Some(HardwareSelfTestFailure::SafeStopFailed),
            true,
            &mut watchdog,
        );
    }
    log::info!("self_test_terminal={{\"schema\":\"self-test-runtime-v1\",\"outcome\":\"passed\",\"safe_stop\":true,\"restart_delay_ms\":{HARDWARE_SELF_TEST_RESTART_DELAY_MS}}}");
    bounded_wait(HARDWARE_SELF_TEST_RESTART_DELAY_MS, &mut watchdog);
    unsafe { sys::esp_restart() };
}

fn run_inner(
    admission: SelfTestAdmission,
    schedule: &mut HardwareSelfTestSchedule,
    actuation: &mut crate::mining_actuation_adapter::Ultra205MiningActuationAdapter,
    watchdog: &mut SelfTestTaskWatchdog,
) -> Result<(), HardwareSelfTestFailure> {
    enter(admission, schedule, HardwareSelfTestStage::Preflight, None)?;
    if unsafe { sys::heap_caps_get_total_size(sys::MALLOC_CAP_SPIRAM) } == 0 {
        return Err(HardwareSelfTestFailure::PsramMissing);
    }
    wait_for_safety_preflight(watchdog)?;
    enter(admission, schedule, HardwareSelfTestStage::Preparing, None)?;
    watchdog.feed(TaskWatchdogOwnerSubphase::EffectPrepareHardware);
    actuation
        .prepare(MiningHardwareProfilePreset::UpstreamDefault.profile())
        .map_err(|_| HardwareSelfTestFailure::PreparationFailed)?;
    crate::asic_adapter::production::execute_self_test_command(Bm1366Command::SetDifficultyMask(
        difficulty_mask_value(f64::from(HARDWARE_SELF_TEST_DIFFICULTY)),
    ))
    .map_err(|_| HardwareSelfTestFailure::PreparationFailed)?;

    let mut measurement = DiagnosticMeasurement::new();
    enter(
        admission,
        schedule,
        HardwareSelfTestStage::Warming,
        Some(HARDWARE_SELF_TEST_WARMUP_TIMEOUT_MS),
    )?;
    crate::mining_actuation_adapter::Ultra205MiningActuationAdapter::set_self_test_fan_duty(
        HARDWARE_SELF_TEST_FAN_MIN_PERCENT,
    )
    .map_err(|_| HardwareSelfTestFailure::PreparationFailed)?;
    while current_temperature().ok_or(HardwareSelfTestFailure::SafetyUnavailable)?
        < HARDWARE_SELF_TEST_WARMUP_C
    {
        if schedule.deadline_expired(crate::runtime_uptime::millis())? {
            return Err(HardwareSelfTestFailure::WarmupTimedOut);
        }
        service_diagnostic_work(&mut measurement, watchdog)?;
    }

    let duration_ms = match admission.case() {
        HardwareSelfTestCase::PlannedFailure => HARDWARE_SELF_TEST_PLANNED_FAILURE_LOAD_MS,
        HardwareSelfTestCase::Pass => HARDWARE_SELF_TEST_MEASUREMENT_MS,
    };
    enter(
        admission,
        schedule,
        HardwareSelfTestStage::Measuring,
        Some(duration_ms),
    )?;
    let started_at_ms = crate::runtime_uptime::millis();
    let mut pid = PidController::new(PidState::default());
    while crate::runtime_uptime::millis().saturating_sub(started_at_ms) < duration_ms {
        let temperature =
            current_temperature().ok_or(HardwareSelfTestFailure::SafetyUnavailable)?;
        if temperature > HARDWARE_SELF_TEST_MAX_C {
            return Err(HardwareSelfTestFailure::TemperatureExceeded);
        }
        measurement.maximum_temperature_c = measurement.maximum_temperature_c.max(temperature);
        let step = pid.step(
            f64::from(HARDWARE_SELF_TEST_TARGET_C),
            f64::from(temperature),
            HARDWARE_SELF_TEST_FAN_MIN_PERCENT,
        );
        pid = PidController::new(step.next_state);
        let duty = step.output_percent.round().clamp(10.0, 100.0) as u8;
        crate::mining_actuation_adapter::Ultra205MiningActuationAdapter::set_self_test_fan_duty(
            duty,
        )
        .map_err(|_| HardwareSelfTestFailure::SafetyUnavailable)?;
        service_diagnostic_work(&mut measurement, watchdog)?;
        thread::sleep(Duration::from_millis(u64::from(PID_SAMPLE_TIME_MS)));
    }
    if admission.case() == HardwareSelfTestCase::PlannedFailure {
        return Err(HardwareSelfTestFailure::PlannedEvaluationFailure);
    }

    enter(admission, schedule, HardwareSelfTestStage::Evaluating, None)?;
    let metrics = measurement.finish(started_at_ms)?;
    evaluate_hardware_self_test_metrics(metrics)?;
    Ok(())
}

fn enter(
    admission: SelfTestAdmission,
    schedule: &mut HardwareSelfTestSchedule,
    stage: HardwareSelfTestStage,
    maybe_timeout_ms: Option<u64>,
) -> Result<(), HardwareSelfTestFailure> {
    schedule.enter(stage, crate::runtime_uptime::millis(), maybe_timeout_ms)?;
    publish_stage(admission, stage, None, false);
    log::info!(
        "self_test_stage={{\"schema\":\"self-test-runtime-v1\",\"stage\":\"{}\",\"case\":\"{}\",\"lease_present\":true}}",
        stage.token(),
        admission.case().token()
    );
    Ok(())
}

fn publish_stage(
    admission: SelfTestAdmission,
    stage: HardwareSelfTestStage,
    maybe_failure: Option<HardwareSelfTestFailure>,
    safe_stop_complete: bool,
) {
    publish(SelfTestRuntimeSnapshot {
        lease_present: admission.lease() != 0,
        case: Some(admission.case()),
        stage: Some(stage),
        maybe_failure,
        safe_stop_complete,
        cancel_requested: CANCEL_REQUESTED.load(Ordering::Acquire),
    });
}

fn safe_stop(
    actuation: &mut crate::mining_actuation_adapter::Ultra205MiningActuationAdapter,
    watchdog: &mut SelfTestTaskWatchdog,
) -> Result<(), HardwareSelfTestFailure> {
    let current = snapshot();
    publish(SelfTestRuntimeSnapshot {
        stage: Some(HardwareSelfTestStage::SafeStopping),
        ..current
    });
    log::info!("self_test_stage={{\"schema\":\"self-test-runtime-v1\",\"stage\":\"safe_stopping\",\"lease_present\":true}}");
    actuation
        .safe_stop(HardwareSafeStopPurpose::Terminal, &mut |_| {
            watchdog.feed(TaskWatchdogOwnerSubphase::EffectSafeStopHardware);
        })
        .map_err(|_| HardwareSelfTestFailure::SafeStopFailed)
}

fn await_cancel(
    admission: SelfTestAdmission,
    maybe_failure: Option<HardwareSelfTestFailure>,
    safe_stop_complete: bool,
    watchdog: &mut SelfTestTaskWatchdog,
) -> ! {
    publish_stage(
        admission,
        HardwareSelfTestStage::AwaitingCancel,
        maybe_failure,
        safe_stop_complete,
    );
    log::info!(
        "self_test_checkpoint={{\"schema\":\"self-test-runtime-v1\",\"checkpoint\":\"cancel_ready\",\"safe_state\":{},\"response_required\":false,\"failure\":\"{}\"}}",
        safe_stop_complete,
        maybe_failure.map_or("unavailable", HardwareSelfTestFailure::token)
    );
    loop {
        watchdog.feed(TaskWatchdogOwnerSubphase::SessionEvaluation);
        if CANCEL_REQUESTED.load(Ordering::Acquire)
            && crate::settings_adapter::clear_self_test_flag_and_record_receipt(
                admission.lease(),
                SelfTestReceipt::Cancelled,
            )
            .is_ok()
        {
            log::info!("self_test_terminal={{\"schema\":\"self-test-runtime-v1\",\"outcome\":\"cancelled\",\"safe_stop\":true}}");
            unsafe { sys::esp_restart() };
        }
        thread::sleep(Duration::from_millis(100));
    }
}

fn publish(next: SelfTestRuntimeSnapshot) {
    let snapshot = SNAPSHOT.get_or_init(|| Mutex::new(SelfTestRuntimeSnapshot::default()));
    if let Ok(mut snapshot) = snapshot.lock() {
        *snapshot = next;
    }
}

fn current_temperature() -> Option<f32> {
    let observations = crate::safety_adapter::observation_snapshot();
    if !observations.chip_temp_celsius.is_fresh() {
        return None;
    }
    observations
        .chip_temp_celsius
        .maybe_last_good()
        .map(|sample| *sample.value() as f32)
        .filter(|value| value.is_finite())
}

fn wait_for_safety_preflight(
    watchdog: &mut SelfTestTaskWatchdog,
) -> Result<(), HardwareSelfTestFailure> {
    let deadline = crate::runtime_uptime::millis().saturating_add(SAFETY_PREFLIGHT_TIMEOUT_MS);
    loop {
        let observations = crate::safety_adapter::observation_snapshot();
        if crate::safety_adapter::safety_actuation_available()
            && observations.is_ultra_205_mining_safe_at(
                bitaxe_safety::observation::MonotonicMillis::new(crate::runtime_uptime::millis()),
            )
        {
            return Ok(());
        }
        if crate::runtime_uptime::millis() >= deadline {
            return Err(HardwareSelfTestFailure::SafetyUnavailable);
        }
        watchdog.feed(TaskWatchdogOwnerSubphase::SessionEvaluation);
        thread::sleep(Duration::from_millis(100));
    }
}

fn service_diagnostic_work(
    measurement: &mut DiagnosticMeasurement,
    watchdog: &mut SelfTestTaskWatchdog,
) -> Result<(), HardwareSelfTestFailure> {
    let now_ms = crate::runtime_uptime::millis();
    if now_ms >= measurement.next_dispatch_ms {
        let payload = Bm1366WorkPayload::new(measurement.job_id, deterministic_work_fields());
        crate::asic_adapter::production::execute_self_test_command(
            Bm1366Command::SendDiagnosticWork(payload),
        )
        .map_err(|_| HardwareSelfTestFailure::PreparationFailed)?;
        measurement.next_dispatch_ms = now_ms.saturating_add(WORK_DISPATCH_MS);
    }
    watchdog.feed(TaskWatchdogOwnerSubphase::EffectPollChip);
    match crate::asic_adapter::production::try_read_self_test_result(
        &measurement.valid_jobs,
        RESULT_POLL_MS,
    )
    .map_err(|_| HardwareSelfTestFailure::MeasurementIncomplete)?
    {
        ProductionReadOutcome::JobNonce(result) => measurement.record_nonce(result.small_core_id),
        ProductionReadOutcome::Pending
        | ProductionReadOutcome::Discarded(_)
        | ProductionReadOutcome::RegisterReadProof(_) => {}
    }
    Ok(())
}

fn deterministic_work_fields() -> Bm1366WorkFields {
    Bm1366WorkFields {
        starting_nonce: [0; 4],
        nbits: [0xff; 4],
        ntime: [0x64, 0x70, 0x25, 0xb5],
        merkle_root: [0x5a; 32],
        prev_block_hash: [0xa5; 32],
        version: [0x20, 0, 0, 4],
    }
}

struct DiagnosticMeasurement {
    job_id: Bm1366JobId,
    valid_jobs: Bm1366ValidJobIds,
    next_dispatch_ms: u64,
    nonce_count: u64,
    domain_nonce_counts: [u32; HARDWARE_SELF_TEST_DOMAIN_COUNT],
    maximum_temperature_c: f32,
}

impl DiagnosticMeasurement {
    fn new() -> Self {
        let job_id = Bm1366JobId::new(0);
        Self {
            job_id,
            valid_jobs: Bm1366ValidJobIds::single(job_id),
            next_dispatch_ms: 0,
            nonce_count: 0,
            domain_nonce_counts: [0; HARDWARE_SELF_TEST_DOMAIN_COUNT],
            maximum_temperature_c: f32::MIN,
        }
    }

    fn record_nonce(&mut self, small_core_id: u8) {
        self.nonce_count = self.nonce_count.saturating_add(1);
        let domain = usize::from(small_core_id) % HARDWARE_SELF_TEST_DOMAIN_COUNT;
        self.domain_nonce_counts[domain] = self.domain_nonce_counts[domain].saturating_add(1);
    }

    fn finish(
        self,
        started_at_ms: u64,
    ) -> Result<HardwareSelfTestMetrics, HardwareSelfTestFailure> {
        let measured_ms = crate::runtime_uptime::millis().saturating_sub(started_at_ms);
        if measured_ms == 0 {
            return Err(HardwareSelfTestFailure::MeasurementIncomplete);
        }
        let hash_unit = f64::from(HARDWARE_SELF_TEST_DIFFICULTY) * 4_294_967_296.0;
        let seconds = measured_ms as f64 / 1_000.0;
        let total_hashrate_ghs = (self.nonce_count as f64 * hash_unit / seconds / 1e9) as f32;
        let domain_hashrate_ghs = self
            .domain_nonce_counts
            .map(|count| (f64::from(count) * hash_unit / seconds / 1e9) as f32);
        let observations = crate::safety_adapter::observation_snapshot();
        Ok(HardwareSelfTestMetrics {
            measured_ms,
            total_hashrate_ghs,
            domain_hashrate_ghs,
            domain_sample_counts: self.domain_nonce_counts,
            domain_rejected_counts: [0; HARDWARE_SELF_TEST_DOMAIN_COUNT],
            input_voltage_volts: observation_f64(&observations.bus_voltage_volts)? as f32,
            core_voltage_mv: observation_f64(&observations.core_voltage_actual_mv)?
                .round()
                .clamp(0.0, f64::from(u16::MAX)) as u16,
            power_watts: observation_f64(&observations.power_watts)? as f32,
            fan_rpm: observation_u16(&observations.fan_rpm)?,
            maximum_temperature_c: self.maximum_temperature_c,
        })
    }
}

fn observation_f64(
    observation: &bitaxe_safety::observation::Observation<f64>,
) -> Result<f64, HardwareSelfTestFailure> {
    if !observation.is_fresh() {
        return Err(HardwareSelfTestFailure::SafetyUnavailable);
    }
    observation
        .maybe_last_good()
        .map(|sample| *sample.value())
        .filter(|value| value.is_finite())
        .ok_or(HardwareSelfTestFailure::SafetyUnavailable)
}

fn observation_u16(
    observation: &bitaxe_safety::observation::Observation<u16>,
) -> Result<u16, HardwareSelfTestFailure> {
    if !observation.is_fresh() {
        return Err(HardwareSelfTestFailure::SafetyUnavailable);
    }
    observation
        .maybe_last_good()
        .map(|sample| *sample.value())
        .ok_or(HardwareSelfTestFailure::SafetyUnavailable)
}

fn bounded_wait(duration_ms: u64, watchdog: &mut SelfTestTaskWatchdog) {
    let deadline = crate::runtime_uptime::millis().saturating_add(duration_ms);
    while crate::runtime_uptime::millis() < deadline {
        watchdog.feed(TaskWatchdogOwnerSubphase::SessionEvaluation);
        thread::sleep(Duration::from_millis(100));
    }
}

struct SelfTestTaskWatchdog {
    subscribed: bool,
    sequence: u64,
}

impl SelfTestTaskWatchdog {
    fn subscribe() -> Self {
        crate::task_watchdog_observation::record_owner_phase(TaskWatchdogOwnerPhase::Subscribing);
        let subscribed = unsafe { sys::esp_task_wdt_add(ptr::null_mut()) } == sys::ESP_OK;
        let mut owner = Self {
            subscribed,
            sequence: 0,
        };
        owner.feed(TaskWatchdogOwnerSubphase::SessionEvaluation);
        owner
    }

    fn feed(&mut self, subphase: TaskWatchdogOwnerSubphase) {
        if !self.subscribed || unsafe { sys::esp_task_wdt_reset() } != sys::ESP_OK {
            crate::task_watchdog_observation::record(TaskWatchdogObservation::FeedFailed);
            self.subscribed = false;
            return;
        }
        self.sequence = self.sequence.saturating_add(1);
        crate::task_watchdog_observation::record_owner_progress(
            subphase,
            Some(TaskWatchdogObservation::fed(
                self.sequence,
                crate::runtime_uptime::millis(),
            )),
        );
    }
}

impl Drop for SelfTestTaskWatchdog {
    fn drop(&mut self) {
        if self.subscribed {
            let _ = unsafe { sys::esp_task_wdt_delete(ptr::null_mut()) };
        }
    }
}
