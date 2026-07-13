---
phase: 11-safety-controller-hardware-regression-evidence
plan: 03
subsystem: hardware-evidence
tags: [ultra-205, hardware-smoke, parity, safety-evidence]
requires:
  - phase: 11-safety-controller-hardware-regression-evidence
    provides: Phase 11 evidence contract and active-control parity guard
provides:
  - Detector-gated Ultra 205 safe boot evidence with wrapper-generated JSON and serial log artifacts.
  - Conservative checklist citations for all Phase 11 affected rows.
  - Final verification record for safety tests, parity tests, checklist validation, full tests, and reference read-only status.
affects: [hardware-evidence, parity-checklist, safety-controller-verification]
tech-stack:
  added: []
  patterns: [detector-gated hardware evidence, conservative checklist promotion, wrapper-owned serial capture]
key-files:
  created:
    - .planning/phases/11-safety-controller-hardware-regression-evidence/11-03-SUMMARY.md
    - docs/parity/evidence/phase-11-safety-controller-hardware-regression-evidence/flash-command-evidence.json
    - docs/parity/evidence/phase-11-safety-controller-hardware-regression-evidence/flash-monitor.log
  modified:
    - docs/parity/checklist.md
    - docs/parity/evidence/phase-11-safety-controller-hardware-regression-evidence.md
    - docs/parity/evidence/phase-11-safety-controller-hardware-regression-evidence/redaction-review.md
    - .planning/phases/11-safety-controller-hardware-regression-evidence/11-VALIDATION.md
key-decisions:
  - "Detector success allowed wrapper-owned flash-monitor capture on /dev/cu.usbmodem1101."
  - "Captured safe boot evidence did not promote active-control, runtime input/display, sensor freshness, or stress/fault behavior to verified."
patterns-established:
  - "Record exact wrapper artifacts and observed markers, then keep unobserved active or mixed claims pending."
requirements-completed: [SAFE-01, SAFE-02, SAFE-03, SAFE-04, SAFE-05, SAFE-06, SAFE-07, SAFE-08, SAFE-09, EVD-05]
generated_by: gsd-execute-plan
lifecycle_mode: yolo
phase_lifecycle_id: 11-2026-06-29T20-23-34
generated_at: 2026-06-29T21:28:52Z
duration: 24min
completed: 2026-06-29
---

# Phase 11 Plan 03 Summary

**Detector-gated Ultra 205 safe boot evidence with conservative checklist closure and no active-control promotion.**

## Performance

- Duration: 24 min
- Started: 2026-06-29T21:05:00Z
- Completed: 2026-06-29T21:28:52Z
- Tasks: 2
- Files modified: 6

## Accomplishments

- Ran `just detect-ultra205`, confirmed exactly one Ultra 205 port, and recorded board-info output.
- Ran `just flash-monitor board=205 port=/dev/cu.usbmodem1101 evidence-dir=docs/parity/evidence/phase-11-safety-controller-hardware-regression-evidence`.
- Captured wrapper-owned `flash-command-evidence.json` and `flash-monitor.log` with `trusted_output=true` and `capture_status=timed_out_after_trusted_output`.
- Updated the evidence ledger, redaction review, validation contract, and checklist rows for every Phase 11 affected safety surface.
- Kept active voltage, DS4432U writes, fan duty, thermal/fault paths, self-test hardware submodes, runtime input/display parity, sensor freshness, and mining/load stress pending.

## Task Commits

No commits were created during plan execution. The wrapper workflow will commit only after full phase verification passes.

## Files Created Or Modified

- `docs/parity/evidence/phase-11-safety-controller-hardware-regression-evidence/flash-command-evidence.json` - Wrapper-generated machine evidence for the flash-monitor run.
- `docs/parity/evidence/phase-11-safety-controller-hardware-regression-evidence/flash-monitor.log` - Wrapper-generated serial log for the detector-gated run.
- `docs/parity/evidence/phase-11-safety-controller-hardware-regression-evidence.md` - Updated execution log, observed safe boot markers, residual risks, conclusion, and final verification.
- `docs/parity/evidence/phase-11-safety-controller-hardware-regression-evidence/redaction-review.md` - Marked generated artifacts reviewed with no redaction required.
- `docs/parity/checklist.md` - Added Phase 11 citations and conservative conclusions for affected rows.
- `.planning/phases/11-safety-controller-hardware-regression-evidence/11-VALIDATION.md` - Marked validation contract passed and Nyquist-compliant.

## Decisions Made

Captured safe boot and watchdog markers can support only exact safe boot/runtime-gap/watchdog startup claims. They do not verify active safety-control behavior, live sensor freshness, API/WebSocket telemetry, runtime input/display parity, or stress/fault scenarios.

## Deviations From Plan

None - plan executed within the detector-gated wrapper-owned command set. The generated monitor capture timed out after trusted output, which is an accepted wrapper outcome.

## Issues Encountered

The plan verification expected the exact lowercase phrase `selected port: /dev/...`; the ledger was normalized to include that literal text while retaining the board-info evidence.

## Verification

- Green: `cargo test -p bitaxe-safety --all-features`
- Green: `cargo test -p bitaxe-parity --all-features`
- Green: `just parity`
- Green: `just test`
- Green: `git diff -- reference/esp-miner --exit-code`
- Green: `git diff --check` for Phase 11 touched scopes

## User Setup Required

None.

## Next Phase Readiness

Phase 11 now has hardware-backed safe boot evidence and strict checklist promotion boundaries. Any future active-control claim needs a separate bounded `hardware-regression` procedure with abort conditions, recovery, and redaction review.
