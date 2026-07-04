# Phase 21 Preflight Evidence

phase21_preflight_status: package_ready
default_safe_package_manifest: docs/parity/evidence/phase-21-live-mining-and-soak-evidence/preflight/package-release-gate/bitaxe-ultra205-package.json
controlled_live_mining_package_manifest: docs/parity/evidence/phase-21-live-mining-and-soak-evidence/live-mining-enablement/package/bitaxe-ultra205-package.json
package_command_log: docs/parity/evidence/phase-21-live-mining-and-soak-evidence/preflight/package-release-gate/package-command.log
release_gate_log: docs/parity/evidence/phase-21-live-mining-and-soak-evidence/preflight/package-release-gate/release-gate.log
release_gate_status: passed
network_scan: disabled
source_commit: caf67601f1cc8087346cb3f7032bd750504e1401
reference_commit: c1915b0a63bfabebdb95a515cedfee05146c1d50
detector_status: pending
safe_baseline_flash_status: pending
hardware_command_status: pending

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
| Source commit | `caf67601f1cc8087346cb3f7032bd750504e1401` |
| Reference commit | `c1915b0a63bfabebdb95a515cedfee05146c1d50` |

## Hardware Gate Status

| Gate | Status | Evidence |
|------|--------|----------|
| Detector command | pending | Task 2 owns `just detect-ultra205` output. |
| Board-info | pending | Task 2 owns board-info evidence after detector pass. |
| Safe baseline flash-monitor | pending | Task 2 owns safe-baseline flash evidence or blocked status. |
| Target discovery | disabled | No network scan or `DEVICE_URL` inference is allowed by this plan. |

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
identity from this file and from the copied default safe manifest only after
Task 2 records detector and board-info status as passed or blocked.
