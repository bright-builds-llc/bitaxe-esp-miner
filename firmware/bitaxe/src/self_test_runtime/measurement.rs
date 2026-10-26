//! The self-test's diagnostic work and its measurement: nonces, domain counters, the regulator's
//! facts and the metrics the evaluation judges.

use bitaxe_asic::bm1366::{
    command::Bm1366Command,
    result::{Bm1366RegisterRead, Bm1366ValidJobIds},
    work::{Bm1366JobId, Bm1366WorkFields, Bm1366WorkPayload},
};
use bitaxe_core::runtime_health::TaskWatchdogOwnerSubphase;
use bitaxe_safety::self_test::{
    HardwareSelfTestContract, HardwareSelfTestFailure, HardwareSelfTestMetrics,
    HARDWARE_SELF_TEST_DIFFICULTY,
};

use crate::asic_adapter::production::ProductionReadOutcome;

use super::domain_measurement::DomainMeasurement;
use super::{board, SelfTestTaskWatchdog, REGISTER_POLL_MS, RESULT_POLL_MS, WORK_DISPATCH_MS};

pub(super) fn service_diagnostic_work(
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
    if now_ms >= measurement.next_register_poll_ms {
        if !crate::asic_adapter::production::request_hashrate_monitor_register_reads_tx() {
            return Err(HardwareSelfTestFailure::MeasurementIncomplete);
        }
        measurement.next_register_poll_ms = now_ms.saturating_add(REGISTER_POLL_MS);
    }
    watchdog.feed(TaskWatchdogOwnerSubphase::EffectPollChip);
    match crate::asic_adapter::production::try_read_self_test_result(
        &measurement.valid_jobs,
        RESULT_POLL_MS,
    )
    .map_err(|_| HardwareSelfTestFailure::MeasurementIncomplete)?
    {
        ProductionReadOutcome::JobNonce(_) => measurement.record_nonce(),
        ProductionReadOutcome::RegisterReadProof(read) => {
            measurement.record_register(read, now_ms)?;
        }
        ProductionReadOutcome::Pending | ProductionReadOutcome::Discarded(_) => {}
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

pub(super) struct DiagnosticMeasurement {
    job_id: Bm1366JobId,
    valid_jobs: Bm1366ValidJobIds,
    next_dispatch_ms: u64,
    next_register_poll_ms: u64,
    nonce_count: u64,
    domains: DomainMeasurement,
    pub(super) maximum_temperature_c: f32,
    maybe_maximum_vr_temperature_c: Option<f32>,
    regulator_fault: bool,
}

impl DiagnosticMeasurement {
    pub(super) fn new(contract: HardwareSelfTestContract) -> Self {
        let job_id: Bm1366JobId = board::first_job_id();
        Self {
            job_id,
            valid_jobs: Bm1366ValidJobIds::single(job_id),
            next_dispatch_ms: 0,
            next_register_poll_ms: 0,
            nonce_count: 0,
            domains: DomainMeasurement::new(contract),
            maximum_temperature_c: f32::MIN,
            maybe_maximum_vr_temperature_c: None,
            regulator_fault: false,
        }
    }

    /// Notes the regulator's temperature and fault; a power fault, or a regulator above its
    /// ceiling, stops the self-test at once, as the chip's temperature does.
    pub(super) fn observe_regulator(
        &mut self,
        contract: &HardwareSelfTestContract,
    ) -> Result<(), HardwareSelfTestFailure> {
        let facts = board::regulator_facts(&crate::safety_adapter::observation_snapshot());
        self.regulator_fault |= facts.fault;
        if let Some(temperature) = facts.maybe_vr_temperature_c {
            self.maybe_maximum_vr_temperature_c = Some(
                self.maybe_maximum_vr_temperature_c
                    .map_or(temperature, |maximum| maximum.max(temperature)),
            );
        }
        let too_hot = contract.maybe_vr_temperature_max_c.is_some_and(|maximum| {
            facts
                .maybe_vr_temperature_c
                .is_some_and(|temperature| temperature > maximum)
        });
        if too_hot {
            return Err(HardwareSelfTestFailure::VrTemperatureExceeded);
        }
        if contract.checks_regulator_faults && facts.fault {
            return Err(HardwareSelfTestFailure::RegulatorFault);
        }
        Ok(())
    }

    fn record_nonce(&mut self) {
        self.nonce_count = self.nonce_count.saturating_add(1);
    }

    fn record_register(
        &mut self,
        read: Bm1366RegisterRead,
        now_ms: u64,
    ) -> Result<(), HardwareSelfTestFailure> {
        self.domains.record(read, now_ms)
    }

    pub(super) fn finish(
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
        let (domain_hashrate_ghs, domain_sample_counts, domain_rejected_counts) =
            self.domains.finish();
        let observations = crate::safety_adapter::observation_snapshot();
        Ok(HardwareSelfTestMetrics {
            measured_ms,
            total_hashrate_ghs,
            domain_hashrate_ghs,
            domain_sample_counts,
            domain_rejected_counts,
            input_voltage_volts: observation_f64(&observations.bus_voltage_volts)? as f32,
            core_voltage_mv: observation_f64(&observations.core_voltage_actual_mv)?
                .round()
                .clamp(0.0, f64::from(u16::MAX)) as u16,
            power_watts: observation_f64(&observations.power_watts)? as f32,
            fan_rpm: observation_u16(&observations.fan_rpm)?,
            maximum_temperature_c: self.maximum_temperature_c,
            maybe_maximum_vr_temperature_c: self.maybe_maximum_vr_temperature_c,
            regulator_fault: self.regulator_fault,
        })
    }
}

pub(super) fn log_self_test_metrics(metrics: &HardwareSelfTestMetrics) {
    log::info!(
        "self_test_metrics={{\"schema\":\"self-test-metrics-v1\",\"measured_ms\":{},\"total_ghs\":{:.3},\"domain_ghs\":[{:.3},{:.3},{:.3},{:.3}],\"domain_samples\":[{},{},{},{}],\"domain_rejected\":[{},{},{},{}],\"input_volts\":{:.3},\"core_mv\":{},\"power_watts\":{:.3},\"fan_rpm\":{},\"maximum_temperature_c\":{:.2}}}",
        metrics.measured_ms,
        metrics.total_hashrate_ghs,
        metrics.domain_hashrate_ghs[0],
        metrics.domain_hashrate_ghs[1],
        metrics.domain_hashrate_ghs[2],
        metrics.domain_hashrate_ghs[3],
        metrics.domain_sample_counts[0],
        metrics.domain_sample_counts[1],
        metrics.domain_sample_counts[2],
        metrics.domain_sample_counts[3],
        metrics.domain_rejected_counts[0],
        metrics.domain_rejected_counts[1],
        metrics.domain_rejected_counts[2],
        metrics.domain_rejected_counts[3],
        metrics.input_voltage_volts,
        metrics.core_voltage_mv,
        metrics.power_watts,
        metrics.fan_rpm,
        metrics.maximum_temperature_c,
    );
}

/// The regulator facts beside the shared metrics line, on a board whose regulator reports them;
/// the 205's metrics line stays as its verified evidence parses it.
pub(super) fn log_regulator_metrics(
    contract: &HardwareSelfTestContract,
    metrics: &HardwareSelfTestMetrics,
) {
    if contract.maybe_vr_temperature_max_c.is_none() && !contract.checks_regulator_faults {
        return;
    }
    log::info!(
        "self_test_regulator_metrics={{\"schema\":\"self-test-regulator-metrics-v1\",\"kind\":\"{}\",\"maximum_vr_temperature_c\":{},\"regulator_fault\":{}}}",
        contract.kind,
        metrics
            .maybe_maximum_vr_temperature_c
            .map_or_else(|| "null".to_owned(), |value| format!("{value:.2}")),
        metrics.regulator_fault,
    );
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
