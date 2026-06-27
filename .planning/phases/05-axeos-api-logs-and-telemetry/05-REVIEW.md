---
phase: 05-axeos-api-logs-and-telemetry
reviewed: 2026-06-27T22:13:49Z
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
  critical: 1
  warning: 6
  info: 0
  total: 7
status: issues_found
---

# Phase 05: Code Review Report

**Reviewed:** 2026-06-27T22:13:49Z
**Depth:** standard
**Files Reviewed:** 40
**Status:** issues_found

## Summary

Reviewed the explicit Phase 05 source, fixture, and parity-doc scope. Generated lockfiles were loaded from the requested context but excluded from the reviewable source count per workflow rules. This review was materially informed by `AGENTS.md`, `AGENTS.bright-builds.md`, `standards-overrides.md`, `standards/index.md`, `standards/core/architecture.md`, `standards/core/code-shape.md`, `standards/core/testing.md`, `standards/core/verification.md`, and `standards/languages/rust.md`.

The pure API crate is generally shaped well around fixture-backed planning and DTO mapping. The higher-risk gaps are in firmware adapter behavior and parity evidence: the Origin gate can allow cross-origin private-network requests, WebSocket live telemetry has no firmware cadence loop, raw log WebSockets replay retained history despite the pure planner claiming no backlog, command/settings effects are not reflected in runtime state, and the API compare tool can miss schema-specific property removals.

## Critical Issues

### CR-01: Origin Parse Failures Bypass The Private-Origin Gate

**File:** `crates/bitaxe-api/src/route_shell.rs:284`
**Issue:** `maybe_origin_ip_from_header()` returns `None` for any Origin host that is not an IPv4 literal, and `is_access_allowed()` treats `None` as if the origin were the private request IP. In firmware, `origin_ip_from_raw()` also returns `None` for truncated or otherwise unparseable Origin headers at `firmware/bitaxe/src/http_api.rs:469`. A browser request from a public origin such as `https://example.com` to a miner on `192.168.x.x` therefore passes the gate because the request IP is private, even though the Origin is not. That exposes unauthenticated POST/WebSocket surfaces such as restart, settings, and logs to cross-origin request forgery from any public web page a LAN user visits. The pinned reference keeps "Origin header present but invalid" distinct from "Origin header absent" by leaving the origin IP as `0`, which then fails the private-range check.
**Fix:**
```rust
pub enum OriginGate {
    Missing,
    Parsed(Ipv4Addr),
    Invalid,
}

pub struct RouteAccessInput {
    pub ap_mode_enabled: bool,
    pub request_ip: Ipv4Addr,
    pub origin: OriginGate,
}

fn is_access_allowed(input: RouteAccessInput) -> bool {
    if input.ap_mode_enabled {
        return true;
    }

    if !is_private_ipv4(input.request_ip) {
        return false;
    }

    match input.origin {
        OriginGate::Missing => true,
        OriginGate::Parsed(origin_ip) => is_private_ipv4(origin_ip),
        OriginGate::Invalid => false,
    }
}
```

Update the firmware header reader to return `Missing` only when `Origin` is absent, `Invalid` when the header exists but cannot be parsed as an allowed private origin, and add HTTP/WebSocket tests for `Origin: https://example.com`, overlong Origin, public IPv4 Origin, missing Origin, and private IPv4 Origin.

## Warnings

### WR-01: Live WebSocket Telemetry Never Runs The 500 ms Cadence

**File:** `firmware/bitaxe/src/http_api.rs:438`
**Issue:** `/api/ws/live` sends one full frame during the WebSocket connect path and then calls `websocket_api::live_cadence_frame(current)` once only to seed the baseline. There is no timer, FreeRTOS task, or broadcast loop that runs `LIVE_TELEMETRY_CADENCE_MS` and sends subsequent diff frames to registered live clients. The checklist and evidence say full/diff/cadence fixtures are covered, but the firmware route currently cannot deliver live updates after connection.
**Fix:** Add a firmware-owned cadence task after HTTP startup that sleeps for `LIVE_TELEMETRY_CADENCE_MS`, collects `collect_api_snapshot()`, computes `websocket_api::live_cadence_frame(...)`, serializes any returned frame, and broadcasts it to every registered `/api/ws/live` session using the ESP-IDF async WebSocket send API. Add a firmware adapter test or live smoke that observes a connect frame followed by at least one diff frame.

### WR-02: Raw Log WebSocket Replays Retained History Despite The No-Backlog Contract

