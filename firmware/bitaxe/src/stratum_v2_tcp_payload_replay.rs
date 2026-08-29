//! Pure replay policy for late receive-only diagnostic attachment.

pub(crate) const REPLAY_CADENCE_MS: u64 = 5_000;
pub(crate) const REPLAY_WINDOW_MS: u64 = 120_000;

#[must_use]
pub(crate) const fn replay_deadline_ms(ordinal: u16) -> Option<u64> {
    if ordinal == 0 {
        return None;
    }
    let deadline_ms = (ordinal as u64) * REPLAY_CADENCE_MS;
    if deadline_ms > REPLAY_WINDOW_MS {
        return None;
    }
    Some(deadline_ms)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn late_attachment_observes_a_replay_before_monitor_admission_deadline() {
        // Arrange
        let attachment_ms = 42_186;

        // Act
        let first_visible = (1..=u16::MAX)
            .filter_map(replay_deadline_ms)
            .find(|deadline_ms| *deadline_ms >= attachment_ms);

        // Assert
        assert_eq!(first_visible, Some(45_000));
        assert!(first_visible.is_some_and(|deadline_ms| deadline_ms <= 60_000));
    }

    #[test]
    fn replay_window_is_bounded_and_covers_post_flash_admission() {
        // Arrange / Act
        let deadlines = (1..=u16::MAX)
            .filter_map(replay_deadline_ms)
            .collect::<Vec<_>>();

        // Assert
        assert_eq!(deadlines.first(), Some(&REPLAY_CADENCE_MS));
        assert_eq!(deadlines.last(), Some(&REPLAY_WINDOW_MS));
        assert_eq!(deadlines.len(), 24);
    }
}
