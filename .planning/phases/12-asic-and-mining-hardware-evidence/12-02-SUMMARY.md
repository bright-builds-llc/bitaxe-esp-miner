---
phase: 12
plan: 02
subsystem: parity-tooling
tags: [parity, checklist, asic, mining, rust]
requires:
  - phase: 12
    provides: "Phase 12 evidence contract and promotion boundaries from Plan 12-01"
provides:
  - "Parity validation for verified live ASIC/mining checklist rows"
  - "Tests covering ASIC-002, ASIC-003, ASIC-004, ASIC-005, STR-006, STR-007, and STR-008 promotion semantics"
affects: [tools-parity, docs-parity-checklist, release-gate]
tech-stack:
  added: []
  patterns:
    - "Verified-row guards are pure checks over parsed checklist rows"
key-files:
  created: []
  modified:
    - tools/parity/src/main.rs
key-decisions:
  - "Live ASIC/mining rows require hardware-smoke, hardware-regression, or soak evidence before verified status is valid."
  - "STR-008 additionally requires share/no-share outcome and board/port/commit/redaction/conclusion metadata."
patterns-established:
  - "Rows below verified remain allowed so pending hardware evidence does not block current reports."
requirements-completed: [ASIC-07, STR-06, STR-07, EVD-05]
generated_by: gsd-execute-plan
lifecycle_mode: yolo
phase_lifecycle_id: 12-2026-06-30T00-13-19
generated_at: 2026-06-30T01:11:44Z
duration: 4 min
completed: 2026-06-30
---

# Phase 12 Plan 02: ASIC/Mining Parity Guard Summary

**Parity validator rejects verified live ASIC and mining claims unless they carry exact hardware or soak evidence**

## Performance

- **Duration:** 4 min
- **Started:** 2026-06-30T01:07:45Z
- **Completed:** 2026-06-30T01:11:44Z
- **Tasks:** 2
- **Files modified:** 1

## Accomplishments

- Added a live ASIC/mining verified-row validator for `ASIC-002`, `ASIC-003`, `ASIC-004`, `ASIC-005`, `STR-006`, and `STR-008`.
- Added `STR-008` metadata validation requiring accepted share, rejected share, or controlled no-share condition plus board, port, firmware commit, reference commit, redaction, and conclusion details.
- Added focused tests proving weak verified ASIC/mining claims are rejected while `STR-007` criteria documentation below verified remains allowed.

## Task Commits

1. **Task 1: Add failing tests for ASIC/mining verified-row overclaims** - covered by `5bdd0c8` after green implementation, because failing Rust commits are prohibited by repo rules.
2. **Task 2: Implement ASIC/mining verified-row validation** - `5bdd0c8`

## Files Created/Modified

- `tools/parity/src/main.rs` - Adds live ASIC/mining verified-row checks and tests.

## Decisions Made

The validation accepts `hardware-smoke`, `hardware-regression`, or `soak` as sufficient evidence tokens for live ASIC/mining verified rows, with stricter `STR-008` notes metadata.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] Did not commit the RED-only failing test state**
- **Found during:** Task 1
- **Issue:** The TDD task asked for failing tests, but repo Rust rules require passing format, clippy, build, and tests before any commit.
- **Fix:** Ran the RED test locally as part of implementation flow, then committed tests and validator together after green verification.
- **Files modified:** `tools/parity/src/main.rs`
- **Verification:** `cargo test -p bitaxe-parity --all-features asic_mining_verified`, `cargo test -p bitaxe-parity --all-features safety_critical`, `just parity`, and the full Rust pre-commit sequence all passed.
- **Committed in:** `5bdd0c8`

**Total deviations:** 1 auto-fixed (blocking)
**Impact on plan:** The final behavior matches the plan while honoring the repo's stricter commit rule.

## Issues Encountered

None.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

Ready for `12-03`: hardware capture can now rely on parity tooling to reject unsupported ASIC/mining promotions.

*Phase: 12-asic-and-mining-hardware-evidence*
*Completed: 2026-06-30*
