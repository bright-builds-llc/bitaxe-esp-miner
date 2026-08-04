//! Pure bounded hashrate monitoring for BM1366 counter observations.
//!
//! Reference breadcrumb:
//! `reference/esp-miner/main/tasks/hashrate_monitor_task.c`.

const HASHRATE_REGISTER_UNIT_HASHES: f64 = 16_777_216.0;
const HASH_COUNTER_UNIT_HASHES: f64 = 4_294_967_296.0;
const MIN_COUNTER_INTERVAL_US: u64 = 1_000_000;
const ONE_MINUTE_SAMPLES: usize = 60;
const TEN_MINUTE_SAMPLES: usize = 10;
const ONE_HOUR_SAMPLES: usize = 6;
const TEN_MINUTE_DIVISOR: u64 = ONE_MINUTE_SAMPLES as u64;
const ONE_HOUR_DIVISOR: u64 = (ONE_MINUTE_SAMPLES * TEN_MINUTE_SAMPLES) as u64;

/// One typed BM1366 hashrate observation category.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HashrateRegister {
    Instantaneous,
    TotalCount,
    DomainCount(u8),
    ErrorCount,
}

/// Closed observation admission failures.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HashrateObservationError {
    AsicOutOfRange,
    DomainOutOfRange,
    TimestampRegression,
}

/// Whether an admitted observation changed a rate measurement.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HashrateObservationOutcome {
    BaselineEstablished,
    Updated,
    IgnoredTooSoon,
    IgnoredRegisterSentinel,
}

/// Public per-ASIC diagnostics matching the upstream system-info shape.
#[derive(Debug, Clone, PartialEq)]
pub struct AsicHashrateSnapshot {
    pub total_ghs: f64,
    pub error_count: u32,
    pub domain_ghs: Vec<f64>,
}

/// One coherent aggregate and diagnostic hashrate snapshot.
#[derive(Debug, Clone, PartialEq)]
pub struct HashrateSnapshot {
    pub current_ghs: f64,
    pub one_minute_ghs: f64,
    pub ten_minute_ghs: f64,
    pub one_hour_ghs: f64,
    pub error_percentage: f64,
    pub asics: Vec<AsicHashrateSnapshot>,
}

impl Default for HashrateSnapshot {
    fn default() -> Self {
        Self {
            current_ghs: 0.0,
            one_minute_ghs: 0.0,
            ten_minute_ghs: 0.0,
            one_hour_ghs: 0.0,
            error_percentage: 0.0,
            asics: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq)]
struct Measurement {
    value: u32,
    maybe_time_us: Option<u64>,
    hashrate_ghs: f64,
}

#[derive(Debug, Clone, PartialEq)]
struct AsicMeasurements {
    total: Measurement,
    error: Measurement,
    domains: Vec<Measurement>,
}

impl AsicMeasurements {
    fn new(domain_count: usize) -> Self {
        Self {
            total: Measurement::default(),
            error: Measurement::default(),
            domains: vec![Measurement::default(); domain_count],
        }
    }

    fn reset(&mut self) {
        self.total = Measurement::default();
        self.error = Measurement::default();
        self.domains.fill(Measurement::default());
    }
}

/// Pure owner of counter measurements and hierarchical rolling windows.
#[derive(Debug, Clone, PartialEq)]
pub struct HashrateMonitor {
    asics: Vec<AsicMeasurements>,
    active: bool,
    poll_count: u64,
    one_minute: [Option<f64>; ONE_MINUTE_SAMPLES],
    ten_minute_previous: Option<f64>,
    ten_minute: [Option<f64>; TEN_MINUTE_SAMPLES],
    one_hour_previous: Option<f64>,
    one_hour: [Option<f64>; ONE_HOUR_SAMPLES],
    snapshot: HashrateSnapshot,
}

impl HashrateMonitor {
    /// Creates a bounded monitor for the supplied fixed device topology.
    #[must_use]
    pub fn new(asic_count: usize, domain_count: usize) -> Self {
        Self {
            asics: (0..asic_count)
                .map(|_| AsicMeasurements::new(domain_count))
                .collect(),
            active: false,
            poll_count: 0,
            one_minute: [None; ONE_MINUTE_SAMPLES],
            ten_minute_previous: None,
            ten_minute: [None; TEN_MINUTE_SAMPLES],
            one_hour_previous: None,
            one_hour: [None; ONE_HOUR_SAMPLES],
            snapshot: HashrateSnapshot::default(),
        }
    }

    /// Admits one parsed register value without performing I/O.
    pub fn observe(
        &mut self,
        asic_index: usize,
        register: HashrateRegister,
        value: u32,
        timestamp_us: u64,
    ) -> Result<HashrateObservationOutcome, HashrateObservationError> {
        let Some(asic) = self.asics.get_mut(asic_index) else {
            return Err(HashrateObservationError::AsicOutOfRange);
        };

        match register {
            HashrateRegister::Instantaneous => {
                let outcome = update_instantaneous(&mut asic.total, value);
                if outcome == HashrateObservationOutcome::Updated {
                    if let Some(first_domain) = asic.domains.first_mut() {
                        update_instantaneous(first_domain, value);
                    }
                }
                Ok(outcome)
            }
            HashrateRegister::TotalCount => update_counter(&mut asic.total, value, timestamp_us),
            HashrateRegister::DomainCount(domain) => {
                let Some(measurement) = asic.domains.get_mut(usize::from(domain)) else {
                    return Err(HashrateObservationError::DomainOutOfRange);
                };
                update_counter(measurement, value, timestamp_us)
            }
            HashrateRegister::ErrorCount => update_counter(&mut asic.error, value, timestamp_us),
        }
    }

