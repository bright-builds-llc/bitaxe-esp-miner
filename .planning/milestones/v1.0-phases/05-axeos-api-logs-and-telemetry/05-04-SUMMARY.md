---
phase: 05-axeos-api-logs-and-telemetry
plan: 04
subsystem: api
tags: [rust, serde-json, axeos, logs, websocket, telemetry]

requires:
  - phase: 05-axeos-api-logs-and-telemetry
    provides: "Plan 05-01 API crate foundation, Serde dependencies, and fixture wiring"
  - phase: 05-axeos-api-logs-and-telemetry
    provides: "Plan 05-03 pure system/mining response mappers for future telemetry state"
provides:
  - "Host-testable retained log buffer and download chunk contract"
  - "Raw `/api/ws` log stream baseline and no-client hibernation semantics"
  - "Pure `/api/ws/live` full update envelope, structured diff, 500 ms cadence, and baseline reset planner"
affects:
  - 05-05-firmware-route-websocket-settings-log-adapters
  - 05-07-api-compare-and-static-evidence
  - axeos-websocket-compatibility

tech-stack:
  added: []
  patterns:
    - "Log download and raw log streaming share one bounded retained-log model with different starting cursors."
    - "Live telemetry update decisions use structured `serde_json::Value` diffs before firmware WebSocket sending."

key-files:
  created:
    - crates/bitaxe-api/src/logs.rs
    - crates/bitaxe-api/src/telemetry.rs
    - crates/bitaxe-api/fixtures/api/log-buffer-cases.json
    - crates/bitaxe-api/fixtures/api/live-telemetry-cases.json
  modified:
    - crates/bitaxe-api/src/lib.rs
    - crates/bitaxe-api/BUILD.bazel

key-decisions:
  - "Keep retained log buffering and raw log stream cursor behavior pure in `bitaxe-api`; ESP log hooks, mutexes, notifications, and WebSocket sends remain firmware adapter work."
  - "Start raw log WebSocket streams at the current absolute log end and reset that cursor while no clients are active to avoid retained-history replay."
  - "Model live telemetry as full `update` envelopes on connect, diff-only cadence frames after baseline, and baseline clearing while no live clients are active."

patterns-established:
  - "RetainedLogBuffer exposes absolute cursor reads with 512 KiB retention, 4096 byte chunks, clamp, and bounded newline resync."
  - "LiveTelemetryPlanner owns baseline comparison while leaving client tracking cadence sleeps and socket I/O outside the API crate."

requirements-completed: [API-05, API-06, API-07]
generated_by: gsd-execute-plan
lifecycle_mode: yolo
phase_lifecycle_id: 5-2026-06-27T18-02-49
generated_at: 2026-06-27T20:42:16Z

duration: 9 min
completed: 2026-06-27
---

# Phase 05 Plan 04: Logs And Live Telemetry Summary

**Bounded retained log contracts plus raw log and live telemetry WebSocket planners with fixture-backed AxeOS semantics**

## Performance

- **Duration:** 9 min
- **Started:** 2026-06-27T20:32:20Z
- **Completed:** 2026-06-27T20:42:16Z
- **Tasks:** 2
- **Files modified:** 6

## Accomplishments

- Added `bitaxe_api::logs` with 512 KiB retention, 4096 byte chunk reads, download headers, absolute-cursor clamping, newline resync, and raw `/api/ws` streaming semantics.
- Added `bitaxe_api::telemetry` with `{"event":"update","data":...}` envelopes, structured nested diffs, no-send unchanged behavior, no-client baseline clearing, and a 500 ms cadence constant.
- Added log and live telemetry fixture cases covering headers, raw text payloads, full connect frames, diff frames, nested diffs, and cadence metadata.

## Task Commits

1. **Task 1: Model retained log download and raw log stream semantics** - `d42b2d0` (feat)
2. **Task 2: Implement live telemetry envelopes, diffs, cadence, and no-send behavior** - `901759c` (feat)

## Files Created/Modified

