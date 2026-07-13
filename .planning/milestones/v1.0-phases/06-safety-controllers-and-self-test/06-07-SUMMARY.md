---
phase: 06-safety-controllers-and-self-test
plan: "07"
subsystem: api
tags: [rust, api, telemetry, safety, statistics, wire]

requires:
  - phase: 06-03
    provides: Power telemetry and evidence semantics
  - phase: 06-04
    provides: Thermal telemetry and evidence semantics
  - phase: 06-06
    provides: Safety evidence gate semantics
provides:
  - Explicit safety telemetry status/report/snapshot model in `bitaxe-api`
  - Public `SafetyTelemetryReport` and `SafetyTelemetryStatus` exports for firmware
  - System info and statistics projections from `safe_telemetry`
  - Live telemetry fixture cases for fresh, stale, faulted, and unavailable safety telemetry
affects: [phase-06, api-telemetry, firmware-safety-adapter, parity]

tech-stack:
  added:
    - bitaxe-safety dependency for `bitaxe-api`
  patterns: [typed API snapshot status, zero-compatible unavailable projection, evidence-preserving telemetry]

key-files:
  modified:
    - Cargo.lock
    - MODULE.bazel.lock
    - crates/bitaxe-api/Cargo.toml
    - crates/bitaxe-api/BUILD.bazel
    - crates/bitaxe-api/src/lib.rs
    - crates/bitaxe-api/src/snapshot.rs
    - crates/bitaxe-api/src/wire.rs
    - crates/bitaxe-api/src/statistics.rs
    - crates/bitaxe-api/fixtures/api/live-telemetry-cases.json

key-decisions:
  - "Keep AxeOS-compatible public numeric fields zero-compatible for unavailable/stale/faulted telemetry."
  - "Preserve safety status and evidence on typed snapshots so firmware and tests do not treat zeroes as trustworthy values."
  - "Require hardware-verified evidence before fresh telemetry values are projected into public numeric fields."

patterns-established:
  - "API snapshots carry explicit `SafetyTelemetryStatus` plus `SafetyCriticalEvidence`."
  - "Wire/statistics projections consume `snapshot.safe_telemetry` rather than raw placeholders."

requirements-completed: [SAFE-01, SAFE-02, SAFE-07, SAFE-08]
generated_by: gsd-execute-plan
lifecycle_mode: yolo
phase_lifecycle_id: 6-2026-06-28T02-37-37
generated_at: 2026-06-28T04:49:03Z

duration: 23 min
completed: 2026-06-28
---

# Phase 06 Plan 07: API Safety Telemetry Summary

**Explicit fresh/stale/faulted/unavailable safety telemetry status for API projections**

## Performance

- **Duration:** 23 min
- **Started:** 2026-06-28T04:26:00Z
- **Completed:** 2026-06-28T04:49:03Z
- **Tasks:** 2
- **Files modified:** 9

## Accomplishments

- Added `SafetyTelemetryStatus`, `SafetyTelemetryReport`, and status/evidence fields on `SafeTelemetrySnapshot`.
- Replaced the Phase 6 unavailable helper with `SafeTelemetrySnapshot::unavailable("safety_telemetry_unavailable")`.
- Added report projection that preserves values only for fresh hardware-verified telemetry.
- Re-exported telemetry report/status types from `bitaxe-api` for firmware use.
- Added statistics and system-info projection tests using `snapshot.safe_telemetry`.
- Extended live telemetry fixtures with fresh, stale, faulted, and unavailable safety telemetry cases.

## Task Commits

1. **Task 1: Define explicit safety telemetry status and public exports** - `36a2c4c` (feat, combined)
2. **Task 2: Project safety telemetry through system info, statistics, and live telemetry** - `36a2c4c` (feat, combined)

## Files Created/Modified

- `crates/bitaxe-api/src/snapshot.rs` - Safety telemetry report/status/snapshot model and D-17/D-18 tests.
- `crates/bitaxe-api/src/lib.rs` - Public exports for firmware.
- `crates/bitaxe-api/src/wire.rs` - System info projection from `safe_telemetry`.
- `crates/bitaxe-api/src/statistics.rs` - Statistics sample projection from snapshots.
- `crates/bitaxe-api/fixtures/api/live-telemetry-cases.json` - Safety telemetry fixture cases.
- `Cargo.lock`, `MODULE.bazel.lock`, API Cargo/Bazel files - Internal safety dependency.

## Decisions Made

- Did not add new public JSON fields to `/api/system/info`; the typed snapshot carries status/evidence while existing AxeOS fields remain compatible.
- Fresh reports with only unit evidence become `safety_telemetry_unverified` and publish zero-compatible numeric fields.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] Combined coupled task commits**
- **Found during:** Task commit boundary
- **Issue:** The snapshot status model and wire/statistics projections are tightly coupled through `SafeTelemetrySnapshot`.
- **Fix:** Committed both task outcomes together after scoped checks and the full Rust gate passed.
- **Files modified:** API snapshot, exports, projections, fixture, and dependency metadata.
- **Verification:** Scoped API tests, full Rust gate, and `just test && just parity`.
- **Committed in:** `36a2c4c`

**Total deviations:** 1 auto-fixed blocking issue
**Impact on plan:** Behavior and verification stayed aligned with the plan; commit granularity was coarser than the ideal task boundary.

## Issues Encountered

No implementation blockers. `MiningStateWire` has no error-percentage field, so the statistics helper keeps that value at the existing zero default.

## Verification

- `cargo test -p bitaxe-api --all-features safety_telemetry_model`
- `cargo test -p bitaxe-api --all-features safety_telemetry_projection`
- `bazel test //crates/bitaxe-api:tests --test_filter='safety_telemetry_model|safety_telemetry_projection'`
- `cargo fmt --all`
- `cargo clippy --all-targets --all-features -- -D warnings`
- `cargo build --all-targets --all-features`
- `cargo test --all-features`
- `just test`
- `just parity` (`validation_errors: none`)

## Known Stubs

Telemetry remains adapter-fed. Firmware still needs Plan 06-08 observe-only safety adapters to populate reports from runtime boundaries.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

Plan 06-08 can use `bitaxe_api::SafetyTelemetryReport` and `SafetyTelemetryStatus` to feed explicit unavailable or observed safety telemetry into firmware snapshots.

## Self-Check: PASSED

- Confirmed scoped API tests passed under Cargo and Bazel.
- Confirmed full Rust gate and project-level `just test && just parity` passed.
- Confirmed task commit `36a2c4c` exists in git history.

---
*Phase: 06-safety-controllers-and-self-test*
*Completed: 2026-06-28*
