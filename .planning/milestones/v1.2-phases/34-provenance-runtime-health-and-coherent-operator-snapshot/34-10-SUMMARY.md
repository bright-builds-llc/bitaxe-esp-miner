---
phase: 34-provenance-runtime-health-and-coherent-operator-snapshot
plan: "10"
subsystem: firmware-package-admission
tags: [esp32-s3, executable-envelope, package-manifest, elf-digest, fail-closed]
requires:
  - phase: 34-08-exact-immutable-package-admission
    provides: Immutable factory/OTA admission and exact package artifact digest checks
  - phase: 34-09-transactional-operator-snapshot-retention
    provides: Final preceding Phase 34 gap-closure wave
provides:
  - Pure typed ESP32-S3 executable-envelope validation for standalone OTA and embedded factory images
  - Pre-effect entry-point, load-address, MMU, checksum, and descriptor admission
  - Producer and consumer binding between the selected firmware ELF and app_elf_sha256
affects: [phase-34-review, phase-34-verification, phase-35-correlated-evidence]
tech-stack:
  added: []
  patterns: [pure-binary-parser, typed-closed-admission-errors, producer-consumer-digest-binding]
key-files:
  created:
    - tools/flash/src/esp32s3_image.rs
  modified:
    - tools/flash/BUILD.bazel
    - tools/flash/src/main.rs
    - tools/flash/src/package_admission.rs
    - tools/parity/src/phase34_source_guard.rs
    - tools/xtask/src/main.rs
    - tools/xtask/src/package_manifest.rs
key-decisions:
  - "The admitted ESP32-S3 envelope is a dependency-free pure parser with checked arithmetic, closed typed errors, and exact conservative address families derived from pinned ESP-IDF evidence."
  - "Zero-length segments retain upstream acceptance, while DROM/IROM file-offset congruence still applies whenever their load address selects a mapped family."
  - "The manifest producer and active flash consumer independently require the actual firmware ELF digest to equal app_elf_sha256 before any later artifact read or effect."
patterns-established:
  - "Executable admission: canonical header -> bounded segments -> address/MMU/entry validation -> descriptor identity -> checksum/digest trailer."
  - "Digest relationship: hash actual ELF bytes -> require one firmware_elf artifact -> compare to top-level app_elf_sha256 -> continue only on equality."
requirements-completed: []
generated_by: gsd-execute-plan
lifecycle_mode: interactive
phase_lifecycle_id: 34-2026-07-15T03-26-15
generated_at: 2026-07-16T20:56:03Z
duration: 34m
completed: 2026-07-16
---

# Phase 34 Plan 10: ESP32-S3 Executable and ELF Identity Admission Summary

**Factory flashing now requires a canonical, executable ESP32-S3 application image whose actual ELF, app descriptor, OTA bytes, and embedded factory slice all share one admitted identity.**

## Performance

- **Duration:** 34m
- **Started:** 2026-07-16T20:21:51Z
- **Completed:** 2026-07-16T20:56:03Z
- **Tasks:** 2
- **Implementation commits:** 2
- **Files:** 7

## Accomplishments

- Added a pure typed ESP32-S3 image validator covering the exact 24-byte header policy, segment-count and length bounds, checked structural parsing, checksum/padding/appended digest, app descriptor identity, and descriptor MMU page size.
- Enforced the conservative DROM, DRAM, IRAM, IROM, RTC data, and RTC fast load envelopes from pinned ESP-IDF sources, including mapped 64 KiB congruence, zero-length semantics, boundary crossing, overflow, and executable entry containment.
- Routed both the standalone OTA artifact and the factory partition's embedded application slice through the same validator before any port, credential, snapshot, or execution effect.
- Added parsed CLI regressions proving zero-load and mapped-congruence failures stop both dry-run and non-dry paths before later effects.
- Added one central producer relationship requiring the unique `firmware_elf.sha256` to equal `app_elf_sha256`, and invoked it while constructing and validating schema-v3 manifests.
- Added the corresponding active consumer check immediately after the unique firmware ELF is read and digest-validated, before OTA/factory reads or external effects.
- Kept fixture manifests coherent and strengthened Phase 34 source guards around producer validation and consumer ordering.

## Task Commits

1. **Task 1: Validate the admitted ESP32-S3 executable envelope** - `b83c7f68`
2. **Task 2: Bind the selected firmware ELF to app_elf_sha256** - `380e06b2`

