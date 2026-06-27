//! Pure WebSocket session state for AxeOS logs and live telemetry.

use std::collections::BTreeSet;

use serde_json::Value;

use crate::{LiveTelemetryPlanner, RawLogStreamPlanner, RetainedLogBuffer, WebSocketRouteKind};

/// Upstream ESP HTTP server WebSocket client cap.
pub const MAX_WEBSOCKET_CLIENTS: usize = 10;

/// Result of registering a WebSocket client.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WebSocketRegisterOutcome {
    /// Client was accepted and counted for its route.
    Accepted { active_clients: usize },
    /// Client was rejected before route registration.
    RejectedMaxClients { max_clients: usize },
}

/// Route-local WebSocket session and stream planner state.
#[derive(Debug, Default)]
pub struct WebSocketState {
    log_clients: BTreeSet<i32>,
    live_clients: BTreeSet<i32>,
    maybe_log_stream: Option<RawLogStreamPlanner>,
    live_telemetry: LiveTelemetryPlanner,
}

impl WebSocketState {
    /// Registers or moves a client session to a WebSocket route.
    #[must_use]
    pub fn register_client(
        &mut self,
        session: i32,
        route: WebSocketRouteKind,
    ) -> WebSocketRegisterOutcome {
        if !self.contains_session(session) && self.active_client_count() >= MAX_WEBSOCKET_CLIENTS {
            return WebSocketRegisterOutcome::RejectedMaxClients {
                max_clients: MAX_WEBSOCKET_CLIENTS,
            };
        }

        let was_log_client = self.log_clients.remove(&session);
        let was_live_client = self.live_clients.remove(&session);
        match route {
            WebSocketRouteKind::Logs => {
                self.log_clients.insert(session);
            }
            WebSocketRouteKind::LiveTelemetry => {
                self.live_clients.insert(session);
            }
        }

        self.hibernate_routes_after_move(was_log_client, was_live_client, route);

        WebSocketRegisterOutcome::Accepted {
            active_clients: self.active_route_client_count(route),
        }
    }

    /// Removes a client session from all WebSocket route state.
    pub fn unregister_client(&mut self, session: i32) {
        self.log_clients.remove(&session);
        self.live_clients.remove(&session);
        self.live_telemetry
            .set_active_client_count(self.live_clients.len());
        if self.log_clients.is_empty() {
            self.maybe_log_stream = None;
        }
    }

    /// Returns a point-in-time list of active sessions for a WebSocket route.
    #[must_use]
    pub fn client_sessions(&self, route: WebSocketRouteKind) -> Vec<i32> {
        match route {
            WebSocketRouteKind::Logs => self.log_clients.iter().copied().collect(),
            WebSocketRouteKind::LiveTelemetry => self.live_clients.iter().copied().collect(),
        }
    }

    /// Plans the full live telemetry frame sent immediately after connection.
    #[must_use]
    pub fn live_connect_frame(&mut self, current: Value) -> Value {
        let live_clients = self.live_clients.len();
        self.live_telemetry.set_active_client_count(live_clients);
        if live_clients == 1 {
            self.live_telemetry.seed_cadence_baseline(current.clone());
        }
        self.live_telemetry.connect_frame(current)
    }

    /// Plans a cadence live telemetry frame for connected clients.
    #[must_use]
    pub fn live_cadence_frame(&mut self, current: Value) -> Option<Value> {
        let live_clients = self.live_clients.len();
        self.live_telemetry.set_active_client_count(live_clients);
        self.live_telemetry.cadence_frame(current)
    }

    /// Updates raw retained-log stream state after a `/api/ws` client connects.
    pub fn log_client_connected(&mut self, buffer: &RetainedLogBuffer) {
        let log_clients = self.log_clients.len();
        let planner = self
            .maybe_log_stream
            .get_or_insert_with(|| RawLogStreamPlanner::new(buffer));
        planner.set_active_client_count(log_clients, buffer);
    }

    /// Drains raw retained-log chunks when `/api/ws` clients are active.
    #[must_use]
    pub fn raw_log_chunks(&mut self, buffer: &RetainedLogBuffer) -> Vec<String> {
        let log_clients = self.log_clients.len();
        let planner = self
            .maybe_log_stream
            .get_or_insert_with(|| RawLogStreamPlanner::new(buffer));
        planner.set_active_client_count(log_clients, buffer);
        planner.drain_raw_chunks(buffer)
    }

    fn active_client_count(&self) -> usize {
        self.log_clients.len() + self.live_clients.len()
    }

    fn active_route_client_count(&self, route: WebSocketRouteKind) -> usize {
        match route {
            WebSocketRouteKind::Logs => self.log_clients.len(),
            WebSocketRouteKind::LiveTelemetry => self.live_clients.len(),
        }
    }

    fn contains_session(&self, session: i32) -> bool {
        self.log_clients.contains(&session) || self.live_clients.contains(&session)
    }

    fn hibernate_routes_after_move(
        &mut self,
        was_log_client: bool,
        was_live_client: bool,
        route: WebSocketRouteKind,
    ) {
        if was_log_client && route != WebSocketRouteKind::Logs && self.log_clients.is_empty() {
            self.maybe_log_stream = None;
        }

        if was_live_client && route != WebSocketRouteKind::LiveTelemetry {
            self.live_telemetry
                .set_active_client_count(self.live_clients.len());
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        RetainedLogBuffer, WebSocketRegisterOutcome, WebSocketRouteKind, WebSocketState,
        MAX_WEBSOCKET_CLIENTS,
    };

    #[test]
    fn route_move_from_logs_to_live_hibernates_raw_log_stream_before_next_cadence() {
        // Arrange
        let mut state = WebSocketState::default();
        let mut buffer = RetainedLogBuffer::new();
        buffer.append("retained old line\n");
        let first_log_registration = state.register_client(1, WebSocketRouteKind::Logs);
        state.log_client_connected(&buffer);
        buffer.append("pending while log client was active\n");

        // Act
        let live_registration = state.register_client(1, WebSocketRouteKind::LiveTelemetry);
        buffer.append("written while no log clients existed\n");
        let second_log_registration = state.register_client(2, WebSocketRouteKind::Logs);
        state.log_client_connected(&buffer);
        let chunks = state.raw_log_chunks(&buffer);

        // Assert
        assert_eq!(
            first_log_registration,
            WebSocketRegisterOutcome::Accepted { active_clients: 1 }
        );
        assert_eq!(
            live_registration,
            WebSocketRegisterOutcome::Accepted { active_clients: 1 }
        );
        assert_eq!(
            second_log_registration,
            WebSocketRegisterOutcome::Accepted { active_clients: 1 }
        );
        assert!(chunks.is_empty());
    }

    #[test]
    fn moving_existing_session_to_another_route_does_not_consume_client_capacity() {
        // Arrange
        let mut state = WebSocketState::default();
        for session in 0..MAX_WEBSOCKET_CLIENTS as i32 {
            let outcome = state.register_client(session, WebSocketRouteKind::Logs);
            assert!(matches!(outcome, WebSocketRegisterOutcome::Accepted { .. }));
        }

        // Act
        let moved = state.register_client(0, WebSocketRouteKind::LiveTelemetry);
        let rejected = state.register_client(99, WebSocketRouteKind::Logs);

        // Assert
        assert_eq!(
            moved,
            WebSocketRegisterOutcome::Accepted { active_clients: 1 }
        );
        assert_eq!(
            rejected,
            WebSocketRegisterOutcome::RejectedMaxClients {
                max_clients: MAX_WEBSOCKET_CLIENTS
            }
        );
    }
}
