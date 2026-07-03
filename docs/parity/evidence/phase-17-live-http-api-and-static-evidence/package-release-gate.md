---
package_status: passed
release_gate_status: passed
source_commit: 9a2bf5850ea042731f6a7947cc7eb04dc4589e90
reference_commit: c1915b0a63bfabebdb95a515cedfee05146c1d50
manifest_path: docs/parity/evidence/phase-17-live-http-api-and-static-evidence/package-release-gate/bitaxe-ultra205-package.json
generated_manifest_path: bazel-bin/firmware/bitaxe/bitaxe-ultra205-package.json
package_command_log: docs/parity/evidence/phase-17-live-http-api-and-static-evidence/package-release-gate/package-command.log
release_gate_log: docs/parity/evidence/phase-17-live-http-api-and-static-evidence/package-release-gate/release-gate.log
recorded_at: 2026-07-03T06:24:40Z
---

# Phase 17 Package And Release-Gate Evidence

Current Phase 17 Ultra 205 package evidence was refreshed before detector,
flash-monitor, HTTP, and WebSocket probes. The copied manifest and release gate
share source commit `9a2bf5850ea042731f6a7947cc7eb04dc4589e90` and reference
commit `c1915b0a63bfabebdb95a515cedfee05146c1d50`.

## Status

- package_status: passed
- release_gate_status: passed
- `just package` completed successfully and wrote
  `bazel-bin/firmware/bitaxe/bitaxe-ultra205-package.json`.
- `bazel run //tools/parity:report -- release-gate --manifest docs/parity/evidence/phase-17-live-http-api-and-static-evidence/package-release-gate/bitaxe-ultra205-package.json`
  completed with `release_gate: passed`.
- The committed manifest copy records the current source/reference identity and
  package checksums.
- No generated binaries were copied into docs evidence.

## Identity

| Field | Value |
| --- | --- |
| source_commit | `9a2bf5850ea042731f6a7947cc7eb04dc4589e90` |
| current git HEAD at package refresh | `9a2bf5850ea042731f6a7947cc7eb04dc4589e90` |
| reference_commit | `c1915b0a63bfabebdb95a515cedfee05146c1d50` |
| reference HEAD | `c1915b0a63bfabebdb95a515cedfee05146c1d50` |
| manifest path | `docs/parity/evidence/phase-17-live-http-api-and-static-evidence/package-release-gate/bitaxe-ultra205-package.json` |
| generated manifest path | `bazel-bin/firmware/bitaxe/bitaxe-ultra205-package.json` |
| default flash image | `bitaxe-ultra205.elf` |

## Commands

| Command | Log |
| --- | --- |
| `just package` | `docs/parity/evidence/phase-17-live-http-api-and-static-evidence/package-release-gate/package-command.log` |
| `bazel run //tools/parity:report -- release-gate --manifest docs/parity/evidence/phase-17-live-http-api-and-static-evidence/package-release-gate/bitaxe-ultra205-package.json` | `docs/parity/evidence/phase-17-live-http-api-and-static-evidence/package-release-gate/release-gate.log` |

## Observed Artifacts

| Kind | Path | Offset | sha256 |
| --- | --- | --- | --- |
| firmware_elf | `bitaxe-ultra205.elf` | `Unavailable` | `2053e11c4ef639797817453486c06c683a3cdb237210019369502d5a0035bbe1` |
| firmware_ota_image | `esp-miner.bin` | `0x10000` | `dd4477c41b59d874525bd4bf7fcecb5671c22bcea6dad94165ee6d927c38705f` |
| www_spiffs_image | `www.bin` | `0x410000` | `dc468896835419242601b2648524167c2bd63a7d3aedb33c1b108c1cd84c4a20` |
| factory_merged_image | `bitaxe-ultra205-factory.bin` | `0x0` | `d4b77d005ffa9395f5edb3722748d2e1a92bd64052f94e9121bbba89a5b4bfa5` |
| partition_table | `firmware/bitaxe/partitions-ultra205.csv` | `Unavailable` | `19f4fe9b96e6807566dcde496697dde11a8c4258f8c74d3439aaee114a33bba5` |
| otadata_initial | `otadata-initial.bin` | `0xf10000` | `7d2c7ac4888bfd75cd5f56e8d61f69595121183afc81556c876732fd3782c62f` |

## Output Summary

`just package` built the ESP32-S3 firmware, created the OTA image, SPIFFS
image, initial OTA data, merged factory image, and package manifest through the
repo-owned Bazel target `//firmware/bitaxe:firmware_image`.

The release gate validated the package manifest and returned:

```text
release_gate: passed
```

## Conclusion

The Phase 17 package and release-gate identity state is ready for downstream
detector, flash-monitor, live HTTP/static/API, and WebSocket evidence for board
`205`.

## Non-Claims

- This package/release-gate evidence does not prove live HTTP.
- This package/release-gate evidence does not prove WebSocket frames.
- This package/release-gate evidence does not prove valid OTA upload.
- This package/release-gate evidence does not prove invalid OTA rejection.
- This package/release-gate evidence does not prove reboot.
- This package/release-gate evidence does not prove rollback.
- This package/release-gate evidence does not prove boot validation.
- This package/release-gate evidence does not prove whole-`www` OTAWWW update behavior.
