//! The test-only Gamma 601 hash check (ADR-0041 item 9, phase F3.2; `gamma601-hash-check`
//! only).
//!
//! After the shared powered check's power-on and detection (`check_power`), the stage:
//! 1. initializes the BM1370 at `FREQUENCY_MHZ` through the executor, the frequency ramp
//!    included, and waits upstream's 2 s;
//! 2. requires two agreeing, plausible diode readings, or sends no work;
//! 3. sends the genesis header as one job and verifies every result against it in firmware,
//!    for at most `WINDOW_MS` or `VERIFIED_TARGET` verified results.
//! Then the runner holds the reset and turns the output off. Telemetry is judged from the init's
//! start (`HashWindowJudge`). The init runs in segments ending at each delay, so a stop or the
//! on-time cap acts within one ramp step. No pool is involved.
//!
//! Reference: `reference/esp-miner/main/power/asic_init.c` (`asic_initialize`),
//! `components/asic/bm1370.c` (`BM1370_init`) and `main/self_test/self_test.c`.

use super::check_guard;
use super::check_power::{self, aborted, CheckSpec, Ended, Powered, TARGET_MV};
use crate::asic_adapter::production::{self, ProductionReadOutcome};
use crate::safety_adapter::{gamma601 as hardware, RuntimeI2cOwner};
use bitaxe_asic::asic::{AsicFamily, ValidJobIds, WorkPayload};
use bitaxe_asic::bm1366::command::{Bm1366AdapterAction, Bm1366Command};
use bitaxe_asic::bm1366::production::ProductionAsicBlocker;
use bitaxe_asic::bm1370::init::Bm1370InitConfig;
use bitaxe_asic::bm1370::known_answer::{GENESIS_NONCE, GENESIS_WORK_FIELDS};
use bitaxe_asic::bm1370::plan;
use bitaxe_safety::gamma601_hash_check::{
    diode_gate, late, CheckResult, HashCheckRecord, HashReadings, HashSample, HashStop, HashTally,
    HashWindowJudge, VerifiedResult, FREQUENCY_MHZ, HASH_START_CAP_MS, INIT_START_CAP_MS,
    ON_TIME_CAP_MS, WINDOW_MS,
};
use bitaxe_safety::gamma601_safe_boot::SafeBootReport;
use bitaxe_safety::sensor_acquisition::AcquisitionOutcome;
use bitaxe_safety::tps546::StatusWord;
use std::sync::Mutex;
use std::time::Duration;

/// Boot-line literal the package and flash tools look for (ADR-0040's pattern).
pub(super) const VARIANT_MARKER: &str = "firmware_variant=gamma601-hash-check";
const SPEC: CheckSpec = CheckSpec {
    tag: "gamma601_hash_check",
    latch_armed: check_power::HASH_CHECK_ARMED,
    latch_done: 0x4853_4b44,
};
/// Upstream's Gamma init waits this long after the init (`mining_actuation_adapter/gamma601.rs`).
const STABILIZE_MS: u64 = 2_000;
const SAMPLE_MS: u64 = 250;
const RESULT_POLL_MS: u32 = 50;

static RECORD: Mutex<Option<HashCheckRecord>> = Mutex::new(None);

/// Whether this boot's check has started; it never runs twice.
pub(super) struct HashCheck {
    started: bool,
}

impl HashCheck {
    pub(super) const fn pending() -> Self {
        Self { started: false }
    }

    /// Runs the check on the first sweep that `check_power::ready` admits. It blocks the
    /// safe-boot thread for up to `ON_TIME_CAP_MS`. Returns whether it ran, so the caller
    /// re-asserts off.
    pub(super) fn poll(
        &mut self,
        owner: &mut RuntimeI2cOwner<'_>,
        report: &SafeBootReport,
    ) -> bool {
        if !check_power::ready(self.started, report) {
            return false;
        }
        self.started = true;
        log::info!("gamma601_hash_check=started target_mv={TARGET_MV} freq_mhz={FREQUENCY_MHZ}");
        if let Ok(mut slot) = RECORD.lock() {
            let mut running = HashCheckRecord::pending();
            running.power.conclude(CheckResult::Running, "none");
            *slot = Some(running);
        }
        check_guard::publish_window_report(report);
        let record = run_check(owner, *report);
        log::info!("{}", record.usb_marker());
        if let Ok(mut slot) = RECORD.lock() {
            *slot = Some(record);
        }
        true
    }
}

/// The check's latest record, logged with the safe boot's and replayed in turn with it.
pub(super) fn usb_marker() -> String {
    RECORD
        .lock()
        .ok()
        .and_then(|slot| slot.as_ref().map(HashCheckRecord::usb_marker))
        .unwrap_or_else(|| HashCheckRecord::pending().usb_marker())
}

/// What the stage leaves for the record, whatever ends it.
#[derive(Default)]
struct StageOut {
    maybe_diode_gate_c: Option<f64>,
    tally: HashTally,
    maybe_first_result_ms: Option<u32>,
    maybe_window_ms: Option<u32>,
}

