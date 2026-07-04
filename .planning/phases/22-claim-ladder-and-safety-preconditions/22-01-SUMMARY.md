---
phase: 22-claim-ladder-and-safety-preconditions
plan: 01
subsystem: parity
tags: [claim-ladder, evidence-governance, parity, rust, bazel]
requires:
  - phase: 21-live-mining-and-soak-evidence
    provides: approved controlled no-share soak closure and exact non-claims
provides:
  - Operator-visible claim ladder distinguishing v1.0 controlled no-share evidence from v1.1 readiness, runtime, share-outcome, and deferred non-claim tiers
  - Test-enforced parity guard that requires stable claim tier ids and rejects controlled no-share overclaims
  - Bazel fixture wiring for validating the committed claim ladder Markdown with Rust tests
affects: [phase-23-evidence-workflow, phase-25-live-stratum-runtime, parity-checklist, evidence-governance]
tech-stack:
  added: []
  patterns: [test-enforced markdown fixture validation, exact claim tier registry]
key-files:
  created:
    - docs/parity/evidence/phase-22-claim-ladder-and-safety-preconditions/claim-ladder.md
    - tools/parity/src/claim_ladder.rs
    - .planning/phases/22-claim-ladder-and-safety-preconditions/22-01-SUMMARY.md
  modified:
    - BUILD.bazel
    - tools/parity/BUILD.bazel
    - tools/parity/src/main.rs
key-decisions:
  - "Kept the claim ladder guard as a test-enforced helper instead of adding a new CLI subcommand."
  - "Declared the claim ladder Markdown as a Bazel compile-time fixture so include_str! validation remains hermetic."
  - "Treat Markdown headings and list items as paragraph boundaries for controlled no-share overclaim detection."
patterns-established:
  - "Claim ladder documents must contain stable tier ids plus allowed claim, blocked claim, and explicit non-claim language."
  - "Prior controlled no-share evidence must be named separately from accepted/rejected share outcome text."
requirements-completed: [EVD-06]
generated_by: gsd-execute-plan
lifecycle_mode: yolo
phase_lifecycle_id: 22-2026-07-04T20-10-36
generated_at: 2026-07-04T20:29:43Z
duration: 5min 14s
completed: 2026-07-04
---

# Phase 22 Plan 01: Claim Ladder And Overclaim Guard Summary

**Operator-facing claim ladder with Rust parity tests that keep Phase 21 controlled no-share evidence from becoming accepted/rejected share proof.**

## Performance

- **Duration:** 5min 14s
- **Started:** 2026-07-04T20:24:29Z
- **Completed:** 2026-07-04T20:29:43Z
- **Tasks:** 2 completed
- **Files modified:** 6

## Accomplishments

- Created the Phase 22 claim ladder with the five required tier ids: `version_1_0_controlled_no_share`, `version_1_1_prerequisite_readiness`, `version_1_1_live_socket_runtime`, `version_1_1_live_asic_share_outcome`, and `explicit_deferred_non_claim`.
- Added `tools/parity/src/claim_ladder.rs` with a stable tier table and `validate_claim_ladder_document` guard for required ids, required claim language, and controlled no-share overclaim paragraphs.
- Wired the guard into the parity Bazel target and validated the committed Markdown fixture with `include_str!`.

## Task Commits

Each task was committed atomically:

1. **Task 1: Write operator claim ladder** - `d4c5718` (`docs`)
2. **Task 2 RED: Add parity claim ladder guard tests** - `2d37c31` (`test`)
3. **Task 2 GREEN: Implement claim ladder guard** - `e6cbe87` (`feat`)

**Plan metadata:** included in the final docs commit for this execution

## Files Created/Modified

- `docs/parity/evidence/phase-22-claim-ladder-and-safety-preconditions/claim-ladder.md` - Operator-visible claim tiers, blocked claims, explicit non-claims, and promotion rules.
- `tools/parity/src/claim_ladder.rs` - Claim tier source of truth, Markdown validator, and focused unit tests.
- `tools/parity/src/main.rs` - Adds the `claim_ladder` module declaration.
- `tools/parity/BUILD.bazel` - Adds the Rust source and Markdown fixture compile data.
- `BUILD.bazel` - Exports the claim ladder Markdown for Bazel fixture access.

