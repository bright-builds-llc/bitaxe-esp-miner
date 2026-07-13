---
phase: 05
fixed_at: 2026-06-27T23:27:39Z
review_path: .planning/phases/05-axeos-api-logs-and-telemetry/05-REVIEW.md
iteration: 3
findings_in_scope: 3
fixed: 3
skipped: 0
status: all_fixed
---

# Phase 05: Code Review Fix Report

**Fixed at:** 2026-06-27T23:27:39Z
**Source review:** .planning/phases/05-axeos-api-logs-and-telemetry/05-REVIEW.md
**Iteration:** 3

**Summary:**
- Findings in scope: 3
- Fixed: 3
- Skipped: 0

Each fix was committed atomically after `cargo fmt --all`, `cargo clippy --all-targets --all-features -- -D warnings`, `cargo build --all-targets --all-features`, and `cargo test --all-features` passed. After the source fix commits, `cargo check -p bitaxe-firmware --target xtensa-esp32s3-espidf` also passed with the ESP environment loaded.

## Fixed Issues

### WR-01: New Live Telemetry Clients Reset The Shared Diff Baseline

**Files modified:** `crates/bitaxe-api/src/telemetry.rs`, `firmware/bitaxe/src/http_api.rs`, `firmware/bitaxe/src/websocket_api.rs`
**Commit:** 7791df6
**Applied fix:** fixed: requires human verification. Live connect frames now use a full update envelope without replacing the shared cadence baseline. The firmware seeds the cadence baseline only for the first active live client and no longer calls the cadence planner from the connect path. Added regression coverage for a second connect preserving the pending A-to-B cadence diff.

### WR-02: Log WebSocket Connect Drains Pending Chunks And Drops Them

**Files modified:** `crates/bitaxe-api/src/logs.rs`, `firmware/bitaxe/src/http_api.rs`, `firmware/bitaxe/src/websocket_api.rs`
**Commit:** b72bd07
**Applied fix:** fixed: requires human verification. Log WebSocket connect now calls a non-draining `log_client_connected` baseline update instead of `raw_log_chunks`. The draining raw-log API remains on the cadence broadcast path. Added regression coverage for an additional client connecting while a live log chunk is pending.

### WR-03: Log WebSocket Unregister Can Rewind The Shared Cursor

**Files modified:** `crates/bitaxe-api/src/logs.rs`, `firmware/bitaxe/src/websocket_api.rs`
**Commit:** 5e1fc25
**Applied fix:** fixed: requires human verification. Log unregister no longer updates the raw planner with a synthetic empty retained buffer; it clears the planner only after the log route reaches zero clients. The raw planner also no longer clamps an active stream cursor backward when presented with a lower-total buffer. Added regression coverage for active-client drop preserving the cursor and not replaying retained history.

---

_Fixed: 2026-06-27T23:27:39Z_
_Fixer: the agent (gsd-code-fixer)_
_Iteration: 3_
