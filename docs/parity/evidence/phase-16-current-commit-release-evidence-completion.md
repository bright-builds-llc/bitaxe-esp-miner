---
phase: 16-current-commit-release-evidence-completion
evidence: final-ledger
redaction_status: passed
source_commit: 8490118a7e7f6fc1a9ac2e4025d983b0f402c8ca
reference_commit: c1915b0a63bfabebdb95a515cedfee05146c1d50
status: mixed
generated_by: gsd-execute-plan
generated_at: 2026-07-01T15:26:58Z
---

# Phase 16 Current Commit Release Evidence Completion

## Scope

This ledger closes Phase 16 evidence governance for Ultra 205 board `205`. It
cites only Phase 16 artifacts under
`docs/parity/evidence/phase-16-current-commit-release-evidence-completion/` as
current release proof.

Phase 13 source commit 190849539700b8f9a7909fd2b6ebd84142557968 is historical
unless it equals current commit. It does not equal the Phase 16 release-candidate
source commit, so Phase 13 package, serial, HTTP, OTA, rollback, erase,
failed-update, interrupted-update, and recovery artifacts are not cited here as
current release proof.

## Current-Commit Identity

| Field | Value |
| --- | --- |
| Phase 16 release-candidate source commit | `8490118a7e7f6fc1a9ac2e4025d983b0f402c8ca` |
| Reference commit | `c1915b0a63bfabebdb95a515cedfee05146c1d50` |
| Board | `205` |
| Device | Ultra 205 BM1366 |
| Package manifest | `docs/parity/evidence/phase-16-current-commit-release-evidence-completion/package-release-gate/bitaxe-ultra205-package.json` |
| Serial flash evidence | `docs/parity/evidence/phase-16-current-commit-release-evidence-completion/serial-boot/flash-command-evidence.json` |
| Identity conclusion | passed for package, release gate, detector, and serial boot at the same release-candidate source commit |

Later evidence and metadata commits advanced repository `HEAD` after the
release-candidate package was flashed. The final release-evidence validator was
run with the explicit post-source evidence-commit allowance, which still rejects
firmware, source, or tool changes after the package source commit. This ledger
does not claim later evidence commits were reflashed or OTA-tested.

## Evidence Pack Matrix

| Evidence pack | Artifact | Status | Supports |
| --- | --- | --- | --- |
| Package and release gate | `package-release-gate.md`, package manifest copy, package/release-gate logs | passed | REL-04 package identity, manifest checksums, release-gate inputs |
| Serial boot | `serial-boot.md`, detector log, flash JSON, monitor log | passed | FND-06 boot identity, safe state, partition, SPIFFS, route registration |
| HTTP static recovery | `http-static-recovery.md`, blocked smoke log | blocked | Evidence that `DEVICE_URL` was absent and network scanning stayed disabled |
| Firmware OTA | `firmware-ota.md`, blocked preflight logs | blocked | Evidence that no OTA upload ran because `DEVICE_URL` was unavailable |
| Recovery regression | `recovery-regression.md`, pending operation logs | pending | Evidence that failed-update, large-erase, and interrupted-update operations did not run without allow flags |
| Redaction review | `redaction-review.md` | passed | All cited artifacts safe; absent artifacts are `absent - not cited` |

## Requirement Claim Matrix

