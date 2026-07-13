//! Pure `/api/ws/live` telemetry envelope, diff, and cadence contract.
//!
//! Reference breadcrumbs:
//! - `reference/esp-miner/main/http_server/websocket_api.c`
//! - `reference/esp-miner/main/http_server/cjson_utils.c`
//! - `reference/esp-miner/main/http_server/axe-os/src/app/services/live-data.service.ts`

use serde_json::{json, Map, Value};

/// Upstream-compatible live telemetry cadence.
pub const LIVE_TELEMETRY_CADENCE_MS: u64 = 500;

/// Wraps telemetry data in the upstream `update` event envelope.
#[must_use]
pub fn live_telemetry_update_envelope(data: Value) -> Value {
    json!({
        "event": "update",
        "data": data,
    })
}

/// Returns a structured diff between the previous and current telemetry payloads.
#[must_use]
pub fn live_telemetry_diff(maybe_old: Option<&Value>, new: &Value) -> Option<Value> {
    let Some(old) = maybe_old else {
        return Some(new.clone());
    };

    if old == new {
        return None;
    }

    match (old, new) {
        (Value::Object(old_object), Value::Object(new_object)) => {
            object_diff(old_object, new_object).map(Value::Object)
        }
        _ => Some(new.clone()),
    }
}

/// Host-testable planner for `/api/ws/live` update decisions.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct LiveTelemetryPlanner {
    maybe_baseline: Option<Value>,
    active_clients: usize,
}

impl LiveTelemetryPlanner {
    /// Updates the active live-client count and clears baseline while hibernating.
    pub fn set_active_client_count(&mut self, active_clients: usize) {
        self.active_clients = active_clients;
        if active_clients == 0 {
            self.maybe_baseline = None;
        }
    }

    /// Plans the full connect-time update frame without changing cadence state.
    #[must_use]
    pub fn connect_frame(&self, current: Value) -> Value {
        live_telemetry_update_envelope(current)
    }

    /// Seeds the shared cadence baseline after the first active client connects.
    pub fn seed_cadence_baseline(&mut self, current: Value) {
        self.maybe_baseline = Some(current);
    }

    /// Plans a cadence update, returning no frame for unchanged state or no clients.
    #[must_use]
    pub fn cadence_frame(&mut self, current: Value) -> Option<Value> {
        if self.active_clients == 0 {
            self.maybe_baseline = None;
            return None;
        }

        let Some(baseline) = self.maybe_baseline.as_ref() else {
            self.maybe_baseline = Some(current);
            return None;
        };

        let maybe_diff = live_telemetry_diff(Some(baseline), &current);
        self.maybe_baseline = Some(current);
        maybe_diff.map(live_telemetry_update_envelope)
    }
}

fn object_diff(
    old_object: &Map<String, Value>,
    new_object: &Map<String, Value>,
) -> Option<Map<String, Value>> {
    let mut diff = Map::new();

    for (key, new_value) in new_object {
        let Some(old_value) = old_object.get(key) else {
            diff.insert(key.clone(), new_value.clone());
            continue;
        };

        if old_value == new_value {
            continue;
        }

        if let (Value::Object(old_nested), Value::Object(new_nested)) = (old_value, new_value) {
            if let Some(sub_diff) = object_diff(old_nested, new_nested) {
                diff.insert(key.clone(), Value::Object(sub_diff));
            }
            continue;
        }

        diff.insert(key.clone(), new_value.clone());
    }

    if diff.is_empty() {
        return None;
    }

    Some(diff)
}

#[cfg(test)]
mod tests {
    use serde::Deserialize;
    use serde_json::{json, Value};

    use crate::telemetry::{
        live_telemetry_diff, live_telemetry_update_envelope, LiveTelemetryPlanner,
        LIVE_TELEMETRY_CADENCE_MS,
    };

    #[derive(Debug, Deserialize)]
    struct LiveTelemetryFixture {
        cadence_ms: u64,
        full_state: Value,
        changed_state: Value,
        expected_connect_frame: Value,
        expected_diff_frame: Value,
        expected_nested_diff: Value,
    }

    fn fixture() -> LiveTelemetryFixture {
        serde_json::from_str(include_str!("../fixtures/api/live-telemetry-cases.json"))
            .expect("live telemetry fixture should parse")
    }

