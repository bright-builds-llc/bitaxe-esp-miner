---
phase: 13-final-ultra-205-release-evidence
plan: "02"
subsystem: release-evidence
tags: [ultra-205, hardware-detection, serial-boot, flash-monitor, evidence, redaction]
requires:
  - phase: 13-final-ultra-205-release-evidence
    provides: Plan 01 package evidence scaffold and release-gate baseline
provides:
  - Detector-gated Ultra 205 USB hardware evidence
  - Wrapper-owned factory flash and serial boot evidence
  - Redaction review for detector and serial artifacts
  - Current package manifest identity tied to live serial boot output
affects: [phase-13, release-evidence, hardware-evidence, parity-checklist]
tech-stack:
  added: []
  patterns:
    - Detector gate before live USB hardware commands
    - Wrapper JSON plus serial log as the trusted serial evidence source
    - Redaction review before generated evidence commits
key-files:
  created:
    - docs/parity/evidence/phase-13-final-ultra-205-release-evidence/hardware-detection.md
    - docs/parity/evidence/phase-13-final-ultra-205-release-evidence/detect-ultra205.log
    - docs/parity/evidence/phase-13-final-ultra-205-release-evidence/serial-boot.md
    - docs/parity/evidence/phase-13-final-ultra-205-release-evidence/serial-boot/flash-command-evidence.json
    - docs/parity/evidence/phase-13-final-ultra-205-release-evidence/serial-boot/flash-monitor.log
  modified:
    - docs/parity/evidence/phase-13-final-ultra-205-release-evidence/hardware-detection.md
    - docs/parity/evidence/phase-13-final-ultra-205-release-evidence/package-release-gate.md
    - docs/parity/evidence/phase-13-final-ultra-205-release-evidence/redaction-review.md
key-decisions:
  - "Plan 13-02 serial evidence cites the package manifest source commit actually rebuilt and flashed by the repo wrapper."
  - "Detector and serial logs may retain the USB port, MAC address, package paths, and commit markers as necessary bench evidence after redaction review."
patterns-established:
  - "A passed detector gate records selected port plus board-info before any live flash command runs."
  - "Passing serial boot evidence requires trusted wrapper JSON and observed commit markers from the monitor log."
requirements-completed: [FND-06, REL-04, EVD-05]
generated_by: gsd-execute-plan
lifecycle_mode: yolo
phase_lifecycle_id: 13-2026-06-30T14-53-46
generated_at: 2026-06-30T16:33:23Z
duration: 11 min
completed: 2026-06-30
---

# Phase 13 Plan 02: Detector-Gated Serial Boot Evidence Summary

**Detector-approved Ultra 205 flash-monitor evidence with trusted serial boot markers and redaction review**

## Performance

- **Duration:** 11 min
- **Started:** 2026-06-30T16:21:47Z
- **Completed:** 2026-06-30T16:33:23Z
- **Tasks:** 2
- **Files modified:** 7

## Accomplishments

- Captured `just detect-ultra205` output in `detect-ultra205.log` and recorded `detector_status: passed` for exactly one ESP32-S3 USB port.
- Ran `just flash-monitor board=205 port=/dev/cu.usbmodem1101 evidence-dir=.../serial-boot capture-timeout-seconds=25` through the repo wrapper.
- Recorded trusted wrapper JSON and serial boot markers for firmware identity, reference identity, OTA boot validation, SPIFFS availability, route shell startup, reset reason, ESP-IDF version, and disabled mining/work/hardware-control state.
- Completed redaction review for detector and serial artifacts before commit.

## Task Commits

Each task was committed atomically:

1. **Task 1: Run and record the Ultra 205 detector gate** - `1908495` (docs)
2. **Task 2: Flash and monitor the release package through the repo wrapper** - `ee2444f` (docs)

**Plan metadata:** committed separately after SUMMARY, STATE, ROADMAP, and REQUIREMENTS updates.

## Files Created/Modified

- `docs/parity/evidence/phase-13-final-ultra-205-release-evidence/hardware-detection.md` - Detector gate summary with selected port, board-info command, board `205`, package identity, and conclusion.
- `docs/parity/evidence/phase-13-final-ultra-205-release-evidence/detect-ultra205.log` - Captured detector stdout/stderr.
- `docs/parity/evidence/phase-13-final-ultra-205-release-evidence/package-release-gate.md` - Current package manifest identity and post-flash release-gate recheck.
- `docs/parity/evidence/phase-13-final-ultra-205-release-evidence/serial-boot.md` - Human-readable wrapper serial boot evidence summary.
- `docs/parity/evidence/phase-13-final-ultra-205-release-evidence/serial-boot/flash-command-evidence.json` - Wrapper-generated machine-readable evidence with `trusted_output: true`.
- `docs/parity/evidence/phase-13-final-ultra-205-release-evidence/serial-boot/flash-monitor.log` - Wrapper-captured noninteractive serial log.
- `docs/parity/evidence/phase-13-final-ultra-205-release-evidence/redaction-review.md` - Redaction review for package, detector, JSON, and serial log artifacts.

