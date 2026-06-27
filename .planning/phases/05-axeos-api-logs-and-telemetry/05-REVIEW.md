---
phase: 05-axeos-api-logs-and-telemetry
reviewed: 2026-06-27T22:53:59Z
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

**Reviewed:** 2026-06-27T22:53:59Z
**Depth:** standard
**Files Reviewed:** 40
**Status:** issues_found

## Summary

Re-reviewed the explicit Phase 05 source, fixture, firmware adapter, parity tooling, and parity documentation scope after code-review fixes. This review was materially informed by `AGENTS.md`, `AGENTS.bright-builds.md`, `standards-overrides.md`, `standards/index.md`, `standards/core/architecture.md`, `standards/core/code-shape.md`, `standards/core/testing.md`, `standards/core/verification.md`, and `standards/languages/rust.md`.

Prior `CR-01` is resolved: Origin handling now distinguishes missing, parsed, and invalid headers, and tests cover public named origins, public IPv4 origins, invalid origins, private origins, and WebSocket rejection. Prior `WR-01`, `WR-05`, and `WR-06` are resolved: live telemetry has a cadence task, API compare checks are schema-scoped, and failed/stale WebSocket sessions are unregistered. Prior `WR-02` was fixed for retained-history replay, but the raw log WebSocket now has no live broadcast path. Prior `WR-03` command-visible state is mostly resolved, but identify mode still ignores its advertised 30 second duration. Prior `WR-04` remains unsafe around NVS reload/persisted snapshot behavior.

## Warnings

### WR-01: Settings PATCH Reload Re-Takes The Default NVS Partition After Commit

**File:** `firmware/bitaxe/src/settings_adapter.rs:80`
**Issue:** `FirmwareSettingsAdapter::open()` takes the singleton default NVS partition at line 23 and stores it inside `self.nvs`. `reload()` then calls `EspDefaultNvsPartition::take()` again while the first partition is still owned by the same adapter. In `esp-idf-svc 0.52.1`, `take()` is guarded by a single `DEFAULT_TAKEN` flag and returns `ESP_ERR_INVALID_STATE` until the existing `NvsDefault` is dropped. That means a valid settings PATCH can write and commit NVS values, then fail at reload, return `Wrong API input`, and skip `apply_persisted_settings_writes()`. The client sees a failed request even though persistent settings may already have changed.
**Fix:**
```rust
pub struct FirmwareSettingsAdapter {
    partition: EspDefaultNvsPartition,
    nvs: EspNvs<NvsDefault>,
}

pub fn open() -> Result<Self, SettingsAdapterFailure> {
    let partition = EspDefaultNvsPartition::take().map_err(settings_failure)?;
    let nvs = EspNvs::new(partition.clone(), NVS_NAMESPACE, true).map_err(settings_failure)?;
    Ok(Self { partition, nvs })
}

fn reload(&mut self) -> Result<(), SettingsAdapterFailure> {
    let _reloaded = EspNvs::new(self.partition.clone(), NVS_NAMESPACE, false)
        .map_err(settings_failure)?;
    Ok(())
}
```

Also make reload or startup populate the shared `NvsSnapshot` from actual NVS values, then add an adapter test/fake proving a committed PATCH returns success and is visible in the next API snapshot.

### WR-02: Raw Log WebSocket Clients Never Receive Live Log Chunks

**File:** `firmware/bitaxe/src/http_api.rs:75`
**Issue:** The `/api/ws` fix removed retained-history replay on connect, which resolves the old no-backlog violation. However, the only background loop now calls `broadcast_live_telemetry_cadence()` and `prune_stale_websocket_sessions()`. The log connect path at line 527 only initializes `websocket_api::raw_log_chunks(&buffer)` and discards the result, and `append_runtime_log_line()` does not notify WebSocket clients. As a result, accepted `/api/ws` clients receive neither retained history nor new live logs, so the Phase 05 raw log WebSocket route is effectively inert.
**Fix:**
```rust
fn live_telemetry_cadence_loop(server_addr: usize) {
    let server = server_addr as sys::httpd_handle_t;
    loop {
        std::thread::sleep(Duration::from_millis(LIVE_TELEMETRY_CADENCE_MS));
        broadcast_live_telemetry_cadence(server);
        broadcast_raw_log_chunks(server);
        prune_stale_websocket_sessions(server);
    }
}

fn broadcast_raw_log_chunks(server: sys::httpd_handle_t) {
    let buffer = log_buffer::retained_log_buffer();
    for chunk in websocket_api::raw_log_chunks(&buffer) {
        broadcast_websocket_text_frame(server, WebSocketRouteKind::Logs, &chunk);
    }
}
```

Keep the baseline-at-current-end behavior on connect, and add a firmware adapter test or live smoke proving an old retained line is not sent while a new appended line is sent to `/api/ws`.

### WR-03: Identify Mode Ignores The Advertised 30 Second Duration

**File:** `firmware/bitaxe/src/runtime_snapshot.rs:69`
**Issue:** `identify_plan()` returns `IdentifyModeEffect::Enable { duration_ms: 30_000 }` and the public response says the device says "Hi!" for 30 seconds, but the firmware state application only sets `IdentifyMode::Active` and drops `duration_ms`. There is no timer, deadline, or expiry check, so the next identify request after 30 seconds still sees active mode and disables it instead of starting a fresh 30 second identify window. This leaves the command-visible state inconsistent with the response contract.
**Fix:** Store an expiry deadline in `CommandVisibleState` or schedule a timer after the response is sent. `identify_mode()` should return `Inactive` once the deadline has passed, and explicit disable should cancel the pending expiry. Add a focused test for enable, post-duration expiry, and explicit disable before expiry.

---

_Reviewed: 2026-06-27T22:53:59Z_
_Reviewer: the agent (gsd-code-reviewer)_
_Depth: standard_
