use bitaxe_asic::bm1366::{registers::Bm1366Register, result::Bm1366RegisterRead};
use bitaxe_core::hashrate::{HashrateMonitor, HashrateObservationOutcome, HashrateRegister};
use bitaxe_safety::self_test::{
    expected_hardware_self_test_domain_hashrate_ghs, HardwareSelfTestFailure,
    HARDWARE_SELF_TEST_DOMAIN_COUNT,
};

const DOMAIN_MAX_PLAUSIBLE_MULTIPLIER: f64 = 3.0;

pub(super) struct DomainMeasurement {
    monitor: HashrateMonitor,
    hashrate_sums: [f64; HARDWARE_SELF_TEST_DOMAIN_COUNT],
    sample_counts: [u32; HARDWARE_SELF_TEST_DOMAIN_COUNT],
    rejected_counts: [u32; HARDWARE_SELF_TEST_DOMAIN_COUNT],
}

impl DomainMeasurement {
    pub(super) fn new() -> Self {
        Self {
            monitor: HashrateMonitor::new(1, HARDWARE_SELF_TEST_DOMAIN_COUNT),
            hashrate_sums: [0.0; HARDWARE_SELF_TEST_DOMAIN_COUNT],
            sample_counts: [0; HARDWARE_SELF_TEST_DOMAIN_COUNT],
            rejected_counts: [0; HARDWARE_SELF_TEST_DOMAIN_COUNT],
        }
    }

    pub(super) fn record(
        &mut self,
        read: Bm1366RegisterRead,
        now_ms: u64,
    ) -> Result<(), HardwareSelfTestFailure> {
        let (register, maybe_domain) = match read.register {
            Bm1366Register::Domain0Count => (HashrateRegister::DomainCount(0), Some(0)),
            Bm1366Register::Domain1Count => (HashrateRegister::DomainCount(1), Some(1)),
            Bm1366Register::Domain2Count => (HashrateRegister::DomainCount(2), Some(2)),
            Bm1366Register::Domain3Count => (HashrateRegister::DomainCount(3), Some(3)),
            Bm1366Register::TotalCount => (HashrateRegister::TotalCount, None),
            Bm1366Register::ErrorCount => (HashrateRegister::ErrorCount, None),
            Bm1366Register::ChipId => return Ok(()),
        };
        let timestamp_us = now_ms
            .checked_mul(1_000)
            .ok_or(HardwareSelfTestFailure::DeadlineOverflow)?;
        let outcome = self
            .monitor
            .observe(
                usize::from(read.asic_index),
                register,
                read.value,
                timestamp_us,
            )
            .map_err(|_| HardwareSelfTestFailure::MeasurementIncomplete)?;
        let Some(domain) = maybe_domain else {
            return Ok(());
        };
        if outcome != HashrateObservationOutcome::Updated {
            return Ok(());
        }
        let snapshot = self.monitor.sample(true);
        let hashrate = snapshot
            .asics
            .first()
            .and_then(|asic| asic.domain_ghs.get(domain))
            .copied()
            .ok_or(HardwareSelfTestFailure::MeasurementIncomplete)?;
        let maximum = f64::from(expected_hardware_self_test_domain_hashrate_ghs())
            * DOMAIN_MAX_PLAUSIBLE_MULTIPLIER;
        if !hashrate.is_finite() || hashrate > maximum {
            self.rejected_counts[domain] = self.rejected_counts[domain].saturating_add(1);
        } else {
            self.hashrate_sums[domain] += hashrate;
            self.sample_counts[domain] = self.sample_counts[domain].saturating_add(1);
        }
        Ok(())
    }

    pub(super) fn finish(
        self,
    ) -> (
        [f32; HARDWARE_SELF_TEST_DOMAIN_COUNT],
        [u32; HARDWARE_SELF_TEST_DOMAIN_COUNT],
        [u32; HARDWARE_SELF_TEST_DOMAIN_COUNT],
    ) {
        let hashrates = std::array::from_fn(|index| {
            let samples = self.sample_counts[index];
            if samples == 0 {
                0.0
            } else {
                (self.hashrate_sums[index] / f64::from(samples)) as f32
            }
        });
        (hashrates, self.sample_counts, self.rejected_counts)
    }
}
