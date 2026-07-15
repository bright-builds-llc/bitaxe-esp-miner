---
phase: 34-provenance-runtime-health-and-coherent-operator-snapshot
plan: 01
subsystem: build-provenance
tags: [git, bazel, esp-idf, lcd, api, package, admission]
requires:
  - phase: 33-confirmed-settings-durability
    provides: Confirmed settings truth and the remapped Phase 35 reboot-qualification boundary
provides:
  - Canonical full-commit build provenance with release/dev and scoped dirty classification
  - Identical human identity on LCD, API, retained logs, ESP-IDF metadata, and package manifests
  - Schema-v3 exact-source package admission that rejects dirty or inconsistent input before hardware discovery
affects: [34-02-operator-snapshot, 34-03-platform-identity, phase-35-correlated-evidence]
tech-stack:
  added: []
  patterns: [stable workspace status, typed provenance stamp, generated sdkconfig, pre-hardware admission]
key-files:
  created:
    - scripts/build-identity-pathspecs.txt
    - scripts/build-identity-status.sh
    - scripts/build_identity.bzl
  modified:
    - crates/bitaxe-api/src/build_identity.rs
    - firmware/bitaxe/build.rs
    - firmware/bitaxe/src/main.rs
    - firmware/bitaxe/src/runtime_snapshot.rs
    - scripts/package-firmware.sh
    - tools/flash/src/main.rs
    - tools/xtask/src/package_manifest.rs
key-decisions:
  - "The full 40-character commit and structured machine fields authenticate source; the suffixed LCD/API label is presentation only."
  - "Clean untagged builds are eligible dev packages, while every dirty package or dirty current workspace fails before port discovery."
  - "Managed esptool elf2image owns the ESP application SHA insertion because espflash save-image does not populate that descriptor field."
patterns-established:
  - "Canonical identity: status primitives -> shared Rust validation -> versioned stamp -> every build/runtime/package surface."
  - "Hardware admission: validate manifest, current clean provenance, embedded identity, descriptor SHA, and artifact digests before resolving a port."
requirements-completed: [SYS-01, SYS-02]
generated_by: gsd-execute-plan
lifecycle_mode: interactive
phase_lifecycle_id: 34-2026-07-15T03-26-15
generated_at: 2026-07-15T05:35:50Z
duration: 67min
completed: 2026-07-15
---

# Phase 34 Plan 01: Canonical Build Identity Summary

**One strict build identity now labels development and dirty firmware visibly while preserving full-commit, descriptor, package-digest, and clean-workspace proof for hardware admission.**

## Performance

- **Duration:** 67 min
- **Started:** 2026-07-14T23:29:01-05:00
- **Completed:** 2026-07-15T00:35:50-05:00
- **Tasks:** 4
- **Implementation commits:** 4

## Accomplishments

- Added the exact four-state label matrix, allowed release-tag grammar, scoped dirty pathspec contract, stable Bazel workspace status, and one strict versioned provenance stamp.
- Propagated the same identity through ESP-IDF application metadata, the full-width LCD fourth line, system-info/live WebSocket, retained machine and human records, and manifest schema v3.
- Replaced package-time live Git identity inference with the declared stamp and generated an exact matching OTA/factory package through managed ESP-IDF artifacts and esptool.
- Added a fail-before-hardware admission boundary that accepts clean release and clean dev packages but rejects dirty state, stale or contradictory identity, embedded-field mismatch, and artifact-digest tampering before port or credential access.
- Built a clean exact-commit dev package at `694cf0ceb72c78fd16b20bc57beeac914f098ac6`; manifest identity, embedded label, descriptor SHA, package artifacts, current HEAD, and current workspace agreed, and inert-port dry-run admission passed.

## Task Commits

1. **Task 1: Build the typed identity core and scoped Git classifier** - `56006fe`
2. **Task 2: Materialize one Bazel identity and stamp the firmware descriptor** - `30e3966`
3. **Task 3: Project canonical identity to LCD, API, WebSocket, and logs** - `c9b9912`
4. **Task 4: Emit manifest v3 and enforce exact pre-hardware admission** - `694cf0c`

## Files Created/Modified

