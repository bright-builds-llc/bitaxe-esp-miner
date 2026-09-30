//! Injectable monotonic clock for bounded production waits.

/// Adapter-owned uptime and cooperative sleep, in milliseconds.
///
/// Physical adapters sleep their task; virtual adapters advance a scheduler.
/// The runtime never selects wall-clock time or bypasses admission checks.
pub trait Clock {
    /// Monotonic uptime since this adapter's boot.
    fn now_ms(&self) -> u64;
    /// Yields for the requested duration; implementations may advance less.
    fn sleep_ms(&mut self, duration_ms: u64);
}

/// Waits in at most 50 ms slices, rechecking authority after every yield.
pub fn wait_with_clock<C: Clock, E>(
    clock: &mut C,
    duration_ms: u64,
    mut check: impl FnMut(u64) -> Result<(), E>,
) -> Result<(), E> {
    let deadline = clock.now_ms().saturating_add(duration_ms);
    loop {
        let now_ms = clock.now_ms();
        check(now_ms)?;
        let remaining = deadline.saturating_sub(now_ms);
        if remaining == 0 {
            return Ok(());
        }
        clock.sleep_ms(remaining.min(50));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Default)]
    struct TestClock {
        now: u64,
        slices: Vec<u64>,
    }
    impl Clock for TestClock {
        fn now_ms(&self) -> u64 {
            self.now
        }
        fn sleep_ms(&mut self, duration_ms: u64) {
            self.now += duration_ms;
            self.slices.push(duration_ms);
        }
    }

    #[test]
    fn cancellation_at_deadline_wins_over_successful_wait() {
        // Arrange
        let mut clock = TestClock::default();
        // Act
        let result = wait_with_clock(&mut clock, 100, |now| {
            if now >= 100 {
                Err("revoked")
            } else {
                Ok(())
            }
        });
        // Assert
        assert_eq!(result, Err("revoked"));
        assert_eq!(clock.slices, [50, 50]);
    }

    #[test]
    fn wait_uses_bounded_slices_with_a_short_final_tail() {
        // Arrange
        let mut clock = TestClock::default();
        // Act
        let result = wait_with_clock(&mut clock, 123, |_| Ok::<_, ()>(()));
        // Assert
        assert_eq!(result, Ok(()));
        assert_eq!(clock.slices, [50, 50, 23]);
    }
}
