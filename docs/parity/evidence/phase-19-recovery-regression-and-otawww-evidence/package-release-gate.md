# Phase 19 Package And Release-Gate Evidence

package_status: passed
release_gate_status: passed
board: 205
source_commit: 6842d7a6d3d4fc64d93900a9847c8a0b97edc16d
reference_commit: c1915b0a63bfabebdb95a515cedfee05146c1d50
manifest_path: docs/parity/evidence/phase-19-recovery-regression-and-otawww-evidence/package-release-gate/bitaxe-ultra205-package.json
firmware_ota_image: bazel-bin/firmware/bitaxe/esp-miner.bin
factory_image: bazel-bin/firmware/bitaxe/bitaxe-ultra205-factory.bin
www_bin_status: present - package/static asset evidence only; not whole-www OTAWWW update proof
otawww_claim_boundary: www.bin is package/static asset evidence only; it is not whole-www OTAWWW update proof
redaction_status: passed - package and release-gate artifacts contain build paths, checksums, and commit identifiers only; no credentials, API tokens, target URLs, or NVS secrets are cited

## Command Evidence

- Package command log: `docs/parity/evidence/phase-19-recovery-regression-and-otawww-evidence/package-release-gate/package-command.log`
- Release-gate log: `docs/parity/evidence/phase-19-recovery-regression-and-otawww-evidence/package-release-gate/release-gate.log`
- Copied manifest source: `bazel-bin/firmware/bitaxe/bitaxe-ultra205-package.json`

## Verification Notes

- `just package` completed successfully before this ledger was written.
- `bazel run //tools/parity:report -- release-gate --manifest docs/parity/evidence/phase-19-recovery-regression-and-otawww-evidence/package-release-gate/bitaxe-ultra205-package.json` returned `release_gate: passed`.
- The copied manifest `source_commit` equals `git rev-parse HEAD` at package time.
