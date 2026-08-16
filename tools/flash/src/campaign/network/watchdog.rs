use bitaxe_api::SystemInfoWire;

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(super) enum WatchdogFailure {
    #[default]
    None,
    SupervisorUnavailable,
    CheckpointUnhealthy,
    CheckpointSequenceMissing,
    WatchdogNotParticipating,
    WatchdogFeedReasonNotFresh,
    WatchdogFeedSequenceMissing,
    WatchdogFeedAgeMissing,
    WatchdogFeedStale,
    HttpCheckpointNotAdvanced,
    HttpFeedNotAdvanced,
    WebsocketCheckpointNotAdvanced,
    WebsocketFeedNotAdvanced,
}

impl WatchdogFailure {
    pub(super) const fn label(self) -> &'static str {
        match self {
            Self::None => "none",
            Self::SupervisorUnavailable => "supervisor_unavailable",
            Self::CheckpointUnhealthy => "checkpoint_unhealthy",
            Self::CheckpointSequenceMissing => "checkpoint_sequence_missing",
            Self::WatchdogNotParticipating => "watchdog_not_participating",
            Self::WatchdogFeedReasonNotFresh => "watchdog_feed_reason_not_fresh",
            Self::WatchdogFeedSequenceMissing => "watchdog_feed_sequence_missing",
            Self::WatchdogFeedAgeMissing => "watchdog_feed_age_missing",
            Self::WatchdogFeedStale => "watchdog_feed_stale",
            Self::HttpCheckpointNotAdvanced => "http_checkpoint_not_advanced",
            Self::HttpFeedNotAdvanced => "http_feed_not_advanced",
            Self::WebsocketCheckpointNotAdvanced => "websocket_checkpoint_not_advanced",
            Self::WebsocketFeedNotAdvanced => "websocket_feed_not_advanced",
        }
    }
}

pub(super) fn sample_failure(sample: &SystemInfoWire) -> WatchdogFailure {
    let health = &sample.runtime_health;
    if health.supervisor_availability != "available" {
        return WatchdogFailure::SupervisorUnavailable;
    }
    if health.checkpoint_health != "healthy" {
        return WatchdogFailure::CheckpointUnhealthy;
    }
    if health.maybe_checkpoint_sequence.is_none() {
        return WatchdogFailure::CheckpointSequenceMissing;
    }
    if health.task_watchdog_participation != "participating" {
        return WatchdogFailure::WatchdogNotParticipating;
    }
    if health.maybe_task_watchdog_reason.as_deref() != Some("feed_fresh") {
        return WatchdogFailure::WatchdogFeedReasonNotFresh;
    }
    if health.maybe_task_watchdog_feed_sequence.is_none() {
        return WatchdogFailure::WatchdogFeedSequenceMissing;
    }
    let Some(feed_age_millis) = health.maybe_task_watchdog_feed_age_millis else {
        return WatchdogFailure::WatchdogFeedAgeMissing;
    };
    if feed_age_millis > 2_000 {
        return WatchdogFailure::WatchdogFeedStale;
    }
    WatchdogFailure::None
}

pub(super) fn window_failure(
    http_checkpoint_advanced: bool,
    http_feed_advanced: bool,
    websocket_checkpoint_advanced: bool,
    websocket_feed_advanced: bool,
) -> WatchdogFailure {
    if !http_checkpoint_advanced {
        return WatchdogFailure::HttpCheckpointNotAdvanced;
    }
    if !http_feed_advanced {
        return WatchdogFailure::HttpFeedNotAdvanced;
    }
    if !websocket_checkpoint_advanced {
        return WatchdogFailure::WebsocketCheckpointNotAdvanced;
    }
    if !websocket_feed_advanced {
        return WatchdogFailure::WebsocketFeedNotAdvanced;
    }
    WatchdogFailure::None
}

#[cfg(test)]
mod tests {
    use super::WatchdogFailure;

    #[test]
    fn watchdog_failure_labels_are_closed_and_value_free() {
        // Arrange
        let cases = [
            (WatchdogFailure::None, "none"),
            (
                WatchdogFailure::SupervisorUnavailable,
                "supervisor_unavailable",
            ),
            (WatchdogFailure::CheckpointUnhealthy, "checkpoint_unhealthy"),
            (
                WatchdogFailure::CheckpointSequenceMissing,
                "checkpoint_sequence_missing",
            ),
            (
                WatchdogFailure::WatchdogNotParticipating,
                "watchdog_not_participating",
            ),
            (
                WatchdogFailure::WatchdogFeedReasonNotFresh,
                "watchdog_feed_reason_not_fresh",
            ),
            (
                WatchdogFailure::WatchdogFeedSequenceMissing,
                "watchdog_feed_sequence_missing",
            ),
            (
                WatchdogFailure::WatchdogFeedAgeMissing,
                "watchdog_feed_age_missing",
            ),
            (WatchdogFailure::WatchdogFeedStale, "watchdog_feed_stale"),
            (
                WatchdogFailure::HttpCheckpointNotAdvanced,
                "http_checkpoint_not_advanced",
            ),
            (
                WatchdogFailure::HttpFeedNotAdvanced,
                "http_feed_not_advanced",
            ),
            (
                WatchdogFailure::WebsocketCheckpointNotAdvanced,
                "websocket_checkpoint_not_advanced",
            ),
            (
                WatchdogFailure::WebsocketFeedNotAdvanced,
                "websocket_feed_not_advanced",
            ),
        ];

        // Act / Assert
        for (failure, expected) in cases {
            let label = failure.label();
            assert_eq!(label, expected);
            assert!(label
                .bytes()
                .all(|byte| byte.is_ascii_lowercase() || byte == b'_'));
        }
    }
}
