---
phase: 09-flash-monitor-evidence-wrapper-hardening
plan: 01
subsystem: tooling
tags: [rust, espflash, evidence, flash-monitor, bazel]
requires:
  - phase: 08-parity-evidence-and-ultra-205-release-gate
    provides: Phase 8 raw fallback evidence gap and Ultra 205 serial boot markers.
provides:
  - Wrapper-owned noninteractive flash-monitor evidence command.
  - Bounded monitor capture with typed timeout and process result status.
  - Trusted Ultra 205 serial marker classifier and fail-closed recovery guidance.
  - Enriched flash-command-evidence.json capture contract.
affects: [flash-monitor, evidence, release-gate, parity]
tech-stack:
  added: []
  patterns:
    - Functional capture classification around a thin espflash process adapter.
    - Typed evidence JSON fields for flash and monitor command provenance.
key-files:
  created:
    - .planning/phases/09-flash-monitor-evidence-wrapper-hardening/09-01-SUMMARY.md
  modified:
    - tools/flash/src/main.rs
key-decisions:
  - "`flash-monitor --evidence-dir` now uses `espflash monitor --chip esp32s3 --port <port> --non-interactive`; ordinary `monitor` stays interactive."
  - "Evidence is trusted only when all seven serial-scope Ultra 205 boot markers are present."
  - "Monitor timeouts are accepted only after trusted output; untrusted timeout or failed monitor exits write JSON and fail visibly."
  - "TDD RED failures were run but not committed because AGENTS.md requires passing Rust checks before every commit."
patterns-established:
  - "Evidence capture outcome: process status plus trusted marker classification determines JSON status and command success."
  - "Recovery guidance stays on repo commands, not raw espflash fallback commands."
requirements-completed: [FND-07, FND-08, EVD-05]
generated_by: gsd-execute-plan
lifecycle_mode: yolo
phase_lifecycle_id: "09-2026-06-29T13-16-47"
generated_at: 2026-06-29T14:08:40Z
duration: 11 min
completed: 2026-06-29
---

# Phase 09 Plan 01: Flash-Monitor Evidence Wrapper Hardening Summary

**Wrapper-owned noninteractive Ultra 205 flash-monitor evidence capture with bounded timeout, trusted marker classification, enriched JSON, and fail-closed recovery guidance**

## Performance

- **Duration:** 11 min
- **Started:** 2026-06-29T13:57:36Z
- **Completed:** 2026-06-29T14:08:40Z
- **Tasks:** 2
- **Files modified:** 1

## Accomplishments

- Added a separate evidence-mode monitor command builder for `espflash monitor --chip esp32s3 --port <port> --non-interactive`.
- Preserved ordinary `just monitor` as interactive `espflash monitor --port <port>`.
- Added bounded monitor capture through Rust `spawn`/`try_wait`/`kill` instead of shell `timeout`.
- Added trusted serial marker classification requiring all seven Phase 9 Ultra 205 markers.
- Enriched `flash-command-evidence.json` with `flash_command`, `monitor_command`, `monitor_log_path`, `capture_mode`, `capture_status`, `capture_timeout_seconds`, `trusted_output`, and `conclusion`.
- Added fail-closed recovery guidance that points to `just detect-ultra205`, wrapper `just flash-monitor`, and diagnostic `just monitor`.

## Task Commits

Each task was committed atomically:

1. **Task 1: Add evidence monitor command and trusted-marker contracts** - `365cd62` (feat)
2. **Task 2: Integrate bounded capture, enriched JSON, and fail-closed guidance** - `de0501c` (feat)

## Files Created/Modified

- `tools/flash/src/main.rs` - Adds capture timeout parsing, evidence monitor command construction, process capture status, trusted marker classification, enriched evidence JSON, and focused unit tests.
- `.planning/phases/09-flash-monitor-evidence-wrapper-hardening/09-01-SUMMARY.md` - Plan execution summary.

## Decisions Made

- Evidence-mode monitor capture is separate from interactive monitor behavior so automation can be noninteractive without breaking manual operator workflows.
- A timed-out monitor process can pass only when the captured log already has all trusted serial markers.
- The wrapper fails untrusted capture paths after writing JSON, so reviewers can inspect both the failure state and the captured log.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] Did not commit failing TDD RED states**
- **Found during:** Task 1 and Task 2
- **Issue:** The TDD flow calls for RED commits, but AGENTS.md requires `cargo fmt`, `cargo clippy`, `cargo build`, and `cargo test` to pass before every Rust commit.
- **Fix:** Ran the RED failures to prove the tests failed for the expected missing contracts, then committed only the verified task outcomes.
- **Files modified:** `tools/flash/src/main.rs`
- **Verification:** RED failures were observed before implementation; final task verification passed.
- **Committed in:** `365cd62`, `de0501c`

**2. [Rule 3 - Blocking] Kept Task 1 commit clippy-clean across the split-task boundary**
- **Found during:** Task 1 pre-commit verification
- **Issue:** Task 1 introduced helpers that Task 2 was responsible for wiring into production flow, so clippy rejected them as dead code.
- **Fix:** Used a narrow temporary allowance in Task 1, then removed it in Task 2 when the helpers were integrated.
- **Files modified:** `tools/flash/src/main.rs`
- **Verification:** `cargo clippy --all-targets --all-features -- -D warnings` passed before each task commit.
- **Committed in:** `365cd62`, resolved in `de0501c`

**3. [Rule 3 - Blocking] Grouped evidence writer arguments for clippy**
- **Found during:** Task 2 pre-commit verification
- **Issue:** The enriched evidence writer initially exceeded clippy's argument-count limit.
- **Fix:** Added `EvidenceRecordInput` to group record-specific fields.
- **Files modified:** `tools/flash/src/main.rs`
- **Verification:** `cargo clippy --all-targets --all-features -- -D warnings` passed.
- **Committed in:** `de0501c`

**Total deviations:** 3 auto-fixed (3 blocking)
**Impact on plan:** All fixes were required to satisfy repo verification and keep the task commits atomic. No scope beyond the wrapper hardening plan was added.

## Issues Encountered

None beyond the auto-fixed verification blockers above.

## Authentication Gates

None.

## Known Stubs

None.

## Verification

- `bazel test //tools/flash:tests` passed after Task 1.
- Task 1 fixed-string acceptance checks passed.
- `cargo fmt --all` passed before each task commit.
- `cargo clippy --all-targets --all-features -- -D warnings` passed before each task commit.
- `cargo build --all-targets --all-features` passed before each task commit.
- `cargo test --all-features` passed before each task commit.
- `bazel test //tools/flash:tests` plus Task 2 fixed-string acceptance checks passed.
- Final `bazel test //tools/flash:tests` passed.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

Plan 09-02 can use the wrapper-owned JSON/log evidence path to capture fresh Ultra 205 hardware evidence and update docs/checklist without relying on raw monitor fallback commands.

*Phase: 09-flash-monitor-evidence-wrapper-hardening*
*Completed: 2026-06-29*

## Self-Check: PASSED

- Found summary file: `.planning/phases/09-flash-monitor-evidence-wrapper-hardening/09-01-SUMMARY.md`
- Found task commit: `365cd62`
- Found task commit: `de0501c`
