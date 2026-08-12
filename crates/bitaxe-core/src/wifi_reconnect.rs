//! Pure station reconnect lifecycle policy.

/// Upstream-compatible delay before each station reconnect attempt.
pub const WIFI_RECONNECT_DELAY_MS: u64 = 5_000;

/// Redaction-safe station disconnect categories.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WifiDisconnectReason {
    Roaming,
    BeaconTimeout,
    AccessPointUnavailable,
    AuthenticationFailed,
    AssociationFailed,
    HandshakeTimeout,
    ConnectionFailed,
    Other,
}

impl WifiDisconnectReason {
    /// Classifies an ESP-IDF reason code without retaining network identifiers.
    #[must_use]
    pub const fn from_esp_reason(reason: u16) -> Self {
        match reason {
            207 => Self::Roaming,
            200 => Self::BeaconTimeout,
            201 | 210..=212 => Self::AccessPointUnavailable,
            2 | 3 | 6 | 9 | 14 | 18..=24 | 202 => Self::AuthenticationFailed,
            4 | 5 | 7 | 8 | 10..=13 | 203 | 208 => Self::AssociationFailed,
            15..=17 | 204 | 209 => Self::HandshakeTimeout,
            205 | 206 => Self::ConnectionFailed,
            _ => Self::Other,
        }
    }

    /// Stable public category used by retained logs and evidence.
    #[must_use]
    pub const fn category(self) -> &'static str {
        match self {
            Self::Roaming => "roaming",
            Self::BeaconTimeout => "beacon_timeout",
            Self::AccessPointUnavailable => "access_point_unavailable",
            Self::AuthenticationFailed => "authentication_failed",
            Self::AssociationFailed => "association_failed",
            Self::HandshakeTimeout => "handshake_timeout",
            Self::ConnectionFailed => "connection_failed",
            Self::Other => "other",
        }
    }
}

/// Events admitted by the reconnect policy.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WifiReconnectEvent {
    StationDisconnected(WifiDisconnectReason),
    ProvisioningClientConnected,
    ProvisioningClientDisconnected,
    RetryDeadline,
    ReconnectLaunchFailed,
    Ipv4Assigned,
}

/// Side effects requested from the firmware shell.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WifiReconnectAction {
    IgnoreRoaming,
    EnableConfigurationNetwork,
    PublishDisconnected {
        reason: WifiDisconnectReason,
        retry_ordinal: u32,
    },
    ScheduleRetry {
        delay_ms: u64,
        retry_ordinal: u32,
    },
    StartReconnect {
        retry_ordinal: u32,
    },
    DisableConfigurationNetwork,
    PublishConnected {
        completed_retry_ordinal: u32,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ReconnectPhase {
    Connected,
    Waiting,
    Connecting,
    Suppressed,
}

/// Stateful reconnect policy owned by one firmware worker.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WifiReconnectState {
    phase: ReconnectPhase,
    retry_ordinal: u32,
    provisioning_clients: u16,
}

impl Default for WifiReconnectState {
    fn default() -> Self {
        Self {
            phase: ReconnectPhase::Connected,
            retry_ordinal: 0,
            provisioning_clients: 0,
        }
    }
}

impl WifiReconnectState {
    /// Applies one event and returns the ordered shell actions.
    #[must_use]
    pub fn apply(&mut self, event: WifiReconnectEvent) -> Vec<WifiReconnectAction> {
        match event {
            WifiReconnectEvent::StationDisconnected(WifiDisconnectReason::Roaming) => {
                vec![WifiReconnectAction::IgnoreRoaming]
            }
            WifiReconnectEvent::StationDisconnected(reason) => self.station_disconnected(reason),
            WifiReconnectEvent::ProvisioningClientConnected => {
                self.provisioning_clients = self.provisioning_clients.saturating_add(1);
                if self.phase != ReconnectPhase::Connected {
                    self.phase = ReconnectPhase::Suppressed;
                }
                Vec::new()
            }
            WifiReconnectEvent::ProvisioningClientDisconnected => {
                self.provisioning_clients = self.provisioning_clients.saturating_sub(1);
                if self.provisioning_clients == 0 && self.phase == ReconnectPhase::Suppressed {
                    self.phase = ReconnectPhase::Waiting;
                    return vec![self.schedule_retry()];
                }
                Vec::new()
            }
            WifiReconnectEvent::RetryDeadline => self.retry_deadline(),
            WifiReconnectEvent::ReconnectLaunchFailed => self.reconnect_launch_failed(),
            WifiReconnectEvent::Ipv4Assigned => self.ipv4_assigned(),
        }
    }

