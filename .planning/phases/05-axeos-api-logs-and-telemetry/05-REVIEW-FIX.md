---
phase: 05
fixed_at: 2026-06-27T22:46:27Z
review_path: .planning/phases/05-axeos-api-logs-and-telemetry/05-REVIEW.md
iteration: 1
findings_in_scope: 7
fixed: 7
skipped: 0
status: all_fixed
---

# Phase 05: Code Review Fix Report

**Fixed at:** 2026-06-27T22:46:27Z
**Source review:** .planning/phases/05-axeos-api-logs-and-telemetry/05-REVIEW.md
**Iteration:** 1

**Summary:**
- Findings in scope: 7
- Fixed: 7
- Skipped: 0

Each fix was committed atomically after `cargo fmt --all`, `cargo clippy --all-targets --all-features -- -D warnings`, `cargo build --all-targets --all-features`, and `cargo test --all-features` passed. Firmware-touching fixes also passed `cargo check -p bitaxe-firmware --target xtensa-esp32s3-espidf`.

## Fixed Issues

### CR-01: Origin Parse Failures Bypass The Private-Origin Gate

**Files modified:** `crates/bitaxe-api/src/lib.rs`, `crates/bitaxe-api/src/route_shell.rs`, `firmware/bitaxe/src/http_api.rs`
**Commit:** de57a5e
**Applied fix:** fixed: requires human verification. Split Origin handling into missing, parsed, and invalid states; missing Origin may use the private peer fallback, while invalid, truncated, or public Origin values fail closed for HTTP and WebSocket access gates.

### WR-01: Live WebSocket Telemetry Never Runs The 500 ms Cadence

**Files modified:** `firmware/bitaxe/src/http_api.rs`, `firmware/bitaxe/src/websocket_api.rs`
**Commit:** 8d3821c
**Applied fix:** Added a firmware-owned live telemetry cadence thread that collects snapshots every `LIVE_TELEMETRY_CADENCE_MS`, serializes diff frames, broadcasts them to registered live sessions, and unregisters sessions when async sends fail.

### WR-02: Raw Log WebSocket Replays Retained History Despite The No-Backlog Contract

**Files modified:** `firmware/bitaxe/src/http_api.rs`
**Commit:** 1677da9
**Applied fix:** Removed retained-history replay from `/api/ws` connection handling so raw WebSocket clients initialize at the current retained-log end without receiving a backlog.

### WR-03: Non-Restart Command Effects Are Dropped Instead Of Mutating Visible State

**Files modified:** `Cargo.lock`, `crates/bitaxe-api/src/lib.rs`, `crates/bitaxe-api/src/snapshot.rs`, `crates/bitaxe-api/src/wire.rs`, `firmware/bitaxe/Cargo.toml`, `firmware/bitaxe/src/http_api.rs`, `firmware/bitaxe/src/runtime_snapshot.rs`
**Commit:** d2696f0
**Applied fix:** fixed: requires human verification. Added command-visible runtime state for mining, identify, and block-found notification effects, then made subsequent command planning and `/api/system/info` snapshots read that state.

### WR-04: Settings PATCH Persistence Is Not Reflected In Runtime Snapshots

**Files modified:** `crates/bitaxe-api/src/snapshot.rs`, `crates/bitaxe-api/src/wire.rs`, `firmware/bitaxe/src/settings_adapter.rs`, `firmware/bitaxe/src/runtime_snapshot.rs`, `firmware/bitaxe/src/http_api.rs`
**Commit:** 8413f5c
**Applied fix:** fixed: requires human verification. Successful PATCH writes now update an in-process NVS snapshot, and runtime API snapshots overlay hostname, frequency, voltage, and fan config fields from that snapshot.

Residual: cold-boot hydration from physical NVS remains outside this Phase 05 fix and should be verified in a later firmware/hardware pass.

### WR-05: API Compare Checks Required Properties Globally, Not In The Target Schema

**Files modified:** `tools/parity/src/api_compare.rs`
**Commit:** 3c6b3b6
**Applied fix:** Scoped OpenAPI property checks to the named component schema or route-local inline schema, added indentation-aware YAML block slicing, and added a regression where a decoy property in another schema no longer satisfies the target schema check.

### WR-06: WebSocket Sessions Can Leak After Failed Sends Or Abrupt Disconnects

**Files modified:** `firmware/bitaxe/src/http_api.rs`
**Commit:** a60c985
**Applied fix:** fixed: requires human verification. WebSocket upgrades now reject invalid sockets, unregister sessions after failed connect-frame sends, unregister on receive errors, and periodically send WebSocket ping control frames to prune stale log and live sessions.

## Skipped Issues

None.

---

_Fixed: 2026-06-27T22:46:27Z_
_Fixer: the agent (gsd-code-fixer)_
_Iteration: 1_
