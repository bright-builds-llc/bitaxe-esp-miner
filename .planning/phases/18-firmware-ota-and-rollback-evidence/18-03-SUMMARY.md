---
phase: 18-firmware-ota-and-rollback-evidence
plan: "03"
subsystem: evidence
tags: [firmware-ota, ultra205, rollback, boot-validation, redaction]
requires:
  - phase: 18-02
    provides: "Package, serial boot, detector, flash-monitor, and redacted target-lock evidence for board 205."
provides:
  - "Phase 18 firmware OTA helper artifacts for invalid rejection, valid upload response, and post-OTA monitor capture."
  - "Claim-specific firmware OTA ledger separating invalid rejection, valid OTA, boot validation, rollback, and non-claims."
  - "Redaction-pending evidence boundary for Plan 18-04 review."
affects: [phase-18, release-evidence, firmware-ota, rollback, redaction-review]
tech-stack:
  added: []
  patterns:
    - "Evidence ledgers keep valid OTA below verified when post-OTA identity or boot-validation markers are missing."
    - "Invalid firmware rejection is cited separately from rollback and boot-validation claims."
key-files:
  created:
    - docs/parity/evidence/phase-18-firmware-ota-and-rollback-evidence/firmware-ota.md
    - docs/parity/evidence/phase-18-firmware-ota-and-rollback-evidence/firmware-ota/firmware-ota-smoke.log
    - docs/parity/evidence/phase-18-firmware-ota-and-rollback-evidence/firmware-ota/post-ota-detect-ultra205.log
    - docs/parity/evidence/phase-18-firmware-ota-and-rollback-evidence/firmware-ota/post-ota-monitor.log
    - docs/parity/evidence/phase-18-firmware-ota-and-rollback-evidence/firmware-ota/invalid-firmware.bin
    - docs/parity/evidence/phase-18-firmware-ota-and-rollback-evidence/firmware-ota/invalid-firmware-ota.headers.txt
    - docs/parity/evidence/phase-18-firmware-ota-and-rollback-evidence/firmware-ota/invalid-firmware-ota.body.txt
    - docs/parity/evidence/phase-18-firmware-ota-and-rollback-evidence/firmware-ota/invalid-firmware-ota.curl-error.txt
    - docs/parity/evidence/phase-18-firmware-ota-and-rollback-evidence/firmware-ota/valid-firmware-ota.headers.txt
    - docs/parity/evidence/phase-18-firmware-ota-and-rollback-evidence/firmware-ota/valid-firmware-ota.body.txt
    - docs/parity/evidence/phase-18-firmware-ota-and-rollback-evidence/firmware-ota/valid-firmware-ota.curl-error.txt
  modified: []
key-decisions:
  - "Plan 18-03 records invalid firmware rejection as passed invalid-rejection evidence only; it is not rollback or boot-validation proof."
  - "Valid firmware OTA remains below verified because the HTTP 200 completion body was not followed by captured post-OTA identity or ota_boot_validation markers."
  - "Destructive rollback, interrupted update, forced boot failure, erase, OTAWWW, active safety, mining, and soak behavior remain non-claims."
patterns-established:
  - "Use redacted target provenance plus a detector rerun immediately before OTA; block or downgrade claims if post-OTA markers are absent."
  - "Keep redaction_status pending for Plan 18-04 even when targeted scans pass during execution."
requirements-completed: [REL-02, REL-08, REL-07, EVD-05]
generated_by: gsd-execute-plan
lifecycle_mode: yolo
phase_lifecycle_id: 18-2026-07-03T14-06-29
generated_at: 2026-07-03T15:48:49Z
duration: 9m52s
completed: 2026-07-03
---

# Phase 18 Plan 03: Firmware OTA Evidence Summary

**Firmware OTA evidence captured invalid rejection and valid HTTP acceptance while keeping valid OTA and boot validation below verified because post-OTA markers were absent.**

## Performance

- **Duration:** 9m52s
- **Started:** 2026-07-03T15:38:57Z
- **Completed:** 2026-07-03T15:48:49Z
- **Tasks:** 2
- **Files modified:** 11

## Accomplishments

- Ran the Phase 18 OTA helper against board `205` on the detector-approved `/dev/cu.usbmodem1101` port using the redacted target lock and trusted local raw flash evidence source.
- Captured invalid firmware rejection: fixed invalid fixture, HTTP 500, `Write Error` body marker, and explicit rejection-only boundary.
- Captured valid firmware OTA HTTP acceptance: manifest `esp-miner.bin`, checksum match, HTTP 200, and `Firmware update complete, rebooting now!` body marker.
- Recorded the exact blocker: bounded post-OTA monitor captured no `firmware_commit=`, `reference_commit=`, or `ota_boot_validation=` markers, so valid OTA, boot validation, rollback, and safe post-OTA claims remain below verified.
- Kept `redaction_status: pending` for Plan 18-04 and preserved raw target/device URL evidence outside committed files.

## Task Commits

1. **Task 1: Run Phase 18 firmware OTA helper or record exact blocker** - `dda07ae` (docs)
2. **Task 2: Write claim-specific firmware OTA ledger** - `b755104` (docs)

**Plan metadata:** pending final metadata commit

## Files Created/Modified

