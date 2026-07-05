---
phase: 26-telemetry-and-parity-closure
plan: 02
subsystem: api
tags: [rust, api, telemetry, websocket, statistics, scoreboard]
requires:
  - phase: 26-telemetry-and-parity-closure
    provides: Plan 26-01 RuntimeTelemetryProjection and RuntimeProjectionSampleMarker contracts.
provides:
  - Projection-backed API views for system info, statistics samples, scoreboard entries, and live telemetry JSON.
  - Tests proving explicit sample-marker gating, conservative scoreboard output, and safe-stop live telemetry payloads.
affects: [api-projection, websocket-telemetry, statistics, scoreboard, firmware-route-wiring]
tech-stack:
  added: []
  patterns: [pure-api-projection-adapter, explicit-sample-marker-gate, conservative-scoreboard-projection]
key-files:
  created:
    - crates/bitaxe-api/src/runtime_projection.rs
  modified:
    - crates/bitaxe-api/src/lib.rs
    - crates/bitaxe-api/BUILD.bazel
key-decisions:
  - "Derive API snapshot mining state and live telemetry JSON from the shared RuntimeTelemetryProjection."
  - "Materialize statistics samples only from an explicit RuntimeProjectionSampleMarker while keeping sample provenance out of public JSON."
  - "Keep scoreboard entries empty until a future parsed-response-backed and redaction-allowed share outcome source exists."
patterns-established:
  - "ProjectedApiViews bundles pure API outputs for firmware route shells without adding endpoint-local mining rules."
  - "Live telemetry tests use the existing WebSocket planner and projection-backed SystemInfoWire JSON."
requirements-completed: [API-11, API-12, API-13]
generated_by: gsd-execute-plan
lifecycle_mode: yolo
phase_lifecycle_id: 26-2026-07-05T03-48-38
generated_at: 2026-07-05T04:20:05Z
duration: 4min
completed: 2026-07-05
---

# Phase 26 Plan 02: API Projection Views Summary

**Projection-backed AxeOS API views with explicit statistics sample markers and safe-stop live telemetry JSON**

## Performance

- **Duration:** 4min
- **Started:** 2026-07-05T04:16:31Z
- **Completed:** 2026-07-05T04:20:05Z
- **Tasks:** 2
- **Files modified:** 3

## Accomplishments

- Added `ProjectedApiViews` and `project_api_views` as a pure adapter from `RuntimeTelemetryProjection` into `ApiSnapshot`, statistics samples, scoreboard entries, and `SystemInfoWire` JSON.
- Proved statistics samples remain empty without an explicit `RuntimeProjectionSampleMarker`, even across repeated request-time reads with changing timestamps.
- Proved scoreboard output remains the upstream-compatible empty array without parsed-response-backed share outcome material.
- Proved `/api/ws/live` full/cadence payload planning receives projection-backed JSON and does not emit stale active-mining state after safe stop.

## Task Commits

Each TDD task was committed atomically:

1. **Task 26-02-01 RED:** `d1d368a` test(26-02): add failing API projection tests
2. **Task 26-02-01 GREEN:** `f1c2175` feat(26-02): implement API projection views
3. **Task 26-02-02 Proof:** `81d43d4` test(26-02): prove projection wire outputs

## Files Created/Modified

- `crates/bitaxe-api/src/runtime_projection.rs` - Pure projection adapter, explicit sample-marker gate, conservative scoreboard view, live telemetry payload serialization, and focused tests.
- `crates/bitaxe-api/src/lib.rs` - Public module export and helper re-exports.
- `crates/bitaxe-api/BUILD.bazel` - Bazel source registration for the new API projection module.

## Decisions Made

- Used `RuntimeTelemetryProjection::state()` as the only mining-state source for the projected `ApiSnapshot`.
- Used the sample marker timestamp for `StatisticsSample::from_snapshot` and ignored request-time timestamp changes when no marker exists.
- Preserved public AxeOS-compatible wire shapes by serializing `SystemInfoWire` and by keeping source, evidence, redaction, pool, device, and raw ASIC semantics out of public JSON.

## Deviations from Plan

None - plan scope and public API compatibility constraints were preserved.

## Issues Encountered

- Task 26-02-02 was proof-focused and reused behavior already covered by the Task 26-02-01 RED contract. It added test-only reinforcement after the adapter was green rather than introducing an artificial second failing implementation surface.

## Known Stubs

None found. The empty scoreboard vector is intentional conservative behavior required by Plan 26-02 until parsed-response-backed and redaction-allowed share outcome material exists.

## Threat Flags

None - this plan implemented the planned projection-core-to-API DTO boundary only, with no new network endpoints, auth paths, file access, or schema changes.

## Verification

- `bazel test //crates/bitaxe-api:tests` failed during RED with unresolved `project_api_views` and `ProjectedApiViews`, as expected.
- `bazel test //crates/bitaxe-api:tests` passed after GREEN implementation.
- `bazel test //crates/bitaxe-api:tests` passed after proof-test reinforcement.
- `node "$HOME/.cursor/get-shit-done/bin/gsd-tools.cjs" verify lifecycle 26 --expect-id 26-2026-07-05T03-48-38 --expect-mode yolo --require-plans` passed.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

Plan 26-03 can wire firmware producers and route consumers to `ProjectedApiViews` without inventing endpoint-local counters, request-time statistics samples, or unsupported scoreboard rows.

## Self-Check: PASSED

- Found summary file at `.planning/phases/26-telemetry-and-parity-closure/26-02-SUMMARY.md`.
- Found key files `crates/bitaxe-api/src/runtime_projection.rs`, `crates/bitaxe-api/src/lib.rs`, and `crates/bitaxe-api/BUILD.bazel`.
- Found task commits `d1d368a`, `f1c2175`, and `81d43d4`.

*Phase: 26-telemetry-and-parity-closure*
*Completed: 2026-07-05*
