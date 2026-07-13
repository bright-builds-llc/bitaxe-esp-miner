---
phase: 15-bm1366-mining-evidence-completion
plan: "01"
subsystem: parity
tags: [bm1366, mining, evidence, parity, redaction]
requires:
  - phase: 14-safety-hardware-evidence-completion
    provides: safety allow-manifest pattern and evidence/redaction scaffold
provides:
  - Typed Phase 15 mining allow-manifest parser and validator
  - `mining-allow` parity CLI gate for evidence-producing mining procedures
  - Phase 15 mining evidence ladder, pack contract, stop conditions, and redaction template
affects: [phase-15, tools-parity, bm1366-mining, parity-evidence]
tech-stack:
  added: []
  patterns:
    - Typed allow-manifest validation in `tools/parity`
    - Procedure-scoped evidence ladder and redaction review scaffold
key-files:
  created:
    - tools/parity/src/mining_allow.rs
    - docs/parity/evidence/phase-15-bm1366-mining-evidence-completion/README.md
    - docs/parity/evidence/phase-15-bm1366-mining-evidence-completion/redaction-review.md
  modified:
    - tools/parity/src/main.rs
    - tools/parity/BUILD.bazel
key-decisions:
  - "Created a separate `mining_allow` module instead of extending `safety_allow` so Phase 15 mining evidence semantics stay distinct from Phase 14 safety-control semantics."
  - "Kept the TDD RED failure uncommitted because repo Rust rules require passing checks before every commit."
patterns-established:
  - "Mining allow manifest: detector, board-info, package identity, surface, claim tier, evidence class, inputs, abort, recovery, safe-state, redaction, and checklist rows are validated before wrappers can proceed."
  - "Phase 15 evidence packs use exact-claim promotion through a six-tier ladder and artifact-specific redaction review."
requirements-completed: [STR-07, SAFE-09, EVD-05]
generated_by: gsd-execute-plan
lifecycle_mode: yolo
phase_lifecycle_id: 15-2026-07-01T02-07-59
generated_at: 2026-07-01T03:21:12Z
duration: 10 min
completed: 2026-07-01
---

# Phase 15 Plan 01: Mining Evidence Gate And Scaffold Summary

**Mining evidence preflight now has a typed `mining-allow` gate plus a Phase 15 evidence ladder and redaction contract.**

## Performance

- **Duration:** 10 min
- **Started:** 2026-07-01T03:11:03Z
- **Completed:** 2026-07-01T03:21:12Z
- **Tasks:** 2
- **Files modified:** 5

## Accomplishments

- Added `tools/parity` support for a `mining-allow` subcommand that validates Phase 15 manifests before evidence-producing BM1366/mining commands can run.
- Covered mining allow behavior with focused Rust tests for board, detector, port, board-info, package identity, surface, live-pool, and bounded-soak gates.
- Created the Phase 15 evidence pack README and redaction review template, including exact ladder tiers, five component packs, stop conditions, prohibited actions, and checklist promotion rules.

## Task Commits

Each task was committed atomically:

1. **Task 1: Add the `mining-allow` manifest gate** - `2e435dc` (feat)
2. **Task 2: Create the Phase 15 evidence scaffold and redaction template** - `e68fe9d` (docs)

**Plan metadata:** pending final metadata commit

## Files Created/Modified

- `tools/parity/src/mining_allow.rs` - Typed Phase 15 mining allow manifest, validation rules, renderer, loader, filters, and tests.
- `tools/parity/src/main.rs` - Registered `mod mining_allow`, `MiningAllow` CLI args, and `run_mining_allow_command`.
- `tools/parity/BUILD.bazel` - Added `src/mining_allow.rs` to the `tools/parity` Rust binary source list.
- `docs/parity/evidence/phase-15-bm1366-mining-evidence-completion/README.md` - Phase 15 scope, hardware gate, manifest contract, evidence ladder, component packs, stop conditions, prohibited actions, and promotion rules.
- `docs/parity/evidence/phase-15-bm1366-mining-evidence-completion/redaction-review.md` - Artifact-specific unchecked redaction review for pool credentials, worker secrets, Wi-Fi credentials, private endpoints, private `DEVICE_URL`, API tokens, NVS secret values, local terminal secrets, serial logs, JSON, Markdown, API responses, WebSocket captures, pasted output, and manual observations.

