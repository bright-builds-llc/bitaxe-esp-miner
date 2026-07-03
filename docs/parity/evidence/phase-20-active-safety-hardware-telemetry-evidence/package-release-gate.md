# Phase 20 Package And Release-Gate Evidence

package_identity_status: passed
release_gate_status: passed
board: 205
source_commit: 16cf0787c15b2258ce470c8a7d27e45c15e1ca2b
reference_commit: c1915b0a63bfabebdb95a515cedfee05146c1d50
manifest_path: docs/parity/evidence/phase-20-active-safety-hardware-telemetry-evidence/package-release-gate/bitaxe-ultra205-package.json
default_flash_image: bitaxe-ultra205.elf
firmware_ota_image: esp-miner.bin
factory_image: bitaxe-ultra205-factory.bin
redaction_status: passed - package and release-gate artifacts contain build paths, checksums, artifact names, and commit identifiers only; no credentials, API tokens, target URLs, Wi-Fi secrets, pool credentials, or NVS secrets are cited

## Command Evidence

- Package command: `just package`
- Package command log: `docs/parity/evidence/phase-20-active-safety-hardware-telemetry-evidence/package-release-gate/package-command.log`
- Copied manifest source: `bazel-bin/firmware/bitaxe/bitaxe-ultra205-package.json`
- Release-gate command: `bazel run //tools/parity:report -- release-gate --manifest docs/parity/evidence/phase-20-active-safety-hardware-telemetry-evidence/package-release-gate/bitaxe-ultra205-package.json`
- Release-gate log: `docs/parity/evidence/phase-20-active-safety-hardware-telemetry-evidence/package-release-gate/release-gate.log`

## Verification Notes

- `just package` completed successfully before this ledger was written.
- `bazel run //tools/parity:report -- release-gate --manifest docs/parity/evidence/phase-20-active-safety-hardware-telemetry-evidence/package-release-gate/bitaxe-ultra205-package.json` returned `release_gate: passed`.
- The copied manifest `source_commit` equals `git rev-parse HEAD` at package time.
- The copied manifest `reference_commit` equals the pinned `reference/esp-miner` commit.

## Non-Claims

safe-baseline non-claim: package identity alone does not prove detector, board-info, flash-monitor, boot, or safe-state evidence.

active safety non-claim: no voltage, fan, thermal, self-test, load, runtime input/display, failure-path, mining, OTA, erase, rollback, or fault-injection command was run by this package gate.

live telemetry non-claim: no live HTTP, WebSocket, API freshness, network target, or `DEVICE_URL` claim is made by this package gate.
