---
phase: 05
fixed_at: 2026-06-27T23:08:25Z
review_path: .planning/phases/05-axeos-api-logs-and-telemetry/05-REVIEW.md
iteration: 2
findings_in_scope: 3
fixed: 3
skipped: 0
status: all_fixed
---

# Phase 05: Code Review Fix Report

**Fixed at:** 2026-06-27T23:08:25Z
**Source review:** .planning/phases/05-axeos-api-logs-and-telemetry/05-REVIEW.md
**Iteration:** 2

**Summary:**
- Findings in scope: 3
- Fixed: 3
- Skipped: 0

Each fix was committed atomically after `cargo fmt --all`, `cargo clippy --all-targets --all-features -- -D warnings`, `cargo build --all-targets --all-features`, and `cargo test --all-features` passed. Firmware-touching fixes also passed `cargo +esp check -p bitaxe-firmware --target xtensa-esp32s3-espidf`.

## Fixed Issues

### WR-01: Settings PATCH Reload Re-Takes The Default NVS Partition After Commit

**Files modified:** `crates/bitaxe-config/src/persistence.rs`, `firmware/bitaxe/src/http_api.rs`, `firmware/bitaxe/src/settings_adapter.rs`
**Commit:** 5144f98
**Applied fix:** fixed: requires human verification. `FirmwareSettingsAdapter` now retains the default NVS partition and reloads through a cloned partition handle instead of calling `take()` again. Startup/open/reload paths refresh the runtime settings snapshot from NVS, and committed writes update the in-process snapshot through a tested pure `NvsSnapshot::apply_writes` helper.

### WR-02: Raw Log WebSocket Clients Never Receive Live Log Chunks

**Files modified:** `firmware/bitaxe/src/http_api.rs`
**Commit:** 271b02b
**Applied fix:** Added raw log chunk broadcasting to the existing live WebSocket cadence loop. `/api/ws` connection handling still initializes the raw log stream at the retained buffer's current end, while the cadence loop now drains new chunks and broadcasts them to active raw log sessions.

### WR-03: Identify Mode Ignores The Advertised 30 Second Duration

**Files modified:** `crates/bitaxe-api/src/commands.rs`, `crates/bitaxe-api/src/lib.rs`, `firmware/bitaxe/src/runtime_snapshot.rs`
**Commit:** d81dd75
**Applied fix:** fixed: requires human verification. Added pure `IdentifyModeState` deadline handling with host tests for post-duration expiry and explicit disable before expiry. Firmware command-visible state now stores the timed identify state and evaluates it against ESP uptime when planning the next identify command.

---

_Fixed: 2026-06-27T23:08:25Z_
_Fixer: the agent (gsd-code-fixer)_
_Iteration: 2_
