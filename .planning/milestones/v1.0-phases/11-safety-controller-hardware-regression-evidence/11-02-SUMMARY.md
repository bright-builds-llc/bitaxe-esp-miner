---
phase: 11-safety-controller-hardware-regression-evidence
plan: 02
subsystem: parity
tags: [rust, parity, safety-evidence, hardware-regression]
requires:
  - phase: 11-safety-controller-hardware-regression-evidence
    provides: Phase 11 context, research, validation contract, and plan set
provides:
  - Active safety-control verified rows require hardware-regression evidence.
  - Narrow read-only hardware-smoke evidence remains valid for telemetry rows such as PWR-006.
  - Targeted parity tests cover active-control overclaim prevention.
affects: [parity, checklist-validation, safety-evidence]
tech-stack:
  added: []
  patterns: [row-id validation helpers, targeted checklist validation tests]
key-files:
  created:
    - .planning/phases/11-safety-controller-hardware-regression-evidence/11-02-SUMMARY.md
  modified:
    - tools/parity/src/main.rs
key-decisions:
  - "Active-control row ids require hardware-regression when marked verified."
  - "Existing safety-critical hardware-smoke/hardware-regression guard remains in place for other safety rows."
patterns-established:
  - "Use explicit row-id helpers for evidence-class validation when broad checklist text would overclaim safety parity."
requirements-completed: [SAFE-01, SAFE-02, SAFE-04, SAFE-05, SAFE-06, SAFE-08, EVD-05]
generated_by: gsd-execute-plan
lifecycle_mode: yolo
phase_lifecycle_id: 11-2026-06-29T20-23-34
generated_at: 2026-06-29T21:16:51Z
duration: 12min
completed: 2026-06-29
---

# Phase 11 Plan 02 Summary

**Parity validation now distinguishes active safety-control evidence from narrow read-only hardware smoke.**

## Performance

- Duration: 12 min
- Started: 2026-06-29T21:04:00Z
- Completed: 2026-06-29T21:16:51Z
- Tasks: 2
- Files modified: 2

## Accomplishments

- Added focused tests for `PWR-003`, `THR-002`, `SELF-001`, and `PWR-006`.
- Added an active safety-control row-id helper covering `PWR-001`, `PWR-002`, `PWR-003`, `PWR-005`, `THR-001`, `THR-002`, `SELF-001`, and `UI-003`.
- Enforced `hardware-regression` for verified active-control rows while preserving `hardware-smoke` for narrow read-only telemetry evidence.

## Task Commits

No commits were created during plan execution. The wrapper workflow will commit only after full phase verification passes.

## Files Created Or Modified

- `tools/parity/src/main.rs` - Added active-control validation and tests.
- `.planning/phases/11-safety-controller-hardware-regression-evidence/11-02-SUMMARY.md` - Recorded plan execution summary and verification evidence.

## Decisions Made

Active-control validation uses explicit checklist row IDs instead of broad text matching. This keeps the guard auditable and avoids blocking narrow rows such as `PWR-006` read-only INA260 telemetry.

## Deviations From Plan

The initial patch combined tests and implementation. To preserve the plan's TDD evidence, the implementation was temporarily disabled, `cargo test -p bitaxe-parity --all-features active_safety_control` was run, and the expected RED failure was observed for `PWR-003`. The implementation was then restored and verified green.

## Issues Encountered

Older safety-critical tests assumed exactly one error per invalid active row. The stricter validator can now report both the general safety-critical error and the active-control regression-evidence error, so those tests were updated to assert the required safety-critical message without relying on exact error count.

## Verification

- RED observed: `cargo test -p bitaxe-parity --all-features active_safety_control` failed before the validator was restored, with `PWR-003` missing the `requires hardware-regression evidence` error.
- Green: `cargo test -p bitaxe-parity --all-features active_safety_control`
- Green: `cargo test -p bitaxe-parity --all-features safety_critical`
- Green: `bazel test //tools/parity:tests --test_filter='active_safety_control|safety_critical'`
- Green: `just parity`
- Green: `git diff --check -- tools/parity/src/main.rs`

## User Setup Required

None.

## Next Phase Readiness

Plan 03 can update checklist and evidence posture with a stronger guard in place. Active voltage, fan actuation, self-test hardware, runtime input, and fault-path rows cannot be promoted to verified from broad hardware smoke evidence alone.
