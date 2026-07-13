---
phase: 19-recovery-regression-and-otawww-evidence
plan: "02"
subsystem: evidence
tags:
  - recovery-regression
  - OTAWWW
  - release-gate
  - ultra-205
  - hardware-smoke
requires:
  - phase: 19-recovery-regression-and-otawww-evidence
    provides: Phase 19 Wave 0 evidence contract and wrapper
  - phase: 18-firmware-ota-and-rollback-evidence
    provides: Firmware OTA evidence and redaction pattern
provides:
  - Current package and release-gate evidence for Phase 19 hardware work
  - Detector-gated Ultra 205 serial flash-monitor evidence
  - Redacted committed serial artifacts
  - Explicit no-scan blocked target lock
affects:
  - phase-19-recovery-regression-and-otawww-evidence
  - release-evidence
  - recovery-regression
  - OTAWWW
tech-stack:
  added: []
  patterns:
    - Manifest-backed release gate before hardware evidence
    - Detector-gated flash-monitor with commit-redacted artifacts
    - Blocked target lock when no raw origin-only target evidence exists
key-files:
  created:
    - docs/parity/evidence/phase-19-recovery-regression-and-otawww-evidence/package-release-gate.md
    - docs/parity/evidence/phase-19-recovery-regression-and-otawww-evidence/package-release-gate/bitaxe-ultra205-package.json
    - docs/parity/evidence/phase-19-recovery-regression-and-otawww-evidence/package-release-gate/package-command.log
    - docs/parity/evidence/phase-19-recovery-regression-and-otawww-evidence/package-release-gate/release-gate.log
    - docs/parity/evidence/phase-19-recovery-regression-and-otawww-evidence/serial-boot.md
    - docs/parity/evidence/phase-19-recovery-regression-and-otawww-evidence/serial-boot/detect-ultra205.log
    - docs/parity/evidence/phase-19-recovery-regression-and-otawww-evidence/serial-boot/flash-command-evidence.json
    - docs/parity/evidence/phase-19-recovery-regression-and-otawww-evidence/serial-boot/flash-monitor.log
    - docs/parity/evidence/phase-19-recovery-regression-and-otawww-evidence/target-lock.json
  modified:
    - docs/parity/evidence/phase-19-recovery-regression-and-otawww-evidence/package-release-gate.md
    - docs/parity/evidence/phase-19-recovery-regression-and-otawww-evidence/package-release-gate/bitaxe-ultra205-package.json
    - docs/parity/evidence/phase-19-recovery-regression-and-otawww-evidence/package-release-gate/package-command.log
    - docs/parity/evidence/phase-19-recovery-regression-and-otawww-evidence/package-release-gate/release-gate.log
key-decisions:
  - "Refresh package/release-gate evidence after the Task 1 evidence commit advanced HEAD so flash-monitor evidence aligns with the copied package manifest."
  - "Commit only redacted serial evidence; without raw origin-only target evidence under target/, target-lock.json remains blocked with network_scan disabled."
  - "Treat www.bin as package/static asset evidence only, not whole-www OTAWWW update proof."
patterns-established:
  - "Evidence commits may advance HEAD after package capture; hardware tasks must refresh package identity before flashing when the firmware source commit would otherwise drift."
  - "Redacted flash-monitor logs can prove serial boot identity but cannot be used to infer a DEVICE_URL."
requirements-completed:
  - REL-07
  - REL-08
  - API-09
  - EVD-05
generated_by: gsd-execute-plan
lifecycle_mode: yolo
phase_lifecycle_id: 19-2026-07-03T17-34-52
generated_at: 2026-07-03T18:45:10Z
duration: 13 min
completed: 2026-07-03
---

# Phase 19 Plan 02: Package Release Gate And Serial Evidence Summary

**Manifest-backed Ultra 205 package identity with detector-gated redacted flash-monitor evidence and a no-scan blocked target lock**

## Performance

- **Duration:** 13 min
- **Started:** 2026-07-03T18:32:42Z
- **Completed:** 2026-07-03T18:45:10Z
- **Tasks:** 2
- **Files modified:** 9

## Accomplishments

- Captured Phase 19 package evidence, copied `bitaxe-ultra205-package.json`, and validated it with the release gate.
- Ran `just detect-ultra205`, confirmed a single detector-approved ESP32-S3 Ultra 205 port, and captured wrapper-owned flash-monitor evidence.
- Committed only redacted serial artifacts while preserving board `205`, source/reference commits, trusted output, and exact command evidence.
- Wrote `target-lock.json` as blocked with `network_scan: disabled` because no raw origin-only target evidence exists under `target/`.

## Task Commits

Each task was committed atomically:

1. **Task 1: Capture current package and manifest-backed release-gate evidence** - `6842d7a` (docs)
1. **Task 2: Capture detector-gated serial identity and sanitized target provenance** - `347b746` (docs)

## Files Created/Modified

