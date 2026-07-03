# Phase 20 Detector-Gated Safe Baseline Evidence

safe_baseline_status: passed
detector_status: passed
board_info_status: passed
flash_monitor_status: passed
board: 205
selected_port: /dev/cu.usbmodem1101
source_commit: c11fba2622a389af533774447956b95f254c0280
reference_commit: c1915b0a63bfabebdb95a515cedfee05146c1d50
package_identity_status: passed
release_gate_status: passed
manifest_path: docs/parity/evidence/phase-20-active-safety-hardware-telemetry-evidence/package-release-gate/bitaxe-ultra205-package.json
trusted_output: true
safe_state: mining=disabled
safe_state_detail: asic_work_submission=disabled
hardware_control=disabled
redaction_mode: commit-redacted
redaction_status: passed - committed detector, flash, and monitor artifacts use redacted placeholders for network and hardware identifiers and contain no credential values
network_scan: disabled
target_lock_status: blocked - no trusted raw origin-only target artifact

## Command Evidence

- Detector command: `just detect-ultra205`
- Detector log: `docs/parity/evidence/phase-20-active-safety-hardware-telemetry-evidence/safe-baseline/detect-ultra205.log`
- Flash-monitor command: `just flash-monitor board=205 port=/dev/cu.usbmodem1101 evidence-dir=docs/parity/evidence/phase-20-active-safety-hardware-telemetry-evidence/safe-baseline capture-timeout-seconds=45 redact-evidence=true wifi-credentials=wifi-credentials.json`
- Flash-monitor evidence JSON: `docs/parity/evidence/phase-20-active-safety-hardware-telemetry-evidence/safe-baseline/flash-command-evidence.json`
- Flash-monitor log: `docs/parity/evidence/phase-20-active-safety-hardware-telemetry-evidence/safe-baseline/flash-monitor.log`
- Target lock: `docs/parity/evidence/phase-20-active-safety-hardware-telemetry-evidence/target-lock.json`

## Observed Evidence

- `just detect-ultra205` found exactly one likely ESP USB serial port and board-info succeeded for ESP32-S3.
- `just flash-monitor` rebuilt the package for source commit `c11fba2622a389af533774447956b95f254c0280`, flashed board `205`, captured trusted boot output, and accepted the monitor timeout only after trusted markers were present.
- The observed firmware marker `c11fba2622a3` is a trusted prefix of the package `source_commit`.
- The observed reference marker matches `c1915b0a63bfabebdb95a515cedfee05146c1d50`.
- The committed flash-monitor log records `safe_state: mining=disabled`, `asic_work_submission=disabled`, and `hardware_control=disabled`.
- Wi-Fi credentials were passed by path only for developer bring-up and are represented in committed evidence as redacted seed status; the credential file contents were not read, printed, summarized, or committed.

## Package Refresh Note

The flash-monitor wrapper rebuilds the package before flashing. After Task 1 committed the package evidence, this Task 2 run refreshed the package manifest and release-gate log so the package ledger, copied manifest, flash evidence, and observed firmware marker all refer to source commit `c11fba2622a389af533774447956b95f254c0280`.

## Target Boundary

`target-lock.json` is blocked with `network_scan: "disabled"` because no trusted raw origin-only target artifact exists for this plan. The committed redacted serial log is not used to infer `DEVICE_URL` or device origin.

## Non-Claims

active safety non-claim: this safe baseline does not run voltage, fan, thermal, self-test, load, runtime input/display, failure-path, mining, erase, rollback, OTA, or fault-injection commands.

live telemetry non-claim: this safe baseline does not prove live HTTP, WebSocket, API freshness, telemetry cadence, route behavior, target reachability, or active-control projection.
