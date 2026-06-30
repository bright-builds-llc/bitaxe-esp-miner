# Phase 13 Serial Boot Evidence

## Command Log

- command: `just flash-monitor board=205 port=/dev/cu.usbmodem1101 evidence-dir=docs/parity/evidence/phase-13-final-ultra-205-release-evidence/serial-boot capture-timeout-seconds=25`
- serial_boot_status: passed
- board: `205`
- port: `/dev/cu.usbmodem1101`
- package manifest: `/Users/peterryszkiewicz/Library/Caches/bazel/_bazel_peterryszkiewicz/79ce772edfc7d2dc3dd5c6889d5a90c9/execroot/_main/bazel-out/darwin_arm64-fastbuild/bin/firmware/bitaxe/bitaxe-ultra205-package.json`
- source commit: `190849539700b8f9a7909fd2b6ebd84142557968`
- reference commit: `c1915b0a63bfabebdb95a515cedfee05146c1d50`
- wrapper JSON: `docs/parity/evidence/phase-13-final-ultra-205-release-evidence/serial-boot/flash-command-evidence.json`
- serial log: `docs/parity/evidence/phase-13-final-ultra-205-release-evidence/serial-boot/flash-monitor.log`

## Wrapper Evidence Fields

| Field | Value |
| --- | --- |
| `trusted_output` | `true` |
| `capture_status` | `timed_out_after_trusted_output` |
| `firmware_commit` | `190849539700b8f9a7909fd2b6ebd84142557968` |
| `reference_commit` | `c1915b0a63bfabebdb95a515cedfee05146c1d50` |
| `observed_firmware_commit` | `190849539700` |
| `observed_reference_commit` | `c1915b0a63bfabebdb95a515cedfee05146c1d50` |
| `flash_command` | `espflash write-bin --chip esp32s3 --port /dev/cu.usbmodem1101 0x0 /Users/peterryszkiewicz/Library/Caches/bazel/_bazel_peterryszkiewicz/79ce772edfc7d2dc3dd5c6889d5a90c9/execroot/_main/bazel-out/darwin_arm64-fastbuild/bin/firmware/bitaxe/bitaxe-ultra205-factory.bin` |
| `monitor_command` | `espflash monitor --chip esp32s3 --port /dev/cu.usbmodem1101 --non-interactive` |
| `conclusion` | `passed - wrapper-owned serial boot evidence captured; HTTP/static/recovery/OTA/rollback parity not claimed` |

## Serial Boot Markers

| Marker | Observed |
| --- | --- |
| `bitaxe-rust boot: board=Ultra 205 asic=BM1366` | yes |
| `ota_boot_validation=not_pending state=factory` | yes |
| `spiffs_mount=available partition=www total_bytes=2884241 used_bytes=4518` | yes |
| `axeos_api_route_shell=started manifest_routes=17 firmware_update_routes=1 otawww_gap_routes=1 recovery_routes=1 static_file_routes=1` | yes |
| `reset_reason=11` | yes |
| `esp_idf_version=v5.5.4` | yes |
| `safe_state: mining=disabled asic_work_submission=disabled hardware_control=disabled` | yes |

## Observed Behavior

The repo wrapper flashed the Ultra 205 factory image generated from package source commit `190849539700b8f9a7909fd2b6ebd84142557968`. The noninteractive monitor capture then observed firmware boot identity, the pinned reference identity, platform boot status, reset reason, SPIFFS availability, route shell startup, and safe no-mining/no-control startup.

safe_state: mining=disabled asic_work_submission=disabled hardware_control=disabled

ota_boot_validation=not_pending state=factory

spiffs_mount=available

axeos_api_route_shell=started

reset_reason=11

esp_idf_version=v5.5.4

The wrapper accepted the continuous monitor timeout only after trusted output was captured, so `capture_status=timed_out_after_trusted_output` is a passing serial evidence result.

## Scope Boundary

This evidence proves detector-gated USB factory flash and serial boot behavior for the package commit above. It does not claim live HTTP/static/recovery/OTA behavior, rollback, failed update recovery, large erase recovery, interrupted update recovery, OTAWWW update parity, ASIC initialization, active mining, voltage, fan, thermal, or power-control parity.

## Conclusion

Conclusion: serial_boot_status: passed - wrapper-owned serial boot evidence binds the current Phase 13 package source commit to live Ultra 205 safe-state boot output.
