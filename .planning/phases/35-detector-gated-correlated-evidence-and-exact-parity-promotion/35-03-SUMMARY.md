---
phase: 35-detector-gated-correlated-evidence-and-exact-parity-promotion
plan: "03"
subsystem: parity
tags: [rust, evidence, parity, atomic-publication]
requires:
  - phase: 35-01
    provides: typed validated Phase 35 evidence
  - phase: 35-02
    provides: detector-gated supervisor evidence
provides:
  - exhaustive exact Phase 35 promotion matrix
  - rollback-capable atomic evidence and checklist publication
  - byte-preserving checklist projection and redaction contract
affects: [35-04]
tech-stack:
  added: []
  patterns:
    - closed typed promotion matrix
    - staged atomic publication
    - byte-preserving checklist projection
key-files:
  created:
    - tools/parity/src/phase35_promotion.rs
    - tools/parity/src/phase35_promotion/types.rs
    - tools/parity/src/phase35_promotion/checklist.rs
    - tools/parity/src/phase35_promotion/evaluator.rs
    - tools/parity/src/phase35_promotion/tests.rs
    - tools/parity/src/operator_evidence/generation/phase35.rs
    - scripts/phase35-promotion-contract-test.sh
  modified:
    - tools/parity/src/main.rs
    - tools/parity/src/phase35_evidence.rs
    - tools/parity/src/operator_evidence/generation/tests/promotion.rs
    - docs/parity/checklist.md
    - tools/parity/BUILD.bazel
    - scripts/BUILD.bazel
key-decisions:
  - "Only four dedicated board-205 passive rows can promote; all broader or excluded scopes receive typed non-promotion decisions."
  - "The live CLI independently rechecks package, reference, lifecycle, detector, root, and no-actuation facts before constructing the matrix."
  - "Evidence and checklist projections are staged and exchanged with rollback; dedicated rows remain implemented until Plan 35-04 admits a real root."
requirements-completed: [EVD-14, EVD-15]
duration: 25m
completed: 2026-07-17
generated_by: gsd-execute-plan
lifecycle_mode: yolo
phase_lifecycle_id: 35-2026-07-17T17-00-37
generated_at: 2026-07-17T22:40:10Z
---

# Phase 35 Plan 03: Exact Parity Promotion Summary

An exhaustive, fail-closed Phase 35 promotion matrix now admits only four narrow passive board-205 claims from one protected-root digest and publishes their redacted evidence/checklist projection atomically.

## Performance

- **Duration:** 25 minutes
- **Started:** 2026-07-17T22:15:45Z
- **Completed:** 2026-07-17T22:40:10Z
- **Tasks:** 2
- **Files modified:** 18

## Accomplishments

- Added a closed 15-scope promotion matrix with four exact promotable claims and typed exclusions for every broader, active, credential-bearing, destructive, archived-lineage, and other-board scope.
- Added `admit-phase35-evidence`, which independently rechecks current package, reference, lifecycle, detector, artifact-digest, root, event, and no-actuation facts before producing a promotion decision.
- Added staged, rollback-capable evidence/checklist publication plus failure-injection tests proving previous-generation preservation at every exchange boundary.
- Added four dedicated checklist rows that remain `implemented` and non-verified until Plan 35-04 supplies an eligible real protected root.
- Added a Bazel-owned preservation/redaction contract proving non-allowlisted rows remain byte-identical and raw canaries never enter published evidence.

## Task Commits

Each task was committed atomically:

1. **Task 1: Build the exhaustive exact promotion matrix and CLI** - `d2aee173` (feat)
2. **Task 2: Prove atomic publication, preservation, and redaction** - `d731c7da` (test)

## Files Created/Modified

- `tools/parity/src/phase35_promotion.rs` - Focused module entrypoint for Phase 35 promotion logic.
- `tools/parity/src/phase35_promotion/types.rs` - Closed scopes, decisions, live rechecks, and matrix data.
- `tools/parity/src/phase35_promotion/evaluator.rs` - Fail-closed exact-claim evaluator.
- `tools/parity/src/phase35_promotion/checklist.rs` - Byte-preserving checklist projection.
- `tools/parity/src/phase35_promotion/tests.rs` - Exhaustiveness, digest, exclusion, and rendering tests.
- `tools/parity/src/operator_evidence/generation/phase35.rs` - Staged atomic publisher with exchange rollback.
- `tools/parity/src/operator_evidence/generation/tests/promotion.rs` - Publication success, drift, and injected-failure coverage.
- `tools/parity/src/main.rs` - Live admission CLI and independent rechecks.
- `tools/parity/src/phase35_evidence.rs` - Preserved package/detector/admission identity and artifact validation helpers.
- `scripts/phase35-promotion-contract-test.sh` - Repository-level preservation and redaction contract.
- `docs/parity/checklist.md` - Four dedicated passive board-205 rows, intentionally non-verified.

## Decisions Made

- The allowlist is represented as an exhaustive typed scope enumeration; adding a new scope requires an explicit promotion or exclusion decision.
- Every promoted row must name the same admitted protected-root digest, and a mismatch in any live recheck fails the matrix closed.
- Publication stages both evidence and checklist content, validates the staged generation, then exchanges both destinations with rollback to preserve the prior generation on failure.
- Phase 30 no-promotion breadcrumbs and historical checklist rows are preserved exactly; this plan does not reinterpret archived evidence.

## Deviations from Plan

None - plan executed exactly as written.

## Issues Encountered

- The first repository-contract run referenced a stale Bazel test binary and lacked cross-package visibility. The test target visibility and source dependencies were made explicit, after which direct and Bazel-owned contract runs passed.

## Known Stubs

None. The four new checklist rows intentionally remain non-verified because real protected-root admission belongs to Plan 35-04.

## Verification

- `cargo fmt --all`
- `cargo clippy --all-targets --all-features -- -D warnings`
- `cargo build --all-targets --all-features`
- `cargo test --all-features`
- `bazel test //tools/parity:tests //scripts:phase35_promotion_contract_test //scripts:phase30_no_promotion_contract_test`
- `just verify-reference`
- `just parity`
- `node "$HOME/.codex/get-shit-done/bin/gsd-tools.cjs" verify lifecycle 35 --require-plans --raw`
- `git diff --check`

## Next Phase Readiness

- Plan 35-04 can run the real detector-gated correlated capture and pass its validated protected root to the admission CLI.
- No checklist row has been promoted from synthetic evidence; all active control, mining, credential, OTA, watchdog, self-test, other-board, and archived Phase 28.1.1 scopes remain excluded.

## Self-Check: PASSED

All declared created files and both task commits exist.
