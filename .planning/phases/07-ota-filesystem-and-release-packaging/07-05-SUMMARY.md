---
phase: 07-ota-filesystem-and-release-packaging
plan: 05
subsystem: release-packaging
tags: [rust, xtask, bazel, espflash, spiffs, manifest-v2]

requires:
  - phase: 07-02
    provides: OTA partition and update metadata context
  - phase: 07-04
    provides: SPIFFS static asset layout and recovery fallback context
  - phase: 07-06
    provides: release install notes, license inventory, and provenance documents
provides:
  - Named Ultra 205 package outputs for ELF, app OTA image, SPIFFS www image, otadata image, merged factory image, and manifest v2
  - Manifest v2 generation and validation with checksums, offsets, source/reference commits, tool versions, release document pointers, and otadata source metadata
  - Flash CLI compatibility coverage proving manifest v2 still resolves top-level default_flash_image to the ELF
affects: [release-packaging, flashing, ota, static-filesystem, provenance]

tech-stack:
  added: []
  patterns:
    - ESP-IDF SPIFFS image generation remains inside the Bazel-owned package wrapper
    - Package manifests that record source commits declare Git metadata as Bazel inputs
    - Manifest v2 keeps top-level default_flash_image for existing flash tooling while adding artifact metadata

key-files:
  created:
    - .planning/phases/07-ota-filesystem-and-release-packaging/07-05-SUMMARY.md
  modified:
    - BUILD.bazel
    - firmware/bitaxe/BUILD.bazel
    - scripts/package-firmware.sh
    - tools/xtask/src/main.rs
    - tools/xtask/src/package_manifest.rs
    - tools/flash/src/main.rs
    - docs/release/provenance-manifest.md

key-decisions:
  - "Keep bitaxe-ultra205.elf as the top-level default_flash_image while listing loose OTA, SPIFFS, otadata, partition, and factory artifacts in manifest v2."
  - "Record partition-table offset as Unavailable for the checked-in CSV artifact; reserve 0x8000 for a future binary partition-table artifact."
  - "Declare .git/HEAD and .git/refs/heads/main as package action inputs so manifest source_commit is refreshed when the main branch advances."

patterns-established:
  - "Release package scripts validate all required manifest v2 metadata before printing success."
  - "Flash tooling ignores extra v2 manifest fields and resolves only the top-level default_flash_image."

requirements-completed: [REL-01, REL-04, REL-06]
generated_by: gsd-execute-plan
lifecycle_mode: yolo
phase_lifecycle_id: 7-2026-06-28T13-30-15
generated_at: 2026-06-28T17:26:07Z

duration: 24min
completed: 2026-06-28
---

# Phase 07 Plan 05: Release Package Manifest Summary

**Ultra 205 release packaging now emits named OTA/static/factory artifacts plus manifest v2 while preserving ELF-based flash defaults.**

## Performance

- **Duration:** 24 min
- **Started:** 2026-06-28T17:02:00Z
- **Completed:** 2026-06-28T17:26:07Z
- **Tasks:** 3
- **Files modified:** 7

## Accomplishments

- `just package` now produces `bitaxe-ultra205.elf`, `esp-miner.bin`, `www.bin`, `otadata-initial.bin`, `bitaxe-ultra205-factory.bin`, and `bitaxe-ultra205-package.json`.
- `xtask package-firmware` writes manifest schema v2 with artifact kinds, offsets, SHA-256 values, source/reference commits, ESP-IDF/Rust target metadata, release document paths, and otadata source metadata.
- `tools/flash` has v2 manifest regression coverage proving normal `just flash board=205` still resolves the ELF through `default_flash_image` and rejects factory images as defaults.

## Task Commits

Each task was committed atomically:

1. **Task 1: Generate www.bin, app OTA image, and otadata artifact** - `25fef22` (feat)
2. **Task 2: Write manifest v2 from real package outputs** - `4c0a908` (feat)
3. **Task 3: Preserve flash compatibility with manifest v2** - `d8504aa` (test)

Additional auto-fix commit:

- `4ce66e8` (fix) - Refresh manifest `source_commit` by making Git HEAD metadata a Bazel package input.

## Files Created/Modified

- `BUILD.bazel` - Exports Git HEAD metadata files used as package action inputs.
- `firmware/bitaxe/BUILD.bazel` - Declares the new package outputs and wires Git metadata into `firmware_image`.
- `scripts/package-firmware.sh` - Generates the SPIFFS image, app OTA image, otadata image, merged factory image, and manifest inputs.
- `tools/xtask/src/main.rs` - Adds manifest v2 CLI arguments and validation flow.
- `tools/xtask/src/package_manifest.rs` - Builds and validates manifest v2 artifact metadata.
- `tools/flash/src/main.rs` - Adds v2 manifest fixture coverage for default image resolution and factory rejection.
- `docs/release/provenance-manifest.md` - Documents the new `www.bin` and `otadata-initial.bin` package artifacts.

## Decisions Made

