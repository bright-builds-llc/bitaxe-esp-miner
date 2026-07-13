---
phase: 05-axeos-api-logs-and-telemetry
plan: 05
subsystem: api
tags: [rust, esp-idf, axeos-api, http, websocket, nvs, telemetry, logs]

requires:
  - phase: 05-axeos-api-logs-and-telemetry
    provides: "Plans 05-02 through 05-06 pure API settings, system, log, telemetry, and command contracts"
provides:
  - "ESP-IDF firmware HTTP/WebSocket route shell for Phase 05 AxeOS API routes"
  - "Private-network/AP-origin access gate reused by HTTP and WebSocket handlers"
  - "Settings PATCH body cap before JSON parsing or NVS persistence"
  - "Firmware NVS settings adapter with write, commit, reload ordering"
  - "Retained log download and WebSocket registration shell for raw logs and live telemetry"
affects: [phase-06-safety-telemetry, phase-07-ota-static-assets, axeos-api, firmware]

tech-stack:
  added: ["bitaxe-api firmware dependency", "ESP-IDF HTTP server WebSocket support"]
  patterns:
    - "Pure API crate owns user-visible decisions; firmware handlers perform I/O and side effects only after pure acceptance"
    - "All HTTP/WebSocket API entrypoints call a common access gate before route work"
    - "Command routes send public responses before executing restart/display/mining-visible effects"

key-files:
  created:
    - firmware/bitaxe/src/http_api.rs
    - firmware/bitaxe/src/runtime_snapshot.rs
    - firmware/bitaxe/src/settings_adapter.rs
    - firmware/bitaxe/src/log_buffer.rs
    - firmware/bitaxe/src/websocket_api.rs
    - crates/bitaxe-api/src/route_shell.rs
  modified:
    - crates/bitaxe-api/src/lib.rs
    - crates/bitaxe-api/BUILD.bazel
    - firmware/bitaxe/Cargo.toml
    - firmware/bitaxe/src/main.rs
    - firmware/bitaxe/sdkconfig.defaults
    - .cargo/config.toml
    - Cargo.lock
    - MODULE.bazel.lock

key-decisions:
  - "Use raw ESP-IDF WebSocket handler registration for `/api/ws` and `/api/ws/live` while keeping unsafe calls behind small firmware helpers."
  - "Use raw ESP-IDF NVS calls in the firmware settings adapter so writes do not auto-commit before the pure executor's commit step."
  - "Apply hostname changes best-effort through ESP-IDF netif handles after successful settings persistence; unavailable netifs are logged, not exposed publicly."
  - "Keep OTA/OTAWWW fail-closed in Phase 05; static assets and recovery-page parity remain Phase 07 scope."

patterns-established:
  - "Route-shell tests prove public denial/error bodies without needing ESP-IDF sockets."
  - "Firmware startup lines are copied into a retained log buffer without logging secret settings."
  - "WebSocket route state is tracked centrally with a 10-client cap and no-client hibernation behavior."

requirements-completed: [API-02, API-03, API-04, API-05, API-06, API-07, API-08, API-09]
generated_by: gsd-execute-plan
lifecycle_mode: yolo
phase_lifecycle_id: 5-2026-06-27T18-02-49
generated_at: 2026-06-27T21:40:45Z

duration: 38 min
completed: 2026-06-27
---

# Phase 05 Plan 05: Firmware API Shell Summary

**ESP-IDF AxeOS API route shell with private-network gating, settings NVS persistence, retained logs, WebSocket registration, and response-before-effect commands**

## Performance

- **Duration:** 38 min
- **Started:** 2026-06-27T21:03:08Z
- **Completed:** 2026-06-27T21:40:45Z
- **Tasks:** 2
- **Files modified:** 14

## Accomplishments

- Registered Phase 05 firmware HTTP routes and raw ESP-IDF WebSocket handlers behind a shared private-network/AP-origin gate.
- Connected handlers to existing `bitaxe-api` pure mappers for system info, ASIC settings, statistics, scoreboard, settings, logs, live telemetry, and commands.
- Added settings PATCH body-cap tests proving oversized requests return `Wrong API input` before JSON parsing, writes, commit, or reload.
- Implemented firmware adapters for ESP-IDF NVS settings persistence, retained log download, raw log WebSocket setup, live telemetry connect frames, and command effects after public responses.
- Kept OTA and OTAWWW routes fail-closed with safe unsupported responses for Phase 05.

## Task Commits

Each task was committed atomically:

1. **Task 1: Add route registration, access gate, and adapter compile spike** - `bf07665` (feat)
2. **Task 2: Wire handlers to settings, logs, telemetry, response mappers, and commands** - `e5f8ae7` (feat)

TDD RED failures were run and recorded in execution notes but not committed because the repo Rust gate requires passing checks before every commit.

## Files Created/Modified