    #[test]
    fn connect_time_telemetry_sends_full_update_envelope() {
        // Arrange
        let fixture = fixture();
        let mut planner = LiveTelemetryPlanner::default();
        planner.set_active_client_count(1);

        // Act
        let frame = planner.connect_frame(fixture.full_state);

        // Assert
        assert_eq!(frame, fixture.expected_connect_frame);
        assert_eq!(frame["event"], "update");
        assert!(frame["data"].is_object());
    }

    #[test]
    fn unchanged_telemetry_after_baseline_sends_no_frame() {
        // Arrange
        let fixture = fixture();
        let mut planner = LiveTelemetryPlanner::default();
        planner.set_active_client_count(1);
        planner.seed_cadence_baseline(fixture.full_state.clone());

        // Act
        let frame = planner.cadence_frame(fixture.full_state);

        // Assert
        assert_eq!(frame, None);
    }

    #[test]
    fn changed_telemetry_sends_diff_only_update_envelope() {
        // Arrange
        let fixture = fixture();
        let mut planner = LiveTelemetryPlanner::default();
        planner.set_active_client_count(1);
        planner.seed_cadence_baseline(fixture.full_state);

        // Act
        let frame = planner.cadence_frame(fixture.changed_state);

        // Assert
        assert_eq!(frame, Some(fixture.expected_diff_frame));
    }

    #[test]
    fn no_live_clients_clear_baseline_so_reconnect_sends_full_state_again() {
        // Arrange
        let fixture = fixture();
        let mut planner = LiveTelemetryPlanner::default();
        planner.set_active_client_count(1);
        planner.seed_cadence_baseline(fixture.full_state.clone());
        planner.set_active_client_count(0);
        let idle_frame = planner.cadence_frame(fixture.changed_state);
        planner.set_active_client_count(1);

        // Act
        let reconnect_frame = planner.connect_frame(fixture.full_state);

        // Assert
        assert_eq!(idle_frame, None);
        assert_eq!(reconnect_frame, fixture.expected_connect_frame);
    }

    #[test]
    fn connect_time_telemetry_does_not_replace_cadence_baseline() {
        // Arrange
        let fixture = fixture();
        let mut planner = LiveTelemetryPlanner::default();
        planner.set_active_client_count(1);
        planner.seed_cadence_baseline(fixture.full_state);
        let changed_state = fixture.changed_state.clone();
        let expected_connect_frame = live_telemetry_update_envelope(changed_state.clone());
        let connect_frame = planner.connect_frame(changed_state.clone());

        // Act
        let cadence_frame = planner.cadence_frame(changed_state);

        // Assert
        assert_eq!(connect_frame, expected_connect_frame);
        assert_eq!(cadence_frame, Some(fixture.expected_diff_frame));
    }

    #[test]
    fn live_telemetry_cadence_is_exactly_500_ms() {
        // Arrange
        let fixture = fixture();

        // Act
        let cadence_ms = LIVE_TELEMETRY_CADENCE_MS;

        // Assert
        assert_eq!(cadence_ms, 500);
        assert_eq!(cadence_ms, fixture.cadence_ms);
    }

    #[test]
    fn nested_object_diff_preserves_granular_changed_fields() {
        // Arrange
        let old = json!({
            "version": "v1",
            "mining": {
                "hashRate": 1.0,
                "sharesAccepted": 2,
                "nested": {
                    "kept": true,
                    "changed": "old"
                }
            }
        });
        let new = json!({
            "version": "v1",
            "mining": {
                "hashRate": 1.0,
                "sharesAccepted": 3,
                "nested": {
                    "kept": true,
                    "changed": "new"
                }
            }
        });
        let fixture = fixture();

        // Act
        let diff = live_telemetry_diff(Some(&old), &new);

        // Assert
        assert_eq!(diff, Some(fixture.expected_nested_diff));
    }

    #[test]
    fn telemetry_truth_change_emits_nested_diff_without_compatibility_numeric_change() {
        // Arrange
        let old = json!({
            "power": 0.0,
            "powerStatus": {
                "state": "unavailable",
                "reason": { "kind": "unavailable", "code": "not_yet_observed" }
            }
        });
        let new = json!({
            "power": 0.0,
            "powerStatus": {
                "state": "fresh",
                "stamp": { "bootSession": 7, "sequence": 1, "acquiredAtMs": 250 }
            }
        });

        // Act
        let diff = live_telemetry_diff(Some(&old), &new);

        // Assert
        assert_eq!(
            diff,
            Some(json!({
                "powerStatus": {
                    "state": "fresh",
                    "stamp": { "bootSession": 7, "sequence": 1, "acquiredAtMs": 250 }
                }
            }))
        );
    }
}