## Files Created/Modified

- `tools/flash/src/esp32s3_image.rs` - Pure parser, typed error vocabulary, conservative memory envelopes, descriptor/trailer validation, and boundary-focused tests.
- `tools/flash/src/package_admission.rs` - Shared standalone/factory image validation and exact embedded-image binding.
- `tools/flash/src/main.rs` - Active pre-effect ELF relationship check plus parsed dry-run/non-dry regression sentinels.
- `tools/flash/BUILD.bazel` - Registered the executable-envelope module in the binary and Phase 34 source group.
- `tools/xtask/src/package_manifest.rs` - Central producer relationship validator and pre-output build-flow enforcement.
- `tools/xtask/src/main.rs` - Coherent manifest validation fixtures.
- `tools/parity/src/phase34_source_guard.rs` - Producer/consumer presence and ordering guards.

## Decisions Made

- Used only existing standard-library and `sha2` facilities; no dependency, package-graph, configuration, generated-file, or reference-tree changes were needed.
- Treated `0x403cb700`, `0x3fcdb700`, and the later bootloader loader/DRAM regions as excluded boundaries even though broader SoC families exist, matching the pinned ESP-IDF bootloader exclusions.
- Required an aligned entry address inside a non-empty admitted IRAM or IROM segment. Load-address family membership alone never authenticates executability.
- Preserved exact upstream zero-length segment acceptance but did not exempt mapped zero-length DROM/IROM segments from file-offset congruence.
- Kept `SYS-02` pending because this plan supplies implementation and regression evidence only; fresh Phase 34 review and independent verification retain authority over requirement completion.

## Deviations from Plan

None - plan executed within its specified architecture, safety scope, and no-tracking-change constraint.

## Issues Encountered

- The first Task 2 Bazel run exposed older xtask manifest fixtures whose placeholder firmware ELF digest did not match their top-level application digest. The fixtures were made coherent with the bytes they create, after which Cargo and Bazel suites passed.
- Long initial firmware/package and repository-wide Bazel invocations continued after their first output yield. Each was allowed to finish and then rerun from cache to capture an unambiguous successful result.

## Verification

- The exact mandatory Rust sequence passed before each implementation commit and again before this summary: `cargo fmt --all`, all-target/all-feature Clippy with warnings denied, all-target/all-feature build, and all-feature tests.
- Focused Cargo tests passed for `esp32s3_image`, package admission, executable admission, producer manifest validation, firmware ELF/application SHA admission, identity admission, and Phase 34 source guards.
- Focused Bazel tests passed for `//tools/xtask:tests`, `//tools/flash:tests`, and `//tools/parity:tests`.
- Repository-wide `bazel test //...` passed all 60 test targets; `just build`, `just package`, `just verify-reference`, ShellCheck, shfmt, and `git diff --check` passed.
- The real packaged image reports six legal ESP32-S3 segments, an IRAM entry point, valid checksum and validation hash, and descriptor MMU log2 value 16.
- Direct artifact inspection proved the unique manifest firmware ELF digest equals `app_elf_sha256`, the actual selected ELF hashes to that value, the app descriptor carries that value, and the factory image embeds the OTA bytes exactly at offset `0x10000`.
- The exact software-only `bazel run //tools/flash -- flash --board 205 --port /dev/null --manifest bazel-bin/firmware/bitaxe/bitaxe-ultra205-package.json --dry-run` admission passed and rendered the expected factory write command without executing it.
- No hardware, USB, serial, credentials, Wi-Fi or network discovery, flashing, OTA execution, direct UART/pins, Phase 35, or archived-lineage operation was used.

## User Setup Required

None.

## Next Phase Readiness

- All ten Phase 34 implementation plans now have summaries and the full software gate passes.
- `SYS-02` remains deliberately pending. Fresh Phase 34 review, regression review, and independent goal verification are the only authorized next actions; Phase 35 remains blocked until those authorities pass.

## Self-Check: PASSED

- Implementation commits `b83c7f68` and `380e06b2` exist and contain the two planned task changes.
- The summary carries lifecycle `34-2026-07-15T03-26-15` and exactly two standalone frontmatter delimiters.
- All focused and repository-wide software gates pass, `SYS-02` remains pending, and no push or prohibited hardware/Phase 35 activity occurred.

***

*Phase: 34-provenance-runtime-health-and-coherent-operator-snapshot*
*Completed: 2026-07-16*