## Decisions Made

- Kept claim ladder validation as pure parity-tooling logic with focused tests and no CLI subcommand, matching the plan boundary.
- Used compile-time fixture validation for the committed Markdown so later edits that remove tier ids or required claim language fail `//tools/parity:tests`.
- Split controlled no-share source text away from accepted/rejected share terms in the doc so the new guard distinguishes non-claim wording from overclaim wording.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] Exported the Markdown fixture for Bazel**
- **Found during:** Task 2 RED
- **Issue:** `bazel test //tools/parity:tests` failed during analysis because `include_str!` needed the claim ladder Markdown declared as a Bazel input.
- **Fix:** Added the Markdown file to root `BUILD.bazel` `exports_files` and referenced it from `tools/parity/BUILD.bazel` `compile_data`.
- **Files modified:** `BUILD.bazel`, `tools/parity/BUILD.bazel`
- **Verification:** RED rerun reached the intended missing-function failure, then GREEN passed `bazel test //tools/parity:tests`.
- **Committed in:** `2d37c31`

**2. [Rule 1 - Bug] Split controlled no-share wording from share-outcome terms**
- **Found during:** Task 2 GREEN
- **Issue:** The Task 1 document put `approved_controlled_no_share_soak` in the same paragraph as accepted/rejected share terms, which correctly triggered the new overclaim guard.
- **Fix:** Reworded the blocked-claim and promotion-rule paragraphs so Phase 21 closure is named separately from the live share outcomes it does not prove.
- **Files modified:** `docs/parity/evidence/phase-22-claim-ladder-and-safety-preconditions/claim-ladder.md`
- **Verification:** `validate_claim_ladder_document(include_str!(...))` passes in `bazel test //tools/parity:tests`.
- **Committed in:** `e6cbe87`

**Total deviations:** 2 auto-fixed (1 blocking, 1 bug)
**Impact on plan:** Both fixes preserve the plan objective and make the test-enforced guard executable under Bazel.

## Issues Encountered

- Initial RED run failed before compiling tests because the Markdown fixture was not exported to Bazel. The fixture export resolved the blocker.
- `gsd-tools state record-metric` returned `recorded: false` because `.planning/STATE.md` has no Performance Metrics section; this summary records the plan timing and file/task counts.
- No authentication gates occurred.

## Known Stubs

None.

## Threat Flags

None - the planned evidence-doc-to-parity-guard trust boundary is covered by T-22-01 and T-22-02, and no new network endpoint, auth path, schema, or runtime file-access surface was added.

## Verification

- `rg "version_1_0_controlled_no_share|version_1_1_prerequisite_readiness|version_1_1_live_socket_runtime|version_1_1_live_asic_share_outcome|explicit_deferred_non_claim|approved_controlled_no_share_soak" docs/parity/evidence/phase-22-claim-ladder-and-safety-preconditions/claim-ladder.md`
- `rg "Allowed Claims|Blocked Claims|Explicit Non-Claims|Promotion Rules" docs/parity/evidence/phase-22-claim-ladder-and-safety-preconditions/claim-ladder.md`
- `rg "accepted shares|rejected shares|unbounded production mining|full active safety closure|non-205 board support|Stratum v2|OTA/recovery trust|runtime display/input parity|BAP behavior" docs/parity/evidence/phase-22-claim-ladder-and-safety-preconditions/claim-ladder.md`
- `bazel test //tools/parity:tests`
- `just parity`
- `just verify-reference`
- `git diff --check`

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

Plan 22-02 can consume the claim ladder language and parity guard while adding typed production-mining preconditions and exact blocker propagation. EVD-06 is ready to be cited as completed for the documented claim-ladder portion of Phase 22.

## Self-Check: PASSED

- Found created files: `claim-ladder.md`, `claim_ladder.rs`, and `22-01-SUMMARY.md`.
- Found task commits: `d4c5718`, `2d37c31`, and `e6cbe87`.

*Phase: 22-claim-ladder-and-safety-preconditions*
*Completed: 2026-07-04*
