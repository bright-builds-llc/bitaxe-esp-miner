---
phase: 07-ota-filesystem-and-release-packaging
verified: 2026-06-28T21:22:55Z
status: passed
score: "5/5 scoped Phase 7 truths verified; live HTTP/OTA/recovery evidence deferred to Phase 8"
generated_by: gsd-verifier
lifecycle_mode: yolo
phase_lifecycle_id: "7-2026-06-28T13-30-15"
generated_at: 2026-06-28T21:22:55Z
lifecycle_validated: true
overrides_applied: 1
deferred_verification:
  - phase: "08-parity-evidence-and-ultra-205-release-gate"
    reason: "Live HTTP/OTA/recovery checks require a reachable DEVICE_URL; the recovered Ultra 205 serial run exposed no IP address, AP address, mDNS name, or other reachable URL."
    items:
      - "GET /"
      - "GET /assets/app.css.gz"
      - "missing static path redirect"
      - "GET /recovery"
      - "POST /api/system/OTAWWW with www.bin expecting Wrong API input"
      - "invalid POST /api/system/OTA safe rejection"
      - "valid POST /api/system/OTA with esp-miner.bin plus reboot and boot-validation logs"
      - "rollback, failed update, large erase, and interrupted-update recovery"
---

# Phase 7: OTA, Filesystem, and Release Packaging Verification Report

**Phase Goal:** Deliver release packaging, partition/filesystem behavior, OTA-capable firmware surfaces, explicit OTAWWW gap handling, release artifacts, license/provenance docs, and safe operator docs, with live network OTA/recovery verification deferred to Phase 8.
**Verified:** 2026-06-28T21:22:55Z
**Status:** passed
**Re-verification:** Yes - refreshed after deferring live network/fault-injection evidence to Phase 8

## Goal Achievement

### Observable Truths

| # | Truth | Status | Evidence |
| --- | --- | --- | --- |
| 1 | Package and flash named Ultra 205 artifacts with checksums, metadata, source/reference commits, install notes, and manifest entries. | VERIFIED | `scripts/package-firmware.sh`, `tools/xtask/src/package_manifest.rs`, `tools/flash/src/main.rs`, and `bazel-bin/firmware/bitaxe/bitaxe-ultra205-package.json` produce/consume manifest v2 with checksummed ELF, OTA image, SPIFFS image, factory image, partition table, otadata, source commit, reference commit, and install notes. `just flash-monitor board=205 port=/dev/cu.usbmodem1101 ...` wrote the merged factory image at `0x0` and verified the flash. |
| 2 | Factory-flashed Ultra 205 boots with the expected partition layout, SPIFFS state, PSRAM, safe startup, boot-validation entry, and HTTP route registration. | VERIFIED | `docs/parity/evidence/phase-07-ultra-205-ota-hardware-smoke.md` records `www`, `ota_0`, `ota_1`, `otadata`, and `coredump` in the partition table, `spiffs_mount=available`, `psram_status=available`, `ota_boot_validation=not_pending state=factory`, safe-state logging, display startup, and `axeos_api_route_shell=started registered_routes=15`. |
| 3 | Firmware OTA, static/recovery, and rollback code paths are implemented, host-tested, packaged, and guarded without claiming live parity. | VERIFIED | `crates/bitaxe-api/src/update_plan.rs`, `crates/bitaxe-api/src/static_plan.rs`, `firmware/bitaxe/src/http_api.rs`, `firmware/bitaxe/src/ota_update.rs`, `firmware/bitaxe/src/boot_validation.rs`, and `firmware/bitaxe/src/static_files.rs` implement the scoped surfaces. Live HTTP requests and OTA uploads are deferred to Phase 8. |
| 4 | OTAWWW/static asset update is explicitly reported as a V1 parity gap with evidence and owner. | VERIFIED | `/api/system/OTAWWW` is deliberately fail-closed with public `Wrong API input`; `AxeOsStaticOtaWww::gap()` records owner `phase-07-release`, impact, and follow-up; docs and evidence record the REL-03 gap. |
| 5 | License inventory, provenance, release docs, parity guardrails, and release-gate checks are in place. | VERIFIED | `docs/release/license-inventory.md`, `docs/release/provenance-manifest.md`, `docs/release/cargo-about.html`, `tools/parity/src/release_gate.rs`, `docs/release/ultra-205.md`, and release/OTA parity guards are validated by the commands below. |

**Score:** 5/5 scoped Phase 7 truths verified.

## Deferred Scope

The user approved splitting live network and destructive recovery verification out of Phase 7. Phase 8 now owns:

- reachable `DEVICE_URL` discovery or firmware network bring-up evidence
- live static HTTP checks for `/`, `/assets/app.css.gz`, missing-static redirect, and `/recovery`
- valid and invalid `/api/system/OTA` request behavior
- OTAWWW public gap response over HTTP
- rollback, failed update, large erase, and interrupted-update recovery evidence

No parity row was promoted to `verified` from package evidence alone.

## Behavioral Spot-Checks

