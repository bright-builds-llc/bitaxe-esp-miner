---
phase: 20-active-safety-hardware-telemetry-evidence
plan: "02"
subsystem: parity-evidence
tags: [safe-baseline, package-release-gate, hardware-evidence, redaction, target-lock]

requires:
  - phase: 20-active-safety-hardware-telemetry-evidence
    provides: Plan 20-01 evidence scaffold, redaction contract, and safety allow context.
  - phase: 19-recovery-regression-and-otawww-evidence
    provides: Prior package-release-gate and detector-gated hardware evidence patterns.
provides:
  - Phase 20 package and release-gate evidence tied to the package-time source commit.
  - Detector-gated Ultra 205 safe-baseline evidence with trusted flash-monitor output.
  - Explicit no-scan target lock that blocks device origin inference from redacted serial logs.
affects: [phase-20, parity-evidence, safe-baseline, active-safety, live-telemetry]

tech-stack:
  added: []
  patterns:
    - Package evidence records source identity at package time because committed manifests cannot contain their own future commit hash.
    - Safe-baseline evidence starts with detector and board-info gates before flash-monitor output is trusted.
    - Target provenance remains blocked unless a trusted raw origin-only target artifact exists.

key-files:
  created:
    - docs/parity/evidence/phase-20-active-safety-hardware-telemetry-evidence/safe-baseline.md
    - docs/parity/evidence/phase-20-active-safety-hardware-telemetry-evidence/safe-baseline/detect-ultra205.log
    - docs/parity/evidence/phase-20-active-safety-hardware-telemetry-evidence/safe-baseline/flash-command-evidence.json
    - docs/parity/evidence/phase-20-active-safety-hardware-telemetry-evidence/safe-baseline/flash-monitor.log
    - docs/parity/evidence/phase-20-active-safety-hardware-telemetry-evidence/target-lock.json
  modified:
    - docs/parity/evidence/phase-20-active-safety-hardware-telemetry-evidence/package-release-gate.md
    - docs/parity/evidence/phase-20-active-safety-hardware-telemetry-evidence/package-release-gate/bitaxe-ultra205-package.json
    - docs/parity/evidence/phase-20-active-safety-hardware-telemetry-evidence/package-release-gate/package-command.log
    - docs/parity/evidence/phase-20-active-safety-hardware-telemetry-evidence/package-release-gate/release-gate.log

key-decisions:
  - "Phase 20 safe-baseline evidence uses detector plus board-info plus wrapper-owned flash-monitor output before downstream active/live packs consume serial logs."
  - "Target provenance remains blocked with `network_scan: disabled`; no `DEVICE_URL` is inferred from committed redacted serial output."
  - "Package and release-gate artifacts were refreshed during Task 2 so the copied manifest and safe-baseline flash evidence agree on source commit `c11fba2`."

patterns-established:
  - "Hardware evidence ledgers cite package-time source commits and explain the package-time convention explicitly."
  - "Redacted serial output can prove boot and safe-state markers but cannot establish a trusted live HTTP/WebSocket origin."

requirements-completed: [SAFE-01, SAFE-02, SAFE-07, SAFE-08, SAFE-09, EVD-05]
generated_by: gsd-execute-plan
lifecycle_mode: yolo
phase_lifecycle_id: 20-2026-07-03T20-48-00
generated_at: 2026-07-03T22:20:34Z

duration: 17 min
completed: 2026-07-03
---

# Phase 20 Plan 02: Safe Baseline Evidence Summary

**Detector-gated Ultra 205 package identity and safe-baseline boot evidence with explicit no-scan target provenance**

## Performance

- **Duration:** 17 min
- **Started:** 2026-07-03T22:04:05Z
- **Completed:** 2026-07-03T22:20:34Z
- **Tasks:** 2
- **Files modified:** 9

## Accomplishments

- Captured Phase 20 package identity and release-gate output for board `205`.
- Ran `just detect-ultra205`, confirmed exactly one ESP32-S3 port through board-info, and captured the redacted detector log.
- Ran repo-owned `just flash-monitor` against the detected Ultra 205 with `redact-evidence=true`, captured trusted boot output, and recorded safe-state markers.
- Wrote `target-lock.json` with `network_scan: "disabled"` and a blocked target origin because no trusted raw origin-only artifact exists.

## Task Commits

Each task was committed atomically:

1. **Task 1: Capture current package and release-gate identity** - `c11fba2` (docs)
2. **Task 2: Capture detector-gated safe baseline or blocked hardware evidence** - `ee71ba8` (docs)

## Files Created/Modified

- `docs/parity/evidence/phase-20-active-safety-hardware-telemetry-evidence/package-release-gate.md` - Records package identity, release-gate status, package-time source commit, redaction boundary, and non-claims.
- `docs/parity/evidence/phase-20-active-safety-hardware-telemetry-evidence/package-release-gate/bitaxe-ultra205-package.json` - Copied package manifest for the package-time source commit used by safe-baseline flashing.
- `docs/parity/evidence/phase-20-active-safety-hardware-telemetry-evidence/package-release-gate/package-command.log` - Captured `just package` output.
- `docs/parity/evidence/phase-20-active-safety-hardware-telemetry-evidence/package-release-gate/release-gate.log` - Captured release-gate output showing `release_gate: passed`.
- `docs/parity/evidence/phase-20-active-safety-hardware-telemetry-evidence/safe-baseline.md` - Summarizes detector, board-info, flash-monitor, safe-state, redaction, and target-lock results.
- `docs/parity/evidence/phase-20-active-safety-hardware-telemetry-evidence/safe-baseline/detect-ultra205.log` - Redacted detector and board-info evidence.
- `docs/parity/evidence/phase-20-active-safety-hardware-telemetry-evidence/safe-baseline/flash-command-evidence.json` - Redacted flash-monitor evidence metadata.
- `docs/parity/evidence/phase-20-active-safety-hardware-telemetry-evidence/safe-baseline/flash-monitor.log` - Redacted trusted serial boot and safe-state output.
- `docs/parity/evidence/phase-20-active-safety-hardware-telemetry-evidence/target-lock.json` - Records no-scan target provenance as blocked.

