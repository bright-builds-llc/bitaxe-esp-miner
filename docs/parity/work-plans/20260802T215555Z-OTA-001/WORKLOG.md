# OTA-001 worklog

## 2026-08-02T21:55:55Z | plan committed

- Source commit: `b0d00c175b2d87be2e1a506d4fa7fe0de0ce939f`
- Actions: selected `OTA-001`, recorded the exact one-attempt hardware contract,
  and committed the immutable plan plus active task.
- Verification: the pre-plan detector found exactly one Ultra 205 on the
  approved USB port; all Rust, managed Bright Builds, 82 Bazel, parity,
  progress, redaction, reference-cleanliness, and diff gates passed.
- Evidence: `PLAN.md` and `TASKS.md`.
- Outcome: implementation authorized; no OTA attempt has run.
- Blocker or next safe action: fix the host monitor-attachment race and prove
  it with deterministic software tests before committing the implementation.

## 2026-08-02T22:02:00Z | monitor race fixed

- Source commit: `b0d00c175b2d87be2e1a506d4fa7fe0de0ce939f`
- Actions: changed the existing OTA smoke helper to start its bounded no-reset
  monitor, wait for verified active ownership, and only then submit the valid
  image; added deterministic cleanup and failure propagation.
- Verification: shell syntax, the focused Bazel test, and `git diff --check`
  pass. The test proves monitor-ready precedes valid upload, all identity and
  boot-validation markers are required, early monitor failure blocks the
  upload, and the readiness file is removed.
- Evidence: `scripts/phase13-firmware-ota-smoke.sh` and
  `scripts/phase13-firmware-ota-smoke-test.sh`.
- Outcome: software implementation complete pending full gates and immutable
  implementation commit.
- Blocker or next safe action: run mandatory implementation verification,
  review/commit the software changes, then build and run the exact hardware
  contract once.

## 2026-08-02T22:06:00Z | dependent wrapper fixture aligned

- Source commit: `b0d00c175b2d87be2e1a506d4fa7fe0de0ce939f`
- Actions: updated the Phase 18 wrapper's fake monitor fixtures to participate
  in the same explicit active-readiness handshake.
- Verification: the first full `just test` exposed the stale fixture; after
  alignment, both `//scripts:phase13_firmware_ota_smoke_test` and
  `//scripts:phase18_firmware_ota_evidence_test` pass together.
- Evidence: the two shell integration test targets.
- Outcome: dependent wrapper behavior is consistent with the corrected helper.
- Blocker or next safe action: rerun the complete mandatory suite before the
  immutable implementation commit.

## 2026-08-02T22:10:00Z | implementation verification complete

- Source commit: `b0d00c175b2d87be2e1a506d4fa7fe0de0ce939f`
- Actions: completed the implementation audit and an explicit simplification
  pass; the existing helper remains the single orchestration surface and reuses
  the monitor's established active-owner handshake.
- Verification: formatting, strict Clippy, all-target/all-feature Cargo build
  and tests, managed Bright Builds checks, all 82 Bazel tests, parity, progress,
  redaction, reference cleanliness, and `git diff --check` pass.
- Evidence: both focused OTA shell tests and the full repository gate output.
- Outcome: software changes are ready for the required immutable implementation
  commit before hardware use.
- Blocker or next safe action: review and commit the exact implementation diff,
  rebuild the package from that clean commit, then run the one-attempt contract.

## 2026-08-02T22:23:00Z | hardware attempt stopped

- Source commit: `afb73fba3b34f4b43250d503d574c92c258f9606`
- Actions: built the exact clean package, passed the fresh detector, flashed
  through the repo-owned wrapper with local Wi-Fi input, admitted exactly one
  origin-only same-session device URL without printing it, and consumed the
  one permitted invalid-plus-valid OTA invocation.
- Verification: package/reference identity and OTA SHA-256 admission passed;
  wrapper flash-monitor trust passed; invalid-image rejection returned HTTP
  500; active monitor readiness preceded the valid upload. The valid request
  then returned curl status 28 and HTTP 000 after 30 seconds with zero response
  bytes. Firmware, reference, boot-validation, and safe-state reboot markers
  were absent. The cleanup detector passed on board 205 at the approved port,
  so the conditional recovery flash did not run. Ignored raw-evidence admission
  passed and no secret or raw origin was committed.
- Evidence: private artifacts under `target/advance-parity-ota001/`, exact
  package digest
  `0dab8e06f08f566a898c8f4b07f315a8a7e8e2d2fd961deb3ad9c2177bdaad7c`,
  and this redacted checkpoint.
- Outcome: `stop_hardware_blocker`; `OTA-001` remains `implemented` with no
  checklist transition or progress sync.
- Blocker or next safe action: none within this authorization. The exact
  attempt is consumed, no retry is permitted, and the task remains active and
  unarchived as a terminal blocker.

## 2026-08-02T22:28:34Z | timeout root cause fixed in software

- Source commit: `99c3f1ca00b99f0f64509d8537989d6001b02385`
- Actions: reproduced the valid-upload failure at the exact host deadline,
  tested the ranked timeout, firmware-stall, restart-race, and monitor-reader
  hypotheses, and changed only the OTA orchestration and its regression seam.
- Verification: the former 30-second boundary failed deterministically on
  three consecutive pre-fix runs with curl status 28, HTTP `000`, and zero
  response bytes; increasing only the valid-upload deadline made the loop
  green. A separate red/green test proved the prearmed capture now selects the
  qualified OS-native reader. Both focused OTA tests and every mandatory
  repository gate pass.
- Evidence: the valid-upload timeout now defaults to a bounded 120 seconds,
  post-upload observation defaults to 360 seconds, the prearmed capture spans
  both budgets, and the helper explicitly requests the OS-native reader.
- Outcome: the proven host-side timeout boundary and the secondary evidence
  transport defect are fixed without another hardware interaction. `OTA-001`
  remains `implemented` and progress remains unchanged.
- Blocker or next safe action: device behavior after the former client deadline
  remains unknowable from the consumed attempt. A separately authorized,
  task-gated hardware attempt is still required for verified parity.

## 2026-08-02T23:13:00Z | terminal result reconciled

- Source commit: `2541818aa23120dd85c711386efadb69a1415ad3`
- Actions: reconciled this original plan after its separately authorized bounded
  retry verified the same parity row and supplied every evidence item that the
  original attempt could not observe past the former timeout boundary.
- Verification: the authoritative checklist records `OTA-001` as `verified`,
  transition receipt `20260802T230503Z-OTA-001` binds the predecessor and
  retry result, and the bounded retry result records current-package admission,
  invalid rejection, HTTP success, reboot identity, safe state, boot
  validation, cleanup, and privacy.
- Evidence: `../20260802T223139Z-OTA-001-RETRY/RESULT.md` and
  `../../checklist-transitions/20260802T230503Z-OTA-001.json`.
- Outcome: this plan now has a terminal `RESULT.md`; its original failed
  attempt remains unchanged while the row-level outcome is verified.
- Blocker or next safe action: none for this plan; rerun the authoritative
  selector and advance to the next eligible parity row.
