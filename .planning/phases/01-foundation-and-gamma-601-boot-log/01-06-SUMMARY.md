---
phase: 01-foundation-and-gamma-601-boot-log
plan: "06"
subsystem: firmware-build
tags: [rust, esp-idf, esp32s3, bazel, firmware, gamma-601, bm1370]

requires:
  - phase: 01-foundation-and-gamma-601-boot-log
    provides: Plan 05 firmware package contract and bitaxe-core safe-state domain values
provides:
  - Safe Gamma 601/BM1370 ESP-IDF boot-log firmware entrypoint
  - Bazel-visible firmware build target at //firmware/bitaxe:firmware
  - Repo wrapper for Cargo/ESP-IDF firmware builds with pinned ESP-IDF settings
affects: [firmware, bazel, cargo, package, flash, evidence]

tech-stack:
  added: [esp-idf-svc, esp-idf-sys, embuild, log, anyhow]
  patterns:
    - ESP-IDF Rust app uses Cargo metadata plus wrapper-exported build environment
    - Bazel firmware target delegates to a repo-owned shell wrapper

key-files:
  created:
    - firmware/bitaxe/BUILD.bazel
    - firmware/bitaxe/build.rs
    - firmware/bitaxe/sdkconfig.defaults
    - scripts/build-firmware.sh
  modified:
    - .cargo/config.toml
    - .gitignore
    - Cargo.lock
    - firmware/bitaxe/Cargo.toml
    - firmware/bitaxe/src/main.rs
    - scripts/BUILD.bazel

key-decisions:
  - "Use Cargo build-std for xtensa-esp32s3-espidf so plain target commands work with the checked-in esp rust-src component."
  - "Export ESP_IDF_VERSION and related esp-idf-sys settings in the Bazel wrapper so Bazel cannot fall back to the crate default ESP-IDF v5.2.3."
  - "Use heap_caps_get_total_size(MALLOC_CAP_SPIRAM) for PSRAM status because the direct esp_psram_is_initialized symbol did not link in this build."

patterns-established:
  - "Firmware boot logs are literal-grep guarded and derived from bitaxe-core domain values where practical."
  - "Firmware Bazel builds run through scripts/build-firmware.sh and copy a declared ELF into bazel-bin."

requirements-completed: [FND-03, FND-04, FND-06]
generated_by: gsd-execute-plan
lifecycle_mode: yolo
phase_lifecycle_id: 1-2026-06-21T00-30-20
generated_at: 2026-06-21T03:17:06Z

duration: 16min
completed: 2026-06-21
---

# Phase 01 Plan 06: Safe Gamma 601 Boot/Log Firmware Summary

**ESP-IDF Rust boot-log firmware for Gamma 601/BM1370 with Bazel-owned xtensa-esp32s3 build output**

## Performance

- **Duration:** 16 min
- **Started:** 2026-06-21T03:01:15Z
- **Completed:** 2026-06-21T03:17:06Z
- **Tasks:** 2
- **Files modified:** 11

## Accomplishments

- Added the real `bitaxe-firmware` ESP-IDF Rust entrypoint with `EspLogger`, ESP-IDF patch linking, reset reason, partition label, PSRAM status, firmware commit fallback, pinned reference commit, ESP-IDF version, Rust target, and exact Gamma 601/BM1370 safe-state logs.
- Added ESP-IDF firmware metadata, sdkconfig defaults, `embuild` build script, and Cargo lock entries for the accepted `esp-idf-svc`/`esp-idf-sys` stack.
- Added `//firmware/bitaxe:firmware`, which invokes `scripts/build-firmware.sh` and produces `bazel-bin/firmware/bitaxe/bitaxe-firmware.elf`.

## Task Commits

1. **Task 1: Implement safe boot/log firmware entrypoint** - `48b35a4` (feat)
2. **Task 2: Add Bazel firmware build wrapper** - `6d502ea` (feat)

## Files Created/Modified

- `firmware/bitaxe/src/main.rs` - Safe ESP-IDF boot/log-only firmware entrypoint.
- `firmware/bitaxe/Cargo.toml` - Firmware dependencies and ESP-IDF v5.5.4 metadata.
- `firmware/bitaxe/build.rs` - ESP-IDF build-script setup and firmware commit capture.
- `firmware/bitaxe/sdkconfig.defaults` - ESP32-S3 Phase 1 SDK defaults.
- `firmware/bitaxe/BUILD.bazel` - Public `firmware` genrule target.
- `scripts/build-firmware.sh` - Rerunnable Cargo/ESP-IDF wrapper used by Bazel.
- `scripts/BUILD.bazel` - Exposes `build_firmware` as a `sh_binary`.
- `.cargo/config.toml` - Enables `build-std` for ESP target builds.
- `.gitignore` - Ignores generated `.embuild/` tool/cache state.
- `Cargo.lock` - Locks ESP-IDF Rust dependency graph.

## Decisions Made

- Kept the firmware entrypoint effect-only and did not introduce Wi-Fi, mining, ASIC work submission, HTTP, OTA, NVS mutation, voltage, fan, thermal, or power-control behavior.
- Configured Cargo `build-std` at the repo level because this ESP toolchain has `rust-src` but no prebuilt `xtensa-esp32s3-espidf` standard library.
- Exported ESP-IDF build settings in the wrapper because Bazel genrule execution did not reliably expose virtual-workspace package metadata to `esp-idf-sys`.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] Enabled Cargo build-std for ESP target**
- **Found during:** Task 1 verification
- **Issue:** The literal plan command `cargo check -p bitaxe-firmware --target xtensa-esp32s3-espidf` initially failed with `can't find crate for core`.
- **Fix:** Added `[unstable] build-std = ["std", "panic_abort"]` to `.cargo/config.toml`.
- **Files modified:** `.cargo/config.toml`
- **Verification:** `cargo check -p bitaxe-firmware --target xtensa-esp32s3-espidf` passed.
- **Committed in:** `6d502ea`

