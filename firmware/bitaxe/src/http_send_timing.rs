//! Times every blocking send on the HTTP server task, so a soak can tell a slow
//! server-task write apart from a Wi-Fi or TCP stall
//! (`task-http-server-task-stall-diagnostics`). Counts and durations only.

use std::sync::Mutex;

use bitaxe_core::runtime_health::{HttpServerSendsSnapshot, SendDurationTracker};
use esp_idf_svc::sys;

/// The server task's two blocking send kinds.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum SendKind {
    HttpResponse,
    WebSocketFrame,
}

struct Trackers {
    http_responses: SendDurationTracker,
    websocket_frames: SendDurationTracker,
}

static TRACKERS: Mutex<Trackers> = Mutex::new(Trackers {
    http_responses: SendDurationTracker::new(),
    websocket_frames: SendDurationTracker::new(),
});

/// Runs one send and records its duration and outcome.
pub(crate) fn timed<T, E>(kind: SendKind, send: impl FnOnce() -> Result<T, E>) -> Result<T, E> {
    let started_ms = crate::runtime_uptime::millis();
    let result = send();
    record(kind, started_ms, result.is_ok());
    result
}

/// Runs one ESP-IDF send and records its duration and outcome.
pub(crate) fn timed_esp(kind: SendKind, send: impl FnOnce() -> sys::esp_err_t) -> sys::esp_err_t {
    let started_ms = crate::runtime_uptime::millis();
    let result = send();
    record(kind, started_ms, result == sys::ESP_OK);
    result
}

fn record(kind: SendKind, started_ms: u64, succeeded: bool) {
    let now_ms = crate::runtime_uptime::millis();
    let duration_ms = u32::try_from(now_ms.saturating_sub(started_ms)).unwrap_or(u32::MAX);
    // Diagnostics only: a poisoned lock skips this sample rather than failing a send.
    let Ok(mut trackers) = TRACKERS.lock() else {
        return;
    };
    let tracker = match kind {
        SendKind::HttpResponse => &mut trackers.http_responses,
        SendKind::WebSocketFrame => &mut trackers.websocket_frames,
    };
    tracker.record(now_ms, duration_ms, succeeded);
}

/// The counts and worst durations for the runtime-health projection.
pub(crate) fn snapshot() -> HttpServerSendsSnapshot {
    let now_ms = crate::runtime_uptime::millis();
    let Ok(trackers) = TRACKERS.lock() else {
        return HttpServerSendsSnapshot::default();
    };
    HttpServerSendsSnapshot {
        http_responses: trackers.http_responses.snapshot(now_ms),
        websocket_sends: trackers.websocket_frames.snapshot(now_ms),
    }
}