// Kept a distinct frame so the build's stack audit can bound it (`firmware-stack-budget.ts`).
#[inline(never)]
fn run_check(owner: &mut RuntimeI2cOwner<'_>, report: SafeBootReport) -> HashCheckRecord {
    let mut maybe_judge = None;
    let mut out = StageOut::default();
    let power = check_power::run_powered(owner, report, &SPEC, |powered| {
        hash_stage(powered, &mut maybe_judge, &mut out)
    });
    HashCheckRecord {
        power,
        freq_mhz: FREQUENCY_MHZ,
        maybe_diode_gate_c: out.maybe_diode_gate_c,
        extremes: maybe_judge
            .map(|judge: HashWindowJudge| judge.extremes())
            .unwrap_or_default(),
        tally: out.tally,
        maybe_first_result_ms: out.maybe_first_result_ms,
        maybe_window_ms: out.maybe_window_ms,
    }
}

fn hash_stage(
    powered: &mut Powered<'_, '_>,
    maybe_judge: &mut Option<HashWindowJudge>,
    out: &mut StageOut,
) -> Result<(), Ended> {
    let init_started_ms = crate::runtime_uptime::millis();
    if late(powered.detected_ms, init_started_ms, INIT_START_CAP_MS) {
        return Err(aborted("init_start_late"));
    }
    let baseline = check_power::observe(powered.owner, powered.report, powered.record)
        .maybe_status
        .ok_or(aborted("controller_unread"))?;
    let judge = maybe_judge.insert(HashWindowJudge::new(
        f64::from(TARGET_MV) / 1000.0,
        crate::board_identity::MINING_SAFETY_LIMITS.max_power_watts(),
        StatusWord(baseline),
    ));
    initialize(powered, judge, init_started_ms)?;
    let diode_first = check_power::read_diode(powered.owner, powered.report);
    std::thread::sleep(Duration::from_millis(SAMPLE_MS));
    let diode_second = check_power::read_diode(powered.owner, powered.report);
    out.maybe_diode_gate_c = diode_first.into_iter().chain(diode_second).reduce(f64::max);
    if diode_gate(diode_first, diode_second).is_none() {
        return Err(aborted("diode_unusable"));
    }
    if late(
        init_started_ms,
        crate::runtime_uptime::millis(),
        HASH_START_CAP_MS,
    ) {
        return Err(aborted("hash_start_late"));
    }
    judge.require_diode(elapsed(init_started_ms, crate::runtime_uptime::millis()));
    hash_window(powered, judge, init_started_ms, out)
}

/// The init and ramp, then upstream's stabilization, judged throughout.
fn initialize(
    powered: &mut Powered<'_, '_>,
    judge: &mut HashWindowJudge,
    init_started_ms: u64,
) -> Result<(), Ended> {
    let actions = plan::mining_ready_actions(Bm1370InitConfig {
        target_frequency_mhz: FREQUENCY_MHZ,
        ..check_power::chip_config()
    })
    .map_err(|_| aborted("plan_invalid"))?;
    let mut maybe_stop = None;
    let mut next_sample_ms = init_started_ms;
    // Each segment ends at a delay, so a stop or the on-time cap acts between ramp steps rather
    // than after the whole ramp. The segments before the last run through the detection entry,
    // which clears production-ready and never sets it; only the last sets it. An external stop
    // that clears it during the init can still be overwritten by that last segment, and the
    // window's own reads then run on.
    let segments: Vec<&[Bm1366AdapterAction]> = actions
        .split_inclusive(|action| matches!(action, Bm1366AdapterAction::DelayMs(_)))
        .collect();
    let last = segments.len().saturating_sub(1);
    for (index, segment) in segments.into_iter().enumerate() {
        let mut progress = || {
            let now_ms = crate::runtime_uptime::millis();
            if maybe_stop.is_none() && now_ms >= next_sample_ms {
                next_sample_ms = now_ms + SAMPLE_MS;
                maybe_stop = sample(powered.owner, powered.report, judge, init_started_ms);
            }
        };
        let executed = if index == last {
            production::execute_mining_ready_actions_guarded_with_progress(
                segment,
                None,
                &mut progress,
            )
        } else {
            production::execute_chip_detection_actions_guarded_with_progress(
                segment,
                None,
                &mut progress,
            )
        };
        if let Err(blocker) = executed {
            powered.record.maybe_blocker = Some(blocker.as_str());
            return Err(aborted("init_failed"));
        }
        if let Some(stop) = maybe_stop {
            return Err(aborted(stop.label()));
        }
        within_on_time_cap(powered)?;
    }
    let stabilized_ms = crate::runtime_uptime::millis().saturating_add(STABILIZE_MS);
    while crate::runtime_uptime::millis() < stabilized_ms {
        std::thread::sleep(Duration::from_millis(SAMPLE_MS));
        if let Some(stop) = sample(powered.owner, powered.report, judge, init_started_ms) {
            return Err(aborted(stop.label()));
        }
        within_on_time_cap(powered)?;
    }
    Ok(())
}

