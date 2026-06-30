# Phase 13 Package Release Gate

## Command Log

- `just package`: not run
- `bazel run //tools/parity:report -- release-gate --manifest bazel-bin/firmware/bitaxe/bitaxe-ultra205-package.json`: not run

## Manifest Fields

- package manifest: not run
- source_commit: not run
- reference_commit: not run
- image_metadata.esp_idf_version: not run
- image_metadata.rust_target: not run
- default_flash_image: not run
- license_inventory: not run
- provenance_manifest: not run
- install_notes.path: not run

## Required Artifacts

| Artifact | Kind | Status |
| --- | --- | --- |
| `bitaxe-ultra205.elf` | firmware_elf | not run |
| `esp-miner.bin` | firmware_ota_image | not run |
| `www.bin` | www_spiffs_image | not run |
| `bitaxe-ultra205-factory.bin` | factory_merged_image | not run |
| `otadata-initial.bin` | otadata_initial | not run |
| `docs/release/license-inventory.md` | license_inventory | not run |
| `docs/release/provenance-manifest.md` | provenance_manifest | not run |

## Release-Gate Result

release_gate: not run

## Blocker Status

package_status: not run

## Conclusion

Conclusion: not run - Task 2 must run `just package`, validate the package manifest with the release gate, and replace every `not run` field before Phase 13 downstream hardware or network evidence is trusted.
