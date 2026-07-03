# Phase 19 Recovery Regression And OTAWWW Evidence Summary

## Identity

- phase: 19
- lifecycle_id: 19-2026-07-03T17-34-52
- mode: yolo
- board: 205
- selected_port: `/dev/cu.usbmodem1101`
- source_commit: `6842d7a6d3d4fc64d93900a9847c8a0b97edc16d`
- reference_commit: `c1915b0a63bfabebdb95a515cedfee05146c1d50`
- network_scan: disabled

## Final Status

- package_status: passed
- release_gate_status: passed
- detector_status: passed
- flash_monitor_status: passed
- target_lock_status: blocked - no explicit origin-only target
- failed_update_status: pending - allow flag not provided
- large_erase_status: pending - allow flag not provided
- interrupted_update_status: pending - allow flag not provided
- rollback_status: pending - no rollback action run
- boot_validation_status: pending - no post-OTA boot-validation marker captured
- otawww_status: blocked - missing DEVICE_URL
- rel_03_status: gap documented
- rel_08_status: below verified - failed-update, large erase, interrupted update, rollback, and boot-validation evidence pending
- api_09_status: supported by prior live static/recovery/admin evidence; Phase 19 adds no whole-www update proof
- redaction_status: passed

## Verified Claim Boundary

Phase 19 verifies package/release-gate identity, Ultra 205 detector evidence,
wrapper-owned factory flash-monitor identity, blocked target-lock provenance,
recovery-regression pending status, explicit OTAWWW REL-03 gap documentation,
and final redaction status for the committed evidence set.

Phase 19 does not verify valid firmware OTA, post-OTA reboot identity, selected
next partition, post-OTA boot-validation, rollback, destructive rollback,
failed-update recovery beyond previously recorded invalid image rejection, large
erase, interrupted update, whole-`www` OTAWWW update behavior, production
mining, pool behavior, active safety telemetry, or long soak behavior.

## Artifact Matrix

| Artifact | Status | Notes |
| --- | --- | --- |
| `package-release-gate.md` | passed | Records package identity, manifest, release-gate output, and `www.bin` as package/static asset evidence only. |
| `serial-boot.md` | passed | Records detector, board-info, and wrapper-owned flash-monitor evidence with redacted committed outputs. |
| `target-lock.json` | blocked | Contains no usable device URL; no target was inferred from redacted serial evidence. |
| `recovery-regression.md` | pending | Failed-update, large erase, interrupted-update, rollback, and boot-validation evidence require explicit allow flags and a trusted target. |
| `otawww.md` | gap documented | Records REL-03 owner, blocker, operator impact, public response boundary, and follow-up path. |
| `redaction-review.md` | passed | Records final redaction review for committed Phase 19 evidence. |

## OTAWWW Gap

OTAWWW remains the explicit REL-03 gap for this release candidate. The current
public route behavior is `Wrong API input` when observed, but Phase 19 has no
trusted DEVICE_URL and did not run a live OTAWWW request. `www.bin` package
presence, static serving, route presence, and fail-closed response behavior are
insufficient proof of whole-`www` OTAWWW update parity.

The follow-up path is to implement or use a whole-`www` partition updater with
size checks, chunked erase/write behavior, recovery access, and
interrupted-update hardware-regression evidence before promoting `OTA-002` or
the OTAWWW portion of `REL-003`.

## Commands

Captured earlier in the phase:

```bash
just package
bazel run //tools/parity:report -- release-gate --manifest docs/parity/evidence/phase-19-recovery-regression-and-otawww-evidence/package-release-gate/bitaxe-ultra205-package.json
just detect-ultra205
just flash-monitor board=205 port=/dev/cu.usbmodem1101 manifest=bazel-bin/firmware/bitaxe/bitaxe-ultra205-package.json wifi-credentials=wifi-credentials.json evidence-dir=docs/parity/evidence/phase-19-recovery-regression-and-otawww-evidence/serial-boot capture-timeout-seconds=45 redact-evidence=true
scripts/phase19-recovery-otawww-evidence.sh --manifest docs/parity/evidence/phase-19-recovery-regression-and-otawww-evidence/package-release-gate/bitaxe-ultra205-package.json --out-dir docs/parity/evidence/phase-19-recovery-regression-and-otawww-evidence --target-lock-out docs/parity/evidence/phase-19-recovery-regression-and-otawww-evidence/target-lock.json --port /dev/cu.usbmodem1101 --flash-evidence-json docs/parity/evidence/phase-19-recovery-regression-and-otawww-evidence/serial-boot/flash-command-evidence.json
```

Final verification commands are recorded in
`.planning/phases/19-recovery-regression-and-otawww-evidence/19-VERIFICATION.md`.