- `crates/bitaxe-api/src/logs.rs` - Host-testable retained ring buffer, download chunking, raw log stream planner, and log behavior tests.
- `crates/bitaxe-api/src/telemetry.rs` - Live telemetry update envelope helpers, structured diffing, planner state, and behavior tests.
- `crates/bitaxe-api/fixtures/api/log-buffer-cases.json` - Log download header and raw payload fixture.
- `crates/bitaxe-api/fixtures/api/live-telemetry-cases.json` - Full state, changed state, expected diff, nested diff, and cadence fixture.
- `crates/bitaxe-api/src/lib.rs` and `crates/bitaxe-api/BUILD.bazel` - Public exports and Bazel source wiring for the new modules.

## Decisions Made

- The API crate models retained log and telemetry payload decisions only; ESP-IDF hooks, sockets, tasks, mutexes, and client admission stay in firmware.
- `/api/ws` and `/api/system/logs` intentionally share the retained log model but not the same cursor baseline: downloads start from absolute beginning, while live raw streams start at current end.
- `/api/ws/live` uses a full connect frame and diff-only cadence frames so AxeOS can merge updates without receiving unchanged full-state payloads every tick.

## Deviations from Plan

### Process Adjustments

**AGENTS.md precedence over TDD RED commits**
- **Found during:** Task 1 and Task 2
- **Issue:** The plan requested TDD RED flow, but repo instructions require `cargo fmt --all`, `cargo clippy --all-targets --all-features -- -D warnings`, `cargo build --all-targets --all-features`, and `cargo test --all-features` to pass before every Rust-project commit.
- **Adjustment:** Ran failing RED tests for both tasks, then committed only passing GREEN implementations after the required Rust gate.
- **Impact:** Preserved higher-priority repo policy without changing feature scope.

**Total deviations:** 0 auto-fixed implementation issues, 1 process adjustment.
**Impact on plan:** Scope stayed within pure log and telemetry contracts; no firmware route, socket, task, or hardware-control behavior was added.

## Issues Encountered

- Task 1 RED setup first failed at compile time because `todo!()` cannot be used in a `const fn`; the stub was adjusted to compile so the RED signal came from behavior tests.
- The Task 1 clamp/resync test fixture initially wrote the retained marker before a full retained tail, which correctly evicted the marker. The fixture was corrected so the retained window begins inside a partial line before implementation.

## Known Stubs

None. No placeholder, TODO, empty mock-data, or UI-flowing hardcoded-empty stubs were found in the files created or modified by this plan.

## Threat Flags

None. The new modules are pure host-testable contracts for planned log and telemetry surfaces; they introduce no network endpoint, authentication path, file access pattern, schema migration, or hardware-control effect.

## Authentication Gates

None.

## User Setup Required

None - no external service configuration required.

## Verification

- `bazel test //crates/bitaxe-api:tests --test_filter=logs` - failed in RED, then passed
- `bazel test //crates/bitaxe-api:tests --test_filter=telemetry` - failed in RED, then passed
- `bazel test //crates/bitaxe-api:tests` - passed after each task and as plan quick verification
- `cargo fmt --all` - passed before each task commit
- `cargo clippy --all-targets --all-features -- -D warnings` - passed before each task commit
- `cargo build --all-targets --all-features` - passed before each task commit
- `cargo test --all-features` - passed before each task commit
- `just test` - passed
- `git status --short reference/esp-miner` - clean output

## Next Phase Readiness

Ready for Plan 05-05. The API crate now has pure log and telemetry contracts that firmware route and WebSocket adapters can call without duplicating retained-buffer cursor rules or live telemetry diff logic.

## Self-Check: PASSED

- Found `.planning/phases/05-axeos-api-logs-and-telemetry/05-04-SUMMARY.md`
- Found `crates/bitaxe-api/src/logs.rs`
- Found `crates/bitaxe-api/src/telemetry.rs`
- Found `crates/bitaxe-api/fixtures/api/log-buffer-cases.json`
- Found `crates/bitaxe-api/fixtures/api/live-telemetry-cases.json`
- Found task commit `d42b2d0`
- Found task commit `901759c`
- Reference implementation remains unmodified.

---
*Phase: 05-axeos-api-logs-and-telemetry*
*Completed: 2026-06-27*
