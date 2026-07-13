---
phase: 24-bm1366-production-work-path
plan: 01
subsystem: asic
tags: [rust, bm1366, production-work, redaction, bazel]
requires:
  - phase: 23-redacted-operator-evidence-workflow
    provides: redaction-safe evidence and non-claim governance for production mining artifacts
provides:
  - BM1366 production-only work payload and command primitives
  - redaction-safe production ASIC blocker and status taxonomy
  - Bazel-registered public BM1366 production module
affects: [phase-24, phase-25, bitaxe-asic, firmware-asic-adapter, parity-evidence]
tech-stack:
  added: []
  patterns:
    - functional-core production BM1366 command primitives
    - redaction-safe category labels for production ASIC failures
key-files:
  created:
    - crates/bitaxe-asic/src/bm1366/production.rs
  modified:
    - crates/bitaxe-asic/src/bm1366.rs
    - crates/bitaxe-asic/BUILD.bazel
key-decisions:
  - "Production BM1366 work uses a distinct command enum and payload wrapper rather than diagnostic work command names."
  - "Production ASIC failure status renders stable category labels only; raw runtime values remain out of committed artifacts."
patterns-established:
  - "ProductionWorkPayload wraps Bm1366WorkPayload while preserving job-id access for downstream correlation."
  - "Bm1366ProductionCommand emits typed Bm1366AdapterAction values so firmware interprets actions instead of constructing frames."
requirements-completed: [ASIC-09, ASIC-12]
generated_by: gsd-execute-plan
lifecycle_mode: yolo
phase_lifecycle_id: 24-2026-07-05T00-27-27
generated_at: 2026-07-05T00:51:28Z
duration: 3min
completed: 2026-07-05
---

# Phase 24 Plan 01: BM1366 Production ASIC Primitives Summary

**BM1366 production command and fail-closed taxonomy primitives with typed adapter actions and redaction-safe labels**

## Performance

- **Duration:** 3 min
- **Started:** 2026-07-05T00:48:43Z
- **Completed:** 2026-07-05T00:51:28Z
- **Tasks:** 2
- **Files modified:** 3

## Accomplishments

- Added `ProductionWorkPayload`, `Bm1366ProductionCommand`, `ProductionAsicBlocker`, and `ProductionAsicStatus` in `crates/bitaxe-asic`.
- Kept production work frame construction inside `bitaxe-asic` while exposing only typed `Bm1366AdapterAction` values to downstream firmware.
- Added tests for production work dispatch, result-read action emission, redaction-safe blocker labels, and fail-closed status reasons.
- Exported the production module through the BM1366 facade and registered it in the Bazel crate target.

## Task Commits

Each task was committed atomically:

1. **Task 24-01-01 RED: Add failing BM1366 production primitive tests** - `26fb1d2` (test)
2. **Task 24-01-01 GREEN: Implement BM1366 production primitives** - `b000139` (feat)
3. **Task 24-01-02: Export production module through Bazel and Rust module roots** - `f55726f` (feat)

## Files Created/Modified

- `crates/bitaxe-asic/src/bm1366/production.rs` - Production-only BM1366 payload, command, blocker, status, and unit tests.
- `crates/bitaxe-asic/src/bm1366.rs` - Public BM1366 production module export.
- `crates/bitaxe-asic/BUILD.bazel` - Bazel source registration for the production module.

## Decisions Made

- Kept production work dispatch separate from diagnostic `SendDiagnosticWork` by introducing `Bm1366ProductionCommand::SendProductionWork`.
- Used stable production blocker category strings for evidence/log surfaces, including timeout, malformed result, stale work, correlation, duplicate, session, and target mismatch categories.
- Used a narrow `expect` inside `bitaxe-asic` frame construction because `ProductionWorkPayload` always wraps the fixed-size `Bm1366WorkPayload` invariant; firmware still receives only typed adapter actions.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] Registered the new module for RED test compilation**
- **Found during:** Task 24-01-01 (Add BM1366 production primitives)
- **Issue:** The TDD task's new `production.rs` tests would not be visible to `bazel test //crates/bitaxe-asic:tests` unless the module and Bazel source were registered before the export task.
- **Fix:** Added the Bazel source entry and a temporary `#[cfg(test)] pub mod production;` hook in the RED commit, then promoted it to a normal public export in Task 24-01-02.
- **Files modified:** `crates/bitaxe-asic/src/bm1366.rs`, `crates/bitaxe-asic/BUILD.bazel`
- **Verification:** `bazel test //crates/bitaxe-asic:tests` failed in RED on missing production types, then passed after implementation and export.
- **Committed in:** `26fb1d2`, finalized by `f55726f`

**Total deviations:** 1 auto-fixed (1 blocking)
**Impact on plan:** The adjustment was limited to making the planned TDD and export workflow verifiable; no production scope was added.

## Issues Encountered

- RED verification failed as intended on unresolved production primitive imports before implementation.
- `gsd-tools state record-metric` returned `recorded: false` because `.planning/STATE.md` has no Performance Metrics section; this summary records the plan timing and file/task counts.
- No stubs were found in created or modified files.
- No unplanned threat flags were introduced beyond the plan's BM1366 production command and redaction-safe status surfaces.

## Verification

- `bazel test //crates/bitaxe-asic:tests`
- `rg "pub enum Bm1366ProductionCommand|SendProductionWork|ReadProductionResult" crates/bitaxe-asic/src/bm1366/production.rs`
- `rg "pub enum ProductionAsicBlocker|production_result_timeout|production_job_uncorrelated|production_target_mismatch" crates/bitaxe-asic/src/bm1366/production.rs`
- `rg "pub enum ProductionAsicStatus|InitializedForProduction|WorkDispatched|ResultCorrelated|FailClosed" crates/bitaxe-asic/src/bm1366/production.rs`
- `! rg -n "SendDiagnosticWork|raw_bm1366_frame|target=|extranonce=|share_payload=|pool_config|device_url|password|token" crates/bitaxe-asic/src/bm1366/production.rs`
- `test "$(rg -c "pub mod production;" crates/bitaxe-asic/src/bm1366.rs)" = "1"`
- `rg "\"src/bm1366/production.rs\"" crates/bitaxe-asic/BUILD.bazel`

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

Plan 24-02 can build on `ProductionWorkPayload` and `Bm1366ProductionCommand` to add session-generation active work tracking in `bitaxe-stratum`. Hardware evidence and live Stratum response claims remain out of scope for this plan.

## Self-Check: PASSED

Confirmed the created/modified files exist and task commits `26fb1d2`, `b000139`, and `f55726f` are present in git history.

*Phase: 24-bm1366-production-work-path*
*Completed: 2026-07-05*