- `crates/bitaxe-api/src/build_identity.rs` - Canonical validation, label/channel derivation, stamp parsing, and workspace-status parsing.
- `scripts/build-identity-status.sh` and `scripts/build-identity-pathspecs.txt` - Closed Git classifier and scoped dirty contract.
- `scripts/build_identity.bzl`, `.bazelrc`, and firmware build wiring - Stable identity transport and cache-correct declared inputs.
- `firmware/bitaxe/build.rs`, runtime snapshot, and startup paths - Compile-time identity, descriptor SHA, LCD, public DTO, and retained records.
- `scripts/package-firmware.sh` and `tools/xtask/src/package_manifest.rs` - Descriptor-aware managed image generation and schema-v3 manifest authority.
- `tools/flash/src/main.rs` - Exact-source, descriptor, digest, and dirty-state admission before port resolution.
- `tools/parity/src/release_gate.rs`, `release_evidence.rs`, and `phase34_source_guard.rs` - Active v3 release/evidence validation and source-boundary regression guards.

## Decisions Made

- Kept dirty and channel orthogonal: a tagged dirty build is `release + dirty`, while an untagged clean build is eligible `dev`.
- Limited dirty classification to firmware-affecting source/build inputs, so planning, evidence, archived reference, ignored scratch, and unrelated host tooling do not mislabel firmware.
- Increased `CONFIG_APP_RETRIEVE_LEN_ELF_SHA` to 64 and treated the nonzero ESP application descriptor hash as a machine field independent of the presentation label.
- Required an admitted v3 manifest for every non-dry explicit image; synthetic dry-run parser tests remain possible without weakening real hardware entrypoints.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Used managed esptool generation after espflash produced a zero application-descriptor SHA**

- **Found during:** Real `just package` descriptor inspection.
- **Issue:** `espflash save-image` preserved the build label but left the ESP-IDF `app_elf_sha256` field as 64 zeros, so the new package gate correctly rejected the image.
- **Fix:** Generate the OTA with managed `esptool.py elf2image --elf-sha256-offset 0xb0`, bound it to the OTA partition size, and merge the factory image from the one matching generated ESP-IDF build's bootloader, partition table, OTA, SPIFFS, and otadata.
- **Verification:** The real descriptor now contains the expected clean-dev label and a matching nonzero 64-character SHA; schema-v3 packaging and dry-run admission pass.
- **Committed in:** `694cf0c`

**2. [Rule 3 - Blocking] Declared the missing package-shell Bazel test target**

- **Found during:** Task 4 focused Bazel verification.
- **Issue:** The plan named `//scripts:package_firmware_test`, but the checked-in shell test had no Bazel target.
- **Fix:** Added the local `sh_test` target with the package script as runfile data.
- **Verification:** Direct shell and focused Bazel package tests pass.
- **Committed in:** `694cf0c`

**Total deviations:** 2 auto-fixed implementation blockers.
**Impact on plan:** Both changes strengthened the declared descriptor/package boundary; no hardware, credential, Phase 35, or public mutation scope was added.

## Issues Encountered

- The repository-wide Bazel sweep passed 56 of 57 tests. Its only failure was an out-of-scope terminal-archive lifecycle test; policy forbids reopening or diagnosing that lineage. Every active Phase 33/34, package, flash, parity, Rust, and reference target passed.
- Firmware cross-build still reports pre-existing dead-code warnings; the canonical build succeeds and host clippy with `-D warnings` is clean.

## Verification

- Exact mandatory Rust sequence passed: format, all-target/all-feature clippy, build, and tests.
- Focused Cargo, package shell, shellcheck, shfmt, and Bazel targets passed.
- Phase 33 serial-session, detector, and confirmed-settings simulations passed.
- `just build`, `just package`, `just verify-reference`, and `git diff --check` passed.
- A real dirty package was rejected before port discovery; a real clean `694cf0ceb72c-dev` package was admitted through dry-run with `/dev/null` and no hardware access.

## User Setup Required

None. No board, credentials, network discovery, flash, reset, monitor, direct UART, or pin access was used.

## Next Phase Readiness

- Plan 34-02 can now bind all operator projections to one boot session and monotonic snapshot revision using the canonical identity fields established here.
- Plans 34-03 and 34-04 remain required before Phase 34 completion.
- Phase 35 remains the sole owner of the final clean current-package detector-gated run that jointly closes CFG-12 and EVD-13.

## Self-Check: PASSED WITH DOCUMENTED ARCHIVE EXCEPTION

- All four implementation commits exist and the clean exact-commit package/admission proof passed.
- Plan 01 focused tests and active software gates are green.
- The only full-sweep exception is the terminal-archive test described above; it was not investigated or modified.
- No push occurred.

***

*Phase: 34-provenance-runtime-health-and-coherent-operator-snapshot*
*Completed: 2026-07-15*
