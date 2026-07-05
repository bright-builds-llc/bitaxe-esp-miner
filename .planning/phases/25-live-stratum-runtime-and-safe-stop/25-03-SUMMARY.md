---
phase: 25-live-stratum-runtime-and-safe-stop
plan: 03
subsystem: evidence
tags: [bash, rust, parity, mining-allow, evidence, redaction, safe-stop, bazel]
requires:
  - phase: 25-live-stratum-runtime-and-safe-stop
    provides: Pure live runtime, submit-response classifier, firmware socket shell, safe-stop markers, and watchdog categories from plans 25-01 and 25-02
provides:
  - Detector-gated Phase 25 evidence wrapper with blocked-safe-prerequisite and hardware modes
  - Phase 25 mining-allow surface and claim-tier validation for live-or-blocked Stratum evidence
  - Redaction-safe Phase 25 evidence docs, Phase 23 handoff updates, checklist rows, and validation metadata
affects: [phase-25-live-stratum-runtime, phase-26-telemetry-and-parity, parity-evidence, mining-allow]
tech-stack:
  added: []
  patterns:
    - detector-gated evidence wrapper
    - redaction-safe category labels
    - strict mining-allow command predicate
    - conservative checklist promotion
key-files:
  created:
    - scripts/phase25-live-stratum-evidence.sh
    - scripts/phase25-live-stratum-evidence-test.sh
    - docs/parity/evidence/phase-25-live-stratum-runtime-and-safe-stop/share-outcome.md
    - docs/parity/evidence/phase-25-live-stratum-runtime-and-safe-stop/safe-stop.md
    - docs/parity/evidence/phase-25-live-stratum-runtime-and-safe-stop/redaction-review.md
    - docs/parity/evidence/phase-25-live-stratum-runtime-and-safe-stop/summary.md
    - .planning/phases/25-live-stratum-runtime-and-safe-stop/25-VALIDATION.md
    - .planning/phases/25-live-stratum-runtime-and-safe-stop/25-03-SUMMARY.md
  modified:
    - scripts/BUILD.bazel
    - Justfile
    - tools/parity/src/mining_allow.rs
    - docs/parity/checklist.md
    - docs/parity/evidence/phase-23-redacted-operator-evidence-workflow/share-outcome.md
    - docs/parity/evidence/phase-23-redacted-operator-evidence-workflow/safe-stop.md
key-decisions:
  - "Recorded Phase 25 committed evidence as blocked-safe-prerequisite rather than accepted/rejected live share proof because no detector-gated live pool response artifact was produced."
  - "Allowed Phase 25 mining-allow manifests only through the repo-owned wrapper command surface while preserving raw Stratum, unsafe hardware-control, erase, rollback, and stale-target rejection."
  - "Promoted STR-11 to verified from deterministic unit coverage, while keeping STR-08, STR-09, SAFE-12, and SAFE-13 at implemented/workflow scope without hardware overclaiming."
patterns-established:
  - "Phase 25 evidence can be generated through `just phase25-evidence` and validated through `//scripts:phase25_live_stratum_evidence_test`."
  - "Phase 25 live-submit proof requires explicit device target category, approved pool input category, safe-stop markers, and the wrapper command."
  - "Committed evidence docs use category labels and exact non-claims for blocked live-share and hardware-watchdog proof."
requirements-completed: [STR-08, STR-09, STR-11, SAFE-12, SAFE-13]
generated_by: gsd-execute-plan
lifecycle_mode: yolo
phase_lifecycle_id: 25-2026-07-05T01-55-45
generated_at: 2026-07-05T02:36:00Z
duration: 7min
completed: 2026-07-05
---

# Phase 25 Plan 03: Evidence Wrapper, Mining Allow, and Checklist Closure Summary

**Detector-gated Phase 25 live Stratum evidence workflow with blocked-safe-prerequisite closure, strict mining-allow rules, and redaction-safe parity documentation**

## Performance

- **Duration:** 7min
- **Started:** 2026-07-05T02:28:59Z
- **Completed:** 2026-07-05T02:36:00Z
- **Tasks:** 3 completed
- **Files modified:** 14

## Accomplishments

- Added `scripts/phase25-live-stratum-evidence.sh` plus deterministic wrapper tests for blocked mode, detector failure, local credential category labels, redaction sentinels, safe-stop markers, watchdog status, and exact non-claims.
- Extended `tools/parity/src/mining_allow.rs` with the `live-stratum-runtime` surface, `live-submit-response` and `safe-prerequisite-blocked` claim tiers, and a strict Phase 25 wrapper command predicate.
- Created the Phase 25 evidence root, updated Phase 23 handoff slots, added conservative v1.1 checklist rows, and marked validation Nyquist-complete after the phase gate passed.

## Task Commits

Each task was committed atomically; TDD tasks have RED and GREEN commits:

