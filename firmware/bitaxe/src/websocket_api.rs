//! Firmware WebSocket state bridge for AxeOS logs and live telemetry.

use std::sync::{Mutex, OnceLock};

pub use bitaxe_api::WebSocketRegisterOutcome;
use bitaxe_api::{RetainedLogBuffer, WebSocketRouteKind, WebSocketState};
use serde_json::Value;

/// Upstream ESP HTTP server WebSocket client cap.
pub const MAX_WEBSOCKET_CLIENTS: usize = bitaxe_api::MAX_WEBSOCKET_CLIENTS;

static WEBSOCKET_STATE: OnceLock<Mutex<WebSocketState>> = OnceLock::new();

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

    state.register_client(session, route)
}

/// Removes a client session from all WebSocket route state.
pub fn unregister_client(session: i32) {
    let state = WEBSOCKET_STATE.get_or_init(|| Mutex::new(WebSocketState::default()));
    let Ok(mut state) = state.lock() else {
        log::warn!("axeos_websocket_state=unavailable reason=mutex_poisoned");
        return;
    };

    state.unregister_client(session);
}

/// Returns a point-in-time list of active sessions for a WebSocket route.
#[must_use]
pub fn client_sessions(route: WebSocketRouteKind) -> Vec<i32> {
    let state = WEBSOCKET_STATE.get_or_init(|| Mutex::new(WebSocketState::default()));
    let Ok(state) = state.lock() else {
        log::warn!("axeos_websocket_state=unavailable reason=mutex_poisoned");
        return Vec::new();
    };

    state.client_sessions(route)
}

/// Plans the full live telemetry frame sent immediately after connection.
#[must_use]
pub fn live_connect_frame(current: Value) -> Option<Value> {
    let state = WEBSOCKET_STATE.get_or_init(|| Mutex::new(WebSocketState::default()));
    let Ok(mut state) = state.lock() else {
        log::warn!("axeos_websocket_state=unavailable reason=mutex_poisoned");
        return None;
    };

    Some(state.live_connect_frame(current))
}

/// Plans a cadence live telemetry frame for connected clients.
#[must_use]
pub fn live_cadence_frame(current: Value) -> Option<Value> {
    let state = WEBSOCKET_STATE.get_or_init(|| Mutex::new(WebSocketState::default()));
    let Ok(mut state) = state.lock() else {
        log::warn!("axeos_websocket_state=unavailable reason=mutex_poisoned");
        return None;
    };

    state.live_cadence_frame(current)
}

/// Updates raw retained-log stream state after a `/api/ws` client connects.
pub fn log_client_connected(buffer: &RetainedLogBuffer) {
    let state = WEBSOCKET_STATE.get_or_init(|| Mutex::new(WebSocketState::default()));
    let Ok(mut state) = state.lock() else {
        log::warn!("axeos_websocket_state=unavailable reason=mutex_poisoned");
        return;
    };

    state.log_client_connected(buffer);
}

/// Drains raw retained-log chunks when `/api/ws` clients are active.
#[must_use]
pub fn raw_log_chunks(buffer: &RetainedLogBuffer) -> Vec<String> {
    let state = WEBSOCKET_STATE.get_or_init(|| Mutex::new(WebSocketState::default()));
    let Ok(mut state) = state.lock() else {
        log::warn!("axeos_websocket_state=unavailable reason=mutex_poisoned");
        return Vec::new();
    };

    state.raw_log_chunks(buffer)
}
