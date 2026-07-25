---
phase: 36-substantive-evidence-admission-and-exact-re-promotion
plan: "09"
subsystem: evidence
tags: [recovery-authority, process-boundary, rust, parser, broker, software-only]
requires:
  - phase: 36-08
    provides: immutable pre-device parser failure and non-promotion history
provides:
  - effect-aware broker state machine that denies recovery before confirmed device effect
  - identity-bound mode-0600 process result contract for partial and completed effects
  - real-parser and deployed-process regressions for no-effect and same-image recovery paths
affects: [36-10, 36-07, phase36-hardware-attempt, evidence-admission]
tech-stack:
  added: []
  patterns:
    - private constructors make confirmed device effect the sole recovery authority
    - closed process results bind operation, package identity, factory image, and process outcome
key-files:
  created: []
  modified:
    - tools/parity/src/phase36_broker/contract.rs
    - tools/parity/src/phase36_broker/hardware.rs
    - tools/parity/src/phase36_broker/hardware_process.rs
    - tools/parity/src/phase36_broker/ledger.rs
    - tools/parity/src/phase36_broker/tests.rs
    - tools/parity/src/main.rs
    - scripts/phase36-hardware-effect.sh
    - scripts/phase36-substantive-evidence-test.sh
    - scripts/BUILD.bazel
    - tools/flash/src/main.rs
key-decisions:
  - "A generic failure, process exit, stderr string, or caller assertion can never construct recovery authority."
  - "Only an identity-bound confirmed partial exact flash or a trusted completed exact flash permits one same-image recovery."
  - "Every pre-effect failure transitions directly to one cleanup while preserving the earliest typed failure and a closed non-authority disposition."
patterns-established:
  - "Closed effect result: schema, operation, process exit, package digest, and factory digest must agree before effect authority exists."
  - "Bounded restoration: at most one same-image recovery follows a confirmed device effect, and cleanup remains mandatory."
requirements-completed: []
requirements-pending: [SYS-02, EVD-11, EVD-12, EVD-14, EVD-15]
generated_by: gsd-execute-plan
lifecycle_mode: yolo
phase_lifecycle_id: 36-2026-07-23T15-20-53
generated_at: 2026-07-25T19:59:12Z
duration: 35min
completed: 2026-07-25
---

# Phase 36 Plan 09: Recovery Authority Closure Summary

**Recovery is now impossible before an identity-bound confirmed device effect, while a partial or completed exact flash retains exactly one bounded same-image restoration path**

## Performance

- **Duration:** 35 min
- **Started:** 2026-07-25T19:24:22Z
- **Completed:** 2026-07-25T19:59:12Z
- **Tasks:** 2
- **Tracked files created or modified:** 11
- **Hardware, detector, credential, private-evidence, and network invocations:** 0

## Accomplishments

- Replaced the former failed-operation recovery rule with an effect-aware state machine. Admission, capability, detector, parser, invocation-construction, and no-effect flash failures now close without recovery authority and still perform one cleanup.
- Added private recovery-authorizing types bound to the admitted package and factory-image identities. Confirmed partial flash and later failure after confirmed completion permit one same-image recovery; mismatched identity fails before invocation.
- Added a mode-0600 `phase36-effect-result-v1` process document whose schema, operation, identity, status, and process exit must agree. Free-form stderr, exit codes, and fake stdout cannot forge device effect.
- Routed the real flash parser’s retired Plan 36-08 dual-plus-redaction rejection through deployed runfiles/process doubles and proved zero recovery. The repaired argument shape remains accepted.
- Preserved the first typed failure through recovery and cleanup, with recovery failure secondary and duplicate, reordered, post-close, or unauthorized recovery rejected.
- Retained ancestry from repair commit `df9cb90008bf47f94434545021a47e237e0c5739` and completed the plan entirely in software.

## Task Commits

1. **Task 1: Make confirmed device effect the sole recovery authority** - `50771a20` (fix)
2. **Task 2: Prove the real parser and deployed process boundary cannot recover before device effect** - `716b1b1e` (fix)

## Files Created/Modified

