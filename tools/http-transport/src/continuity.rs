//! Pure HTTP/WebSocket continuity helpers shared by the legacy campaign observer and the soak observer.
use std::time::Duration;

use bitaxe_api::SystemInfoWire;
use serde_json::Value;

const RECONNECT_BACKOFF_MIN: Duration = Duration::from_secs(1);
const RECONNECT_BACKOFF_MAX: Duration = Duration::from_secs(5);

/// Bounded WebSocket reconnect delay: 1, 2, 4, then 5 seconds, reset by a successful connection.
#[derive(Debug, Clone, Copy)]
pub struct ReconnectBackoff {
    next_delay: Duration,
}

impl Default for ReconnectBackoff {
    fn default() -> Self {
        Self::new()
    }
}

impl ReconnectBackoff {
    #[must_use]
    pub fn new() -> Self {
        Self {
            next_delay: RECONNECT_BACKOFF_MIN,
        }
    }

    pub fn reset(&mut self) {
        self.next_delay = RECONNECT_BACKOFF_MIN;
    }

    pub fn take_delay(&mut self) -> Duration {
        let delay = self.next_delay;
        self.next_delay = (self.next_delay * 2).min(RECONNECT_BACKOFF_MAX);
        delay
    }
}

/// Applies one `/api/ws/live` frame: the first frame must be a full snapshot, later frames are
/// nested diffs merged into it. Returns the coherent snapshot, or `None` for an invalid frame.
pub fn apply_live_frame(bytes: &[u8], projection: &mut Option<Value>) -> Option<SystemInfoWire> {
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

/// A sequence advanced within a window: it appeared, or its last value exceeds its first.
#[must_use]
pub fn advances(first: Option<u64>, last: Option<u64>) -> bool {
    match (first, last) {
        (None, Some(_)) => true,
        (Some(first), Some(last)) => last > first,
        _ => false,
    }
}

#[must_use]
pub fn regresses(previous: Option<(u64, u64)>, current: (u64, u64)) -> bool {
    previous.is_some_and(|previous| current.0 < previous.0 || current.1 < previous.1)
}

pub fn update_gap(maximum: &mut u64, previous: &mut Option<u64>, current: u64) {
    if let Some(previous) = previous.replace(current) {
        *maximum = (*maximum).max(current.saturating_sub(previous));
    }
}

#[cfg(test)]
#[path = "continuity/tests.rs"]
mod tests;
