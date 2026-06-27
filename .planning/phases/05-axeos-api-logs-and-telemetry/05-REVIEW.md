---
phase: 05-axeos-api-logs-and-telemetry
reviewed: 2026-06-27T23:17:15Z
depth: standard
files_reviewed: 40
files_reviewed_list:
  - .cargo/config.toml
  - crates/bitaxe-api/BUILD.bazel
  - crates/bitaxe-api/Cargo.toml
  - crates/bitaxe-api/fixtures/api/asic-settings-ultra205.json
  - crates/bitaxe-api/fixtures/api/command-responses.json
  - crates/bitaxe-api/fixtures/api/live-telemetry-cases.json
  - crates/bitaxe-api/fixtures/api/log-buffer-cases.json
  - crates/bitaxe-api/fixtures/api/scoreboard-empty.json
  - crates/bitaxe-api/fixtures/api/settings-patch-cases.json
  - crates/bitaxe-api/fixtures/api/statistics-empty-compatible.json
  - crates/bitaxe-api/fixtures/api/system-info-ultra205-safe.json
  - crates/bitaxe-api/src/asic.rs
  - crates/bitaxe-api/src/commands.rs
  - crates/bitaxe-api/src/lib.rs
  - crates/bitaxe-api/src/logs.rs
  - crates/bitaxe-api/src/mining.rs
  - crates/bitaxe-api/src/route_shell.rs
  - crates/bitaxe-api/src/scoreboard.rs
  - crates/bitaxe-api/src/settings.rs
  - crates/bitaxe-api/src/snapshot.rs
  - crates/bitaxe-api/src/statistics.rs
  - crates/bitaxe-api/src/system.rs
  - crates/bitaxe-api/src/telemetry.rs
  - crates/bitaxe-api/src/wire.rs
  - docs/parity/checklist.md
  - docs/parity/evidence/phase-05-axeos-api-logs-and-telemetry.md
  - firmware/bitaxe/Cargo.toml
  - firmware/bitaxe/sdkconfig.defaults
  - firmware/bitaxe/src/http_api.rs
  - firmware/bitaxe/src/log_buffer.rs
  - firmware/bitaxe/src/main.rs
  - firmware/bitaxe/src/runtime_snapshot.rs
  - firmware/bitaxe/src/settings_adapter.rs
  - firmware/bitaxe/src/websocket_api.rs
  - tools/parity/BUILD.bazel
  - tools/parity/Cargo.toml
  - tools/parity/fixtures/api/axeos-route-usage.json
  - tools/parity/fixtures/api/phase05-required-routes.json
  - tools/parity/src/api_compare.rs
  - tools/parity/src/main.rs
findings:
  critical: 0
  warning: 3
  info: 0
  total: 3
status: issues_found
---

# Phase 05: Code Review Report

**Reviewed:** 2026-06-27T23:17:15Z
**Depth:** standard
**Files Reviewed:** 40
**Status:** issues_found

## Summary

Re-reviewed the explicit Phase 05 scope after iteration 2 fixes. This review was materially informed by `AGENTS.md`, `AGENTS.bright-builds.md`, `standards-overrides.md`, `standards/index.md`, `standards/core/architecture.md`, `standards/core/code-shape.md`, `standards/core/testing.md`, `standards/core/verification.md`, and `standards/languages/rust.md`.

Prior `CR-01` remains resolved: Origin handling now distinguishes missing, parsed, invalid, public named, public IPv4, and private Origin cases for both HTTP and WebSocket gates. Prior remaining warnings are resolved in their direct failure modes: settings reload reuses the held NVS partition, normal raw log cadence broadcast exists, and identify mode now stores and checks a 30 second expiry. The second fix pass introduced or exposed the WebSocket shared-baseline edge cases below.

Automated Bazel verification was not run because this re-review was constrained to review-only with `.planning/phases/05-axeos-api-logs-and-telemetry/05-REVIEW.md` as the only allowed write. Read-only checks confirmed the scoped JSON fixtures parse and the reviewed files are not Git-ignored.

## Warnings

### WR-01: New Live Telemetry Clients Reset The Shared Diff Baseline

**File:** `firmware/bitaxe/src/websocket_api.rs:118`
**Issue:** `live_connect_frame()` uses the single route-level `LiveTelemetryPlanner` to build a new client's full connect frame. If an existing `/api/ws/live` client has baseline A, the runtime state changes to B, and a second live client connects before the 500 ms cadence tick, this call overwrites the shared baseline with B for all clients. The new client receives B, but the original client never receives the A-to-B diff.
**Fix:**
```rust
// Keep connect-time full frames separate from the shared cadence baseline.
pub fn live_connect_frame(current: Value) -> Option<Value> {
    Some(bitaxe_api::live_telemetry_update_envelope(current))
}
```

Then initialize or clear the cadence planner only on zero-to-one and one-to-zero live-client transitions, or move to per-client live telemetry planners so each session has its own baseline. Add a test for "client 1 baseline A, state B, client 2 connects, next cadence still sends B diff to client 1."

### WR-02: Log WebSocket Connect Drains Pending Chunks And Drops Them

**File:** `firmware/bitaxe/src/http_api.rs:540`
**Issue:** On `/api/ws` connect, the handler calls `websocket_api::raw_log_chunks(&buffer)` and discards the returned chunks. That is safe only for the first client when the stream is inactive. If a log line is appended after an existing client is connected but before the next cadence broadcast, a second client connecting during that interval drains the shared cursor and drops the pending live log for the already-connected client.
**Fix:** Replace the connect-time drain with a state update that only establishes the baseline when the log route transitions from zero clients to one client. Do not call a draining API from the connect path unless the returned chunks are intentionally broadcast.

```rust
match route {
    WebSocketRouteKind::Logs => {
        websocket_api::log_client_connected(&log_buffer::retained_log_buffer());
        sys::ESP_OK
    }
    WebSocketRouteKind::LiveTelemetry => { /* unchanged full-frame send */ }
}
```

Add a regression test for "client 1 connected, one line pending, client 2 connects, next raw-log cadence still emits the pending line."

### WR-03: Log WebSocket Unregister Can Rewind The Shared Cursor

**File:** `firmware/bitaxe/src/websocket_api.rs:96`
**Issue:** `unregister_client()` updates the raw log planner with `RetainedLogBuffer::new()`. When one log client drops while another remains connected, `set_active_client_count()` sees an empty buffer with `total_written() == 0` and can clamp `next_abs` back to zero. The next broadcast with the real retained buffer can replay old retained logs to the remaining client instead of only new live lines.
**Fix:** Do not update the log planner with a synthetic empty buffer. Either leave the planner untouched during unregister and let the next `raw_log_chunks(&actual_buffer)` call reconcile the real client count, or pass the current retained buffer into unregister.

```rust
pub fn unregister_client(session: i32) {
    // remove session sets only; do not mutate RawLogStreamPlanner with an empty buffer
}
```

Add a regression test for "two log clients active, stream cursor after line N, one unregisters, next broadcast does not replay lines before N."

---

_Reviewed: 2026-06-27T23:17:15Z_
_Reviewer: the agent (gsd-code-reviewer)_
_Depth: standard_
