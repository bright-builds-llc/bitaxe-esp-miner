use super::StatisticsSample;

/// Exact upstream statistics history capacity.
pub const MAX_STATISTICS_SAMPLES: usize = 720;

/// Result of one producer-owned history update.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StatisticsHistoryRecord {
    Disabled { cleared: bool },
    Appended,
    Evicted { index: usize },
}

/// Closed failures admitted by the pure history.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StatisticsHistoryError {
    TimestampRegression,
    CapacityUnavailable,
}

/// Bounded chronological statistics history matching the pinned task logic.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct StatisticsHistory {
    samples: Vec<StatisticsSample>,
}

impl StatisticsHistory {
    #[must_use]
    pub const fn new() -> Self {
        Self {
            samples: Vec::new(),
        }
    }

    #[must_use]
    pub fn samples(&self) -> &[StatisticsSample] {
        &self.samples
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.samples.is_empty()
    }

    #[must_use]
    pub fn len(&self) -> usize {
        self.samples.len()
    }

    pub fn record(
        &mut self,
        sample: StatisticsSample,
        frequency_seconds: u16,
    ) -> Result<StatisticsHistoryRecord, StatisticsHistoryError> {
        if frequency_seconds == 0 {
            return Ok(self.disable());
        }
        if self
            .samples
            .last()
            .is_some_and(|latest| sample.timestamp < latest.timestamp)
        {
            return Err(StatisticsHistoryError::TimestampRegression);
        }
        self.reserve_full_history()?;
        if self.samples.len() < MAX_STATISTICS_SAMPLES {
            self.samples.push(sample);
            return Ok(StatisticsHistoryRecord::Appended);
        }

        let index = self.index_to_remove(sample.timestamp, frequency_seconds);
        self.samples.remove(index);
        self.samples.push(sample);
        Ok(StatisticsHistoryRecord::Evicted { index })
    }

    pub fn disable(&mut self) -> StatisticsHistoryRecord {
        let cleared = !self.samples.is_empty();
        self.samples.clear();
        StatisticsHistoryRecord::Disabled { cleared }
    }

    fn reserve_full_history(&mut self) -> Result<(), StatisticsHistoryError> {
        if self.samples.capacity() >= MAX_STATISTICS_SAMPLES {
            return Ok(());
        }
        self.samples
            .try_reserve_exact(MAX_STATISTICS_SAMPLES - self.samples.len())
            .map_err(|_| StatisticsHistoryError::CapacityUnavailable)
    }

    fn index_to_remove(&self, current_timestamp: u64, frequency_seconds: u16) -> usize {
        let current_span = current_timestamp.saturating_sub(self.samples[0].timestamp);
        let target_duration = (MAX_STATISTICS_SAMPLES as u64)
            .saturating_mul(u64::from(frequency_seconds))
            .saturating_mul(1_000);
        if current_span >= target_duration {
            return 0;
        }

        let mut low = 1;
        let mut high = MAX_STATISTICS_SAMPLES - 1;
        while high - low > 1 {
            let low_time = self.samples[low].timestamp;
            let high_time = self.samples[high].timestamp;
            let midpoint = low_time.saturating_add(high_time.saturating_sub(low_time) / 2);
            let mut split = (low..=high)
                .find(|index| self.samples[*index].timestamp >= midpoint)
                .unwrap_or(low);
            if split == low {
                split += 1;
            }
            split = split.min(high);

            let left_count = split - low;
            let right_count = high - split + 1;
            if left_count > right_count {
                high = split - 1;
            } else {
                low = split;
            }
        }
        low
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample(timestamp: u64) -> StatisticsSample {
        StatisticsSample {
            timestamp,
            hashrate: timestamp as f64,
            ..StatisticsSample::default()
        }
    }

    #[test]
    fn zero_frequency_clears_once_and_stays_disabled() {
        // Arrange
        let mut history = StatisticsHistory::new();
        history.record(sample(1_000), 1).expect("append must pass");

        // Act
        let cleared = history.record(sample(2_000), 0);
        let already_empty = history.record(sample(3_000), 0);

        // Assert
        assert_eq!(
            cleared,
            Ok(StatisticsHistoryRecord::Disabled { cleared: true })
        );
        assert_eq!(
            already_empty,
            Ok(StatisticsHistoryRecord::Disabled { cleared: false })
        );
        assert!(history.is_empty());
    }

    #[test]
    fn regressed_timestamp_preserves_existing_history() {
        // Arrange
        let mut history = StatisticsHistory::new();
        history.record(sample(2_000), 1).expect("append must pass");

        // Act
        let result = history.record(sample(1_999), 1);

        // Assert
        assert_eq!(result, Err(StatisticsHistoryError::TimestampRegression));
        assert_eq!(history.samples(), &[sample(2_000)]);
    }

    #[test]
    fn repeated_reads_do_not_create_or_consume_samples() {
        // Arrange
        let mut history = StatisticsHistory::new();
        history.record(sample(1_000), 1).expect("append must pass");
        history.record(sample(2_000), 1).expect("append must pass");

        // Act
        let first = history.samples().to_vec();
        let second = history.samples().to_vec();

        // Assert
        assert_eq!(first, vec![sample(1_000), sample(2_000)]);
        assert_eq!(second, first);
        assert_eq!(history.len(), 2);
    }

    #[test]
    fn full_history_drops_oldest_after_configured_span() {
        // Arrange
        let mut history = StatisticsHistory::new();
        for index in 0..MAX_STATISTICS_SAMPLES {
            history
                .record(sample(index as u64 * 1_000), 1)
                .expect("bounded append must pass");
        }

        // Act
        let result = history.record(sample(720_000), 1);

        // Assert
        assert_eq!(result, Ok(StatisticsHistoryRecord::Evicted { index: 0 }));
        assert_eq!(history.len(), MAX_STATISTICS_SAMPLES);
        assert_eq!(history.samples()[0].timestamp, 1_000);
        assert_eq!(
            history.samples()[MAX_STATISTICS_SAMPLES - 1].timestamp,
            720_000
        );
    }

    #[test]
    fn full_history_uses_reference_median_partition_inside_span() {
        // Arrange
        let mut history = StatisticsHistory::new();
        for index in 0..MAX_STATISTICS_SAMPLES {
            history
                .record(sample(index as u64 * 1_000), u16::MAX)
                .expect("bounded append must pass");
        }

        // Act
        let result = history.record(sample(720_000), u16::MAX);

        // Assert
        assert_eq!(result, Ok(StatisticsHistoryRecord::Evicted { index: 718 }));
        assert_eq!(history.len(), MAX_STATISTICS_SAMPLES);
        assert_eq!(history.samples()[717].timestamp, 717_000);
        assert_eq!(history.samples()[718].timestamp, 719_000);
        assert_eq!(history.samples()[719].timestamp, 720_000);
    }
}
