//! Pure cadence semantics shared by the boot-lifetime runtime owners.
//!
//! Reference breadcrumbs:
//! - `reference/esp-miner/main/tasks/create_jobs_task.c`
//! - `reference/esp-miner/main/tasks/asic_result_task.c`
//! - `reference/esp-miner/main/tasks/power_management_task.c`

/// Safety-supervisor cadence matching the upstream power-management loop.
pub const SAFETY_SUPERVISOR_CADENCE_MS: u64 = 100;
/// Read-only operator observation cadence.
pub const OPERATOR_OBSERVATION_CADENCE_MS: u64 = 500;
/// Maximum interval between authoritative production-readiness reads.
pub const PRODUCTION_REREAD_CADENCE_MS: u64 = 1_000;

/// Failure to construct or advance a periodic deadline.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PeriodicDeadlineError {
    /// A zero cadence would never make forward progress.
    ZeroCadence,
    /// The next deadline cannot be represented by the monotonic clock type.
    DeadlineOverflow,
}

/// Result of advancing one completed periodic step.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PeriodicDeadlineAdvance {
    next_deadline_ms: u64,
    missed_slots: u64,
}

impl PeriodicDeadlineAdvance {
    /// Next monotonic deadline after coalescing any elapsed slots.
    #[must_use]
    pub const fn next_deadline_ms(self) -> u64 {
        self.next_deadline_ms
    }

    /// Number of additional elapsed slots coalesced by this advance.
    #[must_use]
    pub const fn missed_slots(self) -> u64 {
        self.missed_slots
    }
}

/// One periodic owner's next monotonic deadline.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PeriodicDeadline {
    next_deadline_ms: u64,
    cadence_ms: u64,
}

impl PeriodicDeadline {
    /// Starts a schedule that is immediately due at `start_ms`.
    pub const fn new(start_ms: u64, cadence_ms: u64) -> Result<Self, PeriodicDeadlineError> {
        if cadence_ms == 0 {
            return Err(PeriodicDeadlineError::ZeroCadence);
        }
        Ok(Self {
            next_deadline_ms: start_ms,
            cadence_ms,
        })
    }

    /// Returns the current monotonic deadline.
    #[must_use]
    pub const fn next_deadline_ms(self) -> u64 {
        self.next_deadline_ms
    }

    /// Returns whether the owner is due at `now_ms`.
    #[must_use]
    pub const fn is_due(self, now_ms: u64) -> bool {
        now_ms >= self.next_deadline_ms
    }

    /// Advances after one completed step and coalesces elapsed slots.
    ///
    /// Clock regression never creates extra work: if `now_ms` is earlier than
    /// the scheduled boundary, the schedule still advances by exactly one
    /// cadence from the boundary that just ran.
    pub fn advance_past(
        &mut self,
        now_ms: u64,
    ) -> Result<PeriodicDeadlineAdvance, PeriodicDeadlineError> {
        let first_future = self
            .next_deadline_ms
            .checked_add(self.cadence_ms)
            .ok_or(PeriodicDeadlineError::DeadlineOverflow)?;
        let missed_slots = if first_future > now_ms {
            0
        } else {
            now_ms
                .saturating_sub(first_future)
                .checked_div(self.cadence_ms)
                .and_then(|elapsed| elapsed.checked_add(1))
                .ok_or(PeriodicDeadlineError::DeadlineOverflow)?
        };
        let skipped_ms = missed_slots
            .checked_mul(self.cadence_ms)
            .ok_or(PeriodicDeadlineError::DeadlineOverflow)?;
        let next_deadline_ms = first_future
            .checked_add(skipped_ms)
            .ok_or(PeriodicDeadlineError::DeadlineOverflow)?;
        self.next_deadline_ms = next_deadline_ms;
        Ok(PeriodicDeadlineAdvance {
            next_deadline_ms,
            missed_slots,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn schedule_is_due_immediately_then_advances_one_slot() {
        // Arrange
        let mut schedule = PeriodicDeadline::new(1_000, 100).expect("valid cadence");

        // Act
        let advance = schedule
            .advance_past(1_000)
            .expect("deadline should advance");

        // Assert
        assert!(schedule.is_due(1_100));
        assert_eq!(advance.next_deadline_ms(), 1_100);
        assert_eq!(advance.missed_slots(), 0);
    }

    #[test]
    fn delayed_owner_coalesces_elapsed_slots_without_drift() {
        // Arrange
        let mut schedule = PeriodicDeadline::new(1_000, 100).expect("valid cadence");

        // Act
        let advance = schedule
            .advance_past(1_350)
            .expect("deadline should advance");

        // Assert
        assert_eq!(advance.next_deadline_ms(), 1_400);
        assert_eq!(advance.missed_slots(), 3);
    }

    #[test]
    fn regressed_clock_preserves_the_scheduled_boundary() {
        // Arrange
        let mut schedule = PeriodicDeadline::new(1_000, 100).expect("valid cadence");

        // Act
        let advance = schedule.advance_past(900).expect("deadline should advance");

        // Assert
        assert_eq!(advance.next_deadline_ms(), 1_100);
        assert_eq!(advance.missed_slots(), 0);
    }

    #[test]
    fn zero_cadence_and_overflow_fail_without_mutating_deadline() {
        // Arrange
        let zero = PeriodicDeadline::new(0, 0);
        let mut overflowing = PeriodicDeadline::new(u64::MAX, 1).expect("valid cadence");

        // Act
        let overflow = overflowing.advance_past(u64::MAX);

        // Assert
        assert_eq!(zero, Err(PeriodicDeadlineError::ZeroCadence));
        assert_eq!(overflow, Err(PeriodicDeadlineError::DeadlineOverflow));
        assert_eq!(overflowing.next_deadline_ms(), u64::MAX);
    }
}
