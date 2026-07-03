# Phase 18 Package Release Gate

package_status: passed

release_gate_status: passed

source_commit: 22d02f8e97928f1ec29360552179380b92582e6a

reference_commit: c1915b0a63bfabebdb95a515cedfee05146c1d50

manifest_path: docs/parity/evidence/phase-18-firmware-ota-and-rollback-evidence/package-release-gate/bitaxe-ultra205-package.json

firmware_ota_image_path: esp-miner.bin

firmware_ota_image_sha256: c7f4a62872d6662562f89ff8e93e881317d65b8f58003577acfd6a0a50eb6463

factory_image_path: bitaxe-ultra205-factory.bin

www_image_sha256: dc468896835419242601b2648524167c2bd63a7d3aedb33c1b108c1cd84c4a20

command_log: docs/parity/evidence/phase-18-firmware-ota-and-rollback-evidence/package-release-gate/package-command.log

release_gate_log: docs/parity/evidence/phase-18-firmware-ota-and-rollback-evidence/package-release-gate/release-gate.log

network_scan: disabled

conclusion: Phase 18 package identity and manifest-backed release gate passed for the task-start source commit before detector, target-lock, or OTA upload work.

## Commands

Package command:

```bash
just package
```

Release gate command:

```bash
bazel run //tools/parity:report -- release-gate --manifest docs/parity/evidence/phase-18-firmware-ota-and-rollback-evidence/package-release-gate/bitaxe-ultra205-package.json
```

## Evidence Notes

- The copied package manifest was produced by this plan's `just package` run.
- The manifest `source_commit` matched the Task 1 start commit `22d02f8e97928f1ec29360552179380b92582e6a`.
- The manifest lists `firmware_ota_image` path `esp-miner.bin` with a 64-character SHA-256 checksum.
- No generated `.bin` or `.elf` artifact is promoted into committed Phase 18 evidence by this ledger.
