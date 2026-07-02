---
detector_status: passed
flash_monitor_status: passed
network_scan: disabled
board: "205"
selected_port: /dev/cu.usbmodem1101
source_commit: d9e471c9699eb0140749127416640aa1bf077d26
reference_commit: c1915b0a63bfabebdb95a515cedfee05146c1d50
manifest_path: docs/parity/evidence/phase-17-live-http-api-and-static-evidence/package-release-gate/bitaxe-ultra205-package.json
generated_manifest_path: bazel-bin/firmware/bitaxe/bitaxe-ultra205-package.json
flash_evidence_json: docs/parity/evidence/phase-17-live-http-api-and-static-evidence/serial-boot/flash-command-evidence.json
serial_log: docs/parity/evidence/phase-17-live-http-api-and-static-evidence/serial-boot/flash-monitor.log
detector_log: docs/parity/evidence/phase-17-live-http-api-and-static-evidence/serial-boot/detect-ultra205.log
recorded_at: 2026-07-02T02:57:20Z
---

# Phase 17 Serial Boot Evidence

flash_monitor_status: passed

Detector-gated `just flash-monitor` evidence captured wrapper-owned Ultra 205
serial boot output for the same source and reference commits as the copied
Phase 17 package manifest.

## Detector Gate

| Field | Value |
| --- | --- |
| detector_status | passed |
| board | `205` |
| selected port | `/dev/cu.usbmodem1101` |
| network_scan | disabled |
| detector command | `just detect-ultra205` |
| board-info command | `espflash board-info --chip esp32s3 --port /dev/cu.usbmodem1101 --non-interactive` |
| board-info output summary | ESP32-S3 revision v0.2, 16MB flash, WiFi/BLE/embedded flash features, board-info succeeded. |
| detector transcript | `docs/parity/evidence/phase-17-live-http-api-and-static-evidence/serial-boot/detect-ultra205.log` |

The detector reported exactly one `port=` line and exited with status `0`.
Flashing proceeded only after this gate passed. The USB port and MAC address in
the detector transcript are retained only as board identity evidence.

## Flash-Monitor Command

```bash
just flash-monitor board=205 port=/dev/cu.usbmodem1101 manifest=bazel-bin/firmware/bitaxe/bitaxe-ultra205-package.json evidence-dir=docs/parity/evidence/phase-17-live-http-api-and-static-evidence/serial-boot capture-timeout-seconds=35
```

The wrapper resolved the manifest to the factory image and used:

- flash_command: `espflash write-bin --chip esp32s3 --port /dev/cu.usbmodem1101 0x0 /Users/peterryszkiewicz/Repos/bitaxe-esp-miner/bazel-bin/firmware/bitaxe/bitaxe-ultra205-factory.bin`
- monitor_command: `espflash monitor --chip esp32s3 --port /dev/cu.usbmodem1101 --non-interactive`

## Package And Wrapper Identity

| Field | Value |
| --- | --- |
| copied package manifest | `docs/parity/evidence/phase-17-live-http-api-and-static-evidence/package-release-gate/bitaxe-ultra205-package.json` |
| generated package manifest | `bazel-bin/firmware/bitaxe/bitaxe-ultra205-package.json` |
| source_commit | `d9e471c9699eb0140749127416640aa1bf077d26` |
| reference_commit | `c1915b0a63bfabebdb95a515cedfee05146c1d50` |
| command_kind | `flash-monitor` |
| board | `205` |
| trusted_output | `true` |
| capture_status | `timed_out_after_trusted_output` |
| observed_firmware_commit | `d9e471c9699e` |
| observed_reference_commit | `c1915b0a63bfabebdb95a515cedfee05146c1d50` |
| wrapper evidence JSON | `docs/parity/evidence/phase-17-live-http-api-and-static-evidence/serial-boot/flash-command-evidence.json` |
| serial monitor log | `docs/parity/evidence/phase-17-live-http-api-and-static-evidence/serial-boot/flash-monitor.log` |

## Captured Log Summary

The serial log contains the required wrapper-trusted Ultra 205 markers:

- `bitaxe-rust boot: board=Ultra 205 asic=BM1366`
- `safe_state: mining=disabled asic_work_submission=disabled hardware_control=disabled`
- `ota_boot_validation=not_pending state=factory`
- `spiffs_mount=available partition=www total_bytes=2884241 used_bytes=4518`
- `axeos_api_route_shell=started manifest_routes=17 firmware_update_routes=1 otawww_gap_routes=1 recovery_routes=1 static_file_routes=1`
- `reset_reason=11`
- `partition=factory`
- `firmware_commit=d9e471c9699e`
- `reference_commit=c1915b0a63bfabebdb95a515cedfee05146c1d50`
- `esp_idf_version=v5.5.4`
- `rust_target=xtensa-esp32s3-espidf`

The route registration log includes `/recovery`, `/api/system/info`,
`/api/system/OTA`, `/api/system/OTAWWW`, `/api/*`, and `/*`.

## Observed Behavior

The Ultra 205 accepted the wrapper-selected factory image, reset into the
factory partition, mounted SPIFFS, started the AxeOS-compatible route shell, and
emitted firmware and reference commit markers matching the copied Phase 17
package manifest.

## Conclusion

flash_monitor_status: passed - Phase 17 has a current package-to-board identity
chain for board `205` before live HTTP/static/API or WebSocket evidence plans
run.

## Non-Claims

- This serial evidence does not prove live HTTP.
- This serial evidence does not prove WebSocket frames.
- This serial evidence does not prove valid OTA upload.
- This serial evidence does not prove invalid OTA rejection.
- This serial evidence does not prove rollback.
- This serial evidence does not prove boot validation.
- This serial evidence does not prove OTAWWW update behavior.
- No `DEVICE_URL` value was used, inferred, or recorded by this plan.
- No raw `espflash monitor` fallback output is used as trusted evidence.
