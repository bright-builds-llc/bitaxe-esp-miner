# Phase 19 Recovery Regression Evidence

recovery_regression_status: pending - no Phase 19 recovery allow flags were supplied
failed_update_status: pending - allow flag not provided
large_erase_status: pending - allow flag not provided
interrupted_update_status: pending - allow flag not provided
rollback_status: pending - rollback evidence is out of Plan 03 scope
boot_validation_status: pending - boot-validation evidence is out of Plan 03 scope
network_scan: disabled
board: 205
selected_port: /dev/cu.usbmodem1101
source_commit: 6842d7a6d3d4fc64d93900a9847c8a0b97edc16d
reference_commit: c1915b0a63bfabebdb95a515cedfee05146c1d50
manifest_path: docs/parity/evidence/phase-19-recovery-regression-and-otawww-evidence/package-release-gate/bitaxe-ultra205-package.json
target_status: blocked - no explicit origin-only target
redaction_status: pending
claim_boundary: pending evidence only; no failed-update upload, large erase, interrupted upload, rollback, or boot-validation action ran in Plan 03

## Evidence Inputs

| Input | Path | Status |
| --- | --- | --- |
| Package ledger | `docs/parity/evidence/phase-19-recovery-regression-and-otawww-evidence/package-release-gate.md` | passed package and release-gate evidence from Plan 02 |
| Package manifest | `docs/parity/evidence/phase-19-recovery-regression-and-otawww-evidence/package-release-gate/bitaxe-ultra205-package.json` | board `205`, source commit `6842d7a6d3d4fc64d93900a9847c8a0b97edc16d` |
| Serial boot ledger | `docs/parity/evidence/phase-19-recovery-regression-and-otawww-evidence/serial-boot.md` | detector and flash-monitor passed, target remains blocked |
| Target lock | `docs/parity/evidence/phase-19-recovery-regression-and-otawww-evidence/target-lock.json` | `target_status: blocked - no explicit origin-only target`, `network_scan: disabled` |

## Recovery Artifacts

| Action | Artifact | Status | Evidence boundary |
| --- | --- | --- | --- |
| Wrapper transcript | `recovery-regression/recovery-regression.log` | pending | Safe no-allow helper execution only. The transcript records omitted allow flags and no live recovery action. |
| Failed update | `recovery-regression/failed-update.log` | pending | `PHASE19_ALLOW_FAILED_UPDATE` was not set to `1`; no invalid or failed upload was sent. |
| Large erase | `recovery-regression/large-erase.log` | pending | `PHASE19_ALLOW_LARGE_ERASE` was not set to `1`; no erase, factory restore, or monitor recovery sequence ran. |
| Large erase monitor | `recovery-regression/large-erase-post-restore-monitor.log` | pending | No post-restore monitor capture exists because large erase did not run. |
| Interrupted update | `recovery-regression/interrupted-ota.log` | pending | `PHASE19_ALLOW_INTERRUPTED_OTA` was not set to `1`; no bounded upload interruption ran. |

## Action Details

failed_update_status: pending - allow flag not provided

The failed-update path needs an explicit `PHASE19_ALLOW_FAILED_UPDATE=1`,
detector and board-info gates, a current package manifest, an invalid or failed
upload result, and post-failure HTTP/static/recovery/API operability before it
can support a recovery claim. None of those live steps ran in this plan.

large_erase_status: pending - allow flag not provided

The large-erase path needs an explicit `PHASE19_ALLOW_LARGE_ERASE=1`, detector
and board-info gates, the factory image, factory restore, monitor capture, and
post-restore safe-state markers before it can support a recovery claim. None of
those live steps ran in this plan.

interrupted_update_status: pending - allow flag not provided

The interrupted-update path needs an explicit `PHASE19_ALLOW_INTERRUPTED_OTA=1`,
detector and board-info gates, a bounded upload interruption, and
post-interruption HTTP/static/recovery/API or serial safe-state proof before it
can support a recovery claim. None of those live steps ran in this plan.

rollback_status: pending - out of Plan 03 scope

boot_validation_status: pending - out of Plan 03 scope

## Restore And Safe-State Requirements

restore_command: not run - large erase was not allowed; the planned restore path remains `just flash board=205 port=/dev/cu.usbmodem1101 image=bazel-bin/firmware/bitaxe/bitaxe-ultra205-factory.bin manifest=docs/parity/evidence/phase-19-recovery-regression-and-otawww-evidence/package-release-gate/bitaxe-ultra205-package.json evidence-dir=docs/parity/evidence/phase-19-recovery-regression-and-otawww-evidence/recovery-regression/large-erase-restore`

post_action_safe_state_markers: not captured - large erase and restore did not
run. Required markers remain `firmware_commit=`,
`reference_commit=`, `safe_state: mining=disabled`, and
`spiffs_mount=available`.

## Non-Claims

- No failed-update recovery behavior was exercised.
- No large erase, factory restore, or post-restore monitor evidence was captured.
- No interrupted firmware upload behavior was exercised.
- No rollback or boot-validation behavior was exercised.
- No static/recovery/API operability after a failed or interrupted update was proven because no explicit origin-only target exists and no network scan was allowed.

## Next Plan Boundary

Plan 19-04 owns OTAWWW gap/update evidence, final redaction sign-off, release
documentation, checklist updates, and requirements traceability. This Plan 03
ledger is intentionally limited to recovery-regression pending evidence and
does not promote any REL-08 behavior beyond the artifacts listed above.
