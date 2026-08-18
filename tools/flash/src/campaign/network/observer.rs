use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use bitaxe_api::SystemInfoWire;
use bitaxe_http_transport::{PlainWebSocket, StrictHttpClient, WebSocketRead};
use serde_json::Value;

use super::super::CampaignTerminalCategory;
use super::model::{
    CampaignNetworkEvidence, NetworkAccumulator, NetworkCorrelationFailure, NetworkTransport,
    SharedSerialState, TrustedNetworkTarget, TERMINAL_NETWORK_DEADLINE_SECONDS,
};
use super::terminal_settlement::{
    terminal_settlement, TerminalSettlementDecision, TerminalSettlementInput,
};

const HTTP_POLL_INTERVAL: Duration = Duration::from_secs(5);
const HTTP_DEADLINE: Duration = Duration::from_secs(3);
const WEBSOCKET_CONNECT_TIMEOUT: Duration = Duration::from_secs(3);
const WEBSOCKET_IO_TIMEOUT: Duration = Duration::from_millis(250);
const RECONNECT_BACKOFF_MIN: Duration = Duration::from_secs(1);
const RECONNECT_BACKOFF_MAX: Duration = Duration::from_secs(5);

#[derive(Debug, Clone, Copy)]
struct ReconnectBackoff {
    next_delay: Duration,
}

impl ReconnectBackoff {
    fn new() -> Self {
        Self {
            next_delay: RECONNECT_BACKOFF_MIN,
        }
    }

    fn reset(&mut self) {
        self.next_delay = RECONNECT_BACKOFF_MIN;
    }

    fn take_delay(&mut self) -> Duration {
        let delay = self.next_delay;
        self.next_delay = (self.next_delay * 2).min(RECONNECT_BACKOFF_MAX);
        delay
    }
}

