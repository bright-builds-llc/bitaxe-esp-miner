---
phase: 20-active-safety-hardware-telemetry-evidence
plan: "01"
subsystem: parity-evidence
tags: [safety-allow, failure-paths, redaction, hardware-evidence, parity]

requires:
  - phase: 14-safety-hardware-evidence-completion
    provides: Safety allow manifest pattern and prior safety evidence boundaries.
  - phase: 20-active-safety-hardware-telemetry-evidence
    provides: Yolo lifecycle context and active-safety evidence decisions.
provides:
  - First-class `failure-paths` safety allow surface coverage.
  - Phase 20 evidence pack contract.
  - Pending Phase 20 redaction review scaffold.
  - Draft exact-claim evidence ledger.
  - Wave 0 validation mapping for Plan 20-01.
affects: [phase-20, parity-evidence, safety-allow, failure-paths, redaction]

tech-stack:
  added: []
  patterns:
    - Safety allow manifests preserve active-tier hardware-regression requirements.
    - Evidence scaffolds start pending and avoid checklist promotion before reviewed artifacts exist.

key-files:
  created:
    - docs/parity/evidence/phase-20-active-safety-hardware-telemetry-evidence/evidence-contract.md
    - docs/parity/evidence/phase-20-active-safety-hardware-telemetry-evidence/redaction-review.md
    - docs/parity/evidence/phase-20-active-safety-hardware-telemetry-evidence/summary.md
  modified:
    - tools/parity/src/safety_allow.rs
    - .planning/phases/20-active-safety-hardware-telemetry-evidence/20-VALIDATION.md

key-decisions:
  - "`failure-paths` is represented as a standalone safety allow surface while preserving `fault-stimulus` as an active tier requiring `hardware-regression`."
  - "Phase 20 evidence starts with pending scaffold artifacts and no checklist updates until redaction and exact evidence are complete."
  - "The TDD RED failure was run and recorded but not committed because repo Rust pre-commit rules require passing checks before every commit."

patterns-established:
  - "Failure-path evidence can be blocked as `unsupported-pending` plus `deferred` without recovery steps because it is not an active stimulus."
  - "Phase 20 redaction review must clear named artifacts before citation and starts with `raw_artifacts_committed: no`."

requirements-completed: [SAFE-04, SAFE-08, EVD-05]
generated_by: gsd-execute-plan
lifecycle_mode: yolo
phase_lifecycle_id: 20-2026-07-03T20-48-00
generated_at: 2026-07-03T21:57:33Z

duration: 6 min
completed: 2026-07-03
---

# Phase 20 Plan 01: Evidence Foundation Summary

**First-class failure-paths safety surface with pending Phase 20 evidence and redaction scaffolds**

## Performance

- **Duration:** 6 min
- **Started:** 2026-07-03T21:50:26Z
- **Completed:** 2026-07-03T21:56:56Z
- **Tasks:** 2
- **Files modified:** 5

## Accomplishments

- Added `failure-paths` to the safety allow validator without weakening active-tier abort, recovery, safe-state, or `hardware-regression` requirements.
- Added focused safety allow tests for active `fault-stimulus`, rejected `hardware-smoke` active fault evidence, and blocked `unsupported-pending` failure-path evidence.
- Created Phase 20 evidence, redaction, and draft exact-claim scaffolds before any hardware, network, or checklist citation work begins.
- Updated Wave 0 validation so `20-W0-01` points to Plan 20-01 and records `failure-paths` surface coverage.

## Task Commits

Each task was committed atomically:

1. **Task 1: Add `failure-paths` to safety allow validation** - `a77f854` (feat)
2. **Task 2: Create Phase 20 evidence scaffold and redaction contract** - `310480d` (docs)

## Files Created/Modified

- `tools/parity/src/safety_allow.rs` - Adds `failure-paths` and validator tests for active and deferred failure-path evidence.
- `docs/parity/evidence/phase-20-active-safety-hardware-telemetry-evidence/evidence-contract.md` - Defines Phase 20 pack names, claim tiers, metadata, evidence-class requirements, and non-claims.
- `docs/parity/evidence/phase-20-active-safety-hardware-telemetry-evidence/redaction-review.md` - Starts pending artifact redaction review with raw artifact and secret-scan requirements.
- `docs/parity/evidence/phase-20-active-safety-hardware-telemetry-evidence/summary.md` - Starts the draft exact-claim ledger at pending status.
- `.planning/phases/20-active-safety-hardware-telemetry-evidence/20-VALIDATION.md` - Links Wave 0 validation row `20-W0-01` to this plan and marks the scaffold/surface prerequisites complete.

## Decisions Made

- `failure-paths` is a first-class safety allow surface, not hidden under `thermal-fan`, `voltage-control`, or another adjacent surface.
- Active failure-path evidence still uses `fault-stimulus`, so it continues to require `hardware-regression`, recovery steps, abort conditions, and safe-state markers.
- Blocked failure-path evidence is represented as `unsupported-pending` plus `deferred`, with recovery fields allowed to stay empty because no active stimulus ran.
- Redaction remains pending for Phase 20; the scaffold deliberately does not clear artifacts or update checklist rows.

## Deviations from Plan

### Process Adjustments

**1. AGENTS.md precedence - TDD RED was not committed**
- **Found during:** Task 1 (Add `failure-paths` to safety allow validation)
- **Issue:** The GSD TDD flow normally commits the RED test, but this Rust repo requires passing format, clippy, build, and test checks before every commit.
- **Fix:** Ran the RED test and confirmed the expected failures, then committed the passing GREEN state after the full Rust pre-commit sequence.
- **Files modified:** `tools/parity/src/safety_allow.rs`
- **Verification:** RED failed only on missing `failure-paths`; GREEN passed `cargo test -p bitaxe-parity --all-features safety_allow`.
- **Committed in:** `a77f854`

**Total deviations:** 0 auto-fixed; 1 repo-rule process adjustment.
**Impact on plan:** No behavior scope change. The adjustment preserves repo commit policy while still recording the TDD failure signal.

## Issues Encountered

None. The only failing command was the expected TDD RED run before `failure-paths` was added.

## Verification

- `node /Users/peterryszkiewicz/.codex/get-shit-done/bin/gsd-tools.cjs verify lifecycle 20 --require-plans --raw` returned `valid` before execution began.
- `cargo test -p bitaxe-parity --all-features safety_allow` passed.
- `bazel test //tools/parity:tests --test_filter=safety_allow` passed.
- Scaffold `rg` acceptance checks passed for pack names, redaction markers, draft claim status, and `20-W0-01`.
- `git diff -- reference/esp-miner --exit-code` passed.
- Before each task commit: `cargo fmt --all`, `cargo clippy --all-targets --all-features -- -D warnings`, `cargo build --all-targets --all-features`, and `cargo test --all-features` passed.

## Known Stubs

None. The `pending` values in Phase 20 evidence files are intentional scaffold statuses required by the plan and do not block this plan's goal.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

Ready for Plan 20-02. Downstream plans can now consume the `failure-paths` safety allow surface and the Phase 20 evidence/redaction scaffold without promoting unsupported safety claims.

## Self-Check: PASSED

- Found `.planning/phases/20-active-safety-hardware-telemetry-evidence/20-01-SUMMARY.md`.
- Found Phase 20 evidence contract, redaction review, and draft summary files.
- Found task commit `a77f854`.
- Found task commit `310480d`.

*Phase: 20-active-safety-hardware-telemetry-evidence*
*Completed: 2026-07-03*
