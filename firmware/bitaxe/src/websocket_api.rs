//! Firmware WebSocket state bridge for AxeOS logs and live telemetry.

use std::sync::{Mutex, OnceLock};

use bitaxe_api::{RetainedLogBuffer, WebSocketRouteKind, WebSocketState};
pub use bitaxe_api::{WebSocketClientLease, WebSocketRegisterOutcome};
use serde_json::Value;

mod log_stream;

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

    let outcome = state.register_client(session, route);
    crate::telemetry_cadence::RECORDER.subscribers_changed(
        state.active_route_client_count(WebSocketRouteKind::LiveTelemetry) as u32,
    );
    outcome
}

/// Removes a client only when the exact connection generation still owns it.
pub fn unregister_if_current(lease: WebSocketClientLease) -> bool {
    let state = WEBSOCKET_STATE.get_or_init(|| Mutex::new(WebSocketState::default()));
    let Ok(mut state) = state.lock() else {
        log::warn!("axeos_websocket_state=unavailable reason=mutex_poisoned");
        return false;
    };

    let removed = state.unregister_if_current(lease);
    crate::telemetry_cadence::RECORDER.subscribers_changed(
        state.active_route_client_count(WebSocketRouteKind::LiveTelemetry) as u32,
    );
    removed
}

/// Reports whether an exact connection generation still owns its route.
#[must_use]
pub fn is_current(lease: WebSocketClientLease) -> bool {
    let state = WEBSOCKET_STATE.get_or_init(|| Mutex::new(WebSocketState::default()));
    let Ok(state) = state.lock() else {
        log::warn!("axeos_websocket_state=unavailable reason=mutex_poisoned");
        return false;
    };

    state.is_current(lease)
}

/// Returns a point-in-time list of active connection leases for a route.
#[must_use]
pub fn client_leases(route: WebSocketRouteKind) -> Vec<WebSocketClientLease> {
    let state = WEBSOCKET_STATE.get_or_init(|| Mutex::new(WebSocketState::default()));
    let Ok(state) = state.lock() else {
        log::warn!("axeos_websocket_state=unavailable reason=mutex_poisoned");
        return Vec::new();
    };

    state.client_leases(route)
}

/// Plans the full live telemetry frame sent immediately after connection.
#[must_use]
pub fn maybe_live_connect_frame(current: Value) -> Option<Value> {
    let state = WEBSOCKET_STATE.get_or_init(|| Mutex::new(WebSocketState::default()));
    let Ok(mut state) = state.lock() else {
        log::warn!("axeos_websocket_state=unavailable reason=mutex_poisoned");
        return None;
    };

    Some(state.live_connect_frame(current))
}

/// Explicit failure distinguishes poisoned owner state from valid unchanged-data suppression.
#[derive(Debug)]
pub struct LiveCadenceStateUnavailable;

/// Plans a cadence live telemetry frame for connected clients.
#[must_use]
pub fn plan_live_cadence_frame(
    current: Value,
) -> Result<Option<Value>, LiveCadenceStateUnavailable> {
    let state = WEBSOCKET_STATE.get_or_init(|| Mutex::new(WebSocketState::default()));
    let Ok(mut state) = state.lock() else {
        log::warn!("axeos_websocket_state=unavailable reason=mutex_poisoned");
        return Err(LiveCadenceStateUnavailable);
    };

    Ok(state.maybe_live_cadence_frame(current))
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

/// Plans periodic raw-log output without copying retained storage when nobody subscribes.
#[must_use]
pub fn cadence_log_chunks(snapshot: impl FnOnce() -> RetainedLogBuffer) -> Vec<String> {
    let state = WEBSOCKET_STATE.get_or_init(|| Mutex::new(WebSocketState::default()));
    match log_stream::cadence_chunks(state, snapshot) {
        Ok(chunks) => chunks,
        Err(_) => {
            log::warn!("axeos_websocket_state=unavailable reason=mutex_poisoned");
            Vec::new()
        }
    }
}
