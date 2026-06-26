---
phase: 02-ultra-205-config-and-nvs-model
plan: "01"
subsystem: config
tags: [rust, bazel, fixtures, ultra-205, nvs, board-catalog]

requires:
  - phase: 01-foundation-and-gamma-601-boot-log
    provides: "Ultra 205 safe-state identity, Bazel/Cargo workspace, reference guard, and parity tooling"
provides:
  - "Reference-derived Ultra 205 default fixtures with provenance metadata"
  - "Typed Ultra205Defaults API covering pool, ASIC, fan, self-test, device, and board defaults"
  - "Typed board/ASIC catalog with explicit VerificationScope for Ultra 205 and non-205 boards"
  - "NVS schema, migration, and settings update fixtures for downstream config plans"
affects: [phase-02, bitaxe-config, parity-checklist, nvs-model, board-catalog]

tech-stack:
  added: [csv, serde, serde_json, thiserror]
  patterns:
    - "Pure config data modules with reference breadcrumbs"
    - "VerificationScope separates catalog presence from hardware evidence"
    - "Fixture provenance metadata for GPL-derived reference data"

key-files:
  created:
    - crates/bitaxe-config/fixtures/ultra-205-defaults.csv
    - crates/bitaxe-config/fixtures/catalog.json
    - crates/bitaxe-config/fixtures/nvs-schema.json
    - crates/bitaxe-config/fixtures/nvs-migrations.json
    - crates/bitaxe-config/fixtures/settings-updates.json
    - crates/bitaxe-config/src/defaults.rs
    - crates/bitaxe-config/src/catalog.rs
  modified:
    - Cargo.lock
    - MODULE.bazel.lock
    - crates/bitaxe-config/Cargo.toml
    - crates/bitaxe-config/BUILD.bazel
    - crates/bitaxe-config/src/lib.rs
    - docs/parity/checklist.md

key-decisions:
  - "Keep Phase1BoardSelection::ultra_205() as a compatibility shim while exposing Phase 2 defaults/catalog modules."
  - "Represent all non-205 upstream boards in the catalog as NotHardwareVerified so Ultra 205 evidence cannot be inherited."
  - "Treat reference-derived fixture files as GPL-risk source data with explicit provenance metadata."

patterns-established:
  - "Defaults and catalog modules remain pure data with no firmware, NVS adapter, mining, voltage, fan, thermal, power, or ASIC initialization effects."
  - "Plan-level parity evidence updates are kept in docs/parity/checklist.md when pure config evidence lands."

requirements-completed: [CFG-01, CFG-02, CFG-06]
generated_by: gsd-execute-plan
lifecycle_mode: yolo
phase_lifecycle_id: 2-2026-06-26T15-47-58
generated_at: 2026-06-26T16:44:03Z

duration: 14 min
completed: 2026-06-26
---

# Phase 02 Plan 01: Ultra 205 Config Fixture And Catalog Summary

**Reference-derived Ultra 205 defaults, fixture corpus, and scoped board/ASIC catalog for the pure config model**

## Performance

- **Duration:** 14 min
- **Started:** 2026-06-26T16:29:34Z
- **Completed:** 2026-06-26T16:44:03Z
- **Tasks:** 2
- **Files modified:** 15

## Accomplishments

- Added provenance-marked fixtures for Ultra 205 defaults, board catalog, NVS schema keys, migration cases, and settings update cases.
- Added typed `Ultra205Defaults`, `BoardCatalogEntry`, `AsicProfile`, `BoardCapabilities`, and `VerificationScope` APIs in `bitaxe-config`.
- Preserved the Phase 1 identity shim while exposing Phase 2 module exports.
- Updated parity checklist CFG rows for implemented pure config defaults/catalog evidence without claiming hardware verification.

## Task Commits

1. **Task 1: Add Phase 2 fixture and Bazel test infrastructure** - `cb992d6` (feat)
2. **Task 2: Implement Ultra 205 defaults and board catalog types** - `55ec3d3` (feat)

_Note: The TDD RED failure was run and recorded, but not committed because AGENTS.md requires passing Rust verification before any commit._

## Files Created/Modified

