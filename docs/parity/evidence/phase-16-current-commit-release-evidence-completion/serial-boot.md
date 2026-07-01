---
serial_boot_status: passed
detector_status: passed
release_evidence_status: passed
board: "205"
port: /dev/cu.usbmodem1101
source_commit: 8490118a7e7f6fc1a9ac2e4025d983b0f402c8ca
reference_commit: c1915b0a63bfabebdb95a515cedfee05146c1d50
manifest_path: docs/parity/evidence/phase-16-current-commit-release-evidence-completion/package-release-gate/bitaxe-ultra205-package.json
flash_evidence_json: docs/parity/evidence/phase-16-current-commit-release-evidence-completion/serial-boot/flash-command-evidence.json
serial_log: docs/parity/evidence/phase-16-current-commit-release-evidence-completion/serial-boot/flash-monitor.log
detector_log: docs/parity/evidence/phase-16-current-commit-release-evidence-completion/serial-boot/detect-ultra205.log
recorded_at: 2026-07-01T15:26:58Z
---

# Phase 16 Serial Boot Evidence

serial_boot_status: passed

Detector-gated `just flash-monitor` evidence captured wrapper-owned Ultra 205 serial boot output for the same source commit as the copied package manifest.

## Detector Gate

| Field | Value |
| --- | --- |
| detector_status | passed |
| selected port | `/dev/cu.usbmodem1101` |
| board-info command | `espflash board-info --chip esp32s3 --port /dev/cu.usbmodem1101 --non-interactive` |
| board-info result | ESP32-S3, 16MB flash, board-info succeeded |
| detector transcript | `docs/parity/evidence/phase-16-current-commit-release-evidence-completion/serial-boot/detect-ultra205.log` |

The detector reported exactly one `port=` line and exited with status `0`. Flashing proceeded only after this gate passed.

## Flash-Monitor Command

```bash
just flash-monitor board=205 port=/dev/cu.usbmodem1101 manifest=bazel-bin/firmware/bitaxe/bitaxe-ultra205-package.json evidence-dir=docs/parity/evidence/phase-16-current-commit-release-evidence-completion/serial-boot capture-timeout-seconds=35
```

The wrapper resolved the manifest to the factory image and used:

- flash_command: `espflash write-bin --chip esp32s3 --port /dev/cu.usbmodem1101 0x0 /Users/peterryszkiewicz/Repos/bitaxe-esp-miner/bazel-bin/firmware/bitaxe/bitaxe-ultra205-factory.bin`
- monitor_command: `espflash monitor --chip esp32s3 --port /dev/cu.usbmodem1101 --non-interactive`

## Wrapper Evidence JSON

| Field | Value |
| --- | --- |
| command_kind | `flash-monitor` |
| board | `205` |
| firmware_commit | `8490118a7e7f6fc1a9ac2e4025d983b0f402c8ca` |
| reference_commit | `c1915b0a63bfabebdb95a515cedfee05146c1d50` |
| trusted_output | `true` |
| capture_status | `timed_out_after_trusted_output` |
| observed_firmware_commit | `8490118a7e7f` |
| observed_reference_commit | `c1915b0a63bfabebdb95a515cedfee05146c1d50` |
| flash evidence JSON | `docs/parity/evidence/phase-16-current-commit-release-evidence-completion/serial-boot/flash-command-evidence.json` |
| serial monitor log | `docs/parity/evidence/phase-16-current-commit-release-evidence-completion/serial-boot/flash-monitor.log` |

## Trusted Serial Markers

The serial log contains the required FND-06 and release identity markers:

- `bitaxe-rust boot: board=Ultra 205 asic=BM1366`
- `safe_state: mining=disabled asic_work_submission=disabled hardware_control=disabled`
- `reset_reason=11`
- `partition=factory`
- `firmware_commit=8490118a7e7f`
- `reference_commit=c1915b0a63bfabebdb95a515cedfee05146c1d50`
- `esp_idf_version=v5.5.4`
- `rust_target=xtensa-esp32s3-espidf`
- `spiffs_mount=available`
- `axeos_api_route_shell=started`
- route registrations include `/recovery`, `/api/system/OTA`, `/api/system/OTAWWW`, `/api/*`, and `/*`
- `ota_boot_validation=not_pending state=factory`

## Release Evidence Validation

Command:

```bash
bazel run //tools/parity:report -- release-evidence --manifest docs/parity/evidence/phase-16-current-commit-release-evidence-completion/package-release-gate/bitaxe-ultra205-package.json --evidence-root docs/parity/evidence/phase-16-current-commit-release-evidence-completion --flash-evidence-json docs/parity/evidence/phase-16-current-commit-release-evidence-completion/serial-boot/flash-command-evidence.json --redaction-review docs/parity/evidence/phase-16-current-commit-release-evidence-completion/redaction-review.md --require-redaction-passed
```

Output:

```text
release_evidence_status: passed
source_commit: 8490118a7e7f6fc1a9ac2e4025d983b0f402c8ca
reference_commit: c1915b0a63bfabebdb95a515cedfee05146c1d50
evidence_root: /Users/peterryszkiewicz/Repos/bitaxe-esp-miner/docs/parity/evidence/phase-16-current-commit-release-evidence-completion
redaction_status: passed
```

## Redaction

redaction_status: passed

Detector and serial artifacts were reviewed in `docs/parity/evidence/phase-16-current-commit-release-evidence-completion/redaction-review.md`. The phase-level redaction review is passed for cited Phase 16 artifacts; absent live HTTP, OTA, recovery, failed-update, interrupted-update, and large-erase artifacts remain marked `absent - not cited`.

## Non-Claims

- This serial evidence proves wrapper-owned flash-monitor boot evidence for board `205`; it does not prove live HTTP, static, recovery, firmware OTA, OTAWWW, rollback, failed-update, interrupted-update, or large-erase parity.
- No raw `espflash monitor` fallback output is used as trusted evidence.