## Decisions Made

- Used the repo-owned detector and wrapper only; no raw erase, rollback, interrupted update, voltage/fan/mining stress, or ad hoc raw write commands were run.
- Cited the package source commit actually flashed by `just flash-monitor`: `190849539700b8f9a7909fd2b6ebd84142557968`.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] Refreshed package identity to the manifest actually flashed**
- **Found during:** Task 1 and Task 2
- **Issue:** The Plan 01 release-gate Markdown initially reflected an earlier package manifest source/checksum set, while the wrapper rebuilt the package before flashing after the Task 1 commit. Leaving the older values would mix package and hardware evidence identities.
- **Fix:** Updated `package-release-gate.md` and `hardware-detection.md` to cite the current wrapper-built manifest source commit and checksums, then re-ran the manifest-backed release gate.
- **Files modified:** `package-release-gate.md`, `hardware-detection.md`
- **Verification:** `bazel run //tools/parity:report -- release-gate --manifest bazel-bin/firmware/bitaxe/bitaxe-ultra205-package.json` passed; wrapper JSON observed firmware commit `190849539700`, matching the manifest source commit prefix.
- **Committed in:** `1908495`, `ee2444f`

**Total deviations:** 1 auto-fixed (1 blocking)
**Impact on plan:** The deviation preserves D-07 package-to-hardware identity and avoids overclaiming evidence for a package commit that was not flashed.

## Issues Encountered

- The wrapper intentionally rebuilds the package before flashing when no explicit image is supplied. Because GSD requires committing each task before the next task, the live serial evidence is tied to the Task 1 commit `1908495`, not the earlier Plan 01 task commit. The package release gate was rechecked after the wrapper rebuild and passed.

## Verification

- Lifecycle validation before execution: `verify lifecycle 13 --expect-id 13-2026-06-30T14-53-46 --expect-mode yolo --require-plans --raw`: passed.
- Task 1 automated verification for detector log and `hardware-detection.md`: passed.
- Task 2 automated verification for `serial-boot.md`, wrapper JSON/log files, commit-prefix matching, and redaction-review coverage: passed.
- `just flash-monitor board=205 port=/dev/cu.usbmodem1101 evidence-dir=docs/parity/evidence/phase-13-final-ultra-205-release-evidence/serial-boot capture-timeout-seconds=25`: passed.
- `bazel run //tools/parity:report -- release-gate --manifest bazel-bin/firmware/bitaxe/bitaxe-ultra205-package.json`: passed after the wrapper rebuild.
- Rust pre-commit sequence before each task commit: `cargo fmt --all`, `cargo clippy --all-targets --all-features -- -D warnings`, `cargo build --all-targets --all-features`, and `cargo test --all-features`: passed.
- Plan-level verification: `just parity` passed with `validation_errors: none`.
- Reference guard: `git diff -- reference/esp-miner --exit-code` passed.

## Known Stubs

None - stub scan found no placeholder, TODO, FIXME, empty hardcoded UI data, or unresolved `not available` fields. The one `Unavailable` value is the intentional manifest offset for the ELF artifact, not a stub.

## Threat Flags

None - this plan added evidence documentation and generated logs only. It did not introduce new endpoints, auth paths, file-access code, schema changes, or runtime trust-boundary behavior.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

Ready for Plan 13-03. Detector-gated serial boot evidence exists for the current Phase 13 package commit, while live HTTP/static/recovery/OTA evidence remains unclaimed until a reachable `DEVICE_URL` is established by the owning plan.

## Self-Check: PASSED

- Created files exist: `hardware-detection.md`, `detect-ultra205.log`, `serial-boot.md`, `serial-boot/flash-command-evidence.json`, `serial-boot/flash-monitor.log`, `redaction-review.md`, and `13-02-SUMMARY.md`.
- Task commits exist: `1908495` and `ee2444f`.
- SUMMARY frontmatter delimiter check passed with exactly two standalone `---` delimiters.

*Phase: 13-final-ultra-205-release-evidence*
*Completed: 2026-06-30*
