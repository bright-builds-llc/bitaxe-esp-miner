# Phase 18 Firmware OTA And Rollback Evidence Summary

package_status: passed

release_gate_status: passed

detector_status: passed

flash_monitor_status: passed

target_lock_status: passed

firmware_ota_status: blocked - valid upload response was observed, but post-OTA identity and boot-validation markers were not captured

invalid_rejection_status: passed - fixed invalid firmware fixture returned HTTP 500 with `Write Error`

valid_ota_status: below verified - manifest image upload returned HTTP 200, but post-OTA markers were missing

post_reboot_identity_status: blocked - `firmware_commit=` and `reference_commit=` were absent from `firmware-ota/post-ota-monitor.log`

selected_next_partition: unavailable - public route did not expose the selected next partition and no post-OTA boot-validation marker was captured

boot_validation_status: blocked - `ota_boot_validation=` was absent from `firmware-ota/post-ota-monitor.log`

rollback_status: non-claim - invalid rejection and upload acceptance do not prove rollback behavior

destructive_rollback_status: non-claim - no destructive rollback, forced boot failure, erase, or interrupted-update procedure ran in Phase 18

redaction_status: passed

network_scan: disabled

board: `205`

source_commit: `22d02f8e97928f1ec29360552179380b92582e6a`

reference_commit: `c1915b0a63bfabebdb95a515cedfee05146c1d50`

## Scope

Phase 18 records current Ultra 205 package identity, trusted USB flash-monitor
target provenance, one invalid firmware rejection attempt, one valid firmware
upload response, a bounded post-OTA monitor capture, final redaction review, and
conservative release/checklist traceability. It does not perform destructive
rollback, forced boot failure, large erase, interrupted update, OTAWWW update,
mining, soak, or active safety-control verification.

## Exact Commands

Package and release gate:

```bash
just package
bazel run //tools/parity:report -- release-gate --manifest docs/parity/evidence/phase-18-firmware-ota-and-rollback-evidence/package-release-gate/bitaxe-ultra205-package.json
```

Detector and board-info gate:

```bash
just detect-ultra205
espflash board-info --chip esp32s3 --port /dev/cu.usbmodem1101 --non-interactive
```

Flash-monitor evidence:

```bash
just flash-monitor board=205 port=/dev/cu.usbmodem1101 manifest=bazel-bin/firmware/bitaxe/bitaxe-ultra205-package.json wifi-credentials=wifi-credentials.json evidence-dir=docs/parity/evidence/phase-18-firmware-ota-and-rollback-evidence/serial-boot capture-timeout-seconds=45 redact-evidence=true
```

Target-lock and OTA evidence:

```bash
scripts/phase18-firmware-ota-evidence.sh --manifest docs/parity/evidence/phase-18-firmware-ota-and-rollback-evidence/package-release-gate/bitaxe-ultra205-package.json --ota-image bazel-bin/firmware/bitaxe/esp-miner.bin --port /dev/cu.usbmodem1101 --out-dir docs/parity/evidence/phase-18-firmware-ota-and-rollback-evidence --target-lock-out docs/parity/evidence/phase-18-firmware-ota-and-rollback-evidence/target-lock.json --monitor-seconds 45 --use-flash-log-device-url target/phase18-firmware-ota-and-rollback-evidence-dev-raw/flash-command-evidence.json
scripts/phase13-monitor-capture.sh --port /dev/cu.usbmodem1101 --out docs/parity/evidence/phase-18-firmware-ota-and-rollback-evidence/firmware-ota/post-ota-monitor.log --seconds 45 --no-reset
```

The raw local flash evidence path in the command above is intentionally
untracked. The committed target lock records only redacted provenance.

## Artifact Matrix

