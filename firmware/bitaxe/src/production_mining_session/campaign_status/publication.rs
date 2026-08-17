pub(super) const CAMPAIGN_STATUS_PUBLICATION_INTERVAL_MS: u64 = 1_000;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CampaignStatusPublicationError {
    MonotonicTimeRegression,
    DeadlineOverflow,
}

impl CampaignStatusPublicationError {
    pub(crate) const fn label(self) -> &'static str {
        match self {
            Self::MonotonicTimeRegression => "monotonic_time_regression",
            Self::DeadlineOverflow => "deadline_overflow",
        }
    }
}

#[derive(Debug, Default)]
pub(crate) struct CampaignStatusPublicationSchedule {
    maybe_last_published_ms: Option<u64>,
    terminal_published: bool,
}

impl CampaignStatusPublicationSchedule {
    pub(crate) const fn new() -> Self {
        Self {
            maybe_last_published_ms: None,
            terminal_published: false,
        }
    }

    pub(crate) fn should_publish(
        &mut self,
        now_ms: u64,
        terminal: bool,
    ) -> Result<bool, CampaignStatusPublicationError> {
        let Some(last_published_ms) = self.maybe_last_published_ms else {
            self.maybe_last_published_ms = Some(now_ms);
            self.terminal_published = terminal;
            return Ok(true);
        };
        if now_ms < last_published_ms {
            return Err(CampaignStatusPublicationError::MonotonicTimeRegression);
        }
        if terminal && !self.terminal_published {
            self.maybe_last_published_ms = Some(now_ms);
            self.terminal_published = true;
            return Ok(true);
        }
        if self.terminal_published {
            return Ok(false);
        }
        let next_deadline_ms = last_published_ms
            .checked_add(CAMPAIGN_STATUS_PUBLICATION_INTERVAL_MS)
            .ok_or(CampaignStatusPublicationError::DeadlineOverflow)?;
        if now_ms < next_deadline_ms {
            return Ok(false);
        }
        self.maybe_last_published_ms = Some(now_ms);
        Ok(true)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn high_event_rate_publication_is_cadence_bounded_for_ten_minutes() {
        // Arrange
        let mut schedule = CampaignStatusPublicationSchedule::new();
        let mut published_at = Vec::new();

        // Act
        for now_ms in (0..=600_000).step_by(20) {
            if schedule
                .should_publish(now_ms, false)
                .expect("monotonic production timeline should remain valid")
            {
                published_at.push(now_ms);
            }
        }

        // Assert
        assert_eq!(published_at.len(), 601);
        assert_eq!(published_at.first(), Some(&0));
        assert_eq!(published_at.last(), Some(&600_000));
        assert!(published_at
            .windows(2)
            .all(|pair| pair[1].saturating_sub(pair[0]) <= 1_000));
        assert!(published_at.len() < 30_001);
    }

    #[test]
    fn terminal_publication_bypasses_cadence_once() {
        // Arrange
        let mut schedule = CampaignStatusPublicationSchedule::new();
        assert_eq!(schedule.should_publish(0, false), Ok(true));

        // Act
        let terminal = schedule.should_publish(1, true);
        let repeated_terminal = schedule.should_publish(2, true);

        // Assert
        assert_eq!(terminal, Ok(true));
        assert_eq!(repeated_terminal, Ok(false));
    }

    #[test]
    fn publication_schedule_fails_closed_on_clock_regression_and_overflow() {
        // Arrange
        let mut regressed = CampaignStatusPublicationSchedule::new();
        let mut overflowed = CampaignStatusPublicationSchedule::new();
        assert_eq!(regressed.should_publish(10, false), Ok(true));
        assert_eq!(overflowed.should_publish(u64::MAX - 500, false), Ok(true));

        // Act / Assert
        assert_eq!(
            regressed.should_publish(9, false),
            Err(CampaignStatusPublicationError::MonotonicTimeRegression)
        );
        assert_eq!(
            overflowed.should_publish(u64::MAX, false),
            Err(CampaignStatusPublicationError::DeadlineOverflow)
        );
        assert_eq!(
            CampaignStatusPublicationError::MonotonicTimeRegression.label(),
            "monotonic_time_regression"
        );
        assert_eq!(
            CampaignStatusPublicationError::DeadlineOverflow.label(),
            "deadline_overflow"
        );
    }
}
