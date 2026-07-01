---
firmware_ota_status: blocked
device_url_status: blocked - missing DEVICE_URL
detector_status: skipped - preflight blocked before detector rerun
manifest_source_commit: b55d3e68b68060fc6cf271372a75fc86c0a934c6
manifest_reference_commit: c1915b0a63bfabebdb95a515cedfee05146c1d50
preflight_git_head: 50b5868cc444ff91431865212983721ea59a52cb
ota_image_path: bazel-bin/firmware/bitaxe/esp-miner.bin
ota_image_sha256: a494f743187fd9ab137251687a5c7cc59b0486f9594b4b3e946ec74081ddd7cd
invalid_rejection_status: blocked - OTA did not run
invalid_rejection_boundary: not rollback proof
valid_ota_status: blocked - OTA did not run
selected_next_partition: unavailable - OTA did not run
reboot_scheduling_status: blocked - OTA did not run
post_reboot_identity_status: blocked - OTA did not run
boot_validation_status: blocked - OTA did not run
rollback_status: blocked - OTA did not run
ap_apsta_rejection_status: not applicable - device was not contacted
redaction_status: passed - generated blocked logs reviewed; upload bodies and monitor log absent - not cited
checklist_promotion_boundary: no firmware OTA, invalid-rejection, rollback, failed-update, post-reboot identity, or boot-validation row may be promoted to verified from this blocked artifact
conclusion: blocked - package manifest source_commit did not equal the preflight git HEAD and DEVICE_URL was unavailable, so no detector rerun or upload occurred
---

# Phase 16 Firmware OTA Evidence

firmware_ota_status: blocked

The Phase 16 firmware OTA preflight stopped before the detector rerun and before any upload. The copied package manifest is still anchored to the Plan 16-02 flashed release-candidate source commit, while the repository HEAD had advanced to later evidence commits at the time of OTA preflight. No explicit `DEVICE_URL` was present.

## Evidence Inputs

| Field | Value |
| --- | --- |
| device_url_status | blocked - missing DEVICE_URL |
| detector_status | skipped - preflight blocked before detector rerun |
| network_scan | disabled |
| manifest_source_commit | `b55d3e68b68060fc6cf271372a75fc86c0a934c6` |
| manifest_reference_commit | `c1915b0a63bfabebdb95a515cedfee05146c1d50` |
| preflight_git_head | `50b5868cc444ff91431865212983721ea59a52cb` |
| ota_image_path | `bazel-bin/firmware/bitaxe/esp-miner.bin` |
| ota_image_sha256 | `a494f743187fd9ab137251687a5c7cc59b0486f9594b4b3e946ec74081ddd7cd` |
| selected Plan 16-02 port | `/dev/cu.usbmodem1101` |
| helper log | `docs/parity/evidence/phase-16-current-commit-release-evidence-completion/firmware-ota/firmware-ota-smoke.log` |
| detector rerun log | `docs/parity/evidence/phase-16-current-commit-release-evidence-completion/firmware-ota/post-ota-detect-ultra205.log` |

## OTA Results

| Evidence field | Status |
| --- | --- |
| invalid_rejection_status | blocked - OTA did not run |
| invalid_rejection_boundary | not rollback proof |
| valid_ota_status | blocked - OTA did not run |
| selected_next_partition | unavailable - OTA did not run |
| reboot_scheduling_status | blocked - OTA did not run |
| post_reboot_identity_status | blocked - OTA did not run |
| boot_validation_status | blocked - OTA did not run |
| rollback_status | blocked - OTA did not run |
| ap_apsta_rejection_status | not applicable - device was not contacted |

## Claim Boundary

checklist_promotion_boundary: no firmware OTA, invalid-rejection, rollback, failed-update, post-reboot identity, or boot-validation row may be promoted to verified from this blocked artifact.

Invalid rejection remains separate failed-update evidence and is not rollback proof. In this plan it was not observed because the helper stopped before any upload. Route presence from serial boot or HTTP blocked evidence is also not valid firmware OTA evidence.

OTAWWW remains a REL-03 static-update gap unless Plan 16-05 captures whole-www update behavior, recovery access, and interrupted-update hardware-regression.

## Redaction

redaction_status: passed - generated blocked logs reviewed; upload bodies and monitor log absent - not cited.

Only `firmware-ota-smoke.log` and `post-ota-detect-ultra205.log` were generated. No `invalid-firmware.bin`, `.headers.txt`, `.body.txt`, `.curl-error.txt`, or `post-ota-monitor.log` artifact exists because the plan blocked before detector rerun and upload. The generated logs contain source/reference commits, local paths, the retained USB port needed for board identity, a missing `DEVICE_URL` status label, and `network_scan: disabled`; they contain no private endpoint value, IP address, Wi-Fi credential, pool credential, API token, NVS secret value, or terminal secret.

## Conclusion

conclusion: blocked - package manifest source_commit did not equal the preflight git HEAD and `DEVICE_URL` was unavailable, so no detector rerun, invalid upload, valid upload, reboot monitor, boot-validation proof, rollback proof, or OTAWWW proof occurred.
