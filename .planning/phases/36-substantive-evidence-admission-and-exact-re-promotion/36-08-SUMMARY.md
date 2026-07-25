---
phase: 36-substantive-evidence-admission-and-exact-re-promotion
plan: "08"
subsystem: evidence
tags: [hardware-attempt, ultra205, broker, exact-package, privacy, non-promotion]
requires:
  - phase: 36-06
    provides: deployed-runfiles effect-adapter repair and exact pre-effect blocker
provides:
  - exact remaining-plan graph regression for Waves 7 through 9
  - one clean-current-HEAD package and single-use software preflight
  - one immutable typed sealed non-promotion with complete cleanup accounting
  - truthful no-candidate handoff that leaves Plan 36-07 blocked
affects: [36-07, 36-04, phase36-hardware-attempt, evidence-promotion]
tech-stack:
  added: []
  patterns:
    - one broker-owned detector call inside one non-retryable hardware command
    - earliest typed failure preserved separately from recovery and cleanup outcomes
key-files:
  created:
    - .planning/phases/36-substantive-evidence-admission-and-exact-re-promotion/36-08-SUMMARY.md
  modified:
    - scripts/phase36-substantive-evidence-test.sh
key-decisions:
  - "Seal the fresh attempt as non-promotional after flash_failed, without retry or candidate creation."
  - "Record recovery_failed as secondary while preserving flash_failed as the earliest typed failure and cleanup as complete."
  - "Leave Plan 36-07 blocked because only sealed_eligible plus a distinct redacted candidate can unblock it."
patterns-established:
  - "Partial-flash boundary: bounded same-image recovery is restoration-only and can never convert the attempt to eligible."
  - "Fail-closed handoff: a sealed non-promotion creates no candidate and grants no offline promotion authority."
requirements-completed: []
requirements-pending: [SYS-02, EVD-11, EVD-12, EVD-14, EVD-15]
generated_by: gsd-execute-plan
lifecycle_mode: yolo
phase_lifecycle_id: 36-2026-07-23T15-20-53
generated_at: 2026-07-25T05:32:14Z
duration: 8min
completed: 2026-07-25
---

# Phase 36 Plan 08: Progress-Gated Second Attempt Summary

**One exact-current Ultra 205 attempt reached a typed flash failure, recorded failed bounded restoration and complete cleanup, and sealed non-promotional without retry or candidate creation**

## Performance

- **Duration:** 8 min
- **Started:** 2026-07-25T05:24:15Z
- **Completed:** 2026-07-25T05:32:14Z
- **Tasks:** 2
- **Tracked files created or modified:** 2
- **Hardware command invocations:** 1
- **Broker-owned detector invocations:** 1
- **Capture bound:** 360 seconds
- **Adapter wall-clock bound:** 420 seconds

## Accomplishments

- Updated the deployed Phase 36 process regression to require exactly `36-08@7`, `36-07@8`, and `36-04@9`, with all three plans marked as gap closure.
- Passed the ordered Rust checks, all 75 Just targets, fresh no-cache Phase 36 broker/evidence/deployed-process suites, parity, reference, redaction, diff, exact package, and software-only preflight gates at source commit `3e52aa9080401336df55903a91f1c0f94d50bfd9`.
- Invoked the existing hardware mode exactly once. The broker invoked its detector exactly once, preserved `flash_failed` as the earliest typed failure, recorded `recovery_failed` separately, and completed cleanup.
- Sealed the attempt as `sealed_non_promotion`, created no commit-redacted candidate, and left offline Plan 36-07 blocked.
- Preserved the prior Plan 36-06 and Phase 35 evidence histories without access, reuse, mutation, reconciliation, or claim promotion.

## Task Commits

1. **Task 1: Commit the exact graph regression and freeze a clean-current-HEAD preflight** - `3e52aa90` (test)
2. **Task 2: Invoke and seal exactly one broker-owned Ultra 205 attempt** - captured by the task outcome commit containing this summary

## Files Created/Modified

- `scripts/phase36-substantive-evidence-test.sh` - binds the exact remaining gap-plan graph while retaining deployed adapter, fake-detector, failure-ordering, cleanup, and replay-rejection regressions.
- `.planning/phases/36-substantive-evidence-admission-and-exact-re-promotion/36-08-SUMMARY.md` - records the closed typed non-promotion and exact downstream block.

The ignored private attempt remains outside Git. The planned candidate was not created because the terminal disposition is non-promotional.

## Decisions Made

- The authoritative disposition is `sealed_non_promotion`.
- The primary failure is `flash_failed`; the bounded same-image restoration result is the secondary `recovery_failed` category and does not replace the primary boundary.
- Cleanup completed after the failed restoration.
- No retry, ordinal change, standalone detector, or alternate device action is authorized.
- Plan 36-07 remains blocked because no eligible candidate exists.

## Deviations from Plan

None - plan executed exactly as written.

## Issues Encountered

- The first aggregate software-gate run encountered two local process startup races. Each named test passed on a fresh standalone no-cache run, and the complete exact aggregate gate then passed all 75 targets before package and preflight.
- The single hardware attempt reached `flash_failed`; its predeclared same-image restoration then recorded `recovery_failed`. Cleanup completed and the attempt sealed non-promotional without retry.

## Authentication Gates

None.

## User Setup Required

None.

## Verification

- Task 1 acceptance passed with a clean committed HEAD, exact three-plan graph, mode-constrained preflight artifacts, absent broker child before launch, and zero detector or hardware invocations.
- The complete ordered software gate passed, including all 75 Just targets and fresh no-cache Phase 36 broker, evidence, and deployed-process tests.
- Exact packaging and the one software-only preflight passed at `3e52aa9080401336df55903a91f1c0f94d50bfd9`.
- The hardware ledger contains 20 ordered records: exact admission, one detector invocation, failed flash, failed bounded recovery, and completed cleanup.
- The seal is `sealed_non_promotion` with primary `flash_failed` and secondary `recovery_failed`.
- The candidate is absent, so no private-to-public evidence derivation or promotion handoff occurred.
- Reference cleanliness, redaction, and diff checks passed.

## Known Stubs

None. Candidate absence is the required fail-closed result for this typed non-promotion.

## Next Phase Readiness

- Plan 36-07 remains blocked and must not run without a sealed-eligible candidate.
- Plan 36-04 remains the sole Wave 9 independent-review and canonical-reconciliation owner, but its dependency is not satisfied by this result.
- This plan authorizes no hardware retry or alternate recovery.
