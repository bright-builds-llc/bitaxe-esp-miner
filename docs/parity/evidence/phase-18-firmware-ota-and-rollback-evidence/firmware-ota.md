# Phase 18 Firmware OTA Evidence

firmware_ota_status: blocked - post-OTA monitor missing required identity or boot-validation markers

device_url_status: provided - redacted origin from trusted USB flash-monitor evidence

target_lock_status: passed

detector_status: passed - immediate pre-OTA detector rerun found board 205 on `/dev/cu.usbmodem1101`

flash_monitor_status: passed - Plan 18-02 wrapper-owned flash-monitor evidence is trusted for the same board, port, source commit, and reference commit

manifest_source_commit: 22d02f8e97928f1ec29360552179380b92582e6a

manifest_reference_commit: c1915b0a63bfabebdb95a515cedfee05146c1d50

ota_image_path: esp-miner.bin

ota_image_sha256: c7f4a62872d6662562f89ff8e93e881317d65b8f58003577acfd6a0a50eb6463

invalid_fixture_sha256: f08b10f0b8314e397b07526f18a5f81d79da9c20518bf5fd1b720c256d1a9294

invalid_rejection_status: passed - fixed invalid firmware fixture returned HTTP 500 with OTA write error marker

invalid_rejection_http_status: 500

invalid_rejection_body_marker: Write Error

invalid_rejection_boundary: rejection-only evidence; does not prove rollback

valid_ota_status: below verified - HTTP 200 accepted the manifest image, but post-OTA identity and boot-validation markers were missing

valid_ota_http_status: 200

valid_ota_body_marker: Firmware update complete, rebooting now!

same_image_boundary: same-image OTA remains below verified because checksum match and HTTP acceptance were not followed by captured post-OTA markers

selected_next_partition: unavailable - public route does not expose partition; boot-validation marker was required and missing

reboot_scheduling_status: accepted response observed, but bounded no-reset monitor captured no post-reboot identity markers

post_reboot_identity_status: blocked - `firmware_commit=` and `reference_commit=` were missing from `post-ota-monitor.log`

boot_validation_status: blocked - `ota_boot_validation=` was missing from `post-ota-monitor.log`

rollback_status: non-claim - invalid rejection, HTTP acceptance, and missing post-OTA markers do not establish rollback behavior

destructive_rollback_status: non-claim - not exercised in Phase 18

safe_post_ota_status: below verified - post-OTA monitor did not capture safe-state markers

network_scan: disabled

redaction_status: pending

checklist_promotion_boundary: invalid firmware rejection may be cited as invalid-rejection evidence only; valid OTA, boot validation, selected partition, rollback, destructive rollback, whole-www update behavior, interrupted update, large erase, active safety, mining, and soak behavior are not promoted by Plan 18-03.

conclusion: Phase 18 recorded a bounded firmware OTA attempt against board 205 using the manifest-listed `esp-miner.bin` and trusted target provenance. Invalid firmware rejection was captured. The valid firmware upload returned HTTP 200 with the expected reboot body, but the bounded post-OTA monitor did not capture firmware identity or `ota_boot_validation=` markers, so valid OTA, boot validation, selected partition, rollback, destructive rollback, and safe post-OTA claims remain below verified.

## Artifact Evidence

| Artifact | Status | Notes |
| --- | --- | --- |
| `docs/parity/evidence/phase-18-firmware-ota-and-rollback-evidence/package-release-gate/bitaxe-ultra205-package.json` | cited | Manifest source and reference commits match the Plan 18-02 package identity. |
| `docs/parity/evidence/phase-18-firmware-ota-and-rollback-evidence/target-lock.json` | cited | Redacted target provenance with `network_scan: disabled`; raw target remains untracked. |
| `docs/parity/evidence/phase-18-firmware-ota-and-rollback-evidence/firmware-ota/post-ota-detect-ultra205.log` | cited | Immediate pre-OTA detector rerun for the same board 205 port, with MAC redacted. |
| `docs/parity/evidence/phase-18-firmware-ota-and-rollback-evidence/firmware-ota/firmware-ota-smoke.log` | cited | Phase 18 wrapper log and nested OTA helper transcript. |
| `docs/parity/evidence/phase-18-firmware-ota-and-rollback-evidence/firmware-ota/invalid-firmware.bin` | cited | Fixed invalid fixture used only for invalid-rejection evidence. |
| `docs/parity/evidence/phase-18-firmware-ota-and-rollback-evidence/firmware-ota/invalid-firmware-ota.headers.txt` | cited | Invalid OTA selected response headers. |
| `docs/parity/evidence/phase-18-firmware-ota-and-rollback-evidence/firmware-ota/invalid-firmware-ota.body.txt` | cited | Invalid OTA body marker: `Write Error`. |
| `docs/parity/evidence/phase-18-firmware-ota-and-rollback-evidence/firmware-ota/invalid-firmware-ota.curl-error.txt` | cited | Empty curl error artifact for invalid OTA request. |
| `docs/parity/evidence/phase-18-firmware-ota-and-rollback-evidence/firmware-ota/valid-firmware-ota.headers.txt` | cited | Valid OTA selected response headers. |
| `docs/parity/evidence/phase-18-firmware-ota-and-rollback-evidence/firmware-ota/valid-firmware-ota.body.txt` | cited | Valid OTA body marker: `Firmware update complete, rebooting now!`. |
| `docs/parity/evidence/phase-18-firmware-ota-and-rollback-evidence/firmware-ota/valid-firmware-ota.curl-error.txt` | cited | Empty curl error artifact for valid OTA request. |
| `docs/parity/evidence/phase-18-firmware-ota-and-rollback-evidence/firmware-ota/post-ota-monitor.log` | cited | Bounded monitor captured no required post-OTA identity or boot-validation markers. |

## Non-Claims

- Valid OTA remains below verified because post-OTA firmware identity and boot-validation markers were not captured.
- Boot validation remains below verified because `ota_boot_validation=` is absent from the post-OTA monitor artifact.
- Rollback and destructive rollback remain non-claims.
- OTAWWW, failed-update recovery beyond invalid rejection, interrupted update, large erase, active safety, mining, and soak behavior were not exercised in this plan.
- Raw `DEVICE_URL`, network secrets, pool secrets, API tokens, NVS sensitive values, private endpoints, IP addresses, and MAC addresses are not committed in this ledger.