| Artifact | Status | Claim boundary |
| --- | --- | --- |
| `docs/parity/evidence/phase-18-firmware-ota-and-rollback-evidence/package-release-gate.md` | cited | Package and release-gate status, manifest path, source/reference commits, and checksums. |
| `docs/parity/evidence/phase-18-firmware-ota-and-rollback-evidence/package-release-gate/bitaxe-ultra205-package.json` | cited | Manifest for board `205`; `esp-miner.bin` SHA-256 `c7f4a62872d6662562f89ff8e93e881317d65b8f58003577acfd6a0a50eb6463`. |
| `docs/parity/evidence/phase-18-firmware-ota-and-rollback-evidence/serial-boot.md` | cited | Detector and wrapper-owned flash-monitor identity, factory boot, redacted target provenance, and serial non-claims. |
| `docs/parity/evidence/phase-18-firmware-ota-and-rollback-evidence/serial-boot/detect-ultra205.log` | cited | Detector output for exactly one likely Ultra 205 USB port, with MAC redacted. |
| `docs/parity/evidence/phase-18-firmware-ota-and-rollback-evidence/serial-boot/flash-command-evidence.json` | cited | Wrapper-owned flash-monitor command metadata, board, port, source/reference commits, redaction mode, and trusted output. |
| `docs/parity/evidence/phase-18-firmware-ota-and-rollback-evidence/serial-boot/flash-monitor.log` | cited | Factory boot identity, route registration, Wi-Fi connected with redacted values, `firmware_commit=22d02f8e9792`, `reference_commit=c1915b0a63bfabebdb95a515cedfee05146c1d50`, and `ota_boot_validation=not_pending state=factory`. |
| `docs/parity/evidence/phase-18-firmware-ota-and-rollback-evidence/target-lock.json` | cited | Sanitized target provenance, `target_status: passed`, and `network_scan: disabled`. |
| `docs/parity/evidence/phase-18-firmware-ota-and-rollback-evidence/firmware-ota.md` | cited | Claim-specific OTA ledger separating invalid rejection, valid upload response, post-OTA blockers, rollback non-claim, and non-claims. |
| `docs/parity/evidence/phase-18-firmware-ota-and-rollback-evidence/firmware-ota/firmware-ota-smoke.log` | cited | Phase 18 wrapper and nested Phase 13 helper transcript. Records invalid rejection passed and valid upload response, then blocks on missing post-OTA markers. |
| `docs/parity/evidence/phase-18-firmware-ota-and-rollback-evidence/firmware-ota/invalid-firmware.bin` | cited | Fixed invalid binary fixture with SHA-256 `f08b10f0b8314e397b07526f18a5f81d79da9c20518bf5fd1b720c256d1a9294`. |
| `docs/parity/evidence/phase-18-firmware-ota-and-rollback-evidence/firmware-ota/invalid-firmware-ota.headers.txt` | cited | Invalid OTA selected response headers. |
| `docs/parity/evidence/phase-18-firmware-ota-and-rollback-evidence/firmware-ota/invalid-firmware-ota.body.txt` | cited | Invalid OTA body marker: `Write Error`. |
| `docs/parity/evidence/phase-18-firmware-ota-and-rollback-evidence/firmware-ota/invalid-firmware-ota.curl-error.txt` | cited | Empty curl error artifact. |
| `docs/parity/evidence/phase-18-firmware-ota-and-rollback-evidence/firmware-ota/valid-firmware-ota.headers.txt` | cited | Valid OTA selected response headers. |
| `docs/parity/evidence/phase-18-firmware-ota-and-rollback-evidence/firmware-ota/valid-firmware-ota.body.txt` | cited | Valid OTA body marker: `Firmware update complete, rebooting now!`. |
| `docs/parity/evidence/phase-18-firmware-ota-and-rollback-evidence/firmware-ota/valid-firmware-ota.curl-error.txt` | cited | Empty curl error artifact. |
| `docs/parity/evidence/phase-18-firmware-ota-and-rollback-evidence/firmware-ota/post-ota-monitor.log` | cited | Bounded no-reset monitor had empty capture output and no required post-OTA identity or boot-validation markers. |
| `docs/parity/evidence/phase-18-firmware-ota-and-rollback-evidence/redaction-review.md` | cited | Final redaction gate with `redaction_status: passed`. |

## Package And Flash Identity

The copied package manifest and wrapper-owned flash evidence use source commit
`22d02f8e97928f1ec29360552179380b92582e6a` and reference commit
`c1915b0a63bfabebdb95a515cedfee05146c1d50`. The flash-monitor wrapper reported
trusted output and observed `firmware_commit=22d02f8e9792` with the same
reference commit in the factory-boot serial log.

## Target Provenance

target_lock_status: passed

The committed target lock was created from trusted USB flash-monitor evidence,
records `device_url_redacted: http://[redacted]`, and keeps `network_scan:
disabled`. The local developer raw evidence used for target extraction remains
under `target/phase18-firmware-ota-and-rollback-evidence-dev-raw/` and is not
committed.

## Invalid Rejection

invalid_rejection_status: passed

The fixed invalid firmware fixture was uploaded to `POST /api/system/OTA` and
returned HTTP 500 with body marker `Write Error`. This proves invalid image
rejection for the captured route attempt only. It does not prove rollback,
post-OTA boot validation, selected partition behavior, interrupted-update
recovery, large erase recovery, or OTAWWW behavior.

## Valid OTA

valid_ota_status: below verified

