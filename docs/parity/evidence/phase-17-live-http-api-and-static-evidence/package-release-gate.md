---
package_status: passed
release_gate_status: passed
source_commit: d9e471c9699eb0140749127416640aa1bf077d26
reference_commit: c1915b0a63bfabebdb95a515cedfee05146c1d50
manifest_path: docs/parity/evidence/phase-17-live-http-api-and-static-evidence/package-release-gate/bitaxe-ultra205-package.json
generated_manifest_path: bazel-bin/firmware/bitaxe/bitaxe-ultra205-package.json
package_command_log: docs/parity/evidence/phase-17-live-http-api-and-static-evidence/package-release-gate/package-command.log
release_gate_log: docs/parity/evidence/phase-17-live-http-api-and-static-evidence/package-release-gate/release-gate.log
recorded_at: 2026-07-02T02:56:37Z
---

# Phase 17 Package And Release-Gate Evidence

Current Phase 17 Ultra 205 package evidence was refreshed before detector,
flash-monitor, HTTP, or WebSocket probes. The copied manifest and release gate
share source commit `d9e471c9699eb0140749127416640aa1bf077d26` and reference
commit `c1915b0a63bfabebdb95a515cedfee05146c1d50`.

## Status

- package_status: passed
- release_gate_status: passed
- `just package` completed successfully and wrote
  `bazel-bin/firmware/bitaxe/bitaxe-ultra205-package.json`.
- `bazel run //tools/parity:report -- release-gate --manifest bazel-bin/firmware/bitaxe/bitaxe-ultra205-package.json`
  completed with `release_gate: passed`.
- The committed manifest copy is byte-identical to the generated Bazel manifest.
- No generated binaries were copied into docs evidence.

## Identity

| Field | Value |
| --- | --- |
| source_commit | `d9e471c9699eb0140749127416640aa1bf077d26` |
| current git HEAD at package refresh | `d9e471c9699eb0140749127416640aa1bf077d26` |
| reference_commit | `c1915b0a63bfabebdb95a515cedfee05146c1d50` |
| reference HEAD | `c1915b0a63bfabebdb95a515cedfee05146c1d50` |
| manifest path | `docs/parity/evidence/phase-17-live-http-api-and-static-evidence/package-release-gate/bitaxe-ultra205-package.json` |
| generated manifest path | `bazel-bin/firmware/bitaxe/bitaxe-ultra205-package.json` |
| default flash image | `bitaxe-ultra205.elf` |

## Commands

| Command | Log |
| --- | --- |
| `just package` | `docs/parity/evidence/phase-17-live-http-api-and-static-evidence/package-release-gate/package-command.log` |
| `bazel run //tools/parity:report -- release-gate --manifest bazel-bin/firmware/bitaxe/bitaxe-ultra205-package.json` | `docs/parity/evidence/phase-17-live-http-api-and-static-evidence/package-release-gate/release-gate.log` |

## Observed Artifacts

| Kind | Path | Offset | sha256 |
| --- | --- | --- | --- |
| firmware_elf | `bitaxe-ultra205.elf` | `Unavailable` | `34a31260abfaa3ff7a055d632da00795d34df6b002a7c73515c25dbd8796fcd4` |
| firmware_ota_image | `esp-miner.bin` | `0x10000` | `fa774a7c2439a9a3fac9493a7e77eea7cb6959485ff11e3e2122fec22d7b9b27` |
| www_spiffs_image | `www.bin` | `0x410000` | `0dbb0eba0cc4198186d0175557ec134d7829f3426faf35d8baf263ee0a7c65a0` |
| factory_merged_image | `bitaxe-ultra205-factory.bin` | `0x0` | `8d30621401a5deee486c8a6a98334c6f57a39dbf32df56e0077317dff415bff9` |
| partition_table | `firmware/bitaxe/partitions-ultra205.csv` | `Unavailable` | `19f4fe9b96e6807566dcde496697dde11a8c4258f8c74d3439aaee114a33bba5` |
| otadata_initial | `otadata-initial.bin` | `0xf10000` | `7d2c7ac4888bfd75cd5f56e8d61f69595121183afc81556c876732fd3782c62f` |

## Output Summary

`just package` built the ESP32-S3 firmware, created the OTA image, SPIFFS
image, initial OTA data, merged factory image, and package manifest through the
repo-owned Bazel target `//firmware/bitaxe:firmware_image`.

The release gate validated the generated package manifest and returned:

```text
release_gate: passed
```

## Conclusion

The Phase 17 package and release-gate identity state is ready for downstream
detector and flash-monitor evidence. Plans 17-03 and 17-04 must still consume
the copied manifest and wrapper-owned flash evidence before making live route or
WebSocket claims.

## Non-Claims

- This package/release-gate evidence does not prove live HTTP.
- This package/release-gate evidence does not prove static asset serving.
- This package/release-gate evidence does not prove recovery page behavior.
- This package/release-gate evidence does not prove WebSocket frames.
- This package/release-gate evidence does not prove valid OTA upload.
- This package/release-gate evidence does not prove invalid OTA rejection.
- This package/release-gate evidence does not prove reboot.
- This package/release-gate evidence does not prove rollback.
- This package/release-gate evidence does not prove boot validation.
- This package/release-gate evidence does not prove whole-`www` OTAWWW update behavior.
