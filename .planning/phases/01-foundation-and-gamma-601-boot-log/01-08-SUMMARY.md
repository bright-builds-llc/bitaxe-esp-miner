---
phase: 01-foundation-and-gamma-601-boot-log
plan: "08"
subsystem: package-flash-tooling
tags: [rust, bazel, espflash, packaging, manifest, gamma-601, flash]

requires:
  - phase: 01-foundation-and-gamma-601-boot-log
    provides: Safe ESP-IDF firmware ELF target and reference cleanliness guard
provides:
  - Guarded firmware package manifest generation with SHA-256 artifact metadata
  - Public `//firmware/bitaxe:firmware_image` target producing ELF, factory bin, and JSON manifest
  - Typed Gamma 601 flash, monitor, and flash-monitor CLI with dry-run and manifest-default ELF resolution
affects: [just-package, just-flash, hardware-smoke, release-packaging, parity-evidence]

tech-stack:
  added: [sha2, tools/xtask, tools/flash, espflash save-image]
  patterns:
    - Bazel package action consumes declared firmware ELF output without nested Bazel
    - Host flash command execution uses `std::process::Command` argument vectors

key-files:
  created:
    - scripts/package-firmware.sh
    - tools/xtask/BUILD.bazel
    - tools/flash/BUILD.bazel
  modified:
    - Cargo.lock
    - MODULE.bazel.lock
    - firmware/bitaxe/BUILD.bazel
    - scripts/BUILD.bazel
    - tools/xtask/Cargo.toml
    - tools/xtask/src/main.rs
    - tools/flash/Cargo.toml
    - tools/flash/src/main.rs

key-decisions:
  - "Keep `default_flash_image` resolved to `bitaxe-gamma601.elf`; `bitaxe-gamma601-factory.bin` is only an additional package artifact with offset `0x0`."
  - "Run the reference guard from the real checkout path when Bazel actions start in the execroot."
  - "Honor AGENTS.md Rust pre-commit rules by recording TDD RED failures without committing failing intermediate states."

patterns-established:
  - "Package manifests use typed Rust serialization plus `sha2` checksums instead of shell-built JSON or custom hashing."
  - "Flash/monitor commands render and execute process argument vectors, with shell strings limited to display output only."

requirements-completed: [FND-03, FND-08, FND-09, FND-10]
generated_by: gsd-execute-plan
lifecycle_mode: yolo
phase_lifecycle_id: 1-2026-06-21T00-30-20
generated_at: 2026-06-21T03:57:22Z

duration: 20min
completed: 2026-06-21
---

# Phase 01 Plan 08: Package Manifest And Flash Tooling Summary

**Guarded Gamma 601 package manifests plus safe `espflash` dry-run/flash/monitor tooling using ELF defaults**

## Performance

- **Duration:** 20 min
- **Started:** 2026-06-21T03:37:05Z
- **Completed:** 2026-06-21T03:57:22Z
- **Tasks:** 2
- **Files modified:** 11

## Accomplishments

- Added `xtask package-firmware`, producing deterministic JSON with schema version, board/device/ASIC identity, firmware/reference commits, ESP-IDF/Rust target metadata, tool versions, SHA-256 checksums, and artifact entries.
- Added `scripts/package-firmware.sh` and `//firmware/bitaxe:firmware_image`, which run the reference guard first, consume `//firmware/bitaxe:firmware`, copy the declared ELF to `bitaxe-gamma601.elf`, generate `bitaxe-gamma601-factory.bin`, and write the manifest.
- Added `tools/flash`, with typed board parsing, key/value aliases, port discovery errors, dry-run output, manifest-default ELF resolution, and `Command::new("espflash")` execution.

## Task Commits

1. **Task 1: Generate firmware image manifest** - `8d1c70b` (feat)
2. **Task 2: Implement safe flash and monitor CLI** - `1fa494f` (feat)

## Files Created/Modified

- `scripts/package-firmware.sh` - Bazel-visible package wrapper with reference guard, ELF copy, `espflash save-image`, and xtask manifest generation.
- `firmware/bitaxe/BUILD.bazel` - Public `firmware_image` target declaring ELF, factory bin, and manifest outputs.
- `tools/xtask/src/main.rs` - `package-firmware` CLI, manifest model, checksum logic, guard enforcement, and tests.
- `tools/flash/src/main.rs` - `flash`, `monitor`, and `flash-monitor` CLI with dry-run, manifest resolution, port handling, evidence writing, and tests.
- `tools/xtask/BUILD.bazel`, `tools/flash/BUILD.bazel` - Bazel Rust binary/test targets.
- `Cargo.lock`, `MODULE.bazel.lock`, `tools/*/Cargo.toml`, `scripts/BUILD.bazel` - Dependency and target wiring.

## Decisions Made

- The manifest `default_flash_image` is `bitaxe-gamma601.elf`; the factory bin remains an additional artifact only.
- Packaging runs the reference guard from the real workspace path because Bazel execroot symlinks make `reference/esp-miner` look like an invalid submodule path to Git.
- Flash dry-runs with explicit images do not require the image file to exist, so developers can inspect the command shape without hardware or a local artifact.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] Resolved reference guard workspace under Bazel execroot**
- **Found during:** Task 1 package build
- **Issue:** `bazel build //firmware/bitaxe:firmware_image` initially failed because the guard ran from Bazel's execroot, where `reference/` is a symlink and Git rejects the submodule path.
- **Fix:** `scripts/package-firmware.sh` now resolves the real checkout from the `.git` symlink, exports `BUILD_WORKSPACE_DIRECTORY`, and changes to the checkout before guard/package work.
- **Files modified:** `scripts/package-firmware.sh`
- **Verification:** `bazel build //firmware/bitaxe:firmware_image` passed and printed `reference clean: c1915b0a63bfabebdb95a515cedfee05146c1d50`.
- **Committed in:** `8d1c70b`

