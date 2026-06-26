---
phase: 02-ultra-205-config-and-nvs-model
plan: "02"
subsystem: config
tags: [rust, nvs, schema, migrations, fixtures]

requires:
  - phase: 02-ultra-205-config-and-nvs-model
    provides: Ultra 205 defaults and board catalog from Plan 02-01
provides:
  - Typed pure NVS schema rows with exact upstream key and REST names
  - Pure NVS migration and default-load decisions for legacy storage
  - Reference-derived NVS schema and migration fixtures
affects: [config, persistence, api-settings, firmware-nvs-adapter, parity]

tech-stack:
  added: []
  patterns:
    - Pure functional-core NVS schema rows
    - Inert NVS adapter decisions for future firmware shell
    - Reference-derived golden fixtures with provenance

key-files:
  created:
    - crates/bitaxe-config/src/nvs.rs
  modified:
    - crates/bitaxe-config/BUILD.bazel
    - crates/bitaxe-config/src/lib.rs
    - crates/bitaxe-config/fixtures/nvs-schema.json
    - crates/bitaxe-config/fixtures/nvs-migrations.json

key-decisions:
  - "NVS schema and migrations remain pure data/functions; ESP-IDF reads, writes, erases, and commits stay deferred to a future firmware adapter."
  - "Legacy keys `asicfrequency`, `fanspeed`, and `fbSv2ChanType` are represented explicitly so migration decisions preserve upstream compatibility."
  - "Corrupt `FloatString` values fall back to schema defaults, including Ultra 205 `asicfrequency_f = 485.0`."

patterns-established:
  - "Use `NvsKeyName` and `RestFieldName` as separate parsed types so API names cannot replace NVS keys."
  - "Represent future NVS side effects as ordered `MigrationDecision::{Erase, Write}` values."
  - "Load missing or corrupt stored values through `SettingSchema.default_value` without mutating storage."

requirements-completed: [CFG-03, CFG-06]
generated_by: gsd-execute-plan
lifecycle_mode: yolo
phase_lifecycle_id: 2-2026-06-26T15-47-58
generated_at: 2026-06-26T17:04:47Z

duration: 13min
completed: 2026-06-26
---

# Phase 02 Plan 02: NVS Schema And Migration Model Summary

**Typed upstream-compatible NVS schema rows plus pure legacy migration/default-load decisions for Ultra 205 config parity**

## Performance

- **Duration:** 13 min
- **Started:** 2026-06-26T16:52:09Z
- **Completed:** 2026-06-26T17:04:47Z
- **Tasks:** 2
- **Files modified:** 5

## Accomplishments

- Added `crates/bitaxe-config/src/nvs.rs` with exact upstream namespace `main`, 15-byte NVS key validation, typed stored encodings, REST-name separation, defaults, ranges, indexed schema metadata, and reference breadcrumbs.
- Modeled legacy migrations as pure decisions for `asicfrequency` -> `asicfrequency_f`, `fanspeed` -> `manualfanspeed`, stratum protocol u16-to-string, SV2 channel-type u16-to-string, and `fbSv2ChanType` -> `fbsv2chantype`.
- Added default-load behavior for missing values and corrupt float strings, including `asicfrequency_f="bad"` loading as default `485.0`.
- Updated `nvs-schema.json` and `nvs-migrations.json` to capture reference-derived defaults, legacy aliases, and ordered expected erase/write operations.

## Task Commits

1. **Task 1: Implement typed NVS schema rows and key constraints** - `b5409f7` (feat)
2. **Task 2: Implement pure migration and default-load decisions** - `ef375e6` (feat)

## Files Created/Modified

- `crates/bitaxe-config/src/nvs.rs` - Pure typed NVS schema, migration decisions, load defaults, and focused tests.
- `crates/bitaxe-config/src/lib.rs` - Public exports for schema, migration, write/erase, stored, and loaded value types.
- `crates/bitaxe-config/BUILD.bazel` - Adds `src/nvs.rs` to the `bitaxe_config` Rust library.
- `crates/bitaxe-config/fixtures/nvs-schema.json` - Adds active defaults, scoreboard size 20, and legacy alias rows.
- `crates/bitaxe-config/fixtures/nvs-migrations.json` - Adds ordered operation arrays and corrupt-float fallback fixture case.

## Verification

- `bazel test //crates/bitaxe-config:tests --test_filter=nvs_schema` - passed
- `cargo fmt --all` - passed before each task commit
- `cargo clippy --all-targets --all-features -- -D warnings` - passed before each task commit
- `cargo build --all-targets --all-features` - passed before each task commit
- `cargo test --all-features` - passed before each task commit
- `bazel test //crates/bitaxe-config:tests` - passed
- `just parity` - passed, `validation_errors: none`
- `git status --short reference/esp-miner` - passed with no output

## Decisions Made

- Kept legacy aliases in the schema fixture and `all_settings_schema()` so key preservation is directly testable, even though upstream stores some legacy handling in migration code rather than the settings table.
- Used `FloatString` and `BoolAsU16` variants to encode upstream storage quirks without exposing callers to raw C `TYPE_FLOAT` or `TYPE_BOOL` names.
- Kept compatibility writes separate from active writes through `compatibility_writes_for_active()` so a future adapter can decide when to apply legacy mirror writes.

## Deviations from Plan

### Process Adjustments

**1. AGENTS.md pre-commit rule superseded failing RED commits**
- **Found during:** TDD setup for both tasks
- **Issue:** The TDD workflow allows failing RED commits, but AGENTS.md requires all Rust pre-commit checks to pass before every commit.
- **Adjustment:** RED failures were run and recorded in the session, but only passing task states were committed.
- **Impact:** No behavior change; preserves higher-priority repo policy.

### Auto-fixed Issues

**1. [Rule 3 - Blocking] Rewrote optional lookup to satisfy Clippy**
- **Found during:** Task 2 pre-commit Clippy gate
- **Issue:** Clippy denied a `let...else` optional return that should use `?`.
- **Fix:** Replaced the local `maybe_u16` guard with `let stored = maybe_stored_value(stored_values, key)?;`.
- **Files modified:** `crates/bitaxe-config/src/nvs.rs`
- **Verification:** `cargo clippy --all-targets --all-features -- -D warnings`, `cargo build --all-targets --all-features`, `cargo test --all-features`, and focused Bazel tests passed.
- **Committed in:** `ef375e6`

**Total deviations:** 1 process adjustment, 1 auto-fixed blocking issue.
**Impact on plan:** No scope creep; both changes were required to follow repo policy and pass verification.

## Issues Encountered

- Existing worktree state included parent-orchestrator planning artifacts and `.planning/config.json`; these were preserved and not included in task commits.
- `main` is still ahead of and behind `origin/main`; no rebase was attempted because the worktree was already dirty with parent-orchestrator state.

## User Setup Required

None - no external service configuration required.

## Known Stubs

None. The empty string defaults in `sv2authpubkey` and `fbsv2authpubk` are intentional upstream NVS defaults, not UI or data-source stubs.

## Next Phase Readiness

Plan 02-03 can build validation/update behavior on the typed `SettingSchema`, `StoredType`, `LoadedValue`, and inert write/erase decision model without adding ESP-IDF dependencies.

## Self-Check: PASSED

- Created/modified files exist: summary, NVS module, lib exports, Bazel wiring, schema fixture, migration fixture.
- Task commits exist: `b5409f7`, `ef375e6`.

---
*Phase: 02-ultra-205-config-and-nvs-model*
*Completed: 2026-06-26*