- `docs/parity/evidence/phase-19-recovery-regression-and-otawww-evidence/package-release-gate.md` - Package/release-gate ledger and OTAWWW non-claim boundary.
- `docs/parity/evidence/phase-19-recovery-regression-and-otawww-evidence/package-release-gate/bitaxe-ultra205-package.json` - Copied package manifest for the source commit flashed in Task 2.
- `docs/parity/evidence/phase-19-recovery-regression-and-otawww-evidence/package-release-gate/package-command.log` - Package command output.
- `docs/parity/evidence/phase-19-recovery-regression-and-otawww-evidence/package-release-gate/release-gate.log` - Release-gate output.
- `docs/parity/evidence/phase-19-recovery-regression-and-otawww-evidence/serial-boot.md` - Detector, board-info, flash-monitor, redaction, and non-claim ledger.
- `docs/parity/evidence/phase-19-recovery-regression-and-otawww-evidence/serial-boot/detect-ultra205.log` - Redacted detector and board-info log.
- `docs/parity/evidence/phase-19-recovery-regression-and-otawww-evidence/serial-boot/flash-command-evidence.json` - Commit-redacted flash-monitor evidence JSON.
- `docs/parity/evidence/phase-19-recovery-regression-and-otawww-evidence/serial-boot/flash-monitor.log` - Commit-redacted serial boot log.
- `docs/parity/evidence/phase-19-recovery-regression-and-otawww-evidence/target-lock.json` - Blocked no-scan target provenance.

## Decisions Made

- Refreshed package identity before flash-monitor after the Task 1 evidence commit advanced `HEAD`; otherwise the copied manifest would not have matched the flashed firmware commit.
- Kept `target-lock.json` blocked because the only device URL observation was in commit-redacted serial output, and the plan forbids inferring targets from redacted logs.
- Kept failed-update, large erase, interrupted update, rollback, and whole-www OTAWWW as explicit non-claims in this plan.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Refreshed package identity after evidence commit drift**

- **Found during:** Task 2 (Capture detector-gated serial identity and sanitized target provenance)
- **Issue:** Task 1's evidence commit advanced `HEAD`, while flash-monitor would observe firmware built from the new commit. The copied manifest still pointed at the pre-Task 1 source commit.
- **Fix:** Re-ran `just package`, recopied `bitaxe-ultra205-package.json`, re-ran the release gate, and updated `package-release-gate.md` before flash-monitor.
- **Files modified:** `package-release-gate.md`, copied manifest, `package-command.log`, `release-gate.log`
- **Verification:** Refreshed manifest source commit equaled `git rev-parse HEAD` before flash-monitor; release gate passed; flash evidence observed firmware commit `6842d7a6d3d4`.
- **Committed in:** `347b746`

**2. [Rule 2 - Missing Critical] Redacted detector MAC before commit**

- **Found during:** Task 2 (Capture detector-gated serial identity and sanitized target provenance)
- **Issue:** `just detect-ultra205` records board-info output that includes a MAC address; committed/shareable evidence must redact private hardware identifiers.
- **Fix:** Replaced the detector MAC value with `[redacted-mac]` while preserving the board-info command, chip facts, and `port=` evidence.
- **Files modified:** `serial-boot/detect-ultra205.log`
- **Verification:** Targeted redaction scan found only allowed redacted placeholders, labels, NVS metadata, Wi-Fi driver labels, and ESP-IDF memory pool text.
- **Committed in:** `347b746`

**Total deviations:** 2 auto-fixed (1 bug, 1 missing critical)
**Impact on plan:** Both fixes preserved the plan's evidence chain and redaction requirements without expanding recovery, destructive, or OTAWWW scope.

## Issues Encountered

- Flash-monitor capture ended with `timed_out_after_trusted_output`, which is accepted by the wrapper after trusted boot markers are captured.
- No raw origin-only target evidence existed under `target/phase19-recovery-regression-and-otawww-evidence-dev-raw/serial-boot/flash-command-evidence.json`; target provenance was therefore blocked without network scanning.

## Verification

- `just package` passed.
- `bazel run //tools/parity:report -- release-gate --manifest docs/parity/evidence/phase-19-recovery-regression-and-otawww-evidence/package-release-gate/bitaxe-ultra205-package.json` passed with `release_gate: passed`.
- Package identity Python checks passed before Task 1 and after the Task 2 refresh.
- `rg -n "detector_status: (passed|blocked)|flash_monitor_status: (passed|not run|blocked)|board: 205|network_scan: disabled|failed-update.*non-claim|OTAWWW.*non-claim" ...` passed.
- `python3` target-lock redaction assertion passed with `network_scan == "disabled"`.
- `mdformat --check` passed for `package-release-gate.md` and `serial-boot.md`.
- `cargo fmt --all`, `cargo clippy --all-targets --all-features -- -D warnings`, `cargo build --all-targets --all-features`, and `cargo test --all-features` passed before both task commits.
- `just parity` passed with `validation_errors: none`.
- `just verify-reference` passed with reference commit `c1915b0a63bfabebdb95a515cedfee05146c1d50`.
- `node /Users/peterryszkiewicz/.codex/get-shit-done/bin/gsd-tools.cjs verify lifecycle 19 --expect-id 19-2026-07-03T17-34-52 --expect-mode yolo --raw` returned `valid`.

## Known Stubs

None - the blocked target lock is intentional evidence state, not a placeholder implementation.

## Threat Flags

None - package manifest evidence, detector-gated hardware action, redacted flash-monitor artifacts, and blocked target provenance are covered by the plan threat model.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

Ready for `19-03-PLAN.md`. Phase 19 now has current package/release-gate evidence, detector-gated serial evidence, and an explicit no-scan target boundary for recovery-regression planning.

*Phase: 19-recovery-regression-and-otawww-evidence*
*Completed: 2026-07-03*

## Self-Check: PASSED

- Expected evidence and summary files exist.
- Task commits `6842d7a` and `347b746` exist in git history.
- Summary frontmatter delimiter check passed with exactly two standalone delimiters at the top of the file.
