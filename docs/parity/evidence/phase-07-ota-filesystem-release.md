# Phase 7 OTA, Filesystem, And Release Evidence

This rollup separates package, manifest, compile, live hardware, compliance, and
gap evidence so release-readiness language does not outrun the available proof.

Phase 7 now includes live serial hardware evidence for corrected factory flash,
partition layout, SPIFFS mount, PSRAM, boot-validation entry, safe startup, and
HTTP route registration on Ultra 205. Live network HTTP requests, OTA uploads,
rollback, large erase, failed update, and interrupted-update recovery are
deferred to the Phase 8 release evidence gate.

Relevant implementation evidence:

- `.planning/phases/07-ota-filesystem-and-release-packaging/07-04-SUMMARY.md`
  - SPIFFS mount/status adapter, static route handler, `/recovery`, fallback
    static assets, and gzip smoke asset implementation.
- `.planning/phases/07-ota-filesystem-and-release-packaging/07-05-SUMMARY.md`
  - Package generation, manifest v2, named release artifacts, and flash
    compatibility evidence.
- `.planning/phases/07-ota-filesystem-and-release-packaging/07-06-SUMMARY.md`
  - `release-gate`, license inventory, and provenance validation evidence.
- `.planning/phases/07-ota-filesystem-and-release-packaging/07-07-SUMMARY.md`
  - Firmware OTA streaming adapter, rollback boot validation adapter, and
    explicit OTAWWW REL-03 gap route.

## Command: `just package`

- Conclusion: package generation is implemented and host-verified by Plan
  07-05; this is package evidence only.
- Evidence source: `07-05-SUMMARY.md` records `just package` passing and the
  Bazel target producing named Ultra 205 release outputs.
- Expected manifest path:
  `bazel-bin/firmware/bitaxe/bitaxe-ultra205-package.json`.
- Expected artifacts: `bitaxe-ultra205.elf`, `esp-miner.bin`, `www.bin`,
  `otadata-initial.bin`, and `bitaxe-ultra205-factory.bin`.
- Live hardware conclusion: Phase 7 factory flash and serial boot smoke passed;
  live network HTTP/OTA verification is deferred to Phase 8.

## Command: manifest v2 validation

- Conclusion: manifest v2 validation is implemented and host-verified by Plan
  07-05.
- Evidence source: `07-05-SUMMARY.md` records schema version `2`, artifact
  kinds, offsets, SHA-256 values, source/reference commits, tool versions,
  release document paths, and otadata source metadata.
- Flash impact: `tools/flash` still resolves the top-level
  `default_flash_image` to `bitaxe-ultra205.elf`, while the manifest lists
  loose OTA, SPIFFS, otadata, partition, and factory artifacts.
- Live hardware conclusion: Phase 7 manifest-backed factory flashing was
  verified with `espflash write-bin 0x0`; live network HTTP/OTA verification is
  deferred to Phase 8.

## Command: partition/SPIFFS/static/recovery compile evidence

- Conclusion: partition, SPIFFS, static file serving, representative gzip, and
  recovery firmware surfaces compile and package, but live HTTP behavior has
  not been hardware-smoked in this rollup.
- Evidence source: `07-04-SUMMARY.md` records SPIFFS mount/status wiring,
  static route registration, explicit `/recovery`, Rust-owned fallback static
  assets, and representative gzip smoke path `/assets/app.css.gz`.
- Package source: `07-05-SUMMARY.md` records `www.bin` generation from the
  static filesystem tree.
- Serial hardware conclusion: the Ultra 205 boot log showed the expected
  partition table, `spiffs_mount=available`, and HTTP route registration.
- Live `/`, `/assets/app.css.gz`, missing static redirect, `/recovery`, and
  filesystem-unavailable HTTP response conclusions: deferred to Phase 8.

## Command: firmware OTA compile evidence

- Conclusion: firmware OTA runtime wiring is implemented and compile-verified;
  live firmware OTA remains pending.
- Evidence source: `07-07-SUMMARY.md` records streamed `/api/system/OTA`
  handling through ESP-IDF OTA APIs, upload gates, activation, success response
  `Firmware update complete, rebooting now!`, and reboot scheduling.
