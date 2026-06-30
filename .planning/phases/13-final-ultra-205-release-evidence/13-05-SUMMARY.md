______________________________________________________________________

phase: 13-final-ultra-205-release-evidence
plan: "05"
subsystem: release-evidence
tags: [ultra-205, recovery, rollback, large-erase, interrupted-update, evidence, redaction]
requires:

- phase: 13-final-ultra-205-release-evidence
  provides: Plan 13-03 HTTP/static/recovery helper and missing DEVICE_URL blocker evidence
  provides:
- Phase 13 recovery runbook with exact stop conditions and restore commands
- Repo-owned bounded monitor capture helper and recovery regression helper
- Bazel shell tests for monitor safety, pending defaults, failed-update fields, and command rendering
- Pending REL-08 recovery/destructive evidence with redaction review
  affects: [phase-13, release-evidence, ota-recovery, parity-checklist, release-docs]
  tech-stack:
  added: []
  patterns:
  - Destructive recovery helpers default to pending unless explicit allow flags are provided
  - Bounded monitor capture wraps espflash monitor without serial writes or raw flash writes
  - Missing DEVICE_URL produces pending evidence instead of live HTTP or destructive shortcuts
    key-files:
    created:
  - docs/parity/evidence/phase-13-final-ultra-205-release-evidence/recovery-runbook.md
  - docs/parity/evidence/phase-13-final-ultra-205-release-evidence/recovery-regression.md
  - docs/parity/evidence/phase-13-final-ultra-205-release-evidence/recovery-regression/recovery-regression.log
  - docs/parity/evidence/phase-13-final-ultra-205-release-evidence/recovery-regression/large-erase.log
  - docs/parity/evidence/phase-13-final-ultra-205-release-evidence/recovery-regression/large-erase-post-restore-monitor.log
  - docs/parity/evidence/phase-13-final-ultra-205-release-evidence/recovery-regression/interrupted-ota.log
  - scripts/phase13-monitor-capture.sh
  - scripts/phase13-monitor-capture-test.sh
  - scripts/phase13-recovery-regression.sh
  - scripts/phase13-recovery-regression-test.sh
    modified:
  - docs/parity/evidence/phase-13-final-ultra-205-release-evidence/redaction-review.md
  - scripts/BUILD.bazel
  - scripts/phase13-recovery-regression.sh
  - scripts/phase13-recovery-regression-test.sh
    key-decisions:
- "Recovery, failed-update, large-erase, interrupted-update, rollback, and boot-validation evidence remain pending when DEVICE_URL is missing and allow flags are absent."
- "Plan 13-05 helpers record exact command shapes but do not execute destructive or fault-injection actions without explicit allow flags."
- "OTAWWW remains the REL-03 gap with expected public response Wrong API input until whole-www interrupted-update hardware-regression evidence exists."
  patterns-established:
- "Generated evidence artifacts must carry their own pending/blocking reason, even when a sibling summary log already records the gate."
- "Recovery evidence summaries cite pending fields explicitly instead of omitting unrun observations."
  requirements-completed: [REL-08, REL-02, REL-01, EVD-05]
  generated_by: gsd-execute-plan
  lifecycle_mode: yolo
  phase_lifecycle_id: 13-2026-06-30T14-53-46
  generated_at: 2026-06-30T17:09:16Z
  duration: 12 min
  completed: 2026-06-30

______________________________________________________________________

# Phase 13 Plan 05: Recovery Runbook And Pending Regression Evidence Summary

**Recovery runbook and gated helper coverage with conservative pending REL-08 evidence because DEVICE_URL is missing**

## Performance

- **Duration:** 12 min
- **Started:** 2026-06-30T16:57:01Z
- **Completed:** 2026-06-30T17:09:16Z
- **Tasks:** 2
- **Files modified:** 12

## Accomplishments

- Created `recovery-runbook.md` with the current manifest path, factory image, OTA image, stop conditions, exact large erase command, factory restore command, monitor command, failed-update procedure, interrupted-update procedure, OTAWWW gap procedure, and expected artifacts.
- Added `phase13-monitor-capture.sh` and `phase13-recovery-regression.sh` with Bazel shell tests proving bounded monitor safety, pending defaults, failed-update evidence fields, and exact large-erase/restore command rendering.
- Ran the recovery regression helper without unsafe allow flags because `DEVICE_URL` is missing; generated pending failed-update, large-erase, interrupted-update, rollback, and boot-validation evidence.
- Updated the Phase 13 redaction review to include the recovery runbook, evidence summary, and generated pending recovery/destructive logs.

## Task Commits

Each task was committed atomically:

1. **Task 1: Create recovery runbook and destructive evidence helper** - `fe433ed` (feat)
1. **Task 2: Run recovery regression or record pending evidence** - `c9a34d1` (fix)

**Plan metadata:** committed separately after SUMMARY, STATE, ROADMAP, and REQUIREMENTS updates.

## Files Created/Modified

