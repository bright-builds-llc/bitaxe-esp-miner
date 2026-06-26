---
phase: 02-ultra-205-config-and-nvs-model
plan: "03"
subsystem: config
tags: [rust, validation, nvs, settings, fixtures]

requires:
  - phase: 02-ultra-205-config-and-nvs-model
    provides: "Plan 02 pure NVS schema, SettingSchema rows, NvsWrite decisions, and legacy migration helpers"
provides:
  - "Typed validation newtypes and ConfigValidationError for config boundary values"
  - "Pure settings patch decisions that emit inert NVS writes from the Plan 02 schema"
  - "Golden settings update fixture cases for representative valid and invalid updates"
affects: [phase-05-api, firmware-nvs-adapter, phase-06-safety]

tech-stack:
  added: []
  patterns:
    - "Parse raw config input into Rust domain newtypes at the boundary"
    - "Walk all_settings_schema() as the single REST-to-NVS update source"
    - "Return inert NvsWrite values only; adapters own effects later"

key-files:
  created:
    - crates/bitaxe-config/src/validation.rs
    - crates/bitaxe-config/src/settings.rs
  modified:
    - crates/bitaxe-config/src/lib.rs
    - crates/bitaxe-config/BUILD.bazel
    - crates/bitaxe-config/fixtures/settings-updates.json

key-decisions:
  - "Keep frequency, voltage, fan, thermal, and settings validation as pure data checks with no ESP-IDF or hardware side effects."
  - "Use all_settings_schema() as the settings update authority so future API handlers do not duplicate validation or mapping logic."
  - "Preserve upstream legacy mirror writes for frequency and manual fan updates."

patterns-established:
  - "Config validation constructors return typed newtypes or ConfigValidationError."
  - "SettingsPatch uses REST field names and returns all-or-nothing SettingsUpdateDecision values."

requirements-completed: [CFG-04, CFG-06]
generated_by: gsd-execute-plan
lifecycle_mode: yolo
phase_lifecycle_id: 2-2026-06-26T15-47-58
generated_at: 2026-06-26T17:22:39Z

duration: 13 min
completed: 2026-06-26
---

# Phase 02 Plan 03: Typed Validation And Settings Update Decisions Summary

**Typed config boundary validation and schema-driven settings update decisions for Ultra 205 NVS compatibility**

## Performance

- **Duration:** 13 min
- **Started:** 2026-06-26T17:08:58Z
- **Completed:** 2026-06-26T17:22:39Z
- **Tasks:** 2
- **Files modified:** 5

## Accomplishments

- Added `ConfigValidationError` plus domain newtypes for frequency, voltage, fan duty, temperature, hostname, ports, TLS, bool-like values, Stratum protocol, SV2 channel type, board scope, and NVS key names.
- Added `SettingsPatch`, `RawSettingValue`, `SettingsUpdateDecision`, and `apply_settings_patch` for pure all-or-nothing settings update decisions.
- Extended `settings-updates.json` with expected golden writes, including legacy `asicfrequency` and `fanspeed` mirror writes.

## Task Commits

Each task was committed atomically:

1. **Task 1: Add validation newtypes and typed errors** - `4027d79` (`feat`)
2. **Task 2: Add pure settings update/apply decisions** - `989bf89` (`feat`)

## Files Created/Modified

- `crates/bitaxe-config/src/validation.rs` - Typed validation errors, domain newtypes, and focused validation tests.
- `crates/bitaxe-config/src/settings.rs` - Pure REST-name settings patch validation and inert NVS write decisions.
- `crates/bitaxe-config/src/lib.rs` - Public module and type exports.
- `crates/bitaxe-config/BUILD.bazel` - Bazel source, fixture compile data, and test dependency wiring.
- `crates/bitaxe-config/fixtures/settings-updates.json` - Golden valid/invalid settings update cases and expected writes.

## Decisions Made

- Used catalog-backed Ultra 205 BM1366 option validation for active frequency and voltage values, while keeping hardware-control parity unclaimed.
- Kept settings updates schema-driven by iterating `all_settings_schema()` and ignoring unknown REST names that have no schema row.
- Represented accepted writes as existing `NvsWrite` data, including compatibility mirror writes, instead of adding any adapter behavior.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] Added Bazel fixture/test wiring for settings RED tests**
- **Found during:** Task 2
- **Issue:** The RED test surfaced that `include_str!("../fixtures/settings-updates.json")` needed Bazel compile-time fixture visibility and that `serde_json` needed to be visible to the Bazel test target.
- **Fix:** Added `compile_data = glob(["fixtures/**"])` to the config library and `@crates//:serde_json` to the config test target.
- **Files modified:** `crates/bitaxe-config/BUILD.bazel`
- **Verification:** `bazel test //crates/bitaxe-config:tests --test_filter=fixtures`
- **Committed in:** `989bf89`

### Process Adjustments

**AGENTS.md precedence over TDD RED commits**
- **Found during:** Tasks 1 and 2
- **Issue:** The plan requested TDD RED commits, but repo instructions require the full Rust pre-commit sequence before every commit.
- **Adjustment:** Recorded RED failures with focused Bazel commands, then committed only the passing task states after `cargo fmt --all`, `cargo clippy --all-targets --all-features -- -D warnings`, `cargo build --all-targets --all-features`, and `cargo test --all-features`.
- **Impact:** No behavioral scope change; git history contains one green commit per completed task.

**Total deviations:** 1 auto-fixed blocking issue, 1 process adjustment.
**Impact on plan:** Implementation scope stayed within Plan 02-03 and remained pure.

## Issues Encountered

- Phase lifecycle validation reported a pre-existing metadata issue in `02-01-SUMMARY.md` (`generated_by`, `lifecycle_mode`, `phase_lifecycle_id`, and `generated_at` missing). Plan 02-03 and its lifecycle id were valid, so execution continued in plan-executor scope without editing prior plan artifacts.

## Verification

- `bazel test //crates/bitaxe-config:tests --test_filter=validation` - passed
- `bazel test //crates/bitaxe-config:tests --test_filter=fixtures` - passed
- `bazel test //crates/bitaxe-config:tests` - passed
- `cargo fmt --all` - passed before each task commit
- `cargo clippy --all-targets --all-features -- -D warnings` - passed before each task commit
- `cargo build --all-targets --all-features` - passed before each task commit
- `cargo test --all-features` - passed before each task commit
- `just parity` - passed
- `git status --short reference/esp-miner` - clean output

## Known Stubs

None. The only stub-pattern scan hit was an intentional invalid empty-hostname test input.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

Plan 02-04 can build persistence/reload semantics on top of typed validation and `SettingsUpdateDecision` without duplicating REST-to-NVS mapping. ESP-IDF NVS adapters, HTTP handlers, and hardware-control effects remain deferred to later phases and still require their own evidence.

## Self-Check: PASSED

- Created files exist: `validation.rs`, `settings.rs`, and `settings-updates.json`.
- Task commits exist: `4027d79`, `989bf89`.
- Reference implementation remains unmodified.

---
*Phase: 02-ultra-205-config-and-nvs-model*
*Completed: 2026-06-26*