## Decisions Made

- Created a dedicated `mining_allow` module rather than generalizing `safety_allow`; this keeps mining evidence rules explicit and avoids mixing safety-control and mining-smoke claim tiers.
- Required `workflow` evidence for `unsupported-pending` and `parity-redaction`, `hardware-smoke` for diagnostic/smoke tiers, and `soak` for bounded soak, matching the plan contract.
- Preserved conservative documentation: the redaction review remains pending until later evidence artifacts exist, and no checklist rows are promoted by this plan.

## Deviations from Plan

### Process Adjustments

**1. AGENTS.md pre-commit rule superseded TDD RED commit**
- **Found during:** Task 1 (Add the `mining-allow` manifest gate)
- **Issue:** The plan was marked `tdd="true"`, but repo Rust rules require `cargo fmt --all`, `cargo clippy --all-targets --all-features -- -D warnings`, `cargo build --all-targets --all-features`, and `cargo test --all-features` to pass before every commit.
- **Fix:** Ran the RED test and captured the expected failing signal, then implemented the validator and committed only the passing Task 1 state.
- **Files modified:** `tools/parity/src/mining_allow.rs`, `tools/parity/src/main.rs`, `tools/parity/BUILD.bazel`
- **Verification:** RED `cargo test -p bitaxe-parity --all-features mining_allow` failed with eight mining_allow failures; post-implementation targeted, Bazel, parity, and full Rust checks passed.
- **Committed in:** `2e435dc`

**Total deviations:** 1 process adjustment, 0 code auto-fixes.
**Impact on plan:** No behavior scope change. The TDD design loop was followed, while commit history stayed compatible with the repo's mandatory Rust pre-commit policy.

## Issues Encountered

- The initial redaction category scan used shell double quotes around a pattern containing backticks, which attempted command substitution for `DEVICE_URL`; it was rerun with safe quoting and passed.

## Known Stubs

None. The unchecked `redaction-review.md` items are intentional pre-evidence controls for later Phase 15 plans, not runtime or UI stubs.

## User Setup Required

None - no external service configuration required.

## Verification

- `node "$HOME/.codex/get-shit-done/bin/gsd-tools.cjs" verify lifecycle 15 --require-plans --raw` - passed
- RED: `cargo test -p bitaxe-parity --all-features mining_allow` - failed as expected before implementation
- `cargo test -p bitaxe-parity --all-features mining_allow` - passed
- `bazel test //tools/parity:tests --test_filter=mining_allow` - passed
- `just parity` - passed with `validation_errors: none`
- `rg -n "Evidence Ladder|detector/package/safe boot|trusted BM1366 chip-detect|typed diagnostic work/result|controlled mining smoke|bounded mining soak|bm1366-chip-detect|bm1366-work-result|mining-smoke|bounded-soak|parity-redaction|pool credentials|DEVICE_URL|NVS secret" docs/parity/evidence/phase-15-bm1366-mining-evidence-completion/README.md docs/parity/evidence/phase-15-bm1366-mining-evidence-completion/redaction-review.md` - passed
- `rg -n "^---$" docs/parity/evidence/phase-15-bm1366-mining-evidence-completion/README.md docs/parity/evidence/phase-15-bm1366-mining-evidence-completion/redaction-review.md` - no matches, as required
- Task commit pre-checks: `cargo fmt --all`; `cargo clippy --all-targets --all-features -- -D warnings`; `cargo build --all-targets --all-features`; `cargo test --all-features` - passed before each task commit

## Next Phase Readiness

Ready for `15-02-PLAN.md`: package-backed BM1366 chip-detect evidence can use `mining-allow` manifests and the Phase 15 pack scaffold before running hardware commands or citing generated logs.

*Phase: 15-bm1366-mining-evidence-completion*
*Completed: 2026-07-01*

## Self-Check: PASSED

- Summary file exists.
- Created files exist: `tools/parity/src/mining_allow.rs`, Phase 15 evidence README, and Phase 15 redaction review.
- Task commits exist: `2e435dc` and `e68fe9d`.