| Requirement | Conclusion | Evidence basis |
| --- | --- | --- |
| FND-06 | passed | Detector-gated serial boot evidence recorded board, ASIC, reset reason, partition, firmware/reference commits, ESP-IDF version, SPIFFS availability, route shell startup, and safe no-mining/no-control state. |
| API-09 | blocked for live HTTP/static/recovery; implemented evidence remains below verified | Serial boot observed route registration, but live `/`, `/assets/app.css.gz`, missing static, `/recovery`, API, and WebSocket probes did not run because `DEVICE_URL` was missing. |
| REL-01 | partially passed for package, partition, SPIFFS serial boot; below verified for live static/recovery and post-restore behavior | Package and serial evidence support partition/SPIFFS boot identity; live static, recovery, large-erase restore, and post-restore HTTP/static proof remain blocked or pending. |
| REL-02 | blocked | Firmware OTA did not run; valid upload, invalid rejection, reboot identity, boot validation, selected partition, and rollback proof remain below verified. |
| REL-03 | below verified / deferred | OTAWWW remains the explicit REL-03 gap; no whole-`www` update, live `Wrong API input` response, recovery access, or interrupted-update hardware-regression evidence was captured. |
| REL-04 | passed | `just package` and manifest-backed release gate passed for the Phase 16 release-candidate package manifest and artifact checksums. |
| REL-07 | passed for conservative operator documentation inputs; pending final docs update | Existing docs have safe operator instructions; Plan 16-06 updates must cite these Phase 16 artifacts and blockers without overclaiming. |
| REL-08 | pending / blocked | Failed-update, large-erase, interrupted-update, rollback, and boot-validation evidence did not run because `DEVICE_URL`, current OTA preflight, and allow prerequisites were unavailable. |
| EVD-05 | passed for evidence layering; below verified for blocked live layers | Unit/workflow/package/serial/redaction layers exist; live HTTP, OTA, rollback, erase, failed-update, interrupted-update, and OTAWWW layers remain blocked or pending. |

## Package And Release Gate

Status: passed.

Phase 16 package evidence records:

- `source_commit: 8490118a7e7f6fc1a9ac2e4025d983b0f402c8ca`
- `reference_commit: c1915b0a63bfabebdb95a515cedfee05146c1d50`
- `release_gate_status: passed`
- artifacts: `bitaxe-ultra205.elf`, `esp-miner.bin`, `www.bin`,
  `bitaxe-ultra205-factory.bin`, `firmware/bitaxe/partitions-ultra205.csv`,
  and `otadata-initial.bin`

Evidence:

- `docs/parity/evidence/phase-16-current-commit-release-evidence-completion/package-release-gate.md`
- `docs/parity/evidence/phase-16-current-commit-release-evidence-completion/package-release-gate/package-command.log`
- `docs/parity/evidence/phase-16-current-commit-release-evidence-completion/package-release-gate/release-gate.log`
- `docs/parity/evidence/phase-16-current-commit-release-evidence-completion/package-release-gate/bitaxe-ultra205-package.json`

## Serial Boot

Status: passed.

`just detect-ultra205` found one Ultra 205 ESP32-S3 port and board-info
succeeded. `just flash-monitor board=205 ...` then flashed the factory image
from the package manifest and captured trusted wrapper output.

Supported serial claims:

- board `205`, Ultra 205, BM1366 identity
- firmware commit marker `8490118a7e7f`
- reference commit marker `c1915b0a63bfabebdb95a515cedfee05146c1d50`
- reset reason and factory partition boot
- `spiffs_mount=available`
- route registration for `/recovery`, `/api/system/OTA`, `/api/system/OTAWWW`,
  `/api/*`, and `/*`
- `safe_state: mining=disabled asic_work_submission=disabled hardware_control=disabled`
- `ota_boot_validation=not_pending state=factory`

This does not prove live HTTP route behavior, valid OTA, rollback, failed-update,
large erase, interrupted update, or OTAWWW behavior.

## HTTP Static Recovery

Status: blocked.

`DEVICE_URL` was absent, so the helper wrote blocked evidence with
`network_scan: disabled` and did not infer a target from serial logs or local
network state.

Rows that remain below verified from this pack:

- live `/`
- live `/assets/app.css.gz`
- missing static redirect/body
- live `/recovery`
- live API route coexistence
- live `/api/ws` and `/api/ws/live`
- live `POST /api/system/OTA` route presence
- live `POST /api/system/OTAWWW` gap response

## Firmware OTA

Status: blocked.

The firmware OTA preflight stopped before detector rerun or upload because
`DEVICE_URL` was missing.