- `crates/bitaxe-config/fixtures/ultra-205-defaults.csv` - `config-205.cvs` seed defaults with source, commit, and license posture metadata.
- `crates/bitaxe-config/fixtures/catalog.json` - Machine-readable Ultra 205 and non-205 board catalog fixture.
- `crates/bitaxe-config/fixtures/nvs-schema.json` - Active NVS key-name fixture for downstream schema modeling.
- `crates/bitaxe-config/fixtures/nvs-migrations.json` - Legacy key and u16-to-string migration cases.
- `crates/bitaxe-config/fixtures/settings-updates.json` - Representative valid and invalid settings update cases.
- `crates/bitaxe-config/src/defaults.rs` - Typed Ultra 205 defaults API.
- `crates/bitaxe-config/src/catalog.rs` - Typed ASIC profiles, board catalog entries, capabilities, and verification scope.
- `crates/bitaxe-config/src/lib.rs` - Module exports, compatibility shim retention, and behavior tests.
- `crates/bitaxe-config/BUILD.bazel` - Added module sources, dependency labels, and fixture runfiles.
- `crates/bitaxe-config/Cargo.toml`, `Cargo.lock`, `MODULE.bazel.lock` - Dependency wiring and regenerated lock metadata.
- `docs/parity/checklist.md` - CFG-001 and CFG-003 evidence notes.

## Decisions Made

- Kept `Phase1BoardSelection::ultra_205()` unchanged as a compatibility shim rather than forcing existing Phase 1 callers onto the richer Phase 2 defaults model.
- Modeled non-205 boards as present catalog data but `NotHardwareVerified`; only board `205` uses `ActiveUltra205`.
- Kept NVS/runtime validation implementation out of this plan; this plan provides fixtures and typed defaults/catalog only.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 2 - Missing Critical] Updated parity checklist evidence for implemented config surfaces**
- **Found during:** Overall verification
- **Issue:** `docs/parity/checklist.md` still marked the board/device catalog row as `not-started` after Task 2 implemented and tested the pure catalog.
- **Fix:** Updated CFG-001 and CFG-003 notes/statuses to reference `Ultra205Defaults`, `BoardCatalogEntry`, `AsicProfile`, `VerificationScope`, and the relevant Bazel tests while preserving hardware evidence gates.
- **Files modified:** `docs/parity/checklist.md`
- **Verification:** `just parity`
- **Committed in:** Plan metadata commit

### Process Adjustments

**1. AGENTS.md pre-commit rule superseded TDD RED commit**
- **Found during:** Task 2 TDD RED
- **Issue:** The generic TDD workflow asks for a failing RED commit, but repo Rust rules require `cargo fmt --all`, `cargo clippy --all-targets --all-features -- -D warnings`, `cargo build --all-targets --all-features`, and `cargo test --all-features` to pass before any commit.
- **Fix:** Ran the RED test and recorded the expected unresolved import failure, then committed only the passing GREEN implementation.
- **Verification:** RED failure: `bazel test //crates/bitaxe-config:tests --test_filter=ultra_205_defaults`; GREEN checks passed before commit.

---

**Total deviations:** 1 auto-fixed (1 missing critical evidence update), 1 process adjustment.
**Impact on plan:** The implementation scope stayed within the plan; the added checklist update keeps parity evidence aligned with the completed pure config work.

## Issues Encountered

- `origin/main` advanced by one Bright Builds rules refresh commit during startup. Remote refs were fetched, but the working tree already contained uncommitted GSD phase planning files, so no rebase was attempted. The plan work proceeded on the requested main working tree and all verification passed.
- `cargo update` updated compatible existing lock entries as part of the plan-specified dependency refresh, in addition to adding the new `csv` dependency graph.

## Verification

- `cargo update`
- `bazel test //crates/bitaxe-config:tests`
- `bazel test //crates/bitaxe-config:tests --test_filter=ultra_205_defaults`
- `bazel test //crates/bitaxe-config:tests --test_filter=board_catalog`
- `cargo fmt --all`
- `cargo clippy --all-targets --all-features -- -D warnings`
- `cargo build --all-targets --all-features`
- `cargo test --all-features`
- `just parity`
- `git status --short reference/esp-miner`

## Known Stubs

None. The empty Wi-Fi fields in `ultra-205-defaults.csv` are intentional upstream public defaults and do not feed a UI or live runtime path in this plan.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

Ready for Plan 02-02. The downstream NVS schema/default/migration implementation can build on the fixture files, and later safety-critical hardware effects remain blocked on their own evidence.

## Self-Check: PASSED

- Created files verified on disk.
- Task commits `cb992d6` and `55ec3d3` verified in git history.

---
*Phase: 02-ultra-205-config-and-nvs-model*
*Completed: 2026-06-26*
