---
phase: 15-bm1366-mining-evidence-completion
plan: "05"
subsystem: parity-evidence
tags:
  - bm1366
  - mining
  - evidence-ledger
  - redaction
  - lifecycle

requires:
  - phase: 15-04
    provides: controlled no-share mining smoke and unsupported-pending bounded soak evidence
provides:
  - Final Phase 15 evidence ledger with exact claim boundaries
  - Redaction-reviewed citation closure for all cited Phase 15 artifacts
  - Conservative checklist and Nyquist validation updates
  - Final verification report with targeted checks, aggregate checks, reference gates, lifecycle validation, and hardware command inventory
affects:
  - phase-15-final-ledger
  - parity-checklist
  - bm1366-mining-evidence
  - phase-verification

tech-stack:
  added: []
  patterns:
    - Exact evidence subclaims are promoted only when the final ledger, redaction review, and parity guard agree.
    - Missing live pool or DEVICE_URL prerequisites remain optional blockers rather than failed core evidence gates.
    - Verification reports carry lifecycle provenance fields required by the GSD lifecycle validator.

key-files:
  created:
    - docs/parity/evidence/phase-15-bm1366-mining-evidence-completion.md
    - .planning/phases/15-bm1366-mining-evidence-completion/15-VERIFICATION.md
    - .planning/phases/15-bm1366-mining-evidence-completion/15-05-SUMMARY.md
  modified:
    - docs/parity/evidence/phase-15-bm1366-mining-evidence-completion/redaction-review.md
    - docs/parity/checklist.md
    - .planning/phases/15-bm1366-mining-evidence-completion/15-VALIDATION.md

key-decisions:
  - "Phase 15 supports exact BM1366 diagnostic and controlled no-share subclaims only; broad production mining, live pool, API/WebSocket, statistics, frequency, voltage, fan, and release claims remain below verified."
  - "STR-008 was updated to implemented hardware-smoke/workflow evidence for controlled no-share metadata, while accepted/rejected shares and live bounded soak remain below verified."
  - "Missing explicit DEVICE_URL and live pool prerequisites are the only residual optional blockers accepted for passed verification status."

patterns-established:
  - "Final evidence ledgers must name each checklist row, requirement, evidence class, blocker, and non-claim before checklist citation updates."
  - "Lifecycle verification artifacts must include generated_by and lifecycle_validated frontmatter before `verify lifecycle --require-verification` can pass."

requirements-completed:
  - ASIC-07
  - STR-06
  - STR-07
  - SAFE-09
  - EVD-05
generated_by: gsd-execute-plan
lifecycle_mode: yolo
phase_lifecycle_id: 15-2026-07-01T02-07-59
generated_at: 2026-07-01T04:52:40Z

duration: 15m40s
completed: 2026-07-01
---

# Phase 15 Plan 05: Final Evidence Closure Summary

**Final Phase 15 BM1366 evidence ledger, conservative checklist closure, and passed lifecycle verification with optional live-pool and DEVICE_URL boundaries**

## Performance

- **Duration:** 15m40s
- **Started:** 2026-07-01T04:37:00Z
- **Completed:** 2026-07-01T04:52:40Z
- **Tasks:** 3
- **Files modified:** 6

## Accomplishments

- Created the final Phase 15 evidence ledger with exact claim classes, below-verified blockers, residual risks, and non-claims.
- Completed redaction review for cited artifacts while keeping absent API/WebSocket/live-pool artifacts uncited.
- Updated the parity checklist and Nyquist validation without unsupported verified promotions.
- Created `15-VERIFICATION.md` with command summaries, hardware command inventory, optional blockers, and lifecycle status `passed`.

## Task Commits

Each task was committed atomically:

1. **Task 1: Complete redaction review and final Phase 15 ledger** - `43c01aa` (`docs`)
2. **Task 2: Update checklist and Nyquist validation from exact Phase 15 evidence** - `cbf9f42` (`docs`)
3. **Task 3: Run final Phase 15 verification and lifecycle validation** - `adf08c8` (`docs`)

## Files Created/Modified

- `docs/parity/evidence/phase-15-bm1366-mining-evidence-completion.md` - Final evidence ledger and claim matrix for Phase 15.
- `docs/parity/evidence/phase-15-bm1366-mining-evidence-completion/redaction-review.md` - Marks cited Phase 15 artifacts passed or uncited according to actual artifact presence.
- `docs/parity/checklist.md` - Adds conservative Phase 15 citations for BM1366, mining, API/WebSocket, and statistics rows.
- `.planning/phases/15-bm1366-mining-evidence-completion/15-VALIDATION.md` - Marks Wave 0 validation complete with evidence boundaries and final verification closure.
- `.planning/phases/15-bm1366-mining-evidence-completion/15-VERIFICATION.md` - Records final command results, hardware commands, core gates, optional blockers, and passed lifecycle validation.
- `.planning/phases/15-bm1366-mining-evidence-completion/15-05-SUMMARY.md` - This plan summary.

## Decisions Made

