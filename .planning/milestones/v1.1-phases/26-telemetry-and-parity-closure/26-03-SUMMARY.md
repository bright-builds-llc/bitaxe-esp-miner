---
phase: 26-telemetry-and-parity-closure
plan: 03
subsystem: firmware-telemetry
tags: [rust, esp-idf, telemetry, api, websocket, stratum]
requires:
  - phase: 26-telemetry-and-parity-closure
    plan: 01
    provides: RuntimeTelemetryProjection, runtime event fold contract, runtime sample markers.
  - phase: 26-telemetry-and-parity-closure
    plan: 02
    provides: ProjectedApiViews and projection-backed API/WebSocket DTO contract.
provides:
  - Firmware runtime producers fold lifecycle, hashrate, pool difficulty, work-ready, blocked, submit classification, sample marker, and safe-stop events into RuntimeTelemetryProjection.
  - Runtime snapshot bridge exposes projection-backed system info, statistics, scoreboard, and live telemetry helpers.
  - HTTP and live WebSocket consumers serialize projection-backed views instead of route-local mining counter derivation.
affects: [firmware-runtime, http-api, websocket-api, parity-evidence]
tech-stack:
  added: []
  patterns:
    - Shared projection state in firmware snapshot adapter.
    - Producer-boundary statistics sampling through RuntimeProjectionSampleMarker.
    - Projection-backed route helpers for API and WebSocket consumers.
key-files:
  created:
    - .planning/phases/26-telemetry-and-parity-closure/26-03-SUMMARY.md
  modified:
    - firmware/bitaxe/src/live_stratum_runtime.rs
    - firmware/bitaxe/src/runtime_snapshot.rs
    - firmware/bitaxe/src/http_api.rs
key-decisions:
  - "RuntimeTelemetryProjection is stored beside command-visible mining state so firmware producers and API consumers share one telemetry source of truth."
  - "Only the projected statistics helper drains pending sample markers; system-info and live-WebSocket reads do not consume statistics samples."
  - "Scoreboard output remains empty until parsed-response-backed and redaction-allowed share outcome material exists."
patterns-established:
  - "Firmware producers publish typed runtime telemetry events instead of mutating API mining counters directly."
  - "Route serialization calls runtime_snapshot projection helpers before emitting API or live WebSocket payloads."
requirements-completed: [API-11, API-12, API-13]
generated_by: gsd-execute-plan
lifecycle_mode: yolo
phase_lifecycle_id: 26-2026-07-05T03-48-38
generated_at: 2026-07-05T04:27:56Z
duration: 7min
completed: 2026-07-05
---

# Phase 26 Plan 03: Telemetry Wiring Summary

**Firmware Stratum runtime events now feed the shared telemetry projection, and HTTP/live WebSocket consumers serialize the Plan 26-02 projection views.**

## Performance

- **Duration:** 7min
- **Started:** 2026-07-05T04:21:00Z
- **Completed:** 2026-07-05T04:27:56Z
- **Tasks:** 2 completed
- **Files modified:** 3 code files, 1 summary file

## Accomplishments

- Wired Phase 25 runtime lifecycle, hashrate, pool difficulty, work-ready, blocked prerequisite, submit classification, bounded sample, and safe-stop observations into `RuntimeTelemetryProjection`.
- Added runtime snapshot helpers for projection-backed system info, statistics, scoreboard, and live telemetry payloads.
- Replaced `/api/system/info`, `/api/system/statistics`, `/api/system/scoreboard`, and `/api/ws/live` current payload generation with projection-backed helpers while preserving existing route and session gates.

## Task Commits

Each task was committed atomically:

1. **Task 26-03-01: Wire Phase 25 runtime producers into projection fold**
   - `1aa2dbb` test: add firmware projection wiring contract
   - `1852295` feat: wire firmware runtime telemetry projection
2. **Task 26-03-02: Wire firmware HTTP and WebSocket consumers**
   - `1b5984f` test: add projected route helper contract
   - `2119bee` feat: wire projected telemetry consumers

## Files Created/Modified

- `firmware/bitaxe/src/live_stratum_runtime.rs` - publishes typed runtime events from live Stratum shell observations.
- `firmware/bitaxe/src/runtime_snapshot.rs` - stores projection state, drains sample markers, and exposes projected route helpers.
- `firmware/bitaxe/src/http_api.rs` - serializes projection-backed HTTP and live WebSocket payloads.
- `.planning/phases/26-telemetry-and-parity-closure/26-03-SUMMARY.md` - records execution outcome and verification evidence.

## Decisions Made

- `RuntimeTelemetryProjection` lives in the firmware snapshot adapter with monotonic firmware-local sequence generation, keeping pure projection logic in `crates/bitaxe-stratum`.
- Statistics samples are emitted only when `projected_statistics` drains a pending `RuntimeProjectionSampleMarker`; unrelated system info or WebSocket reads preserve the marker.
- Scoreboard remains conservative and empty until a later plan has parsed, current-generation, redaction-allowed share outcome material.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking Issue] Made sample-marker drain production-visible**
- **Found during:** Task 26-03-02 verification
- **Issue:** `bazel build //firmware/bitaxe:firmware` failed because `drain_pending_runtime_sample_marker` was still behind `#[cfg(test)]` after the production statistics helper began using it.
- **Fix:** Removed the test-only gate from the method while keeping test-only helper wrappers test-gated.
- **Files modified:** `firmware/bitaxe/src/runtime_snapshot.rs`
- **Verification:** `bazel test //crates/bitaxe-api:tests && bazel build //firmware/bitaxe:firmware`
- **Committed in:** `2119bee`

**Total deviations:** 1 auto-fixed Rule 3 blocking issue.
**Impact on plan:** Required for production route wiring to compile; no architecture or scope change.

## Issues Encountered

- `cargo test -p bitaxe-firmware --no-run` cannot compile firmware tests on the host `aarch64-apple-darwin` target because `esp-idf-sys` rejects that target. The RED contracts were still committed, and firmware validation used the canonical Bazel firmware target.

## Verification

- `cargo test -p bitaxe-firmware --no-run` - failed at the known ESP-IDF host-target gate before firmware tests could compile.
- `bazel test //crates/bitaxe-api:tests` - passed.
- `bazel test //crates/bitaxe-stratum:tests //crates/bitaxe-api:tests` - passed.
- `bazel build //firmware/bitaxe:firmware` - passed.

## Known Stubs

None found in files modified by this plan.

## Threat Flags

None. The plan reused existing HTTP/WebSocket routes and did not introduce new network endpoints, auth paths, file access patterns, schema changes, or trust-boundary surfaces.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

Plan 26-04 can close parity checklist and evidence state against the projection-backed firmware wiring. Hardware claims remain governed by the existing Ultra 205 detector gate and exact non-claim rules.

## Self-Check: PASSED

- Found summary file: `.planning/phases/26-telemetry-and-parity-closure/26-03-SUMMARY.md`
- Found task commits: `1aa2dbb`, `1852295`, `1b5984f`, `2119bee`