    /// Advances one one-second monitor sample while the ASIC session is active.
    /// Stopping clears counter baselines but deliberately preserves averages.
    #[must_use]
    pub fn sample(&mut self, active: bool) -> HashrateSnapshot {
        if self.active && !active {
            self.reset_measurements();
        }
        self.active = active;

        let current_ghs = if active {
            self.asics.iter().map(|asic| asic.total.hashrate_ghs).sum()
        } else {
            0.0
        };
        let error_ghs = if active {
            self.asics.iter().map(|asic| asic.error.hashrate_ghs).sum()
        } else {
            0.0
        };

        self.snapshot.current_ghs = current_ghs;
        self.snapshot.error_percentage = if current_ghs > 0.0 {
            error_ghs / current_ghs * 100.0
        } else {
            0.0
        };
        self.snapshot.asics = self
            .asics
            .iter()
            .map(|asic| AsicHashrateSnapshot {
                total_ghs: asic.total.hashrate_ghs,
                error_count: asic.error.value,
                domain_ghs: asic
                    .domains
                    .iter()
                    .map(|measurement| measurement.hashrate_ghs)
                    .collect(),
            })
            .collect();

        if current_ghs > 0.0 {
            self.update_averages(current_ghs);
        }

        self.snapshot.clone()
    }

    fn reset_measurements(&mut self) {
        for asic in &mut self.asics {
            asic.reset();
        }
    }

    fn update_averages(&mut self, current_ghs: f64) {
        let one_minute_index = (self.poll_count % ONE_MINUTE_SAMPLES as u64) as usize;
        self.one_minute[one_minute_index] = Some(current_ghs);
        self.snapshot.one_minute_ghs = average(&self.one_minute);

        let ten_minute_blend = self.poll_count % TEN_MINUTE_DIVISOR;
        let ten_minute_index =
            ((self.poll_count / TEN_MINUTE_DIVISOR) % TEN_MINUTE_SAMPLES as u64) as usize;
        if ten_minute_blend == 0 {
            self.ten_minute_previous = self.ten_minute[ten_minute_index];
        }
        let mut one_minute_value = self.snapshot.one_minute_ghs;
        if let Some(previous) = self.ten_minute_previous {
            let fraction = (ten_minute_blend + 1) as f64 / TEN_MINUTE_DIVISOR as f64;
            one_minute_value = fraction * one_minute_value + (1.0 - fraction) * previous;
        }
        self.ten_minute[ten_minute_index] = Some(one_minute_value);
        self.snapshot.ten_minute_ghs = average(&self.ten_minute);

        let one_hour_blend = self.poll_count % ONE_HOUR_DIVISOR;
        let one_hour_index =
            ((self.poll_count / ONE_HOUR_DIVISOR) % ONE_HOUR_SAMPLES as u64) as usize;
        if one_hour_blend == 0 {
            self.one_hour_previous = self.one_hour[one_hour_index];
        }
        let mut ten_minute_value = self.snapshot.ten_minute_ghs;
        if let Some(previous) = self.one_hour_previous {
            let fraction = (one_hour_blend + 1) as f64 / ONE_HOUR_DIVISOR as f64;
            ten_minute_value = fraction * ten_minute_value + (1.0 - fraction) * previous;
        }
        self.one_hour[one_hour_index] = Some(ten_minute_value);
        self.snapshot.one_hour_ghs = average(&self.one_hour);
        self.poll_count = self.poll_count.saturating_add(1);
    }
}

fn update_instantaneous(measurement: &mut Measurement, value: u32) -> HashrateObservationOutcome {
    let long_flag = value & 0x8000_0000 != 0;
    let hashrate_value = value & 0x7fff_ffff;
    if hashrate_value == 0x007f_ffff || long_flag {
        return HashrateObservationOutcome::IgnoredRegisterSentinel;
    }
    measurement.hashrate_ghs = hashrate_value as f64 * HASHRATE_REGISTER_UNIT_HASHES / 1e9;
    HashrateObservationOutcome::Updated
}

fn update_counter(
    measurement: &mut Measurement,
    value: u32,
    timestamp_us: u64,
) -> Result<HashrateObservationOutcome, HashrateObservationError> {
    let Some(previous_time_us) = measurement.maybe_time_us else {
        measurement.value = value;
        measurement.maybe_time_us = Some(timestamp_us);
        return Ok(HashrateObservationOutcome::BaselineEstablished);
    };
    let Some(duration_us) = timestamp_us.checked_sub(previous_time_us) else {
        return Err(HashrateObservationError::TimestampRegression);
    };
    if duration_us < MIN_COUNTER_INTERVAL_US {
        return Ok(HashrateObservationOutcome::IgnoredTooSoon);
    }

    let counter = value.wrapping_sub(measurement.value);
    let seconds = duration_us as f64 / 1_000_000.0;
    measurement.hashrate_ghs = counter as f64 / seconds * HASH_COUNTER_UNIT_HASHES / 1e9;
    measurement.value = value;
    measurement.maybe_time_us = Some(timestamp_us);
    Ok(HashrateObservationOutcome::Updated)
}

fn average<const N: usize>(values: &[Option<f64>; N]) -> f64 {
    let (sum, count) = values
        .iter()
        .flatten()
        .fold((0.0, 0_u32), |(sum, count), value| {
            (sum + value, count.saturating_add(1))
        });
    if count == 0 {
        0.0
    } else {
        sum / f64::from(count)
    }
}

#[cfg(test)]
mod tests;