**File:** `firmware/bitaxe/src/http_api.rs:423`
**Issue:** On `/api/ws` connect, the firmware loops over `log_buffer::download_chunks()` and sends the retained log history before initializing the raw stream baseline. That contradicts the pure `RawLogStreamPlanner` tests in `crates/bitaxe-api/src/logs.rs`, which assert that raw WebSocket clients start at the current end and receive no retained backlog. It also makes `LOG-001` overstate "raw WebSocket baseline semantics" for firmware, because the actual adapter sends retained history on connect.
**Fix:** Keep retained history delivery only on `GET /api/system/logs`. For `/api/ws`, initialize the `RawLogStreamPlanner` with the current retained buffer and do not send download chunks during connect; subsequent log pump iterations should send only `websocket_api::raw_log_chunks(&buffer)` output to active clients. Add a firmware-level test/smoke for "retained old line before connect is not sent over `/api/ws`".

### WR-03: Non-Restart Command Effects Are Dropped Instead Of Mutating Visible State

**File:** `firmware/bitaxe/src/http_api.rs:654`
**Issue:** The pure command layer models pause/resume, identify, and block-found dismiss as effects, but the firmware `apply_command_effect()` only logs those effects. `handle_identify()` always plans from `IdentifyMode::Inactive`, `handle_block_found_dismiss()` builds a hardcoded already-dismissed state, and `collect_api_snapshot()` starts from a fresh safe snapshot each request. As a result, command routes return success messages while `GET /api/system/info` and later command calls cannot reflect the requested visible state changes.
**Fix:** Add a small firmware runtime-state store, for example a `Mutex` holding `MiningRuntimeState`, `IdentifyMode`, and `BlockFoundNotificationState`. Apply `apply_mining_activity_effect()` and `apply_block_found_dismiss_effect()` to that state after the response is sent, toggle identify mode from the stored value, and have `collect_api_snapshot()` merge the stored command-visible state into the returned `ApiSnapshot`. Add route-level tests for pause then info, resume then info, identify on/off, and block-found dismiss.

### WR-04: Settings PATCH Persistence Is Not Reflected In Runtime Snapshots

**File:** `firmware/bitaxe/src/settings_adapter.rs:85`
**Issue:** `current_settings_snapshot()` always returns `NvsSnapshot::new()`, and `collect_api_snapshot()` always starts from `ApiSnapshot::safe_ultra_205()`. A successful `PATCH /api/system` can write and commit NVS values, but subsequent API snapshots still report defaults such as hostname/frequency/fan settings. The empty current snapshot also makes hostname effect planning unreliable because no-op hostname patches look like changes every time.
**Fix:** Implement `current_settings_snapshot()` by reading the actual ESP-IDF NVS namespace into a typed `NvsSnapshot`, and make `reload()` update a shared loaded-settings state that `collect_api_snapshot()` uses. Add an integration-style firmware adapter test with a fake NVS snapshot proving that accepted PATCH writes are visible in the next system-info snapshot and that unchanged hostname patches do not emit a live hostname effect.

### WR-05: API Compare Checks Required Properties Globally, Not In The Target Schema

**File:** `tools/parity/src/api_compare.rs:324`
**Issue:** `validate_schema_evidence()` passes `schema_route.schema` into the error message but `openapi_has_property()` searches the entire OpenAPI text for `property:`. If a required property exists in any other schema, the check passes even when the specific schema named by the route is missing that property. That can produce a false `API-001` verified claim for schema compatibility.
**Fix:**
```rust
if !openapi_schema_has_property(openapi_yaml, &schema_route.schema, property) {
    validation_errors.push(format!(
        "OpenAPI schema {} for {} {} missing property {property}",
        schema_route.schema, schema_route.method, schema_route.path
    ));
}
```

Implement `openapi_schema_has_property()` with a structured YAML parser if available, or at minimum restrict the search to the `components.schemas.<schema>.properties` block. Add a negative test where `message` or another required property exists only in a different schema and verify the compare fails.

### WR-06: WebSocket Sessions Can Leak After Failed Sends Or Abrupt Disconnects

**File:** `firmware/bitaxe/src/http_api.rs:373`
**Issue:** The upgrade handler registers the session before sending connect frames. If `send_websocket_connect_frames()` fails, the function returns the ESP error without unregistering the session. After a successful upgrade, the only unregister path is receipt of a CLOSE frame at `firmware/bitaxe/src/http_api.rs:411`; abrupt socket drops can leave stale session IDs in `WEBSOCKET_STATE`. Stale entries inflate active-client counts and can eventually reject real clients with the max-client guard.
**Fix:** Unregister the session when connect-frame sending fails, and configure an ESP-IDF close callback or periodic prune that removes sessions whose sockets are no longer valid. Add a test around `register_client()` plus failed connect send to prove active-client count is restored.

---

_Reviewed: 2026-06-27T22:13:49Z_
_Reviewer: the agent (gsd-code-reviewer)_
_Depth: standard_
