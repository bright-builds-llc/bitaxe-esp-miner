use bitaxe_api::SystemInfoWire;
use serde::Serialize;

const HASHRATE_UPPER_BOUND_GHS: f64 = 1_000.0;
const HASHRATE_MATCH_TOLERANCE_GHS: f64 = 0.001;

#[derive(Debug, Clone, Copy, Default, Serialize)]
pub(crate) struct HashrateTransportEvidence {
    pub(in crate::campaign) active_sample_count: u64,
    pub(in crate::campaign) positive_coherent_count: u64,
    pub(in crate::campaign) distinct_positive_count: u64,
    pub(in crate::campaign) warm_rolling_window_count: u64,
    pub(in crate::campaign) terminal_zero_confirmed: bool,
}

#[derive(Debug, Clone, Copy, Default, Serialize)]
pub(crate) struct CampaignHashrateEvidence {
    pub(in crate::campaign) monitor_cadence_ms: u64,
    pub(in crate::campaign) asic_count: usize,
    pub(in crate::campaign) domain_count: usize,
    pub(in crate::campaign) http: HashrateTransportEvidence,
    pub(in crate::campaign) websocket: HashrateTransportEvidence,
}

impl CampaignHashrateEvidence {
    pub(super) fn empty() -> Self {
        Self {
            monitor_cadence_ms: 1_000,
            asic_count: 1,
            domain_count: 4,
            ..Self::default()
        }
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub(super) struct HashrateObservationAccumulator {
    pub(super) evidence: HashrateTransportEvidence,
    maybe_last_positive_signature: Option<HashrateSignature>,
}

#[derive(Debug, Clone, Copy, Default)]
pub(super) struct HashrateObservationPair {
    pub(super) http: HashrateObservationAccumulator,
    pub(super) websocket: HashrateObservationAccumulator,
}

impl HashrateObservationPair {
    pub(super) fn observe_terminal(&mut self, http: bool, sample: &SystemInfoWire) {
        if http {
            self.http.observe_terminal(sample);
        } else {
            self.websocket.observe_terminal(sample);
        }
    }

    pub(super) fn evidence(self) -> CampaignHashrateEvidence {
        CampaignHashrateEvidence {
            http: self.http.evidence,
            websocket: self.websocket.evidence,
            ..CampaignHashrateEvidence::empty()
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct HashrateSignature {
    current: u64,
    domains: [u64; 4],
}

impl HashrateObservationAccumulator {
    pub(super) fn observe_active(&mut self, active_ms: u64, sample: &SystemInfoWire) {
        self.evidence.active_sample_count = self.evidence.active_sample_count.saturating_add(1);
        let Some(signature) = coherent_positive_hashrate_signature(sample) else {
            return;
        };
        self.evidence.positive_coherent_count =
            self.evidence.positive_coherent_count.saturating_add(1);
        if self.maybe_last_positive_signature != Some(signature) {
            self.evidence.distinct_positive_count =
                self.evidence.distinct_positive_count.saturating_add(1);
            self.maybe_last_positive_signature = Some(signature);
        }
        if active_ms >= 60_000 && rolling_windows_positive(sample) {
            self.evidence.warm_rolling_window_count =
                self.evidence.warm_rolling_window_count.saturating_add(1);
        }
    }

    pub(super) fn observe_terminal(&mut self, sample: &SystemInfoWire) {
        self.evidence.terminal_zero_confirmed = terminal_hashrate_is_zero(sample);
    }
}

fn coherent_positive_hashrate_signature(sample: &SystemInfoWire) -> Option<HashrateSignature> {
    if !finite_bounded(sample.hash_rate)
        || sample.hash_rate <= 0.0
        || !finite_bounded(sample.hash_rate_1m)
        || !finite_bounded(sample.hash_rate_10m)
        || !finite_bounded(sample.hash_rate_1h)
        || !sample.error_percentage.is_finite()
        || !(0.0..=100.0).contains(&sample.error_percentage)
        || sample.hashrate_monitor.asics.len() != 1
    {
        return None;
    }
    let asic = &sample.hashrate_monitor.asics[0];
    if asic.domains.len() != 4
        || !finite_bounded(asic.total)
        || (asic.total - sample.hash_rate).abs() > HASHRATE_MATCH_TOLERANCE_GHS
        || asic
            .domains
            .iter()
            .any(|value| !finite_bounded(*value) || *value <= 0.0)
    {
        return None;
    }
    Some(HashrateSignature {
        current: sample.hash_rate.to_bits(),
        domains: [
            asic.domains[0].to_bits(),
            asic.domains[1].to_bits(),
            asic.domains[2].to_bits(),
            asic.domains[3].to_bits(),
        ],
    })
}

fn rolling_windows_positive(sample: &SystemInfoWire) -> bool {
    [
        sample.hash_rate_1m,
        sample.hash_rate_10m,
        sample.hash_rate_1h,
    ]
    .into_iter()
    .all(|value| finite_bounded(value) && value > 0.0)
}

fn terminal_hashrate_is_zero(sample: &SystemInfoWire) -> bool {
    sample.hash_rate == 0.0
        && sample.hashrate_monitor.asics.len() == 1
        && sample.hashrate_monitor.asics[0].total == 0.0
        && sample.hashrate_monitor.asics[0].domains.len() == 4
        && sample.hashrate_monitor.asics[0]
            .domains
            .iter()
            .all(|value| *value == 0.0)
}

fn finite_bounded(value: f64) -> bool {
    value.is_finite() && (0.0..=HASHRATE_UPPER_BOUND_GHS).contains(&value)
}

#[cfg(test)]
mod tests {
    use bitaxe_api::{ApiSnapshot, AsicHashrateWire, SystemInfoWire};

    use super::{
        coherent_positive_hashrate_signature, terminal_hashrate_is_zero,
        HashrateObservationAccumulator,
    };

    fn active_hashrate_sample(current: f64) -> SystemInfoWire {
        let mut sample = SystemInfoWire::from_snapshot(&ApiSnapshot::safe_ultra_205());
        sample.hash_rate = current;
        sample.hash_rate_1m = current - 1.0;
        sample.hash_rate_10m = current - 2.0;
        sample.hash_rate_1h = current - 3.0;
        sample.error_percentage = 0.25;
        sample.hashrate_monitor.asics = vec![AsicHashrateWire {
            total: current,
            error_count: 1,
            domains: vec![current / 4.0; 4],
        }];
        sample
    }

    #[test]
    fn active_observations_require_positive_coherent_topology() {
        // Arrange
        let sample = active_hashrate_sample(400.0);

        // Act
        let maybe_signature = coherent_positive_hashrate_signature(&sample);

        // Assert
        assert!(maybe_signature.is_some());
    }

    #[test]
    fn active_observations_reject_aggregate_mismatch() {
        // Arrange
        let mut sample = active_hashrate_sample(400.0);
        sample.hashrate_monitor.asics[0].total = 399.0;

        // Act
        let maybe_signature = coherent_positive_hashrate_signature(&sample);

        // Assert
        assert!(maybe_signature.is_none());
    }

    #[test]
    fn active_observations_reject_nonfinite_and_incoherent_topology() {
        // Arrange
        let mut nonfinite = active_hashrate_sample(400.0);
        nonfinite.hash_rate_1m = f64::NAN;
        let mut wrong_domain_count = active_hashrate_sample(400.0);
        wrong_domain_count.hashrate_monitor.asics[0].domains.pop();

        // Act
        let nonfinite_signature = coherent_positive_hashrate_signature(&nonfinite);
        let topology_signature = coherent_positive_hashrate_signature(&wrong_domain_count);

        // Assert
        assert!(nonfinite_signature.is_none());
        assert!(topology_signature.is_none());
    }

    #[test]
    fn changing_positive_samples_and_warm_windows_are_counted() {
        // Arrange
        let mut accumulator = HashrateObservationAccumulator::default();

        // Act
        accumulator.observe_active(60_000, &active_hashrate_sample(400.0));
        accumulator.observe_active(65_000, &active_hashrate_sample(401.0));

        // Assert
        assert_eq!(accumulator.evidence.active_sample_count, 2);
        assert_eq!(accumulator.evidence.positive_coherent_count, 2);
        assert_eq!(accumulator.evidence.distinct_positive_count, 2);
        assert_eq!(accumulator.evidence.warm_rolling_window_count, 2);
    }

    #[test]
    fn unchanged_and_cold_samples_do_not_complete_the_quorum() {
        // Arrange
        let mut accumulator = HashrateObservationAccumulator::default();
        let repeated = active_hashrate_sample(400.0);
        let mut cold = active_hashrate_sample(401.0);
        cold.hash_rate_1m = 0.0;

        // Act
        accumulator.observe_active(60_000, &repeated);
        accumulator.observe_active(65_000, &repeated);
        accumulator.observe_active(70_000, &cold);

        // Assert
        assert_eq!(accumulator.evidence.positive_coherent_count, 3);
        assert_eq!(accumulator.evidence.distinct_positive_count, 2);
        assert_eq!(accumulator.evidence.warm_rolling_window_count, 2);
    }

    #[test]
    fn terminal_sample_requires_zero_current_asic_and_domains() {
        // Arrange
        let mut sample = active_hashrate_sample(400.0);
        sample.hash_rate = 0.0;
        sample.hashrate_monitor.asics[0].total = 0.0;
        sample.hashrate_monitor.asics[0].domains.fill(0.0);

        // Act
        let zero_confirmed = terminal_hashrate_is_zero(&sample);

        // Assert
        assert!(zero_confirmed);
    }

    #[test]
    fn terminal_sample_rejects_partial_zero_state() {
        // Arrange
        let mut sample = active_hashrate_sample(400.0);
        sample.hash_rate = 0.0;
        sample.hashrate_monitor.asics[0].total = 0.0;
        sample.hashrate_monitor.asics[0].domains = vec![0.0, 0.0, 0.0, 1.0];

        // Act
        let zero_confirmed = terminal_hashrate_is_zero(&sample);

        // Assert
        assert!(!zero_confirmed);
    }
}
