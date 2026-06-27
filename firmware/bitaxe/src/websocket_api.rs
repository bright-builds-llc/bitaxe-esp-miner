//! Firmware WebSocket state bridge for AxeOS logs and live telemetry.

use std::collections::BTreeSet;
use std::sync::{Mutex, OnceLock};

use bitaxe_api::{
    LiveTelemetryPlanner, RawLogStreamPlanner, RetainedLogBuffer, WebSocketRouteKind,
};
use serde_json::Value;

/// Upstream ESP HTTP server WebSocket client cap.
pub const MAX_WEBSOCKET_CLIENTS: usize = 10;

static WEBSOCKET_STATE: OnceLock<Mutex<WebSocketState>> = OnceLock::new();

/// Result of registering a WebSocket client.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WebSocketRegisterOutcome {
    /// Client was accepted and counted for its route.
    Accepted { active_clients: usize },
    /// Client was rejected before route registration.
    RejectedMaxClients { max_clients: usize },
}

#[derive(Default)]
struct WebSocketState {
    log_clients: BTreeSet<i32>,
    live_clients: BTreeSet<i32>,
    maybe_log_stream: Option<RawLogStreamPlanner>,
    live_telemetry: LiveTelemetryPlanner,
}

impl WebSocketState {
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
}

/// Registers or moves a client session to a WebSocket route.
#[must_use]
pub fn register_client(session: i32, route: WebSocketRouteKind) -> WebSocketRegisterOutcome {
    let state = WEBSOCKET_STATE.get_or_init(|| Mutex::new(WebSocketState::default()));
    let Ok(mut state) = state.lock() else {
        log::warn!("axeos_websocket_state=unavailable reason=mutex_poisoned");
        return WebSocketRegisterOutcome::RejectedMaxClients {
            max_clients: MAX_WEBSOCKET_CLIENTS,
        };
    };

    if !state.contains_session(session) && state.active_client_count() >= MAX_WEBSOCKET_CLIENTS {
        return WebSocketRegisterOutcome::RejectedMaxClients {
            max_clients: MAX_WEBSOCKET_CLIENTS,
        };
    }

    state.log_clients.remove(&session);
    state.live_clients.remove(&session);
    match route {
        WebSocketRouteKind::Logs => {
            state.log_clients.insert(session);
        }
        WebSocketRouteKind::LiveTelemetry => {
            state.live_clients.insert(session);
        }
    }

    WebSocketRegisterOutcome::Accepted {
        active_clients: state.active_route_client_count(route),
    }
}

/// Removes a client session from all WebSocket route state.
pub fn unregister_client(session: i32) {
    let state = WEBSOCKET_STATE.get_or_init(|| Mutex::new(WebSocketState::default()));
    let Ok(mut state) = state.lock() else {
        log::warn!("axeos_websocket_state=unavailable reason=mutex_poisoned");
        return;
    };

    state.log_clients.remove(&session);
    state.live_clients.remove(&session);
    let log_clients = state.log_clients.len();
    let live_clients = state.live_clients.len();
    state.live_telemetry.set_active_client_count(live_clients);
    if let Some(planner) = state.maybe_log_stream.as_mut() {
        planner.set_active_client_count(log_clients, &RetainedLogBuffer::new());
    }
}

/// Returns a point-in-time list of active sessions for a WebSocket route.
#[must_use]
pub fn client_sessions(route: WebSocketRouteKind) -> Vec<i32> {
    let state = WEBSOCKET_STATE.get_or_init(|| Mutex::new(WebSocketState::default()));
    let Ok(state) = state.lock() else {
        log::warn!("axeos_websocket_state=unavailable reason=mutex_poisoned");
        return Vec::new();
    };

    match route {
        WebSocketRouteKind::Logs => state.log_clients.iter().copied().collect(),
        WebSocketRouteKind::LiveTelemetry => state.live_clients.iter().copied().collect(),
    }
}

/// Plans the full live telemetry frame sent immediately after connection.
#[must_use]
pub fn live_connect_frame(current: Value) -> Option<Value> {
    let state = WEBSOCKET_STATE.get_or_init(|| Mutex::new(WebSocketState::default()));
    let Ok(mut state) = state.lock() else {
        log::warn!("axeos_websocket_state=unavailable reason=mutex_poisoned");
        return None;
    };

    let live_clients = state.live_clients.len();
    state.live_telemetry.set_active_client_count(live_clients);
    if live_clients == 1 {
        state.live_telemetry.seed_cadence_baseline(current.clone());
    }
    Some(state.live_telemetry.connect_frame(current))
}

/// Plans a cadence live telemetry frame for connected clients.
#[must_use]
pub fn live_cadence_frame(current: Value) -> Option<Value> {
    let state = WEBSOCKET_STATE.get_or_init(|| Mutex::new(WebSocketState::default()));
    let Ok(mut state) = state.lock() else {
        log::warn!("axeos_websocket_state=unavailable reason=mutex_poisoned");
        return None;
    };

    let live_clients = state.live_clients.len();
    state.live_telemetry.set_active_client_count(live_clients);
    state.live_telemetry.cadence_frame(current)
}

/// Drains raw retained-log chunks when `/api/ws` clients are active.
#[must_use]
pub fn raw_log_chunks(buffer: &RetainedLogBuffer) -> Vec<String> {
    let state = WEBSOCKET_STATE.get_or_init(|| Mutex::new(WebSocketState::default()));
    let Ok(mut state) = state.lock() else {
        log::warn!("axeos_websocket_state=unavailable reason=mutex_poisoned");
        return Vec::new();
    };

    let log_clients = state.log_clients.len();
    let planner = state
        .maybe_log_stream
        .get_or_insert_with(|| RawLogStreamPlanner::new(buffer));
    planner.set_active_client_count(log_clients, buffer);
    planner.drain_raw_chunks(buffer)
}
