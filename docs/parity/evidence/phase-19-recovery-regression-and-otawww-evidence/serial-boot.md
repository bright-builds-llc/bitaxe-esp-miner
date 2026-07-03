# Phase 19 Detector-Gated Serial Boot Evidence

detector_status: passed
board_info_status: passed
flash_monitor_status: passed
board: 205
selected_port: /dev/cu.usbmodem1101
source_commit: 6842d7a6d3d4fc64d93900a9847c8a0b97edc16d
reference_commit: c1915b0a63bfabebdb95a515cedfee05146c1d50
manifest_path: docs/parity/evidence/phase-19-recovery-regression-and-otawww-evidence/package-release-gate/bitaxe-ultra205-package.json
trusted_output: true
redaction_mode: commit-redacted
network_scan: disabled
target_lock_status: blocked - no explicit origin-only target

## Command Evidence

- Detector command: `just detect-ultra205`
- Detector log: `docs/parity/evidence/phase-19-recovery-regression-and-otawww-evidence/serial-boot/detect-ultra205.log`
- Flash-monitor command: `just flash-monitor board=205 port=/dev/cu.usbmodem1101 manifest=bazel-bin/firmware/bitaxe/bitaxe-ultra205-package.json evidence-dir=docs/parity/evidence/phase-19-recovery-regression-and-otawww-evidence/serial-boot capture-timeout-seconds=45 redact-evidence=true`
- Flash-monitor evidence JSON: `docs/parity/evidence/phase-19-recovery-regression-and-otawww-evidence/serial-boot/flash-command-evidence.json`
- Flash-monitor log: `docs/parity/evidence/phase-19-recovery-regression-and-otawww-evidence/serial-boot/flash-monitor.log`

## Observed Evidence

- Detector found exactly one likely ESP USB serial port and board-info succeeded for ESP32-S3.
- Flash-monitor wrote the factory image for board `205`, captured trusted boot markers, and timed out only after trusted output.
- The observed firmware marker `6842d7a6d3d4` matches the copied package manifest source commit.
- The observed reference marker matches `c1915b0a63bfabebdb95a515cedfee05146c1d50`.
- Wi-Fi credentials, if used for local bring-up, were passed by path only and are represented in committed evidence as `provided-redacted`.

## Non-Claims

failed-update non-claim: no failed-update upload or recovery behavior was run or verified in this task.
large erase non-claim: no large erase, factory restore regression, or destructive recovery action was run in this task.
interrupted update non-claim: no interrupted firmware upload, rollback, or boot-validation regression was run in this task.
OTAWWW non-claim: no whole-www OTAWWW update, chunked erase/write behavior, recovery access after OTAWWW, or interrupted OTAWWW regression was run in this task.

## Target Boundary

No raw origin-only target evidence exists under `target/phase19-recovery-regression-and-otawww-evidence-dev-raw/serial-boot/flash-command-evidence.json`, so `target-lock.json` is intentionally blocked with `network_scan: disabled`. The committed redacted flash-monitor log is not used to infer a device URL.
