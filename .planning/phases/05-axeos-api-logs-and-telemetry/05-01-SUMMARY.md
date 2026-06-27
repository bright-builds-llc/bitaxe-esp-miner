---
phase: 05-axeos-api-logs-and-telemetry
plan: 01
subsystem: api
tags: [rust, serde, axeos, api-fixtures, bazel]

requires:
  - phase: 04-stratum-v1-and-first-mining-loop
    provides: "MiningRuntimeState and safe blocked mining-loop status inputs"
  - phase: 03-bm1366-asic-protocol-and-safe-initialization
    provides: "BM1366 ASIC init/status input types"
  - phase: 02-ultra-205-config-and-nvs-model
    provides: "Ultra 205 defaults and board catalog facts"
provides:
  - "Handwritten initial AxeOS system and ASIC wire DTOs"
  - "Pure ApiSnapshot input boundary with config/catalog/mining/ASIC/platform facts"
  - "Safe Ultra 205 system-info and ASIC JSON fixtures with structured round-trip tests"
affects:
  - 05-02-settings-patch
  - 05-03-system-asic-statistics-scoreboard-mining
  - 05-04-logs-and-live-telemetry
  - 05-07-api-compare-and-static-evidence

tech-stack:
  added: [serde, serde_json, thiserror, bitaxe-config, bitaxe-stratum, bitaxe-asic]
  patterns:
    - "Handwritten Serde DTOs are separate from internal runtime/domain structs"
    - "Firmware-facing platform facts enter through a pure snapshot boundary"

key-files:
  created:
    - crates/bitaxe-api/src/snapshot.rs
    - crates/bitaxe-api/src/wire.rs
    - crates/bitaxe-api/fixtures/api/system-info-ultra205-safe.json
    - crates/bitaxe-api/fixtures/api/asic-settings-ultra205.json
  modified:
    - crates/bitaxe-api/src/lib.rs
    - crates/bitaxe-api/Cargo.toml
    - crates/bitaxe-api/BUILD.bazel
    - Cargo.lock
    - MODULE.bazel.lock

key-decisions:
  - "AxeOS wire DTOs are handwritten and remain independent from config, Stratum, ASIC, and platform domain structs."
  - "Safe Ultra 205 fixtures use public/synthetic defaults and zero Phase 6-owned hardware telemetry rather than claiming live voltage, fan, thermal, or power data."
  - "The API crate fixture glob allows an empty fixture set so Task 1 can compile before Task 2 creates the first fixture files."

patterns-established:
  - "SystemInfoWire and SystemAsicWire use explicit serde renames for upstream casing and compatibility quirks."
  - "ApiSnapshot::safe_ultra_205 provides a typed, no-ESP-IDF input fixture for early route-contract tests."

requirements-completed: [API-01, API-02, API-10]
generated_by: gsd-execute-plan
lifecycle_mode: yolo
phase_lifecycle_id: 5-2026-06-27T18-02-49
generated_at: 2026-06-27T19:52:05Z

duration: 11 min
completed: 2026-06-27
---

# Phase 05 Plan 01: API Contract Foundation Summary

**Handwritten AxeOS system and ASIC DTOs with typed safe Ultra 205 snapshots and exact JSON fixture coverage**

## Performance

- **Duration:** 11 min
- **Started:** 2026-06-27T19:40:23Z
- **Completed:** 2026-06-27T19:52:05Z
- **Tasks:** 2
- **Files modified:** 9

## Accomplishments

- Replaced the Phase 1 API deferral enum with `wire` and `snapshot` modules.
- Added initial handwritten Serde DTOs for upstream-visible system and ASIC fields, including exact casing and mixed numeric/boolean encodings.
- Added safe Ultra 205 system-info and ASIC JSON fixtures that parse into DTOs and round-trip as structured `serde_json::Value`.

## Task Commits

1. **Task 1: Establish bitaxe-api contracts and module ownership** - `1d254a6` (feat)
2. **Task 2: Add first captured-response fixtures for system and ASIC contracts** - `18a093e` (test)

## Files Created/Modified