    fn station_disconnected(&mut self, reason: WifiDisconnectReason) -> Vec<WifiReconnectAction> {
        self.retry_ordinal = self.retry_ordinal.saturating_add(1).max(1);
        self.phase = if self.provisioning_clients == 0 {
            ReconnectPhase::Waiting
        } else {
            ReconnectPhase::Suppressed
        };
        let mut actions = vec![
            WifiReconnectAction::EnableConfigurationNetwork,
            WifiReconnectAction::PublishDisconnected {
                reason,
                retry_ordinal: self.retry_ordinal,
            },
        ];
        if self.phase == ReconnectPhase::Waiting {
            actions.push(self.schedule_retry());
        }
        actions
    }

    fn retry_deadline(&mut self) -> Vec<WifiReconnectAction> {
        if self.phase != ReconnectPhase::Waiting || self.provisioning_clients != 0 {
            return Vec::new();
        }
        self.phase = ReconnectPhase::Connecting;
        vec![WifiReconnectAction::StartReconnect {
            retry_ordinal: self.retry_ordinal,
        }]
    }

    fn reconnect_launch_failed(&mut self) -> Vec<WifiReconnectAction> {
        if self.phase != ReconnectPhase::Connecting {
            return Vec::new();
        }
        self.retry_ordinal = self.retry_ordinal.saturating_add(1);
        self.phase = if self.provisioning_clients == 0 {
            ReconnectPhase::Waiting
        } else {
            ReconnectPhase::Suppressed
        };
        if self.phase == ReconnectPhase::Waiting {
            vec![self.schedule_retry()]
        } else {
            Vec::new()
        }
    }

    fn ipv4_assigned(&mut self) -> Vec<WifiReconnectAction> {
        if self.phase == ReconnectPhase::Connected {
            return Vec::new();
        }
        let completed_retry_ordinal = self.retry_ordinal;
        self.phase = ReconnectPhase::Connected;
        self.retry_ordinal = 0;
        vec![
            WifiReconnectAction::DisableConfigurationNetwork,
            WifiReconnectAction::PublishConnected {
                completed_retry_ordinal,
            },
        ]
    }

