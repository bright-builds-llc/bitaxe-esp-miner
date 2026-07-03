---
detector_status: passed
flash_monitor_status: passed
redaction_mode: commit-redacted
commit_ready: true
network_scan: disabled
board: "205"
selected_port: /dev/cu.usbmodem1101
source_commit: 9a2bf5850ea042731f6a7947cc7eb04dc4589e90
reference_commit: c1915b0a63bfabebdb95a515cedfee05146c1d50
manifest_path: docs/parity/evidence/phase-17-live-http-api-and-static-evidence/package-release-gate/bitaxe-ultra205-package.json
generated_manifest_path: bazel-bin/firmware/bitaxe/bitaxe-ultra205-package.json
flash_evidence_json: docs/parity/evidence/phase-17-live-http-api-and-static-evidence/serial-boot/flash-command-evidence.json
serial_log: docs/parity/evidence/phase-17-live-http-api-and-static-evidence/serial-boot/flash-monitor.log
detector_log: docs/parity/evidence/phase-17-live-http-api-and-static-evidence/serial-boot/detect-ultra205.log
recorded_at: 2026-07-03T06:31:12Z
---

# Phase 17 Serial Boot Evidence

flash_monitor_status: passed

Detector-gated `just flash-monitor` evidence captured wrapper-owned Ultra 205
serial boot output for the same source and reference commits as the copied
Phase 17 package manifest. The committed flash-monitor evidence is
`commit-redacted`; the raw local USB evidence used only for live target
extraction remains under `target/phase17-gap-current-dev-raw/` and is not
commit-ready.

## Detector Gate

| Field | Value |
| --- | --- |
| detector_status | passed |
| board | `205` |
| selected port | `/dev/cu.usbmodem1101` |
| network_scan | disabled |
| detector command | `just detect-ultra205` |
| board-info command | `espflash board-info --chip esp32s3 --port /dev/cu.usbmodem1101 --non-interactive` |
| board-info output summary | ESP32-S3 revision v0.2, 16MB flash, WiFi/BLE/embedded flash features, board-info succeeded; MAC redacted in committed transcript. |
| detector transcript | `docs/parity/evidence/phase-17-live-http-api-and-static-evidence/serial-boot/detect-ultra205.log` |

The detector reported exactly one `port=` line and exited with status `0`.
Flashing proceeded only after this gate passed.

## Flash-Monitor Command

```bash
just flash-monitor board=205 port=/dev/cu.usbmodem1101 manifest=bazel-bin/firmware/bitaxe/bitaxe-ultra205-package.json wifi-credentials=wifi-credentials.json evidence-dir=docs/parity/evidence/phase-17-live-http-api-and-static-evidence/serial-boot capture-timeout-seconds=45 redact-evidence=true
```

The wrapper resolved the manifest to the factory image and used redacted
evidence mode:

- command_kind: `flash-monitor`
- redaction_mode: `commit-redacted`
- commit_ready: `true`
- nvs_seed_status: `provided`
- wifi_credentials_source: `provided-redacted`
- flash_command: `espflash write-bin --chip esp32s3 --port /dev/cu.usbmodem1101 0x0 .../bitaxe-ultra205-factory.bin`
- monitor_command: `espflash monitor --chip esp32s3 --port /dev/cu.usbmodem1101 --non-interactive`

## Package And Wrapper Identity

| Field | Value |
| --- | --- |
| copied package manifest | `docs/parity/evidence/phase-17-live-http-api-and-static-evidence/package-release-gate/bitaxe-ultra205-package.json` |
| generated package manifest | `bazel-bin/firmware/bitaxe/bitaxe-ultra205-package.json` |
| source_commit | `9a2bf5850ea042731f6a7947cc7eb04dc4589e90` |
| reference_commit | `c1915b0a63bfabebdb95a515cedfee05146c1d50` |
| command_kind | `flash-monitor` |
| board | `205` |
| trusted_output | `true` |
| capture_status | `timed_out_after_trusted_output` |
| observed_firmware_commit | `9a2bf5850ea0` |
| observed_reference_commit | `c1915b0a63bfabebdb95a515cedfee05146c1d50` |
| wrapper evidence JSON | `docs/parity/evidence/phase-17-live-http-api-and-static-evidence/serial-boot/flash-command-evidence.json` |
| serial monitor log | `docs/parity/evidence/phase-17-live-http-api-and-static-evidence/serial-boot/flash-monitor.log` |

## Captured Log Summary

The redacted serial log contains the required wrapper-trusted Ultra 205 markers:

- `bitaxe-rust boot: board=Ultra 205 asic=BM1366`
- `safe_state: mining=disabled asic_work_submission=disabled hardware_control=disabled`
- `ota_boot_validation=not_pending state=factory`
- `wifi_status=connected ipv4=[redacted-ip] device_url=[redacted-url]`
- `spiffs_mount=available partition=www total_bytes=2884241 used_bytes=4518`
- `axeos_api_route_shell=started manifest_routes=17 firmware_update_routes=1 otawww_gap_routes=1 recovery_routes=1 static_file_routes=1`
- `firmware_commit=9a2bf5850ea0`
- `reference_commit=c1915b0a63bfabebdb95a515cedfee05146c1d50`
- `esp_idf_version=v5.5.4`
- `rust_target=xtensa-esp32s3-espidf`

The route registration log includes `/recovery`, `/api/system/info`,
`/api/system/OTA`, `/api/system/OTAWWW`, `/api/*`, and `/*`.

## Observed Behavior

The Ultra 205 accepted the wrapper-selected factory image, reset into the
factory partition, joined Wi-Fi from flash-time NVS credentials, mounted
SPIFFS, started the AxeOS-compatible route shell, and emitted firmware and
reference commit markers matching the copied Phase 17 package manifest.

## Conclusion

flash_monitor_status: passed - Phase 17 has a current package-to-board identity
chain for board `205` before live HTTP/static/API and WebSocket evidence plans
run.

## Non-Claims

- This serial evidence does not prove HTTP route semantics by itself.
- This serial evidence does not prove WebSocket frames by itself.
- This serial evidence does not prove valid OTA upload.
- This serial evidence does not prove invalid OTA rejection.
- This serial evidence does not prove rollback.
- This serial evidence does not prove boot validation.
- This serial evidence does not prove OTAWWW update behavior.
- No raw `DEVICE_URL`, Wi-Fi credential, IP address, or MAC address is committed in this serial evidence.
