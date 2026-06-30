# Phase 13 Package Release Gate

## Command Log

- `just package`: passed
  - Result: Bazel built `//firmware/bitaxe:firmware_image` successfully.
  - Package manifest emitted at `bazel-bin/firmware/bitaxe/bitaxe-ultra205-package.json`.
- `bazel run //tools/parity:report -- release-gate --manifest bazel-bin/firmware/bitaxe/bitaxe-ultra205-package.json`: passed
  - Result: `release_gate: passed`

## Manifest Fields

- package manifest: `bazel-bin/firmware/bitaxe/bitaxe-ultra205-package.json`
- source_commit: `8a93fd0e45f34ea36fd3375dae92fbcf0ce738e8`
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
| `bitaxe-ultra205.elf` | firmware_elf | present | `Unavailable` | `85e39cd914e5e14137eb1d7d623e553821528722d9c9e1163c65080bf6e06315` |
| `esp-miner.bin` | firmware_ota_image | present | `0x10000` | `69079d90ab3bdbd19ed84ccd73498083675f94954a4cf629aebaee085f7b436b` |
| `www.bin` | www_spiffs_image | present | `0x410000` | `0dbb0eba0cc4198186d0175557ec134d7829f3426faf35d8baf263ee0a7c65a0` |
| `bitaxe-ultra205-factory.bin` | factory_merged_image | present | `0x0` | `c6d1c4f9ac139cd8d242eb060cb7636eca04ba0e81c71e23ca6622d968f5bd39` |
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