    const fn schedule_retry(&self) -> WifiReconnectAction {
        WifiReconnectAction::ScheduleRetry {
            delay_ms: WIFI_RECONNECT_DELAY_MS,
            retry_ordinal: self.retry_ordinal,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{
        WifiDisconnectReason, WifiReconnectAction, WifiReconnectEvent, WifiReconnectState,
        WIFI_RECONNECT_DELAY_MS,
    };

    #[test]
    fn disconnect_enables_fallback_and_schedules_first_retry() {
        // Arrange
        let mut state = WifiReconnectState::default();

        // Act
        let actions = state.apply(WifiReconnectEvent::StationDisconnected(
            WifiDisconnectReason::BeaconTimeout,
        ));

        // Assert
        assert_eq!(
            actions,
            vec![
                WifiReconnectAction::EnableConfigurationNetwork,
                WifiReconnectAction::PublishDisconnected {
                    reason: WifiDisconnectReason::BeaconTimeout,
                    retry_ordinal: 1,
                },
                WifiReconnectAction::ScheduleRetry {
                    delay_ms: WIFI_RECONNECT_DELAY_MS,
                    retry_ordinal: 1,
                },
            ]
        );
    }

    #[test]
    fn retry_deadline_launches_nonblocking_attempt() {
        // Arrange
        let mut state = WifiReconnectState::default();
        let _ = state.apply(WifiReconnectEvent::StationDisconnected(
            WifiDisconnectReason::ConnectionFailed,
        ));

        // Act
        let actions = state.apply(WifiReconnectEvent::RetryDeadline);

        // Assert
        assert_eq!(
            actions,
            vec![WifiReconnectAction::StartReconnect { retry_ordinal: 1 }]
        );
    }

    #[test]
    fn ipv4_assignment_resets_retry_and_disables_fallback() {
        // Arrange
        let mut state = WifiReconnectState::default();
        let _ = state.apply(WifiReconnectEvent::StationDisconnected(
            WifiDisconnectReason::Other,
        ));
        let _ = state.apply(WifiReconnectEvent::RetryDeadline);

        // Act
        let actions = state.apply(WifiReconnectEvent::Ipv4Assigned);

        // Assert
        assert_eq!(
            actions,
            vec![
                WifiReconnectAction::DisableConfigurationNetwork,
                WifiReconnectAction::PublishConnected {
                    completed_retry_ordinal: 1,
                },
            ]
        );
        assert!(state.apply(WifiReconnectEvent::Ipv4Assigned).is_empty());
    }

    #[test]
    fn provisioning_client_suppresses_and_then_resumes_retry() {
        // Arrange
        let mut state = WifiReconnectState::default();
        let _ = state.apply(WifiReconnectEvent::ProvisioningClientConnected);

        // Act
        let disconnected = state.apply(WifiReconnectEvent::StationDisconnected(
            WifiDisconnectReason::AccessPointUnavailable,
        ));
        let resumed = state.apply(WifiReconnectEvent::ProvisioningClientDisconnected);

        // Assert
        assert_eq!(disconnected.len(), 2);
        assert_eq!(
            resumed,
            vec![WifiReconnectAction::ScheduleRetry {
                delay_ms: WIFI_RECONNECT_DELAY_MS,
                retry_ordinal: 1,
            }]
        );
    }

    #[test]
    fn roaming_event_does_not_change_connected_state() {
        // Arrange
        let mut state = WifiReconnectState::default();

        // Act
        let ignored = state.apply(WifiReconnectEvent::StationDisconnected(
            WifiDisconnectReason::Roaming,
        ));
        let assignment = state.apply(WifiReconnectEvent::Ipv4Assigned);

        // Assert
        assert_eq!(ignored, vec![WifiReconnectAction::IgnoreRoaming]);
        assert!(assignment.is_empty());
    }

    #[test]
    fn failed_launch_schedules_next_ordinal_without_overflow() {
        // Arrange
        let mut state = WifiReconnectState::default();
        let _ = state.apply(WifiReconnectEvent::StationDisconnected(
            WifiDisconnectReason::Other,
        ));
        let _ = state.apply(WifiReconnectEvent::RetryDeadline);

        // Act
        let actions = state.apply(WifiReconnectEvent::ReconnectLaunchFailed);

        // Assert
        assert_eq!(
            actions,
            vec![WifiReconnectAction::ScheduleRetry {
                delay_ms: WIFI_RECONNECT_DELAY_MS,
                retry_ordinal: 2,
            }]
        );
    }

    #[test]
    fn disconnect_reason_categories_are_closed() {
        // Arrange
        let reasons = [200, 201, 202, 203, 204, 205, 207, u16::MAX];

        // Act
        let categories = reasons.map(WifiDisconnectReason::from_esp_reason);

        // Assert
        assert_eq!(categories[0], WifiDisconnectReason::BeaconTimeout);
        assert_eq!(categories[6], WifiDisconnectReason::Roaming);
        assert_eq!(categories[7], WifiDisconnectReason::Other);
    }
}