pub(super) fn observe_network(
    target: TrustedNetworkTarget,
    shared: Arc<Mutex<SharedSerialState>>,
) -> CampaignNetworkEvidence {
    let http = match StrictHttpClient::new(&target.origin) {
        Ok(http) => http,
        Err(_) => return CampaignNetworkEvidence::from_unobserved(&shared),
    };
    let started = Instant::now();
    let mut accumulator = NetworkAccumulator::new(target.clone());
    let mut maybe_websocket = None;
    let mut websocket_projection = None;
    let mut websocket_connected_once = false;
    let mut next_websocket_attempt = started;
    let mut reconnect_backoff = ReconnectBackoff::new();
    let mut next_http_poll = started;
    let mut maybe_terminal_deadline = None;

    loop {
        let serial = shared_snapshot(&shared);
        if let Some(category) = serial.maybe_failure {
            accumulator.fail(category);
        }
        accumulator.close_elapsed_windows(serial.latest_active_ms, &serial);
        if accumulator.take_recovery_pause_request() {
            let _result = http.post_pause_once(Instant::now() + HTTP_DEADLINE);
        }

        let now = Instant::now();
        if serial.terminal_consumed && maybe_terminal_deadline.is_none() {
            maybe_terminal_deadline =
                Some(now + Duration::from_secs(TERMINAL_NETWORK_DEADLINE_SECONDS));
            next_http_poll = now;
        }
        if now >= next_http_poll {
            observe_http(&http, &serial, started, &mut accumulator);
            next_http_poll = now + HTTP_POLL_INTERVAL;
        }

        if maybe_websocket.is_none() && now >= next_websocket_attempt {
            match PlainWebSocket::connect(
                &target.origin,
                "/api/ws/live",
                WEBSOCKET_CONNECT_TIMEOUT,
                WEBSOCKET_IO_TIMEOUT,
            ) {
                Ok(socket) => {
                    if websocket_connected_once {
                        accumulator.websocket_reconnect_count =
                            accumulator.websocket_reconnect_count.saturating_add(1);
                    }
                    websocket_connected_once = true;
                    reconnect_backoff.reset();
                    websocket_projection = None;
                    maybe_websocket = Some(socket);
                }
                Err(_) => {
                    accumulator.note_websocket_connect_failure();
                    next_websocket_attempt = now + reconnect_backoff.take_delay();
                }
            }
        }

        if let Some(websocket) = maybe_websocket.as_mut() {
            match websocket.read() {
                Ok(WebSocketRead::Text(bytes)) => {
                    let maybe_sample = apply_live_frame(&bytes, &mut websocket_projection);
                    if let Some(sample) = maybe_sample {
                        if serial.terminal_consumed {
                            accumulator
                                .record_terminal_sample(NetworkTransport::WebSocket, &sample);
                        } else if serial.active {
                            accumulator.record_active_sample(
                                NetworkTransport::WebSocket,
                                serial.latest_active_ms,
                                elapsed_millis(started),
                                &sample,
                            );
                        }
                    } else {
                        accumulator.fail_correlation(
                            NetworkCorrelationFailure::WebsocketProjectionInvalid,
                        );
                    }
                }
                Ok(WebSocketRead::Timeout) => {}
                Ok(WebSocketRead::Closed) => {
                    accumulator.note_websocket_peer_close();
                    maybe_websocket = None;
                    websocket_projection = None;
                    next_websocket_attempt = Instant::now() + reconnect_backoff.take_delay();
                }
                Err(kind) => {
                    accumulator.note_websocket_failure(kind);
                    maybe_websocket = None;
                    websocket_projection = None;
                    next_websocket_attempt = Instant::now() + reconnect_backoff.take_delay();
                }
            }
        }

        accumulator.terminal_consumed_observed |= serial.terminal_consumed;
        let settlement = terminal_settlement(TerminalSettlementInput {
            prior_failure: accumulator.maybe_failure.is_some(),
            serial_finished: serial.serial_finished,
            terminal_consumed: serial.terminal_consumed,
            terminal_http_valid: accumulator.terminal_http_valid,
            terminal_websocket_valid: accumulator.terminal_websocket_valid,
            terminal_deadline_expired: maybe_terminal_deadline
                .is_some_and(|deadline| Instant::now() >= deadline),
        });
        accumulator.note_terminal_settlement(settlement);
        match settlement {
            TerminalSettlementDecision::Continue => {}
            TerminalSettlementDecision::RequestSerialClose => request_serial_close(&shared),
            TerminalSettlementDecision::AcceptAfterSerialClose
            | TerminalSettlementDecision::PreserveFailureAfterSerialClose => break,
            TerminalSettlementDecision::FailAfterSerialClose => {
                accumulator.fail(CampaignTerminalCategory::TerminalStateUnconfirmed);
                break;
            }
        }
    }

    if let Some(websocket) = maybe_websocket.as_mut() {
        websocket.close();
    }
    let serial = shared_snapshot(&shared);
    accumulator.finish(&serial)
}

fn observe_http(
    http: &StrictHttpClient,
    serial: &SharedSerialState,
    started: Instant,
    accumulator: &mut NetworkAccumulator,
) {
    let Ok(observation) = http.get_system_info(Instant::now() + HTTP_DEADLINE) else {
        return;
    };
    let Some(response) = observation
        .maybe_http_response()
        .filter(|response| response.status() == 200)
    else {
        return;
    };
    let Ok(sample) = serde_json::from_slice::<SystemInfoWire>(response.body()) else {
        accumulator.fail_correlation(NetworkCorrelationFailure::HttpProjectionInvalid);
        return;
    };
    if serial.terminal_consumed {
        accumulator.record_terminal_sample(NetworkTransport::Http, &sample);
    } else if serial.active {
        accumulator.record_active_sample(
            NetworkTransport::Http,
            serial.latest_active_ms,
            elapsed_millis(started),
            &sample,
        );
    }
}

fn apply_live_frame(bytes: &[u8], projection: &mut Option<Value>) -> Option<SystemInfoWire> {
    let frame: Value = serde_json::from_slice(bytes).ok()?;
    if frame.get("event")?.as_str()? != "update" {
        return None;
    }
    let update = frame.get("data")?.as_object()?;
    match projection {
        Some(current) => merge_object(current.as_object_mut()?, update),
        None => *projection = Some(Value::Object(update.clone())),
    }
    serde_json::from_value(projection.as_ref()?.clone()).ok()
}