| Behavior | Command | Result | Status |
| --- | --- | --- | --- |
| Formatting | `cargo fmt --all --check` | exit 0 | PASS |
| Rust lint | `cargo clippy --all-targets --all-features -- -D warnings` | exit 0 | PASS |
| Rust host build | `cargo build --all-targets --all-features` | exit 0 | PASS |
| Full Cargo suite | `cargo test --all-features` | 329 tests passed across crates and doc-tests | PASS |
| Bazel-backed repo suite | `just test` | 13 Bazel test targets passed | PASS |
| Firmware package build | `just package` | factory, OTA app, SPIFFS, otadata, ELF, and manifest outputs up to date | PASS |
| Firmware OTA planner behavior | `cargo test -p bitaxe-api --all-features update_plan` | 5 passed | PASS |
| Static asset/recovery resolver behavior | `cargo test -p bitaxe-api --all-features static_plan` | 7 passed | PASS |
| Package manifest behavior | `cargo test -p xtask --all-features package_manifest` | 3 passed | PASS |
| Package validation behavior | `cargo test -p xtask --all-features validate_package` | 4 passed | PASS |
| Flash tool manifest behavior | `cargo test -p bitaxe-flash --all-features manifest` | 4 passed | PASS |
| Release gate behavior | `cargo test -p bitaxe-parity --all-features release_gate` | 8 passed | PASS |
| Release/OTA verified-claim guards | `cargo test -p bitaxe-parity --all-features release_ota_verified_guard` | 7 passed | PASS |
| Current package manifest validates against partition table | `cargo run -p xtask -- validate-package --manifest bazel-bin/firmware/bitaxe/bitaxe-ultra205-package.json --partition-table firmware/bitaxe/partitions-ultra205.csv` | exit 0 | PASS |
| Ultra 205 detector | `just detect-ultra205` | selected `/dev/cu.usbmodem1101` after board-info passed | PASS |
| Factory flash command selection | `just flash dry-run=true board=205 port=/dev/cu.usbmodem1101` | rendered `espflash write-bin ... 0x0 ...bitaxe-ultra205-factory.bin` | PASS |
| Corrected factory boot capture | `just flash-monitor board=205 port=/dev/cu.usbmodem1101 evidence-dir=docs/parity/evidence/phase-07-ultra-205-ota-hardware-smoke` | flash verified; serial log showed partition table, SPIFFS mount, PSRAM, safe state, boot validation, and route registration | PASS |
| Release gate validates current release docs | `bazel run //tools/parity:report -- release-gate --manifest bazel-bin/firmware/bitaxe/bitaxe-ultra205-package.json` | `release_gate: passed` | PASS |
| Parity checklist validates evidence status | `just parity` | `validation_errors: none` | PASS |
| Markdown/code whitespace sanity | `git diff --check` | no output | PASS |

## Requirements Coverage

| Requirement | Source Plan | Description | Status | Evidence |
| --- | --- | --- | --- | --- |
| REL-01 | 07-01, 07-02, 07-04, 07-05, 07-09 | Filesystem/static/recovery partition behavior. | SATISFIED FOR PHASE 7 | Partition contract, SPIFFS mount, static resolver, recovery handler, package SPIFFS image, factory boot serial evidence, and checklist guard. |
| REL-02 | 07-01, 07-07, 07-09 | Firmware OTA route implementation and recovery wiring. | SATISFIED FOR PHASE 7 | OTA planner, HTTP handler, streaming adapter, boot validation, rollback entry points, tests, docs, and Phase 8 deferral for live OTA requests. |
| REL-03 | 07-01, 07-03, 07-07, 07-08, 07-09 | Static asset update behavior or explicit V1 parity gap. | SATISFIED | OTAWWW is explicitly reported as a V1 gap with owner, public response, evidence, and follow-up. |
| REL-04 | 07-02, 07-05, 07-08, 07-09 | Release package artifacts and manifest. | SATISFIED | Package script, manifest v2, validate-package command, package JSON in `bazel-bin`. |
| REL-05 | 07-03, 07-06, 07-08, 07-09 | License/provenance inventory and release gate. | SATISFIED | License inventory, provenance manifest, cargo-about report, release-gate tests and command. |
| REL-06 | 07-02, 07-05, 07-09 | Canonical package/flash command surface. | SATISFIED | `just package`, package script, dry-run flash evidence, and hardware `flash-monitor` evidence show the full factory image is selected and written at `0x0`. |
| REL-07 | 07-03, 07-04, 07-08, 07-09 | Operator docs for install/update/recover/release artifacts. | SATISFIED | `docs/release/ultra-205.md` covers build, package, flash, monitor, OTA, OTAWWW gap, recovery, large erase, rollback, and evidence capture. |
| REL-08 | Phase 8 | Rollback, recovery, large erase, failed update, and interrupted-update evidence before release parity. | DEFERRED | Moved to Phase 8 by explicit user scope decision; checklist rows remain below `verified`. |

## Final Status

Phase 7 is passed for the narrowed scope. It delivers package artifacts, manifest/flash behavior, release docs, license/provenance gates, OTA/static/recovery implementation, false-verification parity guards, and live Ultra 205 serial factory-boot evidence.

Live HTTP/OTA/recovery parity is intentionally not claimed and is tracked as Phase 8 release-gate work.
