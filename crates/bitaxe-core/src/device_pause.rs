//! Why the device paused mining itself (ADR-0038). Each reason lasts until the
//! operator resumes. Only the device pauses for these; an operator pause has none.

/// A pause the device chose, shown in the API, the web UI and on the screen.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DevicePauseReason {
    /// Repeated abnormal resets in a row: the crash-loop guard started paused.
    CrashLoop,
    /// An overheat whose pair could not be lowered within the catalog.
    Overheat,
    /// Repeated failed local mining runs: the restart guard holds.
    RestartGuard,
}

impl DevicePauseReason {
    /// The closed API label.
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::CrashLoop => "crash_loop",
            Self::Overheat => "overheat",
            Self::RestartGuard => "restart_guard",
        }
    }

    /// The screen line naming the reason; it fits one Ultra 205 row.
    #[must_use]
    pub const fn screen_line(self) -> &'static str {
        match self {
            Self::CrashLoop => "Repeated resets",
            Self::Overheat => "Overheat stop",
            Self::RestartGuard => "Repeated failures",
        }
    }

    /// A nonzero code for lock-free storage.
    #[must_use]
    pub const fn code(self) -> u8 {
        match self {
            Self::CrashLoop => 1,
            Self::Overheat => 2,
            Self::RestartGuard => 3,
        }
    }

    /// The reason a code names; 0 and unknown codes name none.
    #[must_use]
    pub const fn from_code(code: u8) -> Option<Self> {
        match code {
            1 => Some(Self::CrashLoop),
            2 => Some(Self::Overheat),
            3 => Some(Self::RestartGuard),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::screen::SCREEN_MAX_LINE_CHARS;

    const ALL: [DevicePauseReason; 3] = [
        DevicePauseReason::CrashLoop,
        DevicePauseReason::Overheat,
        DevicePauseReason::RestartGuard,
    ];

    #[test]
    fn codes_round_trip_and_zero_is_none() {
        // Arrange / Act
        let decoded = ALL.map(|reason| DevicePauseReason::from_code(reason.code()));
        // Assert
        assert_eq!(decoded, ALL.map(Some));
        assert_eq!(DevicePauseReason::from_code(0), None);
    }

    #[test]
    fn labels_are_distinct_snake_case() {
        // Arrange / Act
        let labels = ALL.map(DevicePauseReason::label);
        // Assert
        let unique: std::collections::BTreeSet<_> = labels.iter().collect();
        assert_eq!(unique.len(), ALL.len());
        assert!(labels.iter().all(|label| label
            .bytes()
            .all(|byte| byte.is_ascii_lowercase() || byte == b'_')));
    }

    #[test]
    fn screen_lines_fit_one_row() {
        // Arrange / Act / Assert
        assert!(ALL
            .iter()
            .all(|reason| reason.screen_line().chars().count() <= SCREEN_MAX_LINE_CHARS));
    }
}
