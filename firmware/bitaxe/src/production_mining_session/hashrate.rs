//! Thin production-owner adapter around the pure hashrate monitor.

use bitaxe_asic::bm1366::{registers::Bm1366Register, result::Bm1366RegisterRead};
use bitaxe_core::hashrate::{
    HashrateMonitor, HashrateObservationError, HashrateRegister, HashrateSnapshot,
};
use bitaxe_core::runtime_orchestration::{PeriodicDeadline, PeriodicDeadlineError};

const HASHRATE_CADENCE_MS: u64 = 1_000;
const ULTRA_205_ASIC_COUNT: usize = 1;
const BM1366_HASH_DOMAIN_COUNT: usize = 4;

pub(super) struct HashrateServiceTick {
    pub(super) request_registers: bool,
    pub(super) snapshot: HashrateSnapshot,
}

pub(super) struct ProductionHashrateMonitor {
    monitor: HashrateMonitor,
    schedule: PeriodicDeadline,
    active: bool,
    inactive_published: bool,
}

impl ProductionHashrateMonitor {
    pub(super) fn new() -> Self {
        Self {
            monitor: HashrateMonitor::new(ULTRA_205_ASIC_COUNT, BM1366_HASH_DOMAIN_COUNT),
            schedule: PeriodicDeadline::new(0, HASHRATE_CADENCE_MS)
                .expect("hashrate cadence is nonzero"),
            active: false,
            inactive_published: false,
        }
    }

    pub(super) fn observe(&mut self, read: Bm1366RegisterRead, timestamp_us: u64) {
        if !self.active {
            return;
        }
        let Some(register) = monitor_register(read.register) else {
            return;
        };
        if let Err(error) = self.monitor.observe(
            usize::from(read.asic_index),
            register,
            read.value,
            timestamp_us,
        ) {
            log_observation_error(error);
        }
    }

    pub(super) fn service(
        &mut self,
        active: bool,
        now_ms: u64,
    ) -> Result<Option<HashrateServiceTick>, PeriodicDeadlineError> {
        if !active {
            if !self.active && self.inactive_published {
                return Ok(None);
            }
            self.active = false;
            self.inactive_published = true;
            self.schedule = PeriodicDeadline::new(now_ms, HASHRATE_CADENCE_MS)?;
            return Ok(Some(HashrateServiceTick {
                request_registers: false,
                snapshot: self.monitor.sample(false),
            }));
        }

        if !self.active {
            self.active = true;
            self.inactive_published = false;
            self.schedule = PeriodicDeadline::new(now_ms, HASHRATE_CADENCE_MS)?;
        }
        if !self.schedule.is_due(now_ms) {
            return Ok(None);
        }

        let snapshot = self.monitor.sample(true);
        self.schedule.advance_past(now_ms)?;
        Ok(Some(HashrateServiceTick {
            request_registers: true,
            snapshot,
        }))
    }

    pub(super) fn service_snapshot(
        &mut self,
        snapshot: &bitaxe_stratum::v1::production_session::ProductionSessionSnapshot,
        now_ms: u64,
    ) -> Result<Option<HashrateServiceTick>, PeriodicDeadlineError> {
        let active = snapshot.mining.mining_activity
            == bitaxe_stratum::v1::state::MiningActivityStatus::Active
            && snapshot.mining.work_submission
                == bitaxe_stratum::v1::state::WorkSubmissionGate::Ready;
        self.service(active, now_ms)
    }
}

fn monitor_register(register: Bm1366Register) -> Option<HashrateRegister> {
    match register {
        Bm1366Register::ErrorCount => Some(HashrateRegister::ErrorCount),
        Bm1366Register::Domain0Count => Some(HashrateRegister::DomainCount(0)),
        Bm1366Register::Domain1Count => Some(HashrateRegister::DomainCount(1)),
        Bm1366Register::Domain2Count => Some(HashrateRegister::DomainCount(2)),
        Bm1366Register::Domain3Count => Some(HashrateRegister::DomainCount(3)),
        Bm1366Register::TotalCount => Some(HashrateRegister::TotalCount),
        Bm1366Register::ChipId => None,
    }
}

fn log_observation_error(error: HashrateObservationError) {
    let category = match error {
        HashrateObservationError::AsicOutOfRange => "asic_out_of_range",
        HashrateObservationError::DomainOutOfRange => "domain_out_of_range",
        HashrateObservationError::TimestampRegression => "timestamp_regression",
    };
    log::warn!("hashrate_monitor_observation=discarded category={category}");
}
