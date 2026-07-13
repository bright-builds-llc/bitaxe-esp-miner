---
phase: 12
plan: 03
subsystem: hardware-evidence
tags: [hardware, ultra205, asic, mining, evidence]
requires:
  - phase: 12
    provides: "Evidence contract and parity promotion guard from Plans 12-01 and 12-02"
provides:
  - "Detector-gated Ultra 205 safe-boot evidence"
  - "Chip-detect diagnostic capture boundary"
  - "Fail-closed mining smoke preflight output"
  - "Redaction review for generated Phase 12 artifacts"
affects: [docs-parity-evidence, scripts, hardware-evidence]
tech-stack:
  added: []
  patterns:
    - "Live hardware evidence starts with just detect-ultra205 and records exact pending status when a tier is not trusted enough for promotion"
key-files:
  created:
    - scripts/phase12-mining-smoke-preflight.sh
    - docs/parity/evidence/phase-12-asic-and-mining-hardware-evidence/detect-ultra205.log
    - docs/parity/evidence/phase-12-asic-and-mining-hardware-evidence/restore-detect-ultra205.log
    - docs/parity/evidence/phase-12-asic-and-mining-hardware-evidence/safe-boot/flash-command-evidence.json
    - docs/parity/evidence/phase-12-asic-and-mining-hardware-evidence/safe-boot/flash-monitor.log
    - docs/parity/evidence/phase-12-asic-and-mining-hardware-evidence/chip-detect/flash-command-evidence.json
    - docs/parity/evidence/phase-12-asic-and-mining-hardware-evidence/chip-detect/flash-monitor.log
    - docs/parity/evidence/phase-12-asic-and-mining-hardware-evidence/mining-smoke-preflight.log
    - docs/parity/evidence/phase-12-asic-and-mining-hardware-evidence/safe-boot-restore/flash-command-evidence.json
    - docs/parity/evidence/phase-12-asic-and-mining-hardware-evidence/safe-boot-restore/flash-monitor.log
  modified:
    - docs/parity/evidence/phase-12-asic-and-mining-hardware-evidence.md
    - docs/parity/evidence/phase-12-asic-and-mining-hardware-evidence/redaction-review.md
key-decisions:
  - "Treat the ELF-only chip-detect diagnostic capture as wrapper-untrusted because it lacked SPIFFS, even though it emitted fail-closed chip-detect markers."
  - "Restore the connected board to the trusted packaged safe-boot image after the ELF-only diagnostic run."
patterns-established:
  - "Mining smoke/soak must remain not run until a bounded probe and recovery path exist."
requirements-completed: [ASIC-07, STR-06, STR-07, EVD-05]
generated_by: gsd-execute-plan
lifecycle_mode: yolo
phase_lifecycle_id: 12-2026-06-30T00-13-19
generated_at: 2026-06-30T01:28:02Z
duration: 16 min
completed: 2026-06-30
---

# Phase 12 Plan 03: Hardware Capture Summary

**Detector-gated safe boot passed; chip-detect and live mining claims remain conservatively pending**

## Performance

- **Duration:** 16 min
- **Started:** 2026-06-30T01:12:22Z
- **Completed:** 2026-06-30T01:28:02Z
- **Tasks:** 2
- **Files created/modified:** 12

## Accomplishments

- Added `scripts/phase12-mining-smoke-preflight.sh`, a read-only fail-closed hook that classifies wrapper logs before any live mining smoke or soak.
- Ran `just detect-ultra205`; it found exactly one ESP32-S3 port, `/dev/cu.usbmodem1101`, and `board-info` succeeded.
- Captured trusted safe-boot evidence with `spiffs_mount=available`, `mining_loop_status=blocked`, and `work_submission=disabled`.
- Ran the chip-detect diagnostic path. The log recorded chip-detect/no-mining fail-closed markers, but the wrapper rejected the capture because the ELF-only diagnostic flash lacked SPIFFS.
- Ran the mining smoke preflight against the trusted safe-boot log; it recorded `phase12_mining_smoke_preflight=blocked` and did not run mining smoke or soak.
- Restored the connected board to the trusted packaged safe-boot image and captured final safe-state evidence.
- Completed redaction review for all generated artifacts.

## Task Commits

1. **Task 1: Add fail-closed mining smoke preflight hook** - `42bb1f7`
2. **Task 2: Run host checks and detector-gated hardware capture** - included with this summary commit

## Files Created/Modified

- `scripts/phase12-mining-smoke-preflight.sh` - Read-only preflight classifier for trusted wrapper logs.
- `docs/parity/evidence/phase-12-asic-and-mining-hardware-evidence.md` - Execution log, detector, safe-boot, chip-detect, preflight, restore, redaction, and conclusions.
- `docs/parity/evidence/phase-12-asic-and-mining-hardware-evidence/redaction-review.md` - Completed artifact review.
- `docs/parity/evidence/phase-12-asic-and-mining-hardware-evidence/*` - Detector, wrapper JSON, serial logs, and preflight artifacts.

## Decisions Made

The chip-detect capture is not used for checklist promotion because the wrapper marked it untrusted. The observed diagnostic markers are retained as evidence of a fail-closed attempt only.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 4 - Verification] Relative chip-detect image path failed under `bazel run`**
- **Found during:** Task 2
- **Issue:** The plan-specified `--image bazel-bin/firmware/bitaxe/bitaxe-ultra205.elf` was not resolvable from the `bazel run` execution environment.
- **Fix:** Re-ran the same repo wrapper with the absolute workspace symlink path.
- **Files modified:** Phase 12 evidence ledger and generated chip-detect artifacts.
- **Verification:** The corrected command wrote `chip-detect/flash-command-evidence.json` and `chip-detect/flash-monitor.log`.

**2. [Rule 4 - Verification] Chip-detect ELF-only capture failed wrapper trust**
- **Found during:** Task 2
- **Issue:** The diagnostic emitted chip-detect markers, but the wrapper requires `spiffs_mount=available`; the ELF-only diagnostic produced `spiffs_mount=unavailable`.
- **Fix:** Recorded the capture as wrapper-untrusted and left ASIC initialization/work/result promotion pending. Restored the board with the packaged factory image afterward.
- **Files modified:** Phase 12 evidence ledger, redaction review, and restore artifacts.
- **Verification:** Restore safe-boot capture passed with `trusted_output=true` and `spiffs_mount=available`.

**Total deviations:** 2 auto-fixed
**Impact on plan:** The final evidence is more conservative than the optimistic chip-detect path and does not overclaim live ASIC or mining parity.

## Issues Encountered

- Chip-detect UART returned a partial read in the diagnostic capture: `expected 11 bytes, read 9`. The firmware held reset low and kept mining/work submission disabled.

## Verification

- `bash -n scripts/phase12-mining-smoke-preflight.sh` passed.
- `cargo test -p bitaxe-asic -p bitaxe-stratum -p bitaxe-safety -p bitaxe-api --all-features` passed.
- `cargo test -p bitaxe-parity --all-features` passed.
- Evidence ledger marker checks passed.
- Markdown body separator checks passed.
- Redaction scan found only expected WiFi/NVS labels, local paths, and board-info hardware identity.

## User Setup Required

None.

## Next Phase Readiness

Ready for `12-04`: checklist rows and validation should cite Phase 12 safe-boot/preflight evidence while keeping unsupported ASIC initialization, work/result, and live mining claims below `verified`.

*Phase: 12-asic-and-mining-hardware-evidence*
*Completed: 2026-06-30*