1. **Task 25-03-01 RED: Add failing Phase 25 evidence wrapper tests** - `c8553de` (`test`)
2. **Task 25-03-01 GREEN: Add Phase 25 evidence wrapper** - `600fbda` (`feat`)
3. **Task 25-03-02 RED: Add failing Phase 25 mining allow tests** - `7876c86` (`test`)
4. **Task 25-03-02 GREEN: Allow Phase 25 mining evidence surface** - `75e45c5` (`feat`)
5. **Task 25-03-03: Close Phase 25 evidence claims** - `c1643c8` (`docs`)

## Files Created/Modified

- `scripts/phase25-live-stratum-evidence.sh` - Detector-gated Phase 25 evidence wrapper with blocked and hardware modes.
- `scripts/phase25-live-stratum-evidence-test.sh` - Synthetic blocked, detector-failure, credential-label, and redaction tests.
- `scripts/BUILD.bazel` - Bazel `sh_binary` and `sh_test` registration for the Phase 25 wrapper.
- `Justfile` - `phase25-evidence` command surface.
- `tools/parity/src/mining_allow.rs` - Phase 25 live-or-blocked allow-manifest rules and unit tests.
- `docs/parity/evidence/phase-25-live-stratum-runtime-and-safe-stop/*.md` - Redaction-safe Phase 25 evidence docs.
- `docs/parity/evidence/phase-23-redacted-operator-evidence-workflow/share-outcome.md` and `safe-stop.md` - Phase 23 handoff pointers to Phase 25 status.
- `docs/parity/checklist.md` - Conservative v1.1 STR-08, STR-09, STR-11, SAFE-12, and SAFE-13 rows.
- `.planning/phases/25-live-stratum-runtime-and-safe-stop/25-VALIDATION.md` - Completed validation contract.

## Decisions Made

- Treated committed Phase 25 evidence as a blocked-safe-prerequisite closure, not a live accepted/rejected share claim.
- Kept `STR-11` verified because deterministic fake-pool behavior is exactly unit-testable, while live socket/share and hardware watchdog rows remain below verified.
- Allowed workflow blocked manifests to pass mining-allow with blocked board-info status, while live-submit manifests still require board-info passed, explicit target category, and `--mode hardware`.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] Made the new wrapper test executable**
- **Found during:** Task 25-03-01 RED verification
- **Issue:** The initial `sh_test` failed before reaching the intended missing-wrapper RED failure because the new test script was not executable.
- **Fix:** Set executable mode on `scripts/phase25-live-stratum-evidence-test.sh`, then reran the RED check and confirmed it failed for missing `phase25-live-stratum-evidence.sh`.
- **Files modified:** `scripts/phase25-live-stratum-evidence-test.sh`
- **Verification:** `bazel test //scripts:phase25_live_stratum_evidence_test` failed for the expected missing wrapper, then passed after GREEN implementation.
- **Committed in:** `c8553de`

**Total deviations:** 1 auto-fixed blocking issue
**Impact on plan:** The fix was required for the planned Bazel shell test to execute and did not expand scope.

## Issues Encountered

- `cargo fmt --check` over the workspace reported pre-existing formatting drift in unrelated Rust files. Only the touched `tools/parity/src/mining_allow.rs` file was formatted with `rustfmt`, and `bazel test //tools/parity:tests` was rerun successfully.
- No authentication gates occurred.

## Known Stubs

None.

## Threat Flags

None. The new wrapper, evidence, redaction, and allow-manifest surfaces are covered by plan threats T-25-01 through T-25-05.

## Verification

- `bazel test //scripts:phase25_live_stratum_evidence_test`
- `bazel test //tools/parity:tests`
- `bazel test //scripts:phase25_live_stratum_evidence_test //tools/parity:tests`
- `just parity`
- `just verify-reference`
- `node "$HOME/.cursor/get-shit-done/bin/gsd-tools.cjs" verify lifecycle 25 --expect-id 25-2026-07-05T01-55-45 --expect-mode yolo --require-plans`
- Plan acceptance `rg` checks for wrapper/Justfile/Bazel/mining-allow/evidence/checklist/validation passed.
- Redaction scan over Phase 25 evidence docs and Phase 23 updated handoff slots returned no forbidden matches.

## Auth Gates

None.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

Phase 26 can consume the Phase 25 exact claim boundary: live accepted/rejected share proof and hardware watchdog proof remain blocked until detector-gated artifacts exist, while deterministic fake-pool and safe-stop workflow evidence are available for telemetry projection planning.

## Self-Check: PASSED

- Found created files: `scripts/phase25-live-stratum-evidence.sh`, `scripts/phase25-live-stratum-evidence-test.sh`, Phase 25 evidence docs, `25-VALIDATION.md`, and `25-03-SUMMARY.md`.
- Found task commits: `c8553de`, `600fbda`, `7876c86`, `75e45c5`, and `c1643c8`.

*Phase: 25-live-stratum-runtime-and-safe-stop*
*Completed: 2026-07-05*