**2. [Rule 3 - Blocking] Made Bazel wrapper tolerate stripped action environment**
- **Found during:** Task 2 verification
- **Issue:** Bazel actions did not provide `HOME` or a PATH containing `cargo`.
- **Fix:** Inferred and exported `HOME` when absent, prepended `$HOME/.cargo/bin`, and added a clear Cargo availability failure.
- **Files modified:** `scripts/build-firmware.sh`
- **Verification:** `bazel build //firmware/bitaxe:firmware` advanced into Cargo/ESP-IDF and then passed after remaining fixes.
- **Committed in:** `6d502ea`

**3. [Rule 3 - Blocking] Exported pinned ESP-IDF settings in wrapper**
- **Found during:** Task 2 verification
- **Issue:** Inside Bazel, `esp-idf-sys` could not identify the virtual workspace root crate and fell back to ESP-IDF v5.2.3.
- **Fix:** Exported `ESP_IDF_SYS_ROOT_CRATE`, `ESP_IDF_VERSION=tag:v5.5.4`, `ESP_IDF_TOOLS_INSTALL_DIR=workspace`, and sdkconfig paths in the wrapper.
- **Files modified:** `scripts/build-firmware.sh`
- **Verification:** Bazel build logs showed `esp_idf_version=tag:v5.5.4` and produced the firmware ELF.
- **Committed in:** `6d502ea`

**4. [Rule 1 - Bug] Replaced unlinked PSRAM symbol**
- **Found during:** Task 2 Bazel build
- **Issue:** Release linking failed with undefined reference to `esp_psram_is_initialized`.
- **Fix:** Switched PSRAM status to `heap_caps_get_total_size(MALLOC_CAP_SPIRAM)`, which is available through the linked heap component.
- **Files modified:** `firmware/bitaxe/src/main.rs`
- **Verification:** `cargo check`, firmware clippy, and `bazel build //firmware/bitaxe:firmware` passed.
- **Committed in:** `6d502ea`

**5. [Rule 3 - Blocking] Ignored generated ESP build cache**
- **Found during:** Task 1 verification
- **Issue:** ESP-IDF build prep created `.embuild/` as untracked generated tool/cache state.
- **Fix:** Added `.embuild/` to `.gitignore`.
- **Files modified:** `.gitignore`
- **Verification:** `git status --short` no longer reported `.embuild/`.
- **Committed in:** `48b35a4`

***

**Total deviations:** 5 auto-fixed (1 bug, 4 blocking)
**Impact on plan:** All fixes were required to make the planned firmware and Bazel target build in the local ESP/Bazel environment. No scope was added beyond safe boot/log and build integration.

## Verification

Passed:

- `cargo fmt --all`
- `source "$HOME/export-esp.sh" && cargo check -p bitaxe-firmware --target xtensa-esp32s3-espidf`
- `source "$HOME/export-esp.sh" && cargo clippy -p bitaxe-firmware --target xtensa-esp32s3-espidf -- -D warnings`
- `bash -n scripts/build-firmware.sh`
- `bazel query //firmware/bitaxe:firmware`
- `bazel build //firmware/bitaxe:firmware`
- Firmware log and safety greps from the plan, including no active matches for `BM1370_init|send_work|stratum|wifi|fan|thermal|voltage|power`.
- `cargo clippy --workspace --exclude bitaxe-firmware --all-targets --all-features -- -D warnings`
- `cargo build --workspace --exclude bitaxe-firmware --all-targets --all-features`
- `cargo test --workspace --exclude bitaxe-firmware --all-features`

Known local limitation:

- `cargo clippy --all-targets --all-features`, `cargo build --all-targets --all-features`, and `cargo test --all-features` still fail when they try to compile `bitaxe-firmware` for host `aarch64-apple-darwin`; `esp-idf-sys` reports `Unsupported target 'aarch64-apple-darwin'`. Firmware verification must use the explicit `xtensa-esp32s3-espidf` target.

## Issues Encountered

- Bazel strips action environment differently than an interactive shell. The wrapper now restores only the required home, Cargo, and ESP-IDF build settings explicitly.
- No hardware smoke was run in this plan. The build produces an ELF, but boot-log observation on a connected Gamma 601 remains a later hardware evidence item.

## User Setup Required

No external services. Local firmware builds require the ESP Rust setup expected by the plan; `scripts/build-firmware.sh` fails with `espup install --targets esp32s3 --std` guidance if `$HOME/export-esp.sh` is missing.

## Next Phase Readiness

- Plan 07 can use the firmware implementation pointer and exact safe boot-log lines for parity/provenance reporting.
- Plan 08 can build from `//firmware/bitaxe:firmware` when creating package/image manifests.
- Plan 09 still needs live Gamma 601 flash/monitor evidence before any hardware-smoke parity claim.

***
*Phase: 01-foundation-and-gamma-601-boot-log*
*Completed: 2026-06-21*

## Self-Check: PASSED

- Found created files: `firmware/bitaxe/BUILD.bazel`, `firmware/bitaxe/build.rs`, `firmware/bitaxe/sdkconfig.defaults`, `scripts/build-firmware.sh`, and this summary.
- Found task commits: `48b35a4` and `6d502ea`.