fn merge_object(
    current: &mut serde_json::Map<String, Value>,
    update: &serde_json::Map<String, Value>,
) {
    for (key, value) in update {
        if let (Some(Value::Object(current_nested)), Value::Object(update_nested)) =
            (current.get_mut(key), value)
        {
            merge_object(current_nested, update_nested);
        } else {
            current.insert(key.clone(), value.clone());
        }
    }
}

fn shared_snapshot(shared: &Arc<Mutex<SharedSerialState>>) -> SharedSerialState {
    shared.lock().map_or_else(
        |_| SharedSerialState {
            serial_finished: true,
            maybe_failure: Some(CampaignTerminalCategory::NetworkCorrelationFailed),
            ..SharedSerialState::default()
        },
        |state| state.clone(),
    )
}

fn request_serial_close(shared: &Arc<Mutex<SharedSerialState>>) {
    if let Ok(mut state) = shared.lock() {
        state.network_stop_requested = true;
    }
}

fn elapsed_millis(started: Instant) -> u64 {
    u64::try_from(started.elapsed().as_millis()).unwrap_or(u64::MAX)
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use bitaxe_api::{ApiSnapshot, OperatorSnapshotRevision, SystemInfoWire};
    use serde_json::json;

    use super::{apply_live_frame, ReconnectBackoff};

    #[test]
    fn reconnect_backoff_is_one_two_four_then_five_seconds() {
        // Arrange
        let mut backoff = ReconnectBackoff::new();

        // Act
        let delays = [(); 6].map(|()| backoff.take_delay());

        // Assert
        assert_eq!(
            delays,
            [
                Duration::from_secs(1),
                Duration::from_secs(2),
                Duration::from_secs(4),
                Duration::from_secs(5),
                Duration::from_secs(5),
                Duration::from_secs(5),
            ]
        );
    }

    #[test]
    fn successful_connection_resets_reconnect_backoff() {
        // Arrange
        let mut backoff = ReconnectBackoff::new();
        assert_eq!(backoff.take_delay(), Duration::from_secs(1));
        assert_eq!(backoff.take_delay(), Duration::from_secs(2));

        // Act
        backoff.reset();

        // Assert
        assert_eq!(backoff.take_delay(), Duration::from_secs(1));
    }

    #[test]
    fn full_connect_frame_and_nested_diff_reconstruct_one_coherent_snapshot() {
        // Arrange
        let mut full = SystemInfoWire::from_snapshot(&ApiSnapshot::safe_ultra_205());
        full.operator_snapshot_revision =
            OperatorSnapshotRevision::new(1).expect("nonzero revision");
        full.runtime_health.maybe_task_watchdog_feed_sequence = Some(10);
        let full_frame = serde_json::to_vec(&json!({
            "event": "update",
            "data": full,
        }))
        .expect("full frame");
        let diff_frame = serde_json::to_vec(&json!({
            "event": "update",
            "data": {
                "operatorSnapshotRevision": 2,
                "runtimeHealth": {
                    "taskWatchdogFeedSequence": 11,
                },
            },
        }))
        .expect("diff frame");
        let mut projection = None;

        // Act
        let first = apply_live_frame(&full_frame, &mut projection).expect("full snapshot");
        let second = apply_live_frame(&diff_frame, &mut projection).expect("merged snapshot");

        // Assert
        assert_eq!(first.operator_snapshot_revision.get(), 1);
        assert_eq!(second.operator_snapshot_revision.get(), 2);
        assert_eq!(
            second.runtime_health.maybe_task_watchdog_feed_sequence,
            Some(11)
        );
    }

    #[test]
    fn partial_first_frame_is_rejected_without_a_projection() {
        // Arrange
        let frame = serde_json::to_vec(&json!({
            "event": "update",
            "data": { "operatorSnapshotRevision": 2 },
        }))
        .expect("partial frame");
        let mut projection = None;

        // Act
        let maybe_sample = apply_live_frame(&frame, &mut projection);

        // Assert
        assert!(maybe_sample.is_none());
    }
}
