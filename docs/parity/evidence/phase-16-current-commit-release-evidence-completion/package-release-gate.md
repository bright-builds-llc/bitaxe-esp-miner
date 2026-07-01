---
package_status: passed
release_gate_status: passed
source_commit: b55d3e68b68060fc6cf271372a75fc86c0a934c6
reference_commit: c1915b0a63bfabebdb95a515cedfee05146c1d50
manifest_path: docs/parity/evidence/phase-16-current-commit-release-evidence-completion/package-release-gate/bitaxe-ultra205-package.json
generated_manifest_path: bazel-bin/firmware/bitaxe/bitaxe-ultra205-package.json
package_command_log: docs/parity/evidence/phase-16-current-commit-release-evidence-completion/package-release-gate/package-command.log
release_gate_log: docs/parity/evidence/phase-16-current-commit-release-evidence-completion/package-release-gate/release-gate.log
recorded_at: 2026-07-01T14:05:18Z
---

# Phase 16 Package And Release-Gate Evidence

Current-commit Ultra 205 package evidence was generated after Phase 16 Plan 02 started. The package was refreshed after the Task 1 evidence commit because the flash-monitor wrapper rebuilt and flashed commit `b55d3e68b68060fc6cf271372a75fc86c0a934c6`; the copied manifest, release gate, and serial JSON now share that source commit.

## Status

- package_status: passed
- release_gate_status: passed
- `just package` completed successfully for commit `b55d3e68b68060fc6cf271372a75fc86c0a934c6` and wrote `bazel-bin/firmware/bitaxe/bitaxe-ultra205-package.json`.
- `bazel run //tools/parity:report -- release-gate --manifest bazel-bin/firmware/bitaxe/bitaxe-ultra205-package.json` completed with `release_gate: passed`.
- The committed manifest copy is byte-identical to the generated Bazel manifest.
- No generated binaries were copied into docs evidence.

## Identity

| Field | Value |
| --- | --- |
| source_commit | `b55d3e68b68060fc6cf271372a75fc86c0a934c6` |
| current git HEAD at package refresh | `b55d3e68b68060fc6cf271372a75fc86c0a934c6` |
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
| firmware_elf | `bitaxe-ultra205.elf` | `Unavailable` | `e9798435d6656cc993428de333ad82f6b55d42f088b710e1fb078ec9919140eb` |
| firmware_ota_image | `esp-miner.bin` | `0x10000` | `a494f743187fd9ab137251687a5c7cc59b0486f9594b4b3e946ec74081ddd7cd` |
| www_spiffs_image | `www.bin` | `0x410000` | `0dbb0eba0cc4198186d0175557ec134d7829f3426faf35d8baf263ee0a7c65a0` |
| factory_merged_image | `bitaxe-ultra205-factory.bin` | `0x0` | `e86982e9f4f72d59a1018ac423d8fe46530de1829a032870b9b527373e13061d` |
| partition_table | `firmware/bitaxe/partitions-ultra205.csv` | `Unavailable` | `19f4fe9b96e6807566dcde496697dde11a8c4258f8c74d3439aaee114a33bba5` |
| otadata_initial | `otadata-initial.bin` | `0xf10000` | `7d2c7ac4888bfd75cd5f56e8d61f69595121183afc81556c876732fd3782c62f` |

## Release Gate

release_gate_status: passed

The release gate used the generated manifest and validated the Ultra 205 manifest contract, release guide, license inventory, provenance manifest, Cargo license report reference, and artifact review closure.

## Non-Claims

- This package evidence does not prove live HTTP, static, recovery, OTA, rollback, failed-update, interrupted-update, or large-erase parity.
- Historical Phase 13 package and serial evidence are not cited as current-commit proof for this plan.
