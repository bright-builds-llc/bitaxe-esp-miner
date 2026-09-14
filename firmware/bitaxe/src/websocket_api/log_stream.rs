//! Avoid retained-log allocation when the cadence has no raw-log consumers.
use std::sync::Mutex;

use bitaxe_api::{RetainedLogBuffer, WebSocketRouteKind, WebSocketState};

#[derive(Debug)]
pub(super) struct LogStateUnavailable;

pub(super) fn cadence_chunks(
    state: &Mutex<WebSocketState>,
    snapshot: impl FnOnce() -> RetainedLogBuffer,
) -> Result<Vec<String>, LogStateUnavailable> {
    let subscribed = {
        let state = state.lock().map_err(|_| LogStateUnavailable)?;
        state.active_route_client_count(WebSocketRouteKind::Logs) != 0
    };
    if !subscribed {
        return Ok(Vec::new());
    }

    // Never hold route membership while taking the independent retained-log lock.
    let buffer = snapshot();
    let mut state = state.lock().map_err(|_| LogStateUnavailable)?;
    Ok(state.raw_log_chunks(&buffer))
}

#[cfg(test)]
mod tests {
    use std::cell::Cell;
    use std::panic::{catch_unwind, AssertUnwindSafe};

    use super::*;

    #[test]
    fn live_only_subscription_does_not_snapshot_retained_logs() {
        // Arrange
        let mut state = WebSocketState::default();
        let _registration = state.register_client(1, WebSocketRouteKind::LiveTelemetry);
        let state = Mutex::new(state);
        let snapshots = Cell::new(0);

        // Act
        let chunks = cadence_chunks(&state, || {
            snapshots.set(snapshots.get() + 1);
            RetainedLogBuffer::new()
        })
        .expect("healthy route state");

        // Assert
        assert!(chunks.is_empty());
        assert_eq!(snapshots.get(), 0);
    }

    #[test]
    fn active_snapshot_runs_without_membership_lock_and_preserves_cursor() {
        // Arrange
        let mut retained = RetainedLogBuffer::with_capacity(1024);
        retained.append("before connection\n");
        let mut state = WebSocketState::default();
        let _registration = state.register_client(1, WebSocketRouteKind::Logs);
        state.log_client_connected(&retained);
        retained.append("after connection\n");
        let state = Mutex::new(state);

        // Act
        let first = cadence_chunks(&state, || {
            let _available = state
                .try_lock()
                .expect("membership lock released before snapshot");
            retained.clone()
        })
        .expect("healthy route state");
        let second = cadence_chunks(&state, || retained.clone()).expect("healthy route state");

        // Assert
        assert_eq!(first.concat(), "after connection\n");
        assert!(second.is_empty());
    }

    #[test]
    fn disconnect_and_reconnect_keep_existing_log_cursor_baseline() {
        // Arrange
        let mut retained = RetainedLogBuffer::with_capacity(1024);
        let mut state = WebSocketState::default();
        let _registration = state.register_client(1, WebSocketRouteKind::Logs);
        state.log_client_connected(&retained);
        let lease = state.client_leases(WebSocketRouteKind::Logs)[0];
        let state = Mutex::new(state);
        retained.append("old subscriber pending\n");

        // Act
        let disconnected = cadence_chunks(&state, || {
            assert!(state
                .lock()
                .expect("route owner")
                .unregister_if_current(lease));
            retained.clone()
        })
        .expect("healthy route state");
        retained.append("while disconnected\n");
        let inactive = cadence_chunks(&state, || panic!("no retained copy without log clients"))
            .expect("healthy route state");
        {
            let mut state = state.lock().expect("route owner");
            let _registration = state.register_client(2, WebSocketRouteKind::Logs);
            state.log_client_connected(&retained);
        }
        retained.append("after reconnection\n");
        let reconnected = cadence_chunks(&state, || retained.clone()).expect("healthy route state");

        // Assert
        assert!(disconnected.is_empty());
        assert!(inactive.is_empty());
        assert_eq!(reconnected.concat(), "after reconnection\n");
    }

    #[test]
    fn poisoned_membership_reports_failure_without_retained_allocation() {
        // Arrange
        let state = Mutex::new(WebSocketState::default());
        let poisoned = catch_unwind(AssertUnwindSafe(|| {
            let _guard = state.lock().expect("initial lock");
            panic!("poison fixture");
        }));
        assert!(poisoned.is_err());

        // Act
        let result = cadence_chunks(&state, || {
            panic!("poisoned membership must not snapshot logs")
        });

        // Assert
        assert!(result.is_err());
        assert!(state.is_poisoned());
    }
}
