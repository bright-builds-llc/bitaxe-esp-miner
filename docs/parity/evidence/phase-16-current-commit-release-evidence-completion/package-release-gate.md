---
package_status: passed
release_gate_status: passed
source_commit: 025d371f405d6d60774f26f333eb50c8187e72a6
reference_commit: c1915b0a63bfabebdb95a515cedfee05146c1d50
manifest_path: docs/parity/evidence/phase-16-current-commit-release-evidence-completion/package-release-gate/bitaxe-ultra205-package.json
generated_manifest_path: bazel-bin/firmware/bitaxe/bitaxe-ultra205-package.json
package_command_log: docs/parity/evidence/phase-16-current-commit-release-evidence-completion/package-release-gate/package-command.log
release_gate_log: docs/parity/evidence/phase-16-current-commit-release-evidence-completion/package-release-gate/release-gate.log
recorded_at: 2026-07-01T13:59:41Z
---

# Phase 16 Package And Release-Gate Evidence

Current-commit Ultra 205 package evidence was generated after Phase 16 Plan 02 started and before any hardware command in this plan.

## Status

- package_status: passed
- release_gate_status: passed
- `just package` completed successfully and wrote `bazel-bin/firmware/bitaxe/bitaxe-ultra205-package.json`.
- `bazel run //tools/parity:report -- release-gate --manifest bazel-bin/firmware/bitaxe/bitaxe-ultra205-package.json` completed with `release_gate: passed`.
- The committed manifest copy is byte-identical to the generated Bazel manifest.
- No generated binaries were copied into docs evidence.

## Identity

| Field | Value |
| --- | --- |
| source_commit | `025d371f405d6d60774f26f333eb50c8187e72a6` |
| current git HEAD | `025d371f405d6d60774f26f333eb50c8187e72a6` |
| reference_commit | `c1915b0a63bfabebdb95a515cedfee05146c1d50` |
| reference HEAD | `c1915b0a63bfabebdb95a515cedfee05146c1d50` |
| manifest path | `docs/parity/evidence/phase-16-current-commit-release-evidence-completion/package-release-gate/bitaxe-ultra205-package.json` |
| generated manifest path | `bazel-bin/firmware/bitaxe/bitaxe-ultra205-package.json` |

## Commands

| Command | Log |
| --- | --- |
| `just package` | `docs/parity/evidence/phase-16-current-commit-release-evidence-completion/package-release-gate/package-command.log` |
| `bazel run //tools/parity:report -- release-gate --manifest bazel-bin/firmware/bitaxe/bitaxe-ultra205-package.json` | `docs/parity/evidence/phase-16-current-commit-release-evidence-completion/package-release-gate/release-gate.log` |

## Required Artifacts

| Kind | Path | Offset | sha256 |
| --- | --- | --- | --- |
| firmware_elf | `bitaxe-ultra205.elf` | `Unavailable` | `acfc6a3d5fab09e67ec38266e40907d094f69936b980bb263be0c64ed0aa009e` |
| firmware_ota_image | `esp-miner.bin` | `0x10000` | `59a01bb92e2a2edc41b32a1fe315df8a60124f0b8cb23393cfa5cf6226919fdb` |
| www_spiffs_image | `www.bin` | `0x410000` | `0dbb0eba0cc4198186d0175557ec134d7829f3426faf35d8baf263ee0a7c65a0` |
| factory_merged_image | `bitaxe-ultra205-factory.bin` | `0x0` | `4363406af25e30710559658752b3c393c1f118ea06d6c8b224d1d2f6fd92d2a7` |
| partition_table | `firmware/bitaxe/partitions-ultra205.csv` | `Unavailable` | `19f4fe9b96e6807566dcde496697dde11a8c4258f8c74d3439aaee114a33bba5` |
| otadata_initial | `otadata-initial.bin` | `0xf10000` | `7d2c7ac4888bfd75cd5f56e8d61f69595121183afc81556c876732fd3782c62f` |

## Release Gate

release_gate_status: passed

The release gate used the generated manifest and validated the Ultra 205 manifest contract, release guide, license inventory, provenance manifest, Cargo license report reference, and artifact review closure.

## Non-Claims

- This package evidence does not prove live HTTP, static, recovery, OTA, rollback, failed-update, interrupted-update, or large-erase parity.
- Historical Phase 13 package and serial evidence are not cited as current-commit proof for this plan.