- `.cargo/config.toml` - ESP-IDF build environment for direct firmware Cargo builds.
- `Cargo.lock`, `MODULE.bazel.lock` - Dependency graph updates for the firmware API dependency.
- `crates/bitaxe-api/BUILD.bazel` - Exposes the route shell in API crate tests.
- `crates/bitaxe-api/src/route_shell.rs` - Pure route table, access gate, WebSocket denial, unknown-route response, unsupported update response, and settings body-cap decisions.
- `crates/bitaxe-api/src/lib.rs` - Exports route shell and settings body-cap APIs.
- `firmware/bitaxe/Cargo.toml` - Adds API crate dependency.
- `firmware/bitaxe/sdkconfig.defaults` - Enables ESP-IDF HTTP WebSocket support.
- `firmware/bitaxe/src/http_api.rs` - Registers and handles HTTP/WebSocket routes, access gates, settings persistence, log downloads, and command effects.
- `firmware/bitaxe/src/runtime_snapshot.rs` - Collects firmware-facing API snapshot facts from existing runtime/config/platform boundaries.
- `firmware/bitaxe/src/settings_adapter.rs` - Applies accepted settings writes through ESP-IDF NVS, then commits and reloads.
- `firmware/bitaxe/src/log_buffer.rs` - Maintains retained firmware log text for API download/WebSocket use.
- `firmware/bitaxe/src/websocket_api.rs` - Tracks WebSocket clients, route cap, raw log stream baseline, and live telemetry planner state.
- `firmware/bitaxe/src/main.rs` - Starts the API server and mirrors startup log lines into the retained log buffer.

## Decisions Made

- Raw WebSocket registration is used because the route shell needs exact `/api/ws` and `/api/ws/live` upgrade behavior with ESP-IDF's `httpd_uri_t`.
- Settings persistence uses raw `nvs_set_*` calls rather than `EspNvs::set_*` helpers because the helper methods commit each key immediately; the pure settings executor requires write, commit, then reload ordering.
- Hostname live apply is best-effort through ESP-IDF netif handles after the response-safe persistence path succeeds.
- `API-09` in this plan means Phase 05 OTA/static update routes fail closed. Full static AxeOS assets and recovery-page compatibility remain incomplete and should stay with Phase 07/release work.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] Added ESP-IDF build wiring for direct Cargo firmware builds**
- **Found during:** Task 1
- **Issue:** Direct `cargo build -p bitaxe-firmware --target xtensa-esp32s3-espidf` could fall back to stale/default ESP-IDF settings instead of the pinned project settings.
- **Fix:** Added `.cargo/config.toml` ESP-IDF environment values and enabled WebSocket support in `sdkconfig.defaults`.
- **Files modified:** `.cargo/config.toml`, `firmware/bitaxe/sdkconfig.defaults`
- **Verification:** `cargo build -p bitaxe-firmware --target xtensa-esp32s3-espidf`, `bazel build //firmware/bitaxe:firmware`
- **Committed in:** `bf07665`

**2. [Rule 3 - Blocking] Cleared stale ESP-IDF target cache**
- **Found during:** Task 1
- **Issue:** The target-specific ESP-IDF build cache pointed at an older ESP-IDF CMake state and blocked the pinned `v5.5.4` firmware compile.
- **Fix:** Cleaned the affected Xtensa ESP-IDF build artifacts before rerunning the build.
- **Files modified:** None
- **Verification:** `cargo build -p bitaxe-firmware --target xtensa-esp32s3-espidf`
- **Committed in:** Not applicable; cache cleanup only.

**3. [Rule 3 - Blocking] Replaced unavailable POSIX hostname effect**
- **Found during:** Task 2
- **Issue:** `sethostname` appeared in generated bindings but was not provided by the linked ESP-IDF image.
- **Fix:** Switched hostname best-effort apply to `esp_netif_set_hostname` for STA/AP netifs after settings persistence succeeds.
- **Files modified:** `firmware/bitaxe/src/http_api.rs`
- **Verification:** `cargo build -p bitaxe-firmware --target xtensa-esp32s3-espidf`, full Rust gate
- **Committed in:** `e5f8ae7`

***

**Total deviations:** 3 auto-fixed (3 blocking)
**Impact on plan:** All fixes were required to make the planned firmware route shell compile and preserve the settings/command ordering contracts. No architecture change was introduced.

## Issues Encountered

- TDD RED for Task 1 failed as expected before the route shell/access gate existed.
- TDD RED for Task 2 failed as expected before the settings body-cap symbols existed.
- WebSocket frame send support required raw ESP-IDF calls; those calls are isolated in `http_api.rs` helpers and route state remains in safe Rust.

## Known Stubs

None that block this plan. The scan matched runtime `unavailable` log/status values, not UI placeholders. Phase 6-owned hardware telemetry remains intentionally safe/defaulted until the safety telemetry phase provides hardware evidence.

## Verification

- `bazel test //crates/bitaxe-api:tests --test_filter=route_shell`
- `bazel test //crates/bitaxe-api:tests --test_filter=settings_patch_body_cap`
- `cargo build -p bitaxe-firmware --target xtensa-esp32s3-espidf`
- `bazel build //firmware/bitaxe:firmware`
- `cargo fmt --all`
- `cargo clippy --all-targets --all-features -- -D warnings`
- `cargo build --all-targets --all-features`
- `cargo test --all-features`
- `bazel test //crates/bitaxe-api:tests --test_filter='route_shell|settings_patch_body_cap'`
- `just test`

## User Setup Required

None - no external service configuration required. Hardware/API smoke with `just flash-monitor board=205 port=<port>` and representative `curl`/WebSocket checks remains future evidence work.

## Next Phase Readiness

Phase 05 API pure contracts now have firmware route entrypoints. Phase 06 can provide real safety/power/thermal telemetry behind the existing snapshot boundaries, and Phase 07/release work should own full static asset, recovery-page, and OTA update behavior.

## Self-Check: PASSED

- Summary file exists at `.planning/phases/05-axeos-api-logs-and-telemetry/05-05-SUMMARY.md`.
- Task commit `bf07665` is present in git history.
- Task commit `e5f8ae7` is present in git history.

***
*Phase: 05-axeos-api-logs-and-telemetry*
*Completed: 2026-06-27*
