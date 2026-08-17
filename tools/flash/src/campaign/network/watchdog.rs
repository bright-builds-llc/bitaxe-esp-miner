use bitaxe_api::SystemInfoWire;

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(super) enum WatchdogFailure {
    #[default]
    None,
    SupervisorUnavailable,
    CheckpointUnhealthy,
    CheckpointSequenceMissing,
    WatchdogReasonMissing,
    WatchdogUnproved,
    WatchdogInvalidObservation,
    WatchdogSubscriptionFailed,
    WatchdogFeedFailed,
    WatchdogUnsubscriptionFailed,
    WatchdogUnsubscribed,
    WatchdogReasonUnknown,
    WatchdogParticipationInconsistent,
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
            Self::WatchdogReasonMissing => "watchdog_reason_missing",
            Self::WatchdogUnproved => "watchdog_unproved",
            Self::WatchdogInvalidObservation => "watchdog_invalid_observation",
            Self::WatchdogSubscriptionFailed => "watchdog_subscription_failed",
            Self::WatchdogFeedFailed => "watchdog_feed_failed",
            Self::WatchdogUnsubscriptionFailed => "watchdog_unsubscription_failed",
            Self::WatchdogUnsubscribed => "watchdog_unsubscribed",
            Self::WatchdogReasonUnknown => "watchdog_reason_unknown",
            Self::WatchdogParticipationInconsistent => "watchdog_participation_inconsistent",
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

fn reason_failure(maybe_reason: Option<&str>) -> WatchdogFailure {
    match maybe_reason {
        None => WatchdogFailure::WatchdogReasonMissing,
        Some("unproved") => WatchdogFailure::WatchdogUnproved,
        Some("invalid_observation") => WatchdogFailure::WatchdogInvalidObservation,
        Some("subscription_failed") => WatchdogFailure::WatchdogSubscriptionFailed,
        Some("feed_failed") => WatchdogFailure::WatchdogFeedFailed,
        Some("unsubscription_failed") => WatchdogFailure::WatchdogUnsubscriptionFailed,
        Some("unsubscribed") => WatchdogFailure::WatchdogUnsubscribed,
        Some("feed_stale") => WatchdogFailure::WatchdogFeedStale,
        Some("feed_fresh") => WatchdogFailure::None,
        Some(_) => WatchdogFailure::WatchdogReasonUnknown,
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
    let watchdog_failure = reason_failure(health.maybe_task_watchdog_reason.as_deref());
    if watchdog_failure != WatchdogFailure::None {
        return watchdog_failure;
    }
    if health.task_watchdog_participation != "participating" {
        return WatchdogFailure::WatchdogParticipationInconsistent;
    }
    if health.maybe_task_watchdog_feed_sequence.is_none() {
        return WatchdogFailure::WatchdogFeedSequenceMissing;
    }
    if health.maybe_task_watchdog_feed_age_millis.is_none() {
        return WatchdogFailure::WatchdogFeedAgeMissing;
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
                WatchdogFailure::WatchdogReasonMissing,
                "watchdog_reason_missing",
            ),
            (WatchdogFailure::WatchdogUnproved, "watchdog_unproved"),
            (
                WatchdogFailure::WatchdogInvalidObservation,
                "watchdog_invalid_observation",
            ),
            (
                WatchdogFailure::WatchdogSubscriptionFailed,
                "watchdog_subscription_failed",
            ),
            (WatchdogFailure::WatchdogFeedFailed, "watchdog_feed_failed"),
            (
                WatchdogFailure::WatchdogUnsubscriptionFailed,
                "watchdog_unsubscription_failed",
            ),
            (
                WatchdogFailure::WatchdogUnsubscribed,
                "watchdog_unsubscribed",
            ),
            (
                WatchdogFailure::WatchdogReasonUnknown,
                "watchdog_reason_unknown",
            ),
            (
                WatchdogFailure::WatchdogParticipationInconsistent,
                "watchdog_participation_inconsistent",
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
