# Phase 21 Preflight Evidence

phase21_preflight_status: passed
default_safe_package_manifest: docs/parity/evidence/phase-21-live-mining-and-soak-evidence/preflight/package-release-gate/bitaxe-ultra205-package.json
controlled_live_mining_package_manifest: docs/parity/evidence/phase-21-live-mining-and-soak-evidence/live-mining-enablement/package/bitaxe-ultra205-package.json
package_command_log: docs/parity/evidence/phase-21-live-mining-and-soak-evidence/preflight/package-release-gate/package-command.log
release_gate_log: docs/parity/evidence/phase-21-live-mining-and-soak-evidence/preflight/package-release-gate/release-gate.log
release_gate_status: passed
network_scan: disabled
source_commit: a19b9e0660a315e8f0e1aa08d16e4822fd6937a6
reference_commit: c1915b0a63bfabebdb95a515cedfee05146c1d50
detector_status: passed
board: 205
detector_port: /dev/cu.usbmodem1101
board_info_command: espflash board-info --chip esp32s3 --port /dev/cu.usbmodem1101 --non-interactive
board_info_status: passed
safe_baseline_flash_status: passed
safe_baseline_capture_status: timed_out_after_trusted_output
hardware_command_status: passed
safe_state: mining=disabled
hardware_control: disabled
work_submission: disabled

## Scope

This preflight ledger records package identity and release-gate status before
Phase 21 hardware tiers run. It is not BM1366 diagnostic, live mining, share,
soak, API/WebSocket telemetry, frequency-transition, voltage-control, fan, OTA,
erase, rollback, or interrupted-update evidence.

## Package Evidence

| Field | Value |
|-------|-------|
| Package command | `just package` |
| Package command log | `preflight/package-release-gate/package-command.log` |
| Release-gate command | `bazel run //tools/parity:report -- release-gate --manifest docs/parity/evidence/phase-21-live-mining-and-soak-evidence/preflight/package-release-gate/bitaxe-ultra205-package.json` |
| Release-gate log | `preflight/package-release-gate/release-gate.log` |
| Default safe package manifest | `preflight/package-release-gate/bitaxe-ultra205-package.json` |
| Controlled live-mining manifest | `live-mining-enablement/package/bitaxe-ultra205-package.json` |
| Source commit | `a19b9e0660a315e8f0e1aa08d16e4822fd6937a6` |
| Reference commit | `c1915b0a63bfabebdb95a515cedfee05146c1d50` |

## Hardware Gate Status

| Gate | Status | Evidence |
|------|--------|----------|
| Detector command | passed | `preflight/safe-baseline/detect-ultra205.log` records `just detect-ultra205` with redacted board-info output and exactly one selected port. |
| Board-info | passed | Detector-owned `espflash board-info --chip esp32s3 --port /dev/cu.usbmodem1101 --non-interactive` completed before flash. |
| Safe baseline flash-monitor | passed | `preflight/safe-baseline/flash-command-evidence.json` and `preflight/safe-baseline/flash-monitor.log` record trusted boot output and safe-state markers. |
| Target discovery | disabled | No network scan or `DEVICE_URL` inference is allowed by this plan. |

## Safe Baseline Evidence

| Field | Value |
|-------|-------|
| Flash-monitor command | `just flash-monitor board=205 port=/dev/cu.usbmodem1101 evidence-dir=docs/parity/evidence/phase-21-live-mining-and-soak-evidence/preflight/safe-baseline capture-timeout-seconds=45 redact-evidence=true wifi-credentials=wifi-credentials.json` |
| Flash command evidence | `preflight/safe-baseline/flash-command-evidence.json` |
| Flash monitor log | `preflight/safe-baseline/flash-monitor.log` |
| Capture status | `timed_out_after_trusted_output` |
| Redaction mode | `commit-redacted` |
| Wi-Fi credentials source | `provided-redacted`; file path was passed to the repo wrapper and file contents were not read or committed. |
| Safe-state marker | `mining=disabled`, `hardware_control=disabled`, `work_submission=disabled` |

## Non-Claims

- production_mining: not claimed
- accepted_shares: not claimed
- rejected_shares: not claimed
- bm1366_chip_detect: not claimed
- bm1366_work_send: not claimed
- bm1366_result_receive: not claimed
- bounded_soak: not claimed
- live_api_websocket_telemetry: not claimed
- frequency_transition: not claimed
- voltage_control: not claimed
- fan_control: not claimed
- firmware_ota: not claimed
- erase: not claimed
- rollback: not claimed
- interrupted_update: not claimed

## Downstream Use

Later Phase 21 diagnostic, smoke, and soak plans may consume the package
identity from this file and from the copied default safe manifest. This file
does not authorize network target discovery, destructive/fault-injection
commands, live mining, or soak claims.
