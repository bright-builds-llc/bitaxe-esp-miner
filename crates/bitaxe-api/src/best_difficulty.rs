//! All-time and session best nonce difficulty, as the pinned reference keeps them in
//! `SYSTEM_notify_found_nonce` (`main/system.c`).
//!
//! Every nonce on a valid job counts, whether or not it meets the pool target. Values
//! are compared after the reference's `(uint64_t)` cast. The all-time best is restored
//! from NVS key `bestdiff` at boot and written on each new record; the session best
//! starts at 0 on every boot.

/// The reference's best-difficulty pair.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct BestDifficultyState {
    /// `best_nonce_diff`: the all-time best, persisted as `bestdiff`.
    pub all_time: u64,
    /// `best_session_nonce_diff`: the best since this boot.
    pub session: u64,
}

impl BestDifficultyState {
    /// The boot state: the persisted all-time best and an empty session.
    #[must_use]
    pub const fn restored(persisted_all_time: u64) -> Self {
        Self {
            all_time: persisted_all_time,
            session: 0,
        }
    }

    /// Observes one nonce difficulty. Returns the next state and, only when the
    /// all-time best improved, the value to persist.
    #[must_use]
    pub fn observe(self, nonce_difficulty: f64) -> (Self, Option<u64>) {
        let difficulty = reference_u64(nonce_difficulty);
        let session = self.session.max(difficulty);
        if difficulty <= self.all_time {
            return (Self { session, ..self }, None);
        }
        (
            Self {
                all_time: difficulty,
                session,
            },
            Some(difficulty),
        )
    }
}

/// The reference's `(uint64_t)` cast of a difficulty: truncated, and clamped where C
/// would be undefined (NaN, negative, beyond `u64`).
#[must_use]
pub fn reference_u64(value: f64) -> u64 {
    if value.is_nan() || value <= 0.0 {
        return 0;
    }
    // `as` truncates toward zero and saturates at `u64::MAX`.
    value as u64
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_boot_restores_the_all_time_best_and_starts_an_empty_session() {
        // Arrange / Act
        let state = BestDifficultyState::restored(1_234_567);
        // Assert
        assert_eq!((state.all_time, state.session), (1_234_567, 0));
    }

    #[test]
    fn a_new_record_updates_both_and_asks_to_persist() {
        // Arrange
        let state = BestDifficultyState::restored(1_000);
        // Act
        let (next, maybe_persist) = state.observe(2_500.9);
        // Assert: truncated like `(uint64_t)diff`.
        assert_eq!(
            (next.all_time, next.session, maybe_persist),
            (2_500, 2_500, Some(2_500))
        );
    }

    #[test]
    fn a_session_best_below_the_all_time_best_is_not_persisted() {
        // Arrange
        let state = BestDifficultyState::restored(10_000);
        // Act
        let (next, maybe_persist) = state.observe(700.0);
        // Assert
        assert_eq!(
            (next.all_time, next.session, maybe_persist),
            (10_000, 700, None)
        );
    }

    #[test]
    fn an_equal_truncated_value_changes_nothing() {
        // Arrange: the reference compares with `<=` after the cast.
        let (state, _) = BestDifficultyState::restored(0).observe(1_234.2);
        // Act
        let (next, maybe_persist) = state.observe(1_234.9);
        // Assert
        assert_eq!((next, maybe_persist), (state, None));
    }

    #[test]
    fn a_lower_nonce_changes_nothing() {
        // Arrange
        let (state, _) = BestDifficultyState::restored(0).observe(5_000.0);
        // Act
        let (next, maybe_persist) = state.observe(4_999.0);
        // Assert
        assert_eq!((next, maybe_persist), (state, None));
    }

    #[test]
    fn invalid_difficulties_count_as_zero() {
        // Arrange / Act / Assert
        assert_eq!(
            [f64::NAN, -5.0, 0.0, f64::INFINITY].map(reference_u64),
            [0, 0, 0, u64::MAX]
        );
    }
}
