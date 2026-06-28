---
phase: 07
slug: ota-filesystem-and-release-packaging
status: draft
nyquist_compliant: true
wave_0_complete: false
created: 2026-06-28
---

# Phase 07 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

***

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | Rust unit tests through Cargo/Bazel, package validation through `xtask`, parity checks through `tools/parity` |
| **Config file** | `Cargo.toml`, `Cargo.lock`, `MODULE.bazel`, `Justfile`, and Bazel BUILD files |
| **Quick run command** | `cargo test -p bitaxe-api --all-features && cargo test -p xtask --all-features` |
| **Full suite command** | `just test && just package && just parity` |
| **Estimated runtime** | TBD after Wave 0 measurement |

***

## Sampling Rate

- **After every task commit:** Run the touched crate/tool test command plus `git diff --check`.
- **After every plan wave:** Run `just test && just package && just parity`; add `just build` when firmware files or SDK config change.
- **Before `/gsd-verify-work`:** Full suite must be green, package artifacts generated, manifest v2 validated, license/provenance inventory present, operator docs reviewed, and parity/evidence rows updated.
- **Max feedback latency:** TBD after Wave 0 measurement; planner should keep pure/tool feedback under one command cycle before firmware/hardware checks.

***

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Threat Ref | Secure Behavior | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|------------|-----------------|-----------|-------------------|-------------|--------|
| 07-00-01 | 00 | 0 | REL-01 | T-07-static | Static and recovery decisions are testable before firmware effects. | unit | `cargo test -p bitaxe-api --all-features static` | no | pending |
| 07-00-02 | 00 | 0 | REL-02 | T-07-ota | OTA accept/reject/status decisions are typed and testable before ESP-IDF streaming effects. | unit | `cargo test -p bitaxe-api --all-features ota` | no | pending |
| 07-00-03 | 00 | 0 | REL-03 | T-07-otawww | OTAWWW either has tested decision paths or an explicit tested V1 gap path. | unit | `cargo test -p bitaxe-api --all-features otawww` | no | pending |
| 07-00-04 | 00 | 0 | REL-04 | T-07-manifest | Manifest v2 validates artifact kinds, offsets, checksums, source/reference commits, and install-note paths. | unit + package | `cargo test -p xtask --all-features package_manifest` | partial | pending |
| 07-00-05 | 00 | 0 | REL-05 | T-07-provenance | License/provenance gate covers Rust, Bazel/ESP-IDF, flashing tools, and static assets. | tool + docs | `just parity` | no | pending |
| 07-00-06 | 00 | 0 | REL-06 | T-07-flash | `just package` and flash tooling consume manifest v2 without manual artifact discovery. | package + unit | `just package && cargo test -p bitaxe-flash --all-features` | partial | pending |
| 07-00-07 | 00 | 0 | REL-07 | T-07-docs | Operator docs name safe Ultra 205 install, update, monitor, OTA, recovery, and gap procedures. | docs + parity | `just parity` | no | pending |
| 07-00-08 | 00 | 0 | REL-08 | T-07-recovery | Rollback, recovery, large erase, failed update, and interrupted update claims require evidence or explicit gaps. | parity + hardware | `just parity` | no | pending |

*Status: pending / green / red / flaky*

***

## Wave 0 Requirements

- [ ] `crates/bitaxe-api/src/update_plan.rs` and tests for OTA/OTAWWW accept/reject/status decisions.
- [ ] `crates/bitaxe-api/src/static_plan.rs` and tests for `/`, `.gz`, cacheable assets, missing redirect, `/recovery`, and filesystem-unavailable fallback.
- [ ] `tools/xtask` manifest v2 tests for artifact kinds, offsets, checksums, source/reference commits, install notes, and backwards-compatible `default_flash_image`.
- [ ] Partition table validation fixtures and either `esp-idf-part` integration tests or generated partition binary checks.
- [ ] `www.bin` generation fixture or build target with size/provenance checks.
- [ ] Cargo license inventory config plus explicit non-Cargo inventory schema.
- [ ] `docs/release/ultra-205.md` operator guide.
- [ ] `docs/parity/evidence/phase-07-ota-filesystem-release.md` evidence record with separate package, live firmware, hardware, and gap conclusions.

***

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| Live firmware OTA success/reboot/boot validation | REL-02, REL-08 | Requires connected Ultra 205 and real ESP-IDF OTA partition state. | Use the packaged OTA image against `/api/system/OTA`, capture request, response, logs, reboot, running partition, validation/rollback status, firmware commit, and reference commit. |
| Failed firmware OTA and rollback/recovery path | REL-02, REL-08 | Requires controlled bad image or validation failure on hardware. | Upload invalid or intentionally failing image, capture rejection or rollback logs, and record that the device remains recoverable. |
| OTAWWW direct SPIFFS update if implemented | REL-03, REL-08 | Requires real flash erase/write and interruption evidence. | Upload `www.bin`, capture success/failure, recovery page accessibility, and interrupted-update behavior. If not implemented, record explicit V1 gap evidence. |
| Recovery page and static asset smoke | REL-01, REL-07, REL-08 | Requires firmware HTTP server with packaged SPIFFS image. | Verify `/`, representative static assets, representative `.gz`, missing asset redirect, `/recovery`, and filesystem-unavailable fallback. |
| Operator documentation review | REL-07 | Human safety review needed before asking users to update hardware. | Review `docs/release/ultra-205.md` against actual package, flash, OTA, OTAWWW or gap, recovery, and rollback commands. |

***

## Validation Sign-Off

- [x] All requirements have planned automated or manual verification paths.
- [x] Wave 0 covers all missing automated references.
- [x] No watch-mode flags are required.
- [ ] Sampling continuity confirmed after concrete plans exist.
- [ ] Feedback latency measured after Wave 0.
- [x] `nyquist_compliant: true` set in frontmatter.

**Approval:** pending
