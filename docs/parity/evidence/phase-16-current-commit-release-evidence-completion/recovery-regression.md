---
recovery_regression_status: pending
source_commit: b55d3e68b68060fc6cf271372a75fc86c0a934c6
reference_commit: c1915b0a63bfabebdb95a515cedfee05146c1d50
package_manifest: docs/parity/evidence/phase-16-current-commit-release-evidence-completion/package-release-gate/bitaxe-ultra205-package.json
factory_image: bazel-bin/firmware/bitaxe/bitaxe-ultra205-factory.bin
ota_image: bazel-bin/firmware/bitaxe/esp-miner.bin
failed_update_status: pending - allow flag not provided
failed_update_boundary: not run - no invalid firmware upload, public status, body, post-failure state, or recovery proof captured
large_erase_status: pending - allow flag not provided
large_erase_restore_status: pending - erase and factory reflash not run
post_restore_boot_identity_status: pending - post-restore monitor not captured
post_restore_http_static_status: pending - DEVICE_URL missing and no post-restore probe ran
interrupted_update_status: pending - allow flag not provided
post_interruption_operability_status: pending - interrupted upload and post-failure probes not run
rollback_status: blocked - Plan 16-04 OTA did not run
boot_validation_status: blocked - Plan 16-04 OTA did not run
otawww_rel03_status: deferred - Wrong API input alone is not proof and no whole-www interrupted-update regression evidence exists
whole_www_update_status: deferred - no whole-www update behavior captured
redaction_status: passed - generated Plan 16-05 pending logs reviewed; live body/header/detector artifacts absent - not cited
checklist_promotion_boundary: no rollback, boot-validation, failed-update, large-erase, interrupted-update, or OTAWWW row may be promoted to verified from this pending artifact
conclusion: pending - recovery regression actions did not run because unsafe allow flags were omitted under the current Phase 16 blockers
---

# Phase 16 Recovery Regression Evidence

recovery_regression_status: pending

The Phase 16 recovery regression helper ran through the documented command
surface with the Plan 16-02 selected port, but with no unsafe allow flags. That
was the correct branch for this run because Plan 16-03 recorded no explicit
reachable `DEVICE_URL`, and Plan 16-04 recorded that firmware OTA preflight was
blocked before detector rerun or upload.

## Evidence Inputs

| Field | Value |
| --- | --- |
| source_commit | `b55d3e68b68060fc6cf271372a75fc86c0a934c6` |
| reference_commit | `c1915b0a63bfabebdb95a515cedfee05146c1d50` |
| current git HEAD before Task 1 commit | `e5b6303f3abbe7fa240d04e2e44e9e444545e6d1` |
| package_manifest | `docs/parity/evidence/phase-16-current-commit-release-evidence-completion/package-release-gate/bitaxe-ultra205-package.json` |
| factory_image | `bazel-bin/firmware/bitaxe/bitaxe-ultra205-factory.bin` |
| ota_image | `bazel-bin/firmware/bitaxe/esp-miner.bin` |
| selected Plan 16-02 port | `/dev/cu.usbmodem1101` |
| DEVICE_URL status | not provided |
| network_scan | disabled |
| helper log | `docs/parity/evidence/phase-16-current-commit-release-evidence-completion/recovery-regression/recovery-regression.log` |

## Operation Results

| Evidence field | Status |
| --- | --- |
| failed_update_status | pending - allow flag not provided |
| failed_update_boundary | not run - no invalid firmware upload, route response, artifact checksum, failure point, post-failure state, or recovery proof captured |
| large_erase_status | pending - allow flag not provided |
| large_erase_restore_status | pending - erase and factory reflash not run |
| post_restore_boot_identity_status | pending - post-restore monitor not captured |
| post_restore_http_static_status | pending - `DEVICE_URL` missing and no post-restore probe ran |
| interrupted_update_status | pending - allow flag not provided |
| post_interruption_operability_status | pending - interrupted upload and post-failure probes not run |
| rollback_status | blocked - Plan 16-04 OTA did not run |
| boot_validation_status | blocked - Plan 16-04 OTA did not run |
| otawww_rel03_status | deferred - `Wrong API input` alone is not proof and no whole-www interrupted-update hardware-regression evidence exists |
| whole_www_update_status | deferred - no whole-www update behavior captured |

## Claim Boundary

checklist_promotion_boundary: no rollback, boot-validation, failed-update,
large-erase, interrupted-update, or OTAWWW row may be promoted to verified from
this pending artifact.

Invalid image rejection remains separate failed-update evidence and is not
rollback proof. In this plan it was not observed because `--allow-failed-update`
was omitted. Large erase was not run because `--allow-large-erase` was omitted.
Interrupted OTA was not run because `--allow-interrupted-ota` was omitted.

OTAWWW remains the REL-03 gap. A `Wrong API input` response alone would not
verify OTAWWW, and this plan did not capture whole-`www` update behavior,
recovery access, or interrupted-update hardware-regression evidence.

## Redaction

redaction_status: passed - generated Plan 16-05 pending logs reviewed; live
body/header/detector artifacts absent - not cited.

Generated recovery logs contain source/reference commits, local artifact paths,
the retained USB port needed for board identity, a missing `DEVICE_URL` marker,
and pending operation statuses. No failed-update body, failed-update headers,
invalid firmware artifact, detector rerun transcript, large-erase command
transcript, post-restore monitor capture beyond the pending marker, interrupted
OTA body, curl error, private endpoint value, IP address, Wi-Fi credential, pool
credential, API token, NVS secret value, or terminal secret is cited.

## Conclusion

conclusion: pending - recovery regression actions did not run because unsafe
allow flags were omitted under the current Phase 16 blockers. This artifact
preserves the release claim boundary without using raw erase, raw write,
voltage/fan/mining stress, network scanning, or commands outside
`scripts/phase16-recovery-regression.sh`.