**2. [Rule 3 - Blocking] Restored Cargo-installed tools in Bazel action PATH**
- **Found during:** Task 1 package build
- **Issue:** The Bazel package action could not find `espflash` even though it was installed for the user.
- **Fix:** `scripts/package-firmware.sh` prepends `$HOME/.cargo/bin` when present, matching the existing firmware build wrapper pattern.
- **Files modified:** `scripts/package-firmware.sh`
- **Verification:** `bazel build //firmware/bitaxe:firmware_image` produced `bitaxe-gamma601-factory.bin` via `espflash save-image`.
- **Committed in:** `8d1c70b`

**3. [Rule 3 - Blocking] Ran nested package build from the real workspace during `bazel run`**
- **Found during:** Task 2 manifest-default dry-run
- **Issue:** `bazel run //tools/flash:flash -- flash --board 601 --dry-run --port ...` failed because the binary started in a Bazel output directory and then invoked `bazel build`.
- **Fix:** `tools/flash` detects `BUILD_WORKSPACE_DIRECTORY` and runs package-build and `bazel info bazel-bin` with `current_dir` set to the checkout.
- **Files modified:** `tools/flash/src/main.rs`
- **Verification:** Manifest-default dry-run built `//firmware/bitaxe:firmware_image`, printed the manifest path, resolved `bitaxe-gamma601.elf`, and printed the `espflash flash` vector.
- **Committed in:** `1fa494f`

***

**Total deviations:** 3 auto-fixed (3 blocking)
**Impact on plan:** All fixes were required for the planned Bazel/package/flash commands to work in the local Bazel execution environment. No new product scope was added.

## Verification

Passed:

- `cargo test -p xtask`
- `bash -n scripts/package-firmware.sh`
- `bazel test //tools/xtask:tests`
- `bazel query 'deps(//firmware/bitaxe:firmware_image)' | grep '//scripts:verify_reference_clean'`
- `bazel query 'deps(//firmware/bitaxe:firmware_image)' | grep '//firmware/bitaxe:firmware'`
- `bazel build //firmware/bitaxe:firmware_image`
- Manifest checks confirming `"default_flash_image": "bitaxe-gamma601.elf"` and `bitaxe-gamma601-factory.bin` as an additional artifact.
- `cargo test -p bitaxe-flash`
- `bazel test //tools/flash:tests`
- `bazel run //tools/flash:flash -- flash --board 601 --dry-run --port /dev/cu.usbmodem101 --image /tmp/bitaxe-gamma601.elf`
- `bazel run //tools/flash:flash -- flash --board 601 --dry-run --port /dev/cu.usbmodem101`
- `cargo fmt --all`
- `cargo clippy --workspace --exclude bitaxe-firmware --all-targets --all-features -- -D warnings`
- `cargo build --workspace --exclude bitaxe-firmware --all-targets --all-features`
- `cargo test --workspace --exclude bitaxe-firmware --all-features`
- `source "$HOME/export-esp.sh" && cargo clippy -p bitaxe-firmware --target xtensa-esp32s3-espidf -- -D warnings`
- `source "$HOME/export-esp.sh" && cargo build -p bitaxe-firmware --target xtensa-esp32s3-espidf`

Known local limitation:

- The literal `cargo clippy --all-targets --all-features -- -D warnings` still fails for pre-existing firmware-target reasons: `esp-idf-sys` reports `Unsupported target 'aarch64-apple-darwin'` when `bitaxe-firmware` is compiled for the host. This limitation was already recorded in Plan 06; verification used the established host-workspace plus explicit ESP32-S3 firmware-target split.

## Known Stubs

None. The stub scan found only intentional `Unavailable` fallback literals for manifest/evidence fields and empty Bash argument accumulators before parsing.

## Threat Flags

None. The new process execution and package artifact trust surfaces match the plan threat model mitigations: typed CLI parsing, `Command` argument vectors, reference guard before trusted manifest output, and ELF-only default flash image validation.

## Issues Encountered

- Bazel execroot and `bazel run` working-directory behavior required explicit workspace detection in both packaging and flash tooling.
- `tools/flash/src/main.rs` and `tools/xtask/src/main.rs` exceed the advisory file-size refactor trigger because each keeps CLI, workflow logic, and unit tests in one binary file. They are good candidates for module splitting after Phase 1 command wiring stabilizes.

## User Setup Required

No new external services. Local package generation requires the already-established ESP Rust toolchain and `espflash`; the package script fails with install/upgrade guidance if `espflash` is unavailable.

## Next Phase Readiness

- Plan 09 can wire `just package`, `just flash`, `just monitor`, and `just flash-monitor` to the Bazel package and flash targets.
- Hardware smoke evidence is still pending; this plan only proves build/package/dry-run command behavior.

***
*Phase: 01-foundation-and-gamma-601-boot-log*
*Completed: 2026-06-21*

## Self-Check: PASSED

- Found created files: `scripts/package-firmware.sh`, `tools/xtask/BUILD.bazel`, `tools/flash/BUILD.bazel`, and this summary.
- Found task commits: `8d1c70b` and `1fa494f`.