- `crates/bitaxe-api/src/snapshot.rs` - Pure adapter input boundary with config/catalog/mining/ASIC/platform facts and safe Phase 6 telemetry defaults.
- `crates/bitaxe-api/src/wire.rs` - Handwritten AxeOS DTOs, compatibility helper, and fixture/type tests.
- `crates/bitaxe-api/fixtures/api/system-info-ultra205-safe.json` - Safe Ultra 205 system-info response fixture.
- `crates/bitaxe-api/fixtures/api/asic-settings-ultra205.json` - Ultra 205 ASIC settings response fixture.
- `crates/bitaxe-api/src/lib.rs`, `Cargo.toml`, `BUILD.bazel`, `Cargo.lock`, `MODULE.bazel.lock` - Public exports, dependencies, Bazel sources/data, and reproducible dependency metadata.

## Decisions Made

- Handwritten DTOs are the public contract; internal Rust structs only feed them through `ApiSnapshot`.
- Numeric bool-like upstream fields remain numeric (`apEnabled`, `autofanspeed`), while upstream booleans remain booleans (`miningPaused`, `showNewBlock`).
- Safe fixture values deliberately avoid live hardware claims for fan, voltage, thermal, power, and actual frequency.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] Allowed empty fixture glob before Task 2**
- **Found during:** Task 1
- **Issue:** Bazel failed while Task 1 compiled the API crate because `glob(["fixtures/**"])` was empty before Task 2 created fixture files.
- **Fix:** Added `allow_empty = True` to the API crate fixture globs.
- **Files modified:** `crates/bitaxe-api/BUILD.bazel`
- **Verification:** `bazel test //crates/bitaxe-api:tests`
- **Committed in:** `1d254a6`

**2. [Rule 3 - Blocking] Committed dependency lock metadata**
- **Found during:** Task 1
- **Issue:** Adding `serde`, `serde_json`, `thiserror`, `bitaxe-config`, `bitaxe-stratum`, and `bitaxe-asic` changed Cargo and crate_universe lock metadata.
- **Fix:** Included `Cargo.lock` and `MODULE.bazel.lock` with the task commit.
- **Files modified:** `Cargo.lock`, `MODULE.bazel.lock`
- **Verification:** `cargo build --all-targets --all-features`, `bazel test //crates/bitaxe-api:tests`
- **Committed in:** `1d254a6`

***

**Total deviations:** 2 auto-fixed (2 blocking)
**Impact on plan:** Both fixes kept the planned foundation reproducible and compilable without expanding route/module scope.

## Issues Encountered

- TDD RED failures were run but not committed because `AGENTS.md` requires passing Rust verification before every commit.

## Known Stubs

None blocking. Synthetic fixture values are intentional safe/public contract data for API comparison; live firmware API smoke and Phase 6 hardware-control telemetry remain future evidence.

## Authentication Gates

None.

## Verification

- `bazel test //crates/bitaxe-api:tests` - passed
- `bazel test //crates/bitaxe-api:tests --test_filter=wire` - passed
- `cargo fmt --all` - passed before each task commit
- `cargo clippy --all-targets --all-features -- -D warnings` - passed before each task commit
- `cargo build --all-targets --all-features` - passed before each task commit
- `cargo test --all-features` - passed before each task commit
- `just test` - passed
- `git status --short reference/esp-miner` - clean

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

Ready for Plan 05-02. The shared API crate now has module ownership, dependency wiring, and the first response-contract fixtures; later plans can add their own module files and build wiring without reviving the Phase 1 deferral.

## Self-Check: PASSED

- Found `.planning/phases/05-axeos-api-logs-and-telemetry/05-01-SUMMARY.md`
- Found `crates/bitaxe-api/src/snapshot.rs`
- Found `crates/bitaxe-api/src/wire.rs`
- Found `crates/bitaxe-api/fixtures/api/system-info-ultra205-safe.json`
- Found `crates/bitaxe-api/fixtures/api/asic-settings-ultra205.json`
- Found task commit `1d254a6`
- Found task commit `18a093e`

***
*Phase: 05-axeos-api-logs-and-telemetry*
*Completed: 2026-06-27*