- Artifact source: `07-05-SUMMARY.md` records `esp-miner.bin` as the app OTA
  image in manifest v2.
- Live `/api/system/OTA` accepted upload, invalid image rejection, AP/APSTA
  rejection, reboot, and post-update identity conclusions: deferred to Phase 8.

## Command: boot validation compile evidence

- Conclusion: boot validation and rollback adapter code is implemented and
  compile-verified; live rollback behavior remains pending.
- Evidence source: `07-07-SUMMARY.md` records ESP-IDF OTA state inspection,
  valid-image marking, invalid-image rollback/reboot, and retained boot
  validation logs.
- Live rollback, bad-pending-image, failed update recovery, and return to
  operable-state conclusions: deferred to Phase 8.

## Command: `just parity`

- Conclusion: release-gate/license/provenance evidence exists for docs and
  release inputs, while publication remains gated on package artifacts and
  hardware evidence.
- Evidence source: `07-06-SUMMARY.md` records the `tools/parity release-gate`
  command, required Cargo report, non-Cargo license inventory, and provenance
  manifest checks.
- Release documents:
  - `docs/release/license-inventory.md`
  - `docs/release/provenance-manifest.md`
  - `docs/release/ultra-205.md`
- License/provenance conclusion: documentation evidence initialized and
  release-gate validated; generated artifact publication review remains tied to
  manifest checksums and hardware evidence.

## OTAWWW REL-03 Gap

- Conclusion: REL-03 remains an explicit V1 release gap for direct AxeOS static
  update through `/api/system/OTAWWW`.
- Evidence source: `07-07-SUMMARY.md` records the OTAWWW route preserving
  access gates, returning status 400 body `Wrong API input`, and logging
  `otawww_update=gap reason=interruption_evidence_missing
  owner=phase-07-release`.
- Public response: `Wrong API input`.
- Required operator gap copy:

```text
AxeOS update is not available in this release candidate. Use just package to create www.bin and flash the factory image, or use /recovery only after the documented evidence gate is complete.
```

- Release impact: owners can generate `www.bin` and a merged factory image, but
  the firmware does not claim safe whole-`www` partition replacement through
  AxeOS for this release candidate.
- Follow-up: implement and prove whole-partition SPIFFS erase/write, size
  checks, successful update, `/recovery` availability, and interrupted-update
  recovery on Ultra 205 before moving REL-03 above explicit gap status.
- Live static update and interrupted-update conclusions: deferred to Phase 8.

## Live Firmware OTA

- Conclusion: deferred to Phase 8.
- Required future evidence: board `Ultra 205`, port, firmware commit,
  reference commit, package manifest path, `esp-miner.bin` checksum, accepted
  upload response, invalid image rejection, reboot logs, running partition, and
  boot validation result.

## Live Rollback And Failed Update Recovery

- Conclusion: deferred to Phase 8.
- Required future evidence: pending-image state, validation pass/failure logs,
  invalid-image rollback/reboot logs, resulting running partition, and recovery
  steps that return the device to an operable state.

## Live Static And Recovery Smoke

- Conclusion: deferred to Phase 8.
- Required future evidence: `/`, `/assets/app.css.gz`, missing static redirect,
  `/recovery`, and API coexistence responses from the packaged firmware running
  on Ultra 205.

## Large Erase

- Conclusion: deferred to Phase 8.
- Required future evidence: erase command, port, package manifest path,
  `bitaxe-ultra205-factory.bin`, flash command, boot logs, static/recovery
  reachability, and final device state.

## Interrupted Update

- Conclusion: deferred to Phase 8.
- Required future evidence: point of interruption, route (`/api/system/OTA` or
  `/api/system/OTAWWW`), artifact checksum, post-interruption reachability,
  recovery procedure, and final conclusion.

## Final Release-Readiness Status

- Conclusion: Phase 7 release packaging and serial factory-boot readiness are
  established; final V1 release readiness is deferred to Phase 8.
- Package, manifest v2, partition/SPIFFS/static/recovery compile, firmware OTA
  compile, boot validation compile, release-gate, license, provenance, and
  REL-03 gap evidence are documented separately above.
- Live firmware OTA, rollback, recovery, large erase, failed update, and
  interrupted update are explicit Phase 8 release-gate evidence.
