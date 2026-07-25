---
phase: 36-substantive-evidence-admission-and-exact-re-promotion
plan: "10"
subsystem: evidence
tags: [hardware-attempt, ultra205, broker, exact-package, privacy, non-promotion]
requires:
  - phase: 36-09
    provides: effect-aware recovery authority and deployed process regressions
provides:
  - exact remaining-plan graph regression for Waves 9 through 11
  - one clean-current-HEAD package and software-only preflight
  - one broker-owned hardware command with a sealed non-promotion disposition
  - truthful no-candidate handoff that leaves Plan 36-07 blocked
affects: [36-07, 36-04, phase36-hardware-attempt, evidence-promotion]
tech-stack:
  added: []
  patterns:
    - one broker-owned hardware command after clean exact-package preflight
    - protected accounting remains private when no shareable projection exists
key-files:
  created:
    - .planning/phases/36-substantive-evidence-admission-and-exact-re-promotion/36-10-SUMMARY.md
  modified:
    - scripts/phase36-substantive-evidence-test.sh
key-decisions:
  - "Honor sealed_non_promotion without retry or candidate creation."
  - "Do not infer the protected first-failure, recovery, cleanup, detector, or restoration details from timing or source structure."
  - "Leave Plan 36-07 blocked and device restoration unresolved because no eligible shareable candidate exists."
patterns-established:
  - "Public non-promotion boundary: record only the closed disposition and absence facts actually projected by the repository-owned command."
requirements-completed: []
requirements-pending: [SYS-02, EVD-11, EVD-12, EVD-14, EVD-15]
generated_by: gsd-execute-plan
lifecycle_mode: yolo
phase_lifecycle_id: 36-2026-07-23T15-20-53
generated_at: 2026-07-25T20:23:40Z
duration: 8min
completed: 2026-07-25
---

# Phase 36 Plan 10: Fresh Broker Attempt Summary

**One exact-current Ultra 205 broker command sealed non-promotional, created no candidate, and stopped without retry or unsupported restoration claims**

## Performance

- **Duration:** 8 min
- **Started:** 2026-07-25T20:15:28Z
- **Completed:** 2026-07-25T20:23:40Z
- **Tasks:** 2
- **Tracked files created or modified:** 2
- **Hardware command invocations:** 1
- **Capture bound:** 360 seconds
- **Adapter wall-clock bound:** 420 seconds

## Accomplishments

- Committed a deployed regression that binds the exact remaining Phase 36 graph to `36-10@9`, `36-07@10`, and `36-04@11`, with every plan marked gap closure and all three predecessor edges asserted.
- Passed the ordered Rust checks, four fresh no-cache Phase 36/flash suites, the full repository test graph, parity, reference cleanliness, redaction, exact-current package, and software-only preflight gates at source commit `15d14c82aee9f31b73bfc0e46e4f8fdc297da874`.
- Invoked the authorized repository-owned hardware command exactly once with board 205 and a 360-second capture bound. The command returned `sealed_non_promotion`.
- Created no commit-redacted candidate, performed no retry or follow-up device action, and left Plan 36-07 blocked.
- Preserved the prior Plan 36-06, Plan 36-08, and Phase 35 histories without access, reuse, mutation, reconciliation, or claim promotion.

## Task Commits

1. **Task 1: Commit the exact graph regression and complete one software preflight** - `15d14c82` (test)
2. **Task 2: Run exactly one broker-owned Ultra 205 attempt and seal its typed outcome** - pending outcome commit

## Files Created/Modified

- `scripts/phase36-substantive-evidence-test.sh` - asserts the exact remaining waves, gap-closure flags, and predecessor chain.
- `.planning/phases/36-substantive-evidence-admission-and-exact-re-promotion/36-10-SUMMARY.md` - records the shareable non-promotion disposition and downstream block.

The ignored private attempt remains outside Git and was not inspected. The planned candidate was not created because the terminal disposition is non-promotional.

## Decisions Made

- The authoritative public disposition is `sealed_non_promotion`.
- The command count is exactly one and no second attempt, standalone detector, manual recovery, network inspection, or other device action is authorized.
- The repository-owned wrapper exposed no shareable first-failure, secondary-failure, recovery-disposition, cleanup-result, detector-count, or restoration fields. Those details remain protected; this summary does not infer them from elapsed time, code structure, or prior attempts.
- Candidate absence blocks offline Plan 36-07. Plan 36-04 remains the sole canonical reconciliation owner after its exact predecessor can truthfully complete.
- Device restoration remains unresolved because this attempt produced no shareable typed restoration observation.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] Built the exact-current package before the deployed process aggregate**

- **Found during:** Task 1 full software gate
- **Issue:** The graph-regression commit rotated `HEAD`, while the deployed process suite's synthetic preflight correctly rejected the existing package manifest before the plan's later package step.
- **Fix:** Built the exact-current board-205 package, verified its source commit matched clean `HEAD`, and reran the complete prescribed gate from the start in the original order.
- **Files modified:** None; package artifacts are generated and ignored.
- **Verification:** The rerun passed all ordered Rust, no-cache deployed, full repository, parity, reference, redaction, package, and preflight gates.

**Total deviations:** 1 auto-fixed blocking issue.

**Impact:** Verification ordering gained the package state its deployed preflight requires; no source behavior, hardware authority, or evidence policy changed.

## Issues Encountered

- The sole hardware command returned `sealed_non_promotion` and created no candidate.
- The public wrapper projected only the terminal disposition. Exact protected failure/recovery/cleanup accounting cannot be repeated in this shareable summary without violating the explicit private-root inspection prohibition.
- Because no fresh shareable typed restoration observation exists, device restoration remains unresolved.

## Authentication Gates

None.

## User Setup Required

None.

## Verification

- `git merge-base --is-ancestor df9cb90008bf47f94434545021a47e237e0c5739 HEAD`
- `cargo fmt --all`
- `cargo clippy --all-targets --all-features -- -D warnings`
- `cargo build --all-targets --all-features`
- `cargo test --all-features`
- `bazel test --nocache_test_results //tools/flash:tests //tools/parity:phase36_broker_tests //tools/parity:phase36_evidence_tests //scripts:phase36_substantive_evidence_test`
- `just test`
- `just parity`
- `just verify-reference`
- `just verify-redaction`
- `just package`
- Software-only preflight returned `preflight_ready` at clean exact-current `HEAD`.
- The sole hardware command returned `sealed_non_promotion`.
- The public candidate path is absent and the worktree contains no private evidence.

## Known Stubs

None. Candidate absence is the required fail-closed result for a non-promotional attempt.

## Next Phase Readiness

- Plan 36-07 remains blocked and must not run without a sealed-eligible candidate.
- Plan 36-04 remains the sole Wave 11 independent-review and canonical-reconciliation owner, but its dependency is not satisfied by this result.
- This plan authorizes no hardware retry, alternate recovery, or private-root inspection.
- SYS-02, EVD-11, EVD-12, EVD-14, and EVD-15 remain unchanged.

## Self-Check: PENDING

- Awaiting the Task 2 outcome commit and final tracking update.