- Exact chip-detect evidence supports only chip-detect/no-mining fail-closed behavior, not full BM1366 initialization.
- Exact work/result evidence supports only typed diagnostic dispatch and bounded no-result/fail-closed handling, not production work or valid live nonce/share behavior.
- Controlled no-share evidence supports mining evidence governance and safe blocking, not live pool mining or accepted/rejected share behavior.
- `15-VERIFICATION.md` includes `generated_by` and `lifecycle_validated: true` because the lifecycle validator requires those fields for verification artifacts.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] Split invalid Cargo multi-filter verification commands**
- **Found during:** Task 3 (Run final Phase 15 verification and lifecycle validation)
- **Issue:** The plan listed Cargo commands with multiple test filters in one invocation, but Cargo accepts one trailing test filter per command.
- **Fix:** Ran the intended ASIC filters separately (`adapter_gate`, `work`, `result`, `transcript`) and Stratum filters separately (`mining_loop`, `fake_pool`, `queue`).
- **Files modified:** `.planning/phases/15-bm1366-mining-evidence-completion/15-VERIFICATION.md`
- **Verification:** All seven equivalent filtered commands passed and are recorded in `15-VERIFICATION.md`.
- **Committed in:** `adf08c8`

**2. [Rule 3 - Blocking] Added lifecycle-required verification metadata**
- **Found during:** Task 3 (Run final Phase 15 verification and lifecycle validation)
- **Issue:** Lifecycle validation rejected the draft verification report until `generated_by` and `lifecycle_validated` frontmatter were present.
- **Fix:** Added `generated_by: gsd-execute-plan` and `lifecycle_validated: true` to `15-VERIFICATION.md`, then reran the required lifecycle command.
- **Files modified:** `.planning/phases/15-bm1366-mining-evidence-completion/15-VERIFICATION.md`
- **Verification:** `node /Users/peterryszkiewicz/.codex/get-shit-done/bin/gsd-tools.cjs verify lifecycle 15 --expect-id 15-2026-07-01T02-07-59 --expect-mode yolo --require-plans --require-verification --raw` returned `valid`.
- **Committed in:** `adf08c8`

**Total deviations:** 2 auto-fixed (2 blocking)
**Impact on plan:** Both deviations were process-level fixes needed to run the planned verification accurately. No evidence claims were broadened.

## Issues Encountered

- Early inspection commands hit shell quoting pitfalls: a zsh loop variable named `path` shadowed command lookup, and backticks inside a double-quoted `rg` pattern caused shell substitution. Both checks were rerun with safe variable names and quoting.
- Initial lifecycle validation of the draft verification report returned `invalid` until verification-specific frontmatter metadata was added.

## Auth Gates

None. Missing live pool variables and missing explicit `DEVICE_URL` are accepted optional evidence boundaries for this plan, not authentication gates.

## Known Stubs

None. Stub-pattern scan across created and modified files found no TODO, FIXME, placeholder, coming-soon, or hardcoded empty UI/data stubs.

## Verification

- `test -f docs/parity/evidence/phase-15-bm1366-mining-evidence-completion.md && rg -n "Phase 15 BM1366 Mining Evidence Completion|ASIC-002|ASIC-003|ASIC-004|STR-006|STR-007|STR-008|SAFE-09|EVD-05|redaction|below verified|hardware evidence pending|non-claims" docs/parity/evidence/phase-15-bm1366-mining-evidence-completion.md docs/parity/evidence/phase-15-bm1366-mining-evidence-completion/redaction-review.md`
- `just parity`
- `bash -n scripts/phase15-*.sh`
- `bazel test //scripts:phase15_bm1366_diagnostic_package_test //scripts:phase15_controlled_mining_test`
- `cargo test -p bitaxe-asic --all-features adapter_gate`
- `cargo test -p bitaxe-asic --all-features work`
- `cargo test -p bitaxe-asic --all-features result`
- `cargo test -p bitaxe-asic --all-features transcript`
- `cargo test -p bitaxe-stratum --all-features mining_loop`
- `cargo test -p bitaxe-stratum --all-features fake_pool`
- `cargo test -p bitaxe-stratum --all-features queue`
- `cargo test -p bitaxe-api --all-features mining`
- `cargo test -p bitaxe-parity --all-features mining_allow`
- `cargo fmt --all`
- `cargo clippy --all-targets --all-features -- -D warnings`
- `cargo build --all-targets --all-features`
- `cargo test --all-features`
- `just test`
- `just verify-reference`
- `git diff -- reference/esp-miner --exit-code`
- `node /Users/peterryszkiewicz/.codex/get-shit-done/bin/gsd-tools.cjs verify lifecycle 15 --expect-id 15-2026-07-01T02-07-59 --expect-mode yolo --require-plans --require-verification --raw`

## User Setup Required

None for plan completion. Future live pool or API/WebSocket promotion requires an explicit `DEVICE_URL` and disposable/non-secret live pool prerequisites, with new redaction-reviewed artifacts.

## Next Phase Readiness

Phase 15 is closed with passed lifecycle verification. Future phases can rely on exact diagnostic and controlled no-share evidence, but must keep production mining, accepted/rejected shares, live pool behavior, live API/WebSocket/statistics, frequency, voltage, fan, and release claims below verified until new hardware-regression evidence exists.

*Phase: 15-bm1366-mining-evidence-completion*
*Completed: 2026-07-01*

## Self-Check: PASSED

- Created files exist: final ledger, `15-VERIFICATION.md`, and this summary.
- Task commits `43c01aa`, `cbf9f42`, and `adf08c8` exist in git history.
- Summary frontmatter uses only the opening and closing standalone delimiter lines.
- Stub scan found no actionable stubs; the only match was this summary's "Known Stubs" statement.