No invalid firmware upload, valid OTA upload, selected next partition, reboot
scheduling, post-reboot identity, boot-validation proof, rollback proof, or
AP/APSTA rejection proof was captured.

## Recovery Regression

Status: pending.

The Phase 16 recovery helper ran with no unsafe allow flags. It produced pending
markers and did not run failed-update upload, invalid firmware generation, large
erase, factory reflash, post-restore monitor capture, interrupted upload,
detector rerun, board-info rerun, or post-action HTTP/static proof.

Rollback and boot-validation also remain blocked because firmware OTA did not
run in Plan 16-04.

## OTAWWW REL-03 Gap

Status: deferred / below verified.

OTAWWW remains the REL-03 static-update gap. Phase 16 did not capture a live
`Wrong API input` response, a whole-`www` partition update, recovery access
after static update, or interrupted-update hardware-regression evidence. A
generated `www.bin` artifact and route registration are not OTAWWW parity proof.

## Redaction Review

redaction_status: passed.

`docs/parity/evidence/phase-16-current-commit-release-evidence-completion/redaction-review.md`
reviewed the package, detector, flash JSON, serial, blocked HTTP, blocked OTA,
pending recovery, failed-update, interrupted-update, and large-erase artifacts.

Absent live body, header, curl error, WebSocket, OTA upload, detector rerun,
board-info, erase, reflash, and interrupted-upload artifacts are marked
`absent - not cited`. The scan found expected labels, ESP boot terms, USB/MAC
board identity, and missing-`DEVICE_URL` markers only; no private endpoint,
pool credential, worker secret, Wi-Fi credential, API token, NVS secret value,
or local terminal secret was found.

## Exact Claims Supported

- Phase 16 package manifest and release gate passed for source commit
  `8490118a7e7f6fc1a9ac2e4025d983b0f402c8ca`.
- Phase 16 detector-gated wrapper flash-monitor evidence passed for board `205`
  with trusted output at the same release-candidate source commit.
- Serial boot supports firmware identity, reference identity, factory partition,
  reset reason, SPIFFS availability, route registration, safe no-mining/no-control
  state, and factory boot-validation status.
- Phase 16 helpers respected explicit-`DEVICE_URL`, current-manifest, detector,
  and allow-flag gates by producing blocked or pending evidence instead of
  scanning, uploading, erasing, interrupting, or issuing raw recovery commands.
- All cited Phase 16 artifacts passed final redaction review.

## Claims Remaining Below Verified

- Live HTTP/static/recovery/API/WebSocket behavior.
- Valid firmware OTA upload.
- Invalid image rejection.
- AP/APSTA OTA rejection.
- Selected next OTA partition.
- Reboot scheduling and post-reboot firmware identity after OTA.
- Rollback and boot-validation proof after OTA.
- Failed-update recovery.
- Large erase recovery and factory restore.
- Interrupted-update recovery.
- OTAWWW whole-`www` update behavior.
- Final release parity for rows that require any blocked or pending live evidence above.

## Residual Risks

- A reachable explicit `DEVICE_URL` is still required before live HTTP/static,
  recovery, firmware OTA, and OTAWWW evidence can run.
- A refreshed package manifest must match the git HEAD used for any future live
  OTA or recovery run.
- Destructive or fault-injection recovery work still requires detector-approved
  board `205`, current manifest and factory image, explicit allow flags, recovery
  steps, abort conditions, and redaction review.
- The package and serial evidence are strong for the flashed release-candidate
  commit, but later documentation/evidence commits were not reflashed and must
  not be treated as firmware proof.

## Final Verification

Final verification is recorded in
`.planning/phases/16-current-commit-release-evidence-completion/16-VERIFICATION.md`.
This ledger supports a final phase status only if the required Task 3 checks,
`release-evidence --require-redaction-passed`, `just parity`,
`just verify-reference`, reference diff, and lifecycle validation pass while
preserving the blocked and pending evidence boundaries above.
