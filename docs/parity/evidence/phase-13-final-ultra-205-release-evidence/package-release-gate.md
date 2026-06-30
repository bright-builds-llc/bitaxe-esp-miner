# Phase 13 Package Release Gate

## Command Log

- `just package`: passed
  - Result: Bazel built `//firmware/bitaxe:firmware_image` successfully.
  - Package manifest emitted at `bazel-bin/firmware/bitaxe/bitaxe-ultra205-package.json`.
- `bazel run //tools/parity:report -- release-gate --manifest bazel-bin/firmware/bitaxe/bitaxe-ultra205-package.json`: passed
  - Result: `release_gate: passed`
- `just flash-monitor board=205 port=/dev/cu.usbmodem1101 evidence-dir=docs/parity/evidence/phase-13-final-ultra-205-release-evidence/serial-boot capture-timeout-seconds=25`: passed
  - Result: Wrapper rebuilt and flashed the package manifest for source commit `190849539700b8f9a7909fd2b6ebd84142557968`.
- `bazel run //tools/parity:report -- release-gate --manifest bazel-bin/firmware/bitaxe/bitaxe-ultra205-package.json`: passed after serial flash
  - Result: `release_gate: passed`

## Manifest Fields

- package manifest: `bazel-bin/firmware/bitaxe/bitaxe-ultra205-package.json`
- source_commit: `190849539700b8f9a7909fd2b6ebd84142557968`
- reference_commit: `c1915b0a63bfabebdb95a515cedfee05146c1d50`
- image_metadata.esp_idf_version: `v5.5.4`
- image_metadata.rust_target: `xtensa-esp32s3-espidf`
- default_flash_image: `bitaxe-ultra205.elf`
- license_inventory: `docs/release/license-inventory.md`
- provenance_manifest: `docs/release/provenance-manifest.md`
- install_notes.path: `docs/release/ultra-205.md`

## Required Artifacts

| Artifact | Kind | Manifest status | Offset | SHA-256 |
| --- | --- | --- | --- | --- |
| `bitaxe-ultra205.elf` | firmware_elf | present | `Unavailable` | `dbe8c778ede1b721a06f44c4d5ab4a1a7558439dd2e6446d97e668e0b5fb9735` |
| `esp-miner.bin` | firmware_ota_image | present | `0x10000` | `e55e22da45f510b124beba62f56425fd468a95b1efd17949cfc140e15f220c42` |
| `www.bin` | www_spiffs_image | present | `0x410000` | `0dbb0eba0cc4198186d0175557ec134d7829f3426faf35d8baf263ee0a7c65a0` |
| `bitaxe-ultra205-factory.bin` | factory_merged_image | present | `0x0` | `b354279c76d6ab05741c4444eb525beeef9be27c7285da394f5ee5396256d37a` |
| `otadata-initial.bin` | otadata_initial | present | `0xf10000` | `7d2c7ac4888bfd75cd5f56e8d61f69595121183afc81556c876732fd3782c62f` |
| `docs/release/license-inventory.md` | license_inventory | present via manifest field `license_inventory` | `n/a` | `n/a` |
| `docs/release/provenance-manifest.md` | provenance_manifest | present via manifest field `provenance_manifest` | `n/a` | `n/a` |

## Release-Gate Result

release_gate: passed

## Blocker Status

package_status: passed

No release-critical artifact from D-03 is missing. Downstream Phase 13 hardware or network evidence may trust this package identity only when it cites this same package manifest and source commit.

## Conclusion

Conclusion: passed - release-candidate package identity is recorded and the manifest-backed release gate passed before Phase 13 downstream hardware or network evidence is trusted.

Redaction review: passed for this file and the Task 2 command outputs. The reviewed package manifest fields and command results contain source/reference commits, artifact paths, checksums, ESP-IDF/Rust target metadata, and release document paths; they do not contain Wi-Fi credentials, pool credentials, API tokens, private endpoints, NVS secret values, private DEVICE_URL values, raw terminal secrets, or local private IP values.