- `tools/parity/src/phase36_broker/contract.rs` - defines closed effect and recovery-disposition contracts.
- `tools/parity/src/phase36_broker/hardware.rs` - owns effect-aware transaction sequencing, same-image recovery, primary/secondary failure preservation, and cleanup.
- `tools/parity/src/phase36_broker/ledger.rs` - validates effect-aware recovery transitions and closed dispositions.
- `tools/parity/src/phase36_broker/tests.rs` - exercises unauthorized recovery, earliest failure, ledger order, and privacy invariants.
- `tools/parity/src/phase36_broker/hardware_process.rs` - validates identity-bound process results and constructs recovery authority only from trusted classifications.
- `tools/parity/src/main.rs` - wires the repaired hardware transaction contract.
- `scripts/phase36-hardware-effect.sh` - emits closed owner-only effect results from parser or trusted flash-stage observations.
- `scripts/phase36-substantive-evidence-test.sh` - covers real-parser, deployed-runfiles, forged-output, identity-mismatch, partial-effect, completed-effect, recovery, and cleanup paths with synthetic doubles.
- `scripts/BUILD.bazel` - supplies the deployed flash parser binary to the process regression.
- `tools/flash/src/main.rs` - writes closed pre-effect parser and invocation-construction outcomes before any environment detection.

## Decisions Made

- Device-effect authority is a private typed capability, not a property inferred from a generic `Result`, process launch, exit code, output text, or caller input.
- Exact-package flash identity and factory-image identity are immutable across the sole permitted restoration attempt.
- A completed exact flash may authorize restoration after a later passive/read-only failure; that same later failure has no recovery authority without the completed-effect boundary.
- Cleanup is mandatory on every terminal path and cannot overwrite the earliest failure.
- Plan 36-09 changes no requirement truth and authorizes no hardware action; Plan 36-10 remains the separately gated hardware owner.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] Kept the deployed planning-graph regression valid across summary creation**

- **Found during:** Task 2 final verification
- **Issue:** The exact incomplete-plan fixture still named the completed Plan 36-08 graph and would become stale again when the Plan 36-09 summary appeared.
- **Fix:** Bound the exact graph to the presence of the Plan 36-09 summary, requiring Waves 8-11 before completion and Waves 9-11 afterward while checking every incomplete plan’s gap-closure marker.
- **Files modified:** `scripts/phase36-substantive-evidence-test.sh`
- **Commit:** `716b1b1e`

## Issues Encountered

- The deployed supervisor intentionally rejects a dirty source tree. The orchestrator had already made a targeted `STATE.md` planning update, so the final deployed and full-repository gates were run only after preserving that update in the plan metadata commit and rebuilding the exact-current package.
- Existing firmware release-build dead-code warnings remain unchanged and are outside this plan’s recovery-boundary scope. The required host Rust Clippy gate passed with warnings denied.

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
- `bazel test --nocache_test_results //tools/flash:tests //tools/parity:phase36_broker_tests //scripts:phase36_substantive_evidence_test`
- `just test`
- `just parity`
- `just verify-reference`
- `just verify-redaction`
- `git diff --check`
- `just package` built the exact-current Ultra 205 image without invoking hardware.

## Known Stubs

None. Synthetic process doubles are deliberate test boundaries and do not substitute for Plan 36-10 hardware evidence.

## Next Phase Readiness

- Plan 36-10 may begin from this clean software gate and remains the sole owner of the separately authorized bounded Ultra 205 attempt.
- Plan 36-07 remains offline and blocked unless Plan 36-10 produces an eligible redacted candidate.
- Plan 36-04 remains the sole canonical reconciliation owner after its exact predecessors complete.
- SYS-02, EVD-11, EVD-12, EVD-14, and EVD-15 remain unchanged pending the planned evidence and reconciliation sequence.

## Self-Check: PASSED

- This summary exists with exactly one opening and one closing YAML frontmatter delimiter.
- Task commits `50771a20` and `716b1b1e` exist in Git history.
- All listed source and test files exist.
- No hardware, detector, credential, private-evidence, network, or device command ran.