fn within_on_time_cap(powered: &Powered<'_, '_>) -> Result<(), Ended> {
    if late(
        powered.started_ms,
        crate::runtime_uptime::millis(),
        ON_TIME_CAP_MS,
    ) {
        Err(aborted("on_time_cap"))
    } else {
        Ok(())
    }
}

/// One genesis job, its results verified as they arrive, and a sample every `SAMPLE_MS`.
fn hash_window(
    powered: &mut Powered<'_, '_>,
    judge: &mut HashWindowJudge,
    init_started_ms: u64,
    out: &mut StageOut,
) -> Result<(), Ended> {
    let job_id = AsicFamily::Bm1370.first_job_id();
    let valid = ValidJobIds::single(job_id);
    production::execute_self_test_command(Bm1366Command::SendDiagnosticWork(WorkPayload::new(
        job_id,
        GENESIS_WORK_FIELDS,
    )))
    .map_err(|blocker| stage_blocker(powered, blocker, "job_send_failed"))?;
    let sent_ms = crate::runtime_uptime::millis();
    let mut next_sample_ms = sent_ms + SAMPLE_MS;
    loop {
        let now_ms = crate::runtime_uptime::millis();
        let elapsed_ms = elapsed(sent_ms, now_ms);
        out.maybe_window_ms = Some(elapsed_ms);
        if elapsed_ms >= WINDOW_MS || out.tally.target_reached() {
            break;
        }
        within_on_time_cap(powered)?;
        match production::try_read_self_test_result(&valid, RESULT_POLL_MS) {
            Ok(ProductionReadOutcome::Pending) => {}
            Ok(ProductionReadOutcome::JobNonce(result)) => {
                let check = bitaxe_stratum::v1::verify_asic_result(&GENESIS_WORK_FIELDS, result);
                out.tally.observe(VerifiedResult {
                    nonce: result.nonce,
                    version_bits: result.version_bits,
                    difficulty: check.difficulty,
                    meets_ticket: check.meets_ticket,
                    genesis: result.nonce == GENESIS_NONCE && result.version_bits == 0,
                });
                if check.meets_ticket && out.maybe_first_result_ms.is_none() {
                    out.maybe_first_result_ms =
                        Some(elapsed(sent_ms, crate::runtime_uptime::millis()));
                }
            }
            Ok(
                ProductionReadOutcome::Discarded(_) | ProductionReadOutcome::RegisterReadProof(_),
            ) => {
                out.tally.discard();
            }
            Err(blocker) => return Err(stage_blocker(powered, blocker, "result_read_failed")),
        }
        if crate::runtime_uptime::millis() >= next_sample_ms {
            next_sample_ms += SAMPLE_MS;
            if let Some(stop) = sample(powered.owner, powered.report, judge, init_started_ms) {
                return Err(aborted(stop.label()));
            }
        }
    }
    if out.tally.passed() {
        Ok(())
    } else {
        Err(aborted("pass_rule_failed"))
    }
}

/// Records the executor's blocker. A lost `production_ready`, such as a production session's
/// stop clearing it, reads as its own reason.
fn stage_blocker(
    powered: &mut Powered<'_, '_>,
    blocker: ProductionAsicBlocker,
    reason: &'static str,
) -> Ended {
    powered.record.maybe_blocker = Some(blocker.as_str());
    if blocker == ProductionAsicBlocker::AsicInitFailed {
        aborted("production_ready_lost")
    } else {
        aborted(reason)
    }
}

/// Reads the controller and the thermal chip, and judges the sample.
fn sample(
    owner: &mut RuntimeI2cOwner<'_>,
    report: SafeBootReport,
    judge: &mut HashWindowJudge,
    init_started_ms: u64,
) -> Option<HashStop> {
    let mut reading = report;
    hardware::read_controller(owner, &mut reading);
    let maybe_diode_c = match hardware::read_thermal(owner, &mut reading).chip_temperature {
        AcquisitionOutcome::Success(celsius) => Some(celsius),
        _ => None,
    };
    judge.observe(HashSample {
        elapsed_ms: elapsed(init_started_ms, crate::runtime_uptime::millis()),
        maybe_readings: readings(&reading, maybe_diode_c),
    })
}

/// The judge's reading, when the controller answered every read.
fn readings(reading: &SafeBootReport, maybe_diode_c: Option<f64>) -> Option<HashReadings> {
    Some(HashReadings {
        vout_volts: reading.maybe_vout_volts?,
        vin_volts: reading.maybe_vin_volts?,
        iout_amps: reading.maybe_iout_amps?,
        vr_temp_c: reading.maybe_vr_temperature_celsius?,
        maybe_diode_c,
        maybe_fan_rpm: reading.maybe_fan_rpm,
        status: StatusWord(reading.maybe_status?),
    })
}

#[allow(clippy::cast_possible_truncation)]
fn elapsed(since_ms: u64, now_ms: u64) -> u32 {
    now_ms.saturating_sub(since_ms).min(u64::from(u32::MAX)) as u32
}