- Kept the ELF as the operator flash default so `espflash flash` continues to receive the same kind of input as earlier package manifests.
- Kept loose `esp-miner.bin` and `www.bin` as first-class artifacts instead of hiding them inside an archive.
- Used `Unavailable` as an explicit manifest value for artifact offsets that are not meaningful for the current file type.
- Treated manifest `source_commit` freshness as release correctness and added Bazel inputs rather than relying on a runtime `git rev-parse` alone.

## Verification

- `node /Users/peterryszkiewicz/.codex/get-shit-done/bin/gsd-tools.cjs verify lifecycle 07 --require-plans` passed before execution.
- `cargo fmt --all` passed before each implementation/fix commit.
- `cargo clippy --all-targets --all-features -- -D warnings` passed before each implementation/fix commit.
- `cargo build --all-targets --all-features` passed before each implementation/fix commit.
- `cargo test --all-features` passed before each implementation/fix commit.
- `cargo test -p xtask --all-features package_manifest` passed.
- `cargo test -p xtask --all-features validate_package` passed.
- `cargo test -p bitaxe-flash --all-features manifest` passed.
- `bazel test //tools/xtask:tests` passed.
- `bazel build //firmware/bitaxe:firmware_image` passed and produced the named package outputs.
- `just package` passed.
- Manifest check passed: schema version `2`, `source_commit` matched the then-current Git HEAD, and required artifact paths are present.
- `just flash board=205 --dry-run port=/dev/cu.usbmodem101` passed and resolved `default_flash_image` to `bitaxe-ultra205.elf`.
- `git diff --check` passed.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] Fixed clippy failures introduced by manifest v2 changes**
- **Found during:** Task 2
- **Issue:** The new CLI argument payload triggered `large_enum_variant`, and artifact construction triggered `vec_init_then_push` under `-D warnings`.
- **Fix:** Boxed the package subcommand payload and initialized artifact entries with `vec![]`.
- **Files modified:** `tools/xtask/src/main.rs`, `tools/xtask/src/package_manifest.rs`
- **Verification:** `cargo clippy --all-targets --all-features -- -D warnings` passed.
- **Committed in:** `4c0a908`

**2. [Rule 2 - Missing Critical] Recorded otadata source in manifest metadata**
- **Found during:** Task 2
- **Issue:** Task 1 required recording whether `otadata-initial.bin` was copied from ESP-IDF output or generated as erased flash, but the original Task 2 CLI argument list did not include that field.
- **Fix:** Added `--otadata-source` and manifest `otadata_source` metadata.
- **Files modified:** `scripts/package-firmware.sh`, `tools/xtask/src/main.rs`, `tools/xtask/src/package_manifest.rs`
- **Verification:** `bazel build //firmware/bitaxe:firmware_image` logged the source and generated manifest v2 with `otadata_source`.
- **Committed in:** `4c0a908`

**3. [Rule 1 - Bug] Fixed stale manifest source commits from Bazel caching**
- **Found during:** Whole-plan verification after Task 3
- **Issue:** `xtask package-firmware` read `git rev-parse HEAD`, but Bazel did not know the package action depended on Git HEAD. After new commits, a cached manifest could retain an old `source_commit`.
- **Fix:** Exported `.git/HEAD` and `.git/refs/heads/main` from the root package and added them as `firmware_image` inputs.
- **Files modified:** `BUILD.bazel`, `firmware/bitaxe/BUILD.bazel`
- **Verification:** Rebuilding after the fix reran package generation; manifest `source_commit` matched the then-current Git HEAD.
- **Committed in:** `4ce66e8`

### Process Adjustments

- The plan marked tasks as TDD. RED failure signals were captured before implementation, but separate RED commits were not created because repo-local pre-commit rules require the full Rust verification suite before every commit.

**Total deviations:** 3 auto-fixed, 1 process adjustment.
**Impact on plan:** All changes stayed within release packaging correctness and flash compatibility.

## Known Stubs

None. Stub scan found only intentional shell argument initializers and the manifest `Unavailable` sentinel for non-applicable offsets/tool values.

## Threat Flags

None. The plan's intended package-artifact and manifest-to-flash trust boundaries were implemented with checksums, source/reference commits, metadata validation, and flash default tests; no unplanned network, auth, file-write, or hardware-control surface was added.

## Issues Encountered

- Bazel action caching initially hid stale manifest `source_commit` metadata. This was fixed with explicit Git metadata inputs and verified by rebuilding after the fix commit.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

Future release inspection can consume a manifest v2 with loose OTA/static artifacts, checksums, release document pointers, and an ELF default flash path. Hardware flashing still remains an explicit operator action; this plan verified dry-run command resolution only.

## Self-Check

PASSED.

- Confirmed summary and all modified files exist.
- Confirmed commits `25fef22`, `4c0a908`, `d8504aa`, and `4ce66e8` exist.

---
*Phase: 07-ota-filesystem-and-release-packaging*
*Completed: 2026-06-28*