The manifest-listed `esp-miner.bin` SHA-256 matched the packaged artifact and
the upload returned HTTP 200 with body marker `Firmware update complete,
rebooting now!`. The helper then attempted a bounded no-reset post-OTA monitor
capture, but no `firmware_commit=`, `reference_commit=`, or
`ota_boot_validation=` marker appeared in `firmware-ota/post-ota-monitor.log`.
The valid upload response is evidence of route acceptance only; it is not enough
to promote valid OTA, reboot identity, selected partition, or boot validation to
verified.

## Reboot Identity

post_reboot_identity_status: blocked

Phase 18 has factory-boot identity from `serial-boot/flash-monitor.log`, but the
post-OTA monitor captured no output between `capture_output_start` and
`capture_output_end`. Post-reboot identity after the valid upload remains
blocked.

## Selected Partition

selected_next_partition: unavailable

The public OTA route did not expose selected next partition information. The
helper required post-OTA boot-validation markers as the supporting evidence, and
those markers were absent.

## Boot Validation

boot_validation_status: blocked

`serial-boot/flash-monitor.log` recorded factory boot state
`ota_boot_validation=not_pending state=factory` before the OTA upload. The
post-OTA monitor did not capture `ota_boot_validation=`, so Phase 18 does not
prove post-OTA boot validation.

## Rollback

rollback_status: non-claim

No bootloader rollback transition, ESP-IDF rollback state transition, forced
boot failure, recovery command, erase command, or interrupted-update procedure
ran in Phase 18. Invalid image rejection and valid upload response are not
rollback evidence.

## Checklist Promotion Matrix

| Row | Phase 18-supported update | Required boundary |
| --- | --- | --- |
| `OTA-001` | Keep `implemented`; add Phase 18 invalid rejection and valid upload-response evidence. | Do not mark `verified` because valid OTA and boot-validation terms are blocked by missing post-OTA markers. |
| `REL-001` | Keep `implemented`; cite package/flash identity and factory partition evidence only. | Do not claim selected next partition or interrupted-update partition behavior. |
| `REL-002` | Keep `implemented`; cite factory boot `ota_boot_validation=not_pending state=factory` only. | Do not claim post-OTA boot validation or rollback behavior. |
| `REL-003` | Keep `implemented`; cite package, release-gate, flash-monitor, target, redaction, invalid rejection, and upload-response evidence. | Do not mark `verified`; rollback, recovery, large erase, failed-update recovery beyond invalid rejection, and interrupted-update terms are not satisfied. |
| `REL-02` | Requirement traceability may record partial Phase 18 evidence. | Release parity cannot rely on valid OTA or rollback until markers exist. |
| `REL-08` | Requirement traceability may record invalid rejection and non-claim boundaries. | Destructive rollback/recovery cases remain pending or future-owned. |
| `REL-07` | Operator docs may cite exact Phase 18 artifacts. | Docs must preserve unsupported non-claims. |
| `EVD-05` | Evidence layer includes helper tests, package, release-gate, parity, reference, redaction, and lifecycle validation. | Hardware conclusions remain scoped to captured artifacts. |

## Explicit Non-Claims

- Valid OTA remains below verified because post-OTA firmware identity and
  boot-validation markers were not captured.
- Reboot identity after the valid upload remains blocked.
- Selected next partition remains unavailable.
- Boot validation after the valid upload remains blocked.
- Rollback and destructive rollback remain non-claims.
- Failed-update recovery beyond invalid image rejection remains below verified.
- Large erase recovery did not run.
- Interrupted-update recovery did not run.
- OTAWWW whole-`www` update behavior did not run.
- Active safety-control behavior, mining, pool behavior, share behavior, and
  soak behavior did not run.

## Conclusions

Phase 18 passed package identity, release-gate, detector, factory flash-monitor,
target-lock, invalid firmware rejection, redaction review, and documentation
traceability preparation for board `205`. It also recorded a valid upload HTTP
200 response for the manifest image. The final release claim boundary remains
conservative: valid OTA, post-OTA boot validation, selected partition, rollback,
destructive rollback, failed-update recovery beyond invalid rejection, large
erase, interrupted update, OTAWWW, mining, active safety, and soak behavior are
below verified or non-claims.

## Residual Risks

- The valid upload may have rebooted outside the bounded no-reset monitor
  window, but no committed artifact proves post-reboot identity.
- Same-image upload behavior may differ from a true version-changing OTA, and
  the current evidence does not distinguish that path.
- Future rollback or destructive recovery evidence still needs a documented
  allow gate, abort conditions, restore path, and safe-state markers before any
  release parity claim.
