# Phase 8 Ultra 205 Release Summary

This summary closes the Phase 8 release gate from recorded evidence only. Package,
USB detector, and serial boot evidence exist for Ultra 205; live HTTP, firmware
OTA, recovery, rollback, failed-update, large erase, and interrupted-update
evidence remains blocked by no reachable `DEVICE_URL`.

## Commands Run

| Command | Result | Evidence |
| --- | --- | --- |
| `just parity` | passed | Task 1 checklist validation passed with `validation_errors: none`. |
| `just package` | passed | Ran before reading `bazel-bin/firmware/bitaxe/bitaxe-ultra205-package.json`; Bazel built `//firmware/bitaxe:firmware_image` and produced the Ultra 205 package artifacts. |
| `just test` | required final gate | Final Task 3 gate command; result is appended after execution. |
| `bazel run //tools/parity:report -- release-gate --manifest bazel-bin/firmware/bitaxe/bitaxe-ultra205-package.json` | required final gate | Final Task 3 gate command; result is appended after execution. |

## Package Manifest Used

Manifest path: `bazel-bin/firmware/bitaxe/bitaxe-ultra205-package.json`

| Field | Value |
| --- | --- |
| schema_version | 2 |
| release_name | `bitaxe-ultra205` |
| source commit | `c33e4f7143482be6566dd2e8249265b68af71c30` |
| reference commit | `c1915b0a63bfabebdb95a515cedfee05146c1d50` |
| default_flash_image | `bitaxe-ultra205.elf` |
| board | `205` |
| device_model | `Ultra 205` |
| asic | `BM1366` |
| esp_idf_version | `v5.5.4` |
| rust_target | `xtensa-esp32s3-espidf` |
| cargo | `cargo 1.88.0-nightly (873a06493 2025-05-10) (1.88.0.0)` |
| rustc | `rustc 1.88.0-nightly (2ab28d2e7 2025-06-24) (1.88.0.0)` |
| bazel | `Unavailable` |
| espflash | `espflash 4.0.1` |
| license_inventory | `docs/release/license-inventory.md` |
| provenance_manifest | `docs/release/provenance-manifest.md` |

| Artifact | Kind | Offset | SHA-256 |
| --- | --- | --- | --- |
| `esp-miner.bin` | `firmware_ota_image` | `0x10000` | `28af3f014328748977d446cff86a70d9c8c2773eece14a32b058abe723b99197` |
| `www.bin` | `www_spiffs_image` | `0x410000` | `0dbb0eba0cc4198186d0175557ec134d7829f3426faf35d8baf263ee0a7c65a0` |
| `bitaxe-ultra205-factory.bin` | `factory_merged_image` | `0x0` | `9ba7f0171382b51733fe894705d31a25840b0cb3d5dfaf4ba392361733e3b169` |
| `otadata-initial.bin` | `otadata_initial` | `0xf10000` | `7d2c7ac4888bfd75cd5f56e8d61f69595121183afc81556c876732fd3782c62f` |
| `bitaxe-ultra205-package.json` | manifest output | `n/a` | manifest file validated by release gate; artifact checksums listed above. |

## Hardware Evidence Files

| File | Evidence status |
| --- | --- |
| `docs/parity/evidence/phase-08-ultra-205-release-gate.md` | Package, detector, factory flash attempt, bounded serial boot capture, and explicit `DEVICE_URL status: blocked - no reachable DEVICE_URL`. |
| `docs/parity/evidence/phase-08-ultra-205-release-gate/flash-monitor-noninteractive.log` | Sanitized serial boot log with SPIFFS mount, boot-validation factory state, and route shell registration. |
| `docs/parity/evidence/phase-07-ultra-205-ota-hardware-smoke.md` | Phase 7 serial scope evidence for corrected factory flash, partition layout, SPIFFS mount, boot-validation entry, and HTTP route registration. |

## Checklist Rows Promoted

None. No Phase 8 row moved to `verified` because the live evidence file records
`DEVICE_URL status: blocked - no reachable DEVICE_URL`.

## Checklist Rows Deferred Or Pending

| Row | Status after Phase 8 closure | Reason |
| --- | --- | --- |
| `FS-001` | `implemented` | Live static, `/assets/app.css.gz`, missing static redirect, and `/recovery` HTTP smoke were not run. |
| `OTA-001` | `implemented` | Valid OTA, invalid image rejection, reboot, rollback, and boot-validation hardware evidence were not run. |
| `OTA-002` | `deferred` | OTAWWW remains the REL-03 gap with public response `Wrong API input`; no whole-`www` hardware-regression or interrupted-update evidence exists. |
| `REL-001` | `implemented` | Package and partition evidence exists; live OTA/recovery partition behavior remains blocked. |
| `REL-002` | `implemented` | SDK/package evidence exists; live rollback, recovery, and interrupted-update behavior remains blocked. |
| `REL-003` | `implemented` | Release-gate, provenance, and package workflow evidence exists, but rollback, recovery, large erase, failed update recovery, post-failed-update operability, recovery outcome, and interrupted-update evidence remain not run. |

## Reference Breadcrumb Audit

Task 3 performs the final audit over `crates`, `firmware`, and `tools` for
`Reference breadcrumb`, `Reference breadcrumbs`, and `reference/esp-miner`.
The final audit count and any source-module follow-up items are appended before
the Phase 8 plan summary is written.

## Deferred Scope Confirmation

The current release scope remains Ultra 205/BM1366 only.

| Scope | Current status |
| --- | --- |
| `CFG-002` | deferred for Gamma 601/BM1370; no Ultra 205 evidence applies. |
| `ASIC-008` | deferred BM1370 parity; no Ultra 205 evidence applies. |
| `ASIC-009` | not-started BM1368 parity. |
| `ASIC-010` | not-started BM1397 parity. |
| `STR-005` | deferred Stratum v2 protocol. |
| `BAP-001` | not-started BAP interface initialization. |
| `BAP-002` | not-started BAP protocol behavior. |
| `Angular` | out of V1 scope; API/static compatibility remains the current boundary. |
| `all-board` | deferred release matrix until each board has its own evidence set. |

## License And Provenance Status

- `docs/release/provenance-manifest.md` now records package manifest evidence
  for manifest-present artifacts and keeps each generated firmware image as a
  GPL-risk-reviewed release artifact.
- `docs/release/license-inventory.md` now cites this Phase 8 summary for
  release artifacts and keeps `Publication waits for final release approval`.
- `reference/esp-miner` remains pinned read-only at
  `c1915b0a63bfabebdb95a515cedfee05146c1d50`.
- No private URLs, Wi-Fi credentials, pool credentials, private endpoints, or
  NVS secret values are recorded in this summary.

## Residual Release Risk

- Live HTTP/static/recovery evidence is blocked because no IP, DHCP, Wi-Fi
  association, AP address, mDNS, hostname, or operator-supplied reachable URL
  was recorded.
- Firmware OTA accepted upload, invalid-image rejection, failed update recovery,
  post-failed-update operability, recovery outcome, rollback, large erase, and
  interrupted-update behavior remain unverified on hardware.
- OTAWWW remains a documented release gap with public response `Wrong API input`.
- Generated firmware artifacts remain GPL-risk-reviewed release artifacts until
  final release approval records source availability, installation information,
  and notice obligations.

## Final Conclusion

Phase 8 closes the current evidence ledger conservatively: package, manifest,
USB detector, serial boot, license, provenance, and release-gate inputs are
recorded, but final V1 release parity is not claimed for live HTTP, OTA,
rollback, recovery, failed-update, large erase, interrupted-update, OTAWWW, or
deferred roadmap scope.