## Decisions Made

- Safe-baseline evidence is trusted only after the repo detector finds exactly one likely ESP USB serial port and `espflash board-info --chip esp32s3` succeeds.
- `wifi-credentials.json` was passed by path only to repo-owned flashing automation for developer bring-up; its contents were not read, printed, summarized, or committed.
- Redacted serial output is sufficient for boot and safe-state claims but not for target-origin claims, so `target-lock.json` remains blocked until a trusted raw origin-only target artifact exists.
- Package identity is interpreted as current at package time, matching prior Phase 19 evidence; the committed docs commit necessarily advances repository `HEAD` after the manifest is copied.

## Deviations from Plan

### Process Adjustments

**1. Package evidence refreshed after Task 1 commit**
- **Found during:** Task 2 (Capture detector-gated safe baseline or blocked hardware evidence)
- **Issue:** `just flash-monitor` rebuilds the package before flashing. After Task 1 committed package evidence, the hardware run used the Task 1 source commit.
- **Fix:** Refreshed the copied package manifest, package log, release-gate log, and package ledger so package evidence and safe-baseline flash evidence both cite source commit `c11fba2`.
- **Files modified:** `package-release-gate.md`, `package-release-gate/bitaxe-ultra205-package.json`, `package-release-gate/package-command.log`, `package-release-gate/release-gate.log`
- **Verification:** Release gate passed against the refreshed manifest, and `safe-baseline.md` cites the same source commit.
- **Committed in:** `ee71ba8`

**2. Generated serial logs normalized before commit**
- **Found during:** Task 2 commit review
- **Issue:** `git diff --cached --check` flagged generated detector and monitor logs for CRLF/trailing whitespace line endings.
- **Fix:** Mechanically normalized the generated logs to LF and trimmed line endings without changing evidence content.
- **Files modified:** `safe-baseline/detect-ultra205.log`, `safe-baseline/flash-monitor.log`
- **Verification:** `git diff --cached --check` passed.
- **Committed in:** `ee71ba8`

**Total deviations:** 0 auto-fixed; 2 repo-rule/process adjustments.
**Impact on plan:** No scope change. The adjustments preserve evidence integrity, redaction safety, and normal commit hygiene.

## Issues Encountered

None requiring blockers. Hardware detection, board-info, package build, release gate, flash-monitor, and redaction checks all completed successfully.

## Verification

- `node /Users/peterryszkiewicz/.codex/get-shit-done/bin/gsd-tools.cjs verify lifecycle 20 --require-plans --raw` returned `valid` before execution began and again after task commits.
- `just package` passed during Task 1, during the Task 2 package refresh, and during plan-level verification.
- `bazel run //tools/parity:report -- release-gate --manifest docs/parity/evidence/phase-20-active-safety-hardware-telemetry-evidence/package-release-gate/bitaxe-ultra205-package.json` passed.
- `just detect-ultra205` passed and recorded exactly one detected Ultra 205-compatible ESP32-S3 port.
- `just flash-monitor board=205 port=/dev/cu.usbmodem1101 evidence-dir=docs/parity/evidence/phase-20-active-safety-hardware-telemetry-evidence/safe-baseline capture-timeout-seconds=45 redact-evidence=true wifi-credentials=wifi-credentials.json` passed and captured trusted boot markers.
- Task acceptance checks passed for package manifest presence, release-gate evidence, detector status, safe-baseline status, target lock `network_scan: "disabled"`, and targeted redaction scanning.
- `just parity` passed.
- `just verify-reference` passed.
- `git diff -- reference/esp-miner --exit-code` passed.
- Before each task commit: `cargo fmt --all`, `cargo clippy --all-targets --all-features -- -D warnings`, `cargo build --all-targets --all-features`, and `cargo test --all-features` passed.

## Known Stubs

None. The blocked `target-lock.json` state is intentional and prevents unsupported target-origin claims.

## Auth Gates

None.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

Ready for downstream Phase 20 active safety and live telemetry plans to consume package identity and safe-baseline boot evidence. Live telemetry remains blocked from using a network origin until a later plan supplies a trusted raw origin-only target artifact.

## Self-Check: PASSED

- Found `.planning/phases/20-active-safety-hardware-telemetry-evidence/20-02-SUMMARY.md`.
- Found Phase 20 package-release-gate evidence files.
- Found Phase 20 safe-baseline evidence files.
- Found `target-lock.json`.
- Found task commit `c11fba2`.
- Found task commit `ee71ba8`.
- Confirmed the summary uses only the opening and closing frontmatter delimiters.
- Stub scan found no implementation stubs; the only placeholder-pattern match is intentional redaction wording in `safe-baseline.md`.

*Phase: 20-active-safety-hardware-telemetry-evidence*
*Completed: 2026-07-03*