- `docs/parity/evidence/phase-18-firmware-ota-and-rollback-evidence/firmware-ota/firmware-ota-smoke.log` - Phase 18 wrapper transcript, nested OTA helper output, and blocked post-OTA marker conclusion.
- `docs/parity/evidence/phase-18-firmware-ota-and-rollback-evidence/firmware-ota/post-ota-detect-ultra205.log` - Immediate pre-OTA detector rerun for the same board 205 port, with MAC redacted.
- `docs/parity/evidence/phase-18-firmware-ota-and-rollback-evidence/firmware-ota/post-ota-monitor.log` - Bounded post-OTA monitor capture showing no required identity or boot-validation markers.
- `docs/parity/evidence/phase-18-firmware-ota-and-rollback-evidence/firmware-ota/invalid-firmware.bin` - Fixed invalid firmware fixture used only for invalid-rejection evidence.
- `docs/parity/evidence/phase-18-firmware-ota-and-rollback-evidence/firmware-ota/invalid-firmware-ota.headers.txt` - Invalid OTA selected response headers.
- `docs/parity/evidence/phase-18-firmware-ota-and-rollback-evidence/firmware-ota/invalid-firmware-ota.body.txt` - Invalid OTA body marker.
- `docs/parity/evidence/phase-18-firmware-ota-and-rollback-evidence/firmware-ota/invalid-firmware-ota.curl-error.txt` - Invalid OTA curl error artifact.
- `docs/parity/evidence/phase-18-firmware-ota-and-rollback-evidence/firmware-ota/valid-firmware-ota.headers.txt` - Valid OTA selected response headers.
- `docs/parity/evidence/phase-18-firmware-ota-and-rollback-evidence/firmware-ota/valid-firmware-ota.body.txt` - Valid OTA body marker.
- `docs/parity/evidence/phase-18-firmware-ota-and-rollback-evidence/firmware-ota/valid-firmware-ota.curl-error.txt` - Valid OTA curl error artifact.
- `docs/parity/evidence/phase-18-firmware-ota-and-rollback-evidence/firmware-ota.md` - Claim-specific ledger for firmware OTA, invalid rejection, boot validation, rollback, and non-claims.

## Decisions Made

- Invalid rejection is promoted only as invalid-rejection evidence; it does not establish rollback behavior.
- Valid OTA remains below verified when post-OTA identity and `ota_boot_validation=` markers are absent, even when HTTP upload acceptance is observed.
- Destructive rollback and fault-injection surfaces remain non-claims because this plan did not document or execute those recovery gates.
- Plan 18-04 still owns redaction closure; Plan 18-03 only ran a targeted pre-commit scan.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] Adjusted ledger wording for contradictory verification guard**
- **Found during:** Task 2 (Write claim-specific firmware OTA ledger)
- **Issue:** The plan requested exact `not rollback proof` wording while its negative verification guard rejected `invalid.*rollback proof`.
- **Fix:** Used equivalent wording, `rejection-only evidence; does not prove rollback`, preserving the claim boundary while satisfying the machine guard.
- **Files modified:** `docs/parity/evidence/phase-18-firmware-ota-and-rollback-evidence/firmware-ota.md`
- **Verification:** Ledger guard passed after wording adjustment.
- **Committed in:** `b755104`

**2. [Rule 2 - Missing Critical] Redacted generated detector MAC before commit**
- **Found during:** Task 1 (Run Phase 18 firmware OTA helper or record exact blocker)
- **Issue:** `just detect-ultra205` wrote an unredacted MAC address to the post-OTA detector transcript, which is commit-destined evidence.
- **Fix:** Replaced the MAC value with `[redacted-mac]`.
- **Files modified:** `docs/parity/evidence/phase-18-firmware-ota-and-rollback-evidence/firmware-ota/post-ota-detect-ultra205.log`
- **Verification:** Targeted redaction scan passed over 12 Plan 18-03 artifacts.
- **Committed in:** `dda07ae`

**Total deviations:** 2 auto-fixed (1 blocking, 1 missing critical)
**Impact on plan:** Both fixes preserved the plan's evidence and security boundaries without adding scope.

## Issues Encountered

- The valid OTA HTTP upload returned `200` with the expected reboot body, but the post-OTA monitor captured no firmware identity or boot-validation markers. This is recorded as blocked/below-verified evidence rather than a failed execution.

## Verification

- `bash -n scripts/phase18-firmware-ota-evidence.sh scripts/phase13-firmware-ota-smoke.sh scripts/phase13-monitor-capture.sh` passed.
- `bazel test //scripts:phase18_firmware_ota_evidence_test //scripts:phase13_firmware_ota_smoke_test //scripts:phase13_monitor_capture_test` passed.
- Plan task-level evidence and ledger `rg`/Python checks passed.
- `git diff --check -- docs/parity/evidence/phase-18-firmware-ota-and-rollback-evidence/firmware-ota.md docs/parity/evidence/phase-18-firmware-ota-and-rollback-evidence/firmware-ota` passed.
- Targeted redaction scan over Plan 18-03 artifacts passed.
- Before each task commit: `cargo fmt --all`, `cargo clippy --all-targets --all-features -- -D warnings`, `cargo build --all-targets --all-features`, and `cargo test --all-features` all passed.

## Known Stubs

None.

## User Setup Required

None - the plan used trusted local raw flash evidence for redacted target provenance, and no secrets were committed.

## Next Phase Readiness

Plan 18-04 can perform redaction review with a precise claim boundary: invalid rejection captured, valid OTA HTTP acceptance observed, but valid OTA, boot validation, selected partition, rollback, destructive rollback, and safe post-OTA behavior remain below verified because post-OTA markers were absent.

## Self-Check: PASSED

- Verified 12 created evidence/summary files exist.
- Verified task commits `dda07ae` and `b755104` exist in git history.

*Phase: 18-firmware-ota-and-rollback-evidence*
*Completed: 2026-07-03*