- `docs/parity/evidence/phase-13-final-ultra-205-release-evidence/recovery-runbook.md` - Phase 13 recovery procedure, destructive stop conditions, exact commands, and artifact contract.
- `scripts/phase13-monitor-capture.sh` - Bounded wrapper around `espflash monitor --chip esp32s3 --port <path> --non-interactive`.
- `scripts/phase13-monitor-capture-test.sh` - Fake-espflash tests for monitor command safety, no raw writes, no-reset rendering, and timeout status.
- `scripts/phase13-recovery-regression.sh` - Gated failed-update, large-erase, and interrupted-update helper with pending defaults.
- `scripts/phase13-recovery-regression-test.sh` - Fake-command tests for pending defaults, failed-update evidence fields, and exact erase/restore command rendering.
- `scripts/BUILD.bazel` - Registers `phase13_monitor_capture`, `phase13_monitor_capture_test`, `phase13_recovery_regression`, and `phase13_recovery_regression_test`.
- `docs/parity/evidence/phase-13-final-ultra-205-release-evidence/recovery-regression.md` - Pending recovery/destructive evidence summary.
- `docs/parity/evidence/phase-13-final-ultra-205-release-evidence/recovery-regression/*.log` - Helper-generated pending evidence logs.
- `docs/parity/evidence/phase-13-final-ultra-205-release-evidence/redaction-review.md` - Redaction review for Plan 13-05 recovery artifacts.

## Decisions Made

- Did not run failed-update live HTTP, large erase, or interrupted OTA because `DEVICE_URL` is missing and the corresponding allow flags were intentionally absent.
- Kept rollback and boot-validation status as `pending - Plan 04 OTA evidence not run yet`.
- Kept OTAWWW as the REL-03 gap; `Wrong API input` is recorded as the expected public response, not observed live evidence.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 2 - Missing Critical] Added pending status to the post-restore monitor artifact**

- **Found during:** Task 2 (Run recovery regression or record pending evidence)
- **Issue:** The helper wrote `large_erase_status: pending - allow flag not provided` to the large-erase log, but left `large-erase-post-restore-monitor.log` empty when large erase was not allowed.
- **Fix:** Updated `scripts/phase13-recovery-regression.sh` to write `large_erase_post_restore_monitor_status: pending - allow flag not provided` and `capture_status=pending`, and extended the shell test to assert both lines.
- **Files modified:** `scripts/phase13-recovery-regression.sh`, `scripts/phase13-recovery-regression-test.sh`
- **Verification:** `bash -n scripts/phase13-recovery-regression.sh`, `bash scripts/phase13-recovery-regression-test.sh`, `bazel test //scripts:phase13_recovery_regression_test`, Task 2 evidence grep, `just parity`, reference cleanliness, and the full Rust pre-commit sequence passed.
- **Committed in:** `c9a34d1`

**Total deviations:** 1 auto-fixed (1 missing critical)
**Impact on plan:** The fix improves evidence completeness without widening recovery scope or running unsafe actions.

## Issues Encountered

- `DEVICE_URL` was missing, matching the Plan 13-03 blocker. This is an evidence gate, not a task failure. The helper wrote pending evidence and did not run failed-update live HTTP, large erase, interrupted upload, rollback, raw write, voltage/fan/mining stress, or ad hoc recovery commands.

## Verification

- Lifecycle validation before execution: `verify lifecycle 13 --expect-id 13-2026-06-30T14-53-46 --expect-mode yolo --require-plans --raw`: passed.
- Sync before implementation: `git fetch origin` and `git pull --rebase`: branch already up to date.
- Task 1 checks: `bash -n` for all new shell scripts, direct fake-command tests, `bazel test //scripts:phase13_monitor_capture_test //scripts:phase13_recovery_regression_test`, runbook grep checks, `mdformat --check`, and `shfmt -l -d`: passed.
- Task 2 checks: recovery helper run without unsafe allow flags, required evidence files present, Task 2 grep verification, `mdformat --check`, `bash -n scripts/phase13-recovery-regression.sh`, and `bazel test //scripts:phase13_recovery_regression_test`: passed.
- Plan-level verification: `just parity` passed with `validation_errors: none`; `git diff -- reference/esp-miner --exit-code` passed.
- Rust pre-commit sequence before each task commit: `cargo fmt --all`, `cargo clippy --all-targets --all-features -- -D warnings`, `cargo build --all-targets --all-features`, and `cargo test --all-features`: passed.

## Known Stubs

None - stub scan found only shell variable initializers and fake-command test fixture initializers (`port=""`, `out=""`, `manifest=""`, `body_file=""`, `url=""`, `out_dir=""`). These are not UI-rendered placeholder data or incomplete product behavior.

## Threat Flags

None - the new helper surfaces match the plan threat model: destructive local command, interrupted OTA upload, recovery logs, factory recovery, and OTAWWW fail-closed handling. No unplanned endpoint, auth path, file-access trust boundary, schema change, or runtime firmware trust boundary was introduced.

## User Setup Required

None for local tooling. A reachable `DEVICE_URL` for the just-flashed Ultra 205 remains required before live failed-update, interrupted-update, HTTP/static recovery, OTA, rollback, and boot-validation evidence can pass.

## Next Phase Readiness

Recovery procedures and helper coverage are ready for the valid OTA/recovery evidence path. Plan 13-04 or any later live OTA plan must still provide a reachable `DEVICE_URL` before running live HTTP, OTA, failed-update, or interrupted-update checks.

## Self-Check: PASSED

- Created files exist: `recovery-runbook.md`, `recovery-regression.md`, `phase13-monitor-capture.sh`, `phase13-recovery-regression.sh`, and `13-05-SUMMARY.md`.
- Task commits exist: `fe433ed` and `c9a34d1`.
- SUMMARY frontmatter delimiter check passed with exactly two standalone `---` delimiters.

*Phase: 13-final-ultra-205-release-evidence*
*Completed: 2026-06-30*
