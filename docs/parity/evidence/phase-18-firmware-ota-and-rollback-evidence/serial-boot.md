---
detector_status: passed
flash_monitor_status: passed
target_lock_status: passed
redaction_mode: commit-redacted
commit_ready: true
network_scan: disabled
board: "205"
selected_port: /dev/cu.usbmodem1101
source_commit: 22d02f8e97928f1ec29360552179380b92582e6a
reference_commit: c1915b0a63bfabebdb95a515cedfee05146c1d50
manifest_path: docs/parity/evidence/phase-18-firmware-ota-and-rollback-evidence/package-release-gate/bitaxe-ultra205-package.json
generated_manifest_path: bazel-bin/firmware/bitaxe/bitaxe-ultra205-package.json
flash_evidence_json: docs/parity/evidence/phase-18-firmware-ota-and-rollback-evidence/serial-boot/flash-command-evidence.json
serial_log: docs/parity/evidence/phase-18-firmware-ota-and-rollback-evidence/serial-boot/flash-monitor.log
detector_log: docs/parity/evidence/phase-18-firmware-ota-and-rollback-evidence/serial-boot/detect-ultra205.log
target_lock: docs/parity/evidence/phase-18-firmware-ota-and-rollback-evidence/target-lock.json
device_url_source: usb_flash_monitor_log
recorded_at: 2026-07-03T15:29:00Z
---

# Phase 18 Serial Boot Evidence

flash_monitor_status: passed

Detector-gated `just flash-monitor` evidence captured wrapper-owned Ultra 205
serial boot output for the same source and reference commits as the copied
Phase 18 package manifest. The committed flash-monitor evidence is
`commit-redacted`; the raw local USB evidence used only for target extraction
remains under `target/phase18-firmware-ota-and-rollback-evidence-dev-raw/` and
is not commit-ready.

## Detector Gate

| Field | Value |
| --- | --- |
| detector_status | passed |
| board | `205` |
| selected_port | `/dev/cu.usbmodem1101` |
| network_scan | disabled |
| detector command | `just detect-ultra205` |
| board-info command | `espflash board-info --chip esp32s3 --port /dev/cu.usbmodem1101 --non-interactive` |
| board-info output summary | ESP32-S3 revision v0.2, 16MB flash, WiFi/BLE/embedded flash features, board-info succeeded; MAC redacted in committed transcript. |
| detector transcript | `docs/parity/evidence/phase-18-firmware-ota-and-rollback-evidence/serial-boot/detect-ultra205.log` |

The detector reported exactly one `port=` line and exited with status `0`.
Flashing proceeded only after this gate passed.

## Flash-Monitor Command

```bash
just flash-monitor board=205 port=/dev/cu.usbmodem1101 manifest=bazel-bin/firmware/bitaxe/bitaxe-ultra205-package.json wifi-credentials=wifi-credentials.json evidence-dir=docs/parity/evidence/phase-18-firmware-ota-and-rollback-evidence/serial-boot capture-timeout-seconds=45 redact-evidence=true
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
| copied package manifest | `docs/parity/evidence/phase-18-firmware-ota-and-rollback-evidence/package-release-gate/bitaxe-ultra205-package.json` |
| generated package manifest | `bazel-bin/firmware/bitaxe/bitaxe-ultra205-package.json` |
| source_commit | `22d02f8e97928f1ec29360552179380b92582e6a` |
| reference_commit | `c1915b0a63bfabebdb95a515cedfee05146c1d50` |
| command_kind | `flash-monitor` |
| board | `205` |
| trusted_output | `true` |
| capture_status | `timed_out_after_trusted_output` |
| observed_firmware_commit | `22d02f8e9792` |
| observed_reference_commit | `c1915b0a63bfabebdb95a515cedfee05146c1d50` |
| wrapper evidence JSON | `docs/parity/evidence/phase-18-firmware-ota-and-rollback-evidence/serial-boot/flash-command-evidence.json` |
| serial monitor log | `docs/parity/evidence/phase-18-firmware-ota-and-rollback-evidence/serial-boot/flash-monitor.log` |

The flash-monitor command was run from a temporary package-source clone under
`target/` at `22d02f8e97928f1ec29360552179380b92582e6a` so the wrapper-owned
`firmware_commit` field remained aligned with the copied package manifest after
the package evidence commit. No files from that temporary clone are committed.

## Captured Log Summary

The redacted serial log contains the required wrapper-trusted Ultra 205 markers:

- `bitaxe-rust boot: board=Ultra 205 asic=BM1366`
- `safe_state: mining=disabled asic_work_submission=disabled hardware_control=disabled`
- `ota_boot_validation=not_pending state=factory`
- `wifi_status=connected ipv4=[redacted-ip] device_url=[redacted-url]`
- `spiffs_mount=available partition=www`
- `axeos_api_route_shell=started manifest_routes=17 firmware_update_routes=1 otawww_gap_routes=1 recovery_routes=1 static_file_routes=1`
- `firmware_commit=22d02f8e9792`
- `reference_commit=c1915b0a63bfabebdb95a515cedfee05146c1d50`
- `esp_idf_version=v5.5.4`
- `rust_target=xtensa-esp32s3-espidf`

The route registration log includes `/recovery`, `/api/system/info`,
`/api/system/OTA`, `/api/system/OTAWWW`, `/api/*`, and `/*`.

## Target Lock

| Field | Value |
| --- | --- |
| target_lock_status | passed |
| target lock | `docs/parity/evidence/phase-18-firmware-ota-and-rollback-evidence/target-lock.json` |
| device_url_source | `usb_flash_monitor_log` |
| device_url_redacted | `http://[redacted]` |
| created_from_explicit_input | `true` |
| raw evidence source | `target/phase18-firmware-ota-and-rollback-evidence-dev-raw/flash-command-evidence.json` |
| network_scan | disabled |

The target lock was produced by `scripts/phase18-firmware-ota-evidence.sh
--target-lock-only` from trusted local raw flash-monitor evidence. The committed
target lock redacts the raw origin and records `network_scan: disabled`.

## Observed Behavior

The Ultra 205 accepted the wrapper-selected factory image, reset into the
factory partition, joined Wi-Fi from flash-time NVS credentials, mounted SPIFFS,
started the AxeOS-compatible route shell, and emitted firmware and reference
commit markers matching the copied Phase 18 package manifest.

## Conclusion

target_lock_status: passed

flash_monitor_status: passed - Phase 18 has current package-to-board identity
and sanitized target provenance for board `205` before OTA upload evidence runs.

## Non-Claims

- This serial evidence does not prove valid OTA upload.
- This serial evidence does not prove invalid OTA rejection.
- This serial evidence does not prove rollback.
- This serial evidence records boot-validation state as `not_pending state=factory`; it does not prove post-OTA boot validation.
- This serial evidence does not prove OTAWWW update behavior.
- No raw `DEVICE_URL`, Wi-Fi credential, IP address, MAC address, SSID, pool credential, API token, or NVS secret value is committed in this serial evidence.
