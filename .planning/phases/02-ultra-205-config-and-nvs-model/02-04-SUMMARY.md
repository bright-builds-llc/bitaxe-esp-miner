---
phase: 02-ultra-205-config-and-nvs-model
plan: "04"
subsystem: config
tags: [rust, nvs, persistence, parity, evidence]

requires:
  - phase: 02-ultra-205-config-and-nvs-model
    provides: "Plan 02 NVS schema and Plan 03 schema-driven SettingsUpdateDecision writes"
provides:
  - "Pure in-memory NVS snapshot load, update, migration, and reload semantics"
  - "Phase 2 parity evidence for pure config/default/schema/validation/persistence behavior"
  - "CFG checklist rows scoped to pure evidence without firmware adapter or hardware-control claims"
affects: [phase-05-api, firmware-nvs-adapter, phase-06-safety, phase-03-asic]

tech-stack:
  added: []
  patterns:
    - "Represent persistence as deterministic in-memory snapshots plus inert NvsWrite/NvsErase decisions"
    - "Reload snapshots through the same pure migration/default-load path used for initial load"
    - "Keep parity rows explicit about pure evidence versus hardware/API/adapter evidence"

key-files:
  created:
    - crates/bitaxe-config/src/persistence.rs
    - docs/parity/evidence/phase-02-ultra-205-config-nvs-model.md
  modified:
    - crates/bitaxe-config/src/lib.rs
    - crates/bitaxe-config/BUILD.bazel
    - docs/parity/checklist.md

key-decisions:
  - "Keep persistence in crates/bitaxe-config pure: no ESP-IDF NVS calls, HTTP handlers, flashing, mining, ASIC, voltage, fan, thermal, power, or hardware side effects."
  - "Use SettingsUpdateDecision and existing NvsWrite/NvsErase data as the adapter contract for future firmware storage work."
  - "Leave CFG-001 and CFG-005 implemented rather than verified where the checklist row would otherwise overclaim hardware-control or API route evidence."

patterns-established:
  - "NvsSnapshot::from_values accepts raw StoredValue data and load_snapshot/reload_snapshot return loaded defaults plus typed migration commands."
  - "apply_patch_to_snapshot is all-or-nothing: rejected settings patches preserve the original snapshot and emit no writes."
  - "Phase evidence docs list command headings and scoped conclusions before checklist rows cite them."

requirements-completed: [CFG-05, CFG-06]
generated_by: gsd-execute-plan
lifecycle_mode: yolo
phase_lifecycle_id: 2-2026-06-26T15-47-58
generated_at: 2026-06-26T17:37:21Z

duration: 10 min
completed: 2026-06-26
---

# Phase 02 Plan 04: Persistence And Parity Evidence Summary

**Pure NVS snapshot persistence/reload semantics with Phase 2 parity evidence scoped to config-only behavior**

## Performance

- **Duration:** 10 min
- **Started:** 2026-06-26T17:27:24Z
- **Completed:** 2026-06-26T17:37:21Z
- **Tasks:** 2
- **Files modified:** 5

## Accomplishments

- Added `NvsSnapshot`, `PersistenceDecision`, `load_snapshot`, `apply_patch_to_snapshot`, and `reload_snapshot` for host-testable persistence behavior.
- Covered default load, missing-key load, valid update/reload roundtrip, invalid update rejection, legacy migration, and corrupt float fallback with focused Rust tests.
- Added Phase 2 parity evidence and updated CFG-001 through CFG-005 without claiming firmware NVS adapter, API PATCH route, non-205 board, or hardware-control verification.

## Task Commits

Each task was committed atomically:

1. **Task 1: Add pure snapshot persistence and reload semantics** - `3800fae` (`feat`)
2. **Task 2: Record parity checklist and evidence for Phase 2** - `4f38c25` (`docs`)

## Files Created/Modified

- `crates/bitaxe-config/src/persistence.rs` - Pure in-memory snapshot model, migration application, settings patch application, reload behavior, and tests.
- `crates/bitaxe-config/src/lib.rs` - Public persistence module and API re-exports.
- `crates/bitaxe-config/BUILD.bazel` - Adds `src/persistence.rs` to the Bazel Rust library.
- `docs/parity/evidence/phase-02-ultra-205-config-nvs-model.md` - Phase 2 command evidence and scoped conclusion.
- `docs/parity/checklist.md` - CFG rows updated with implementation pointers, pure evidence, and deferred boundaries.

## Decisions Made

- Kept persistence storage as a deterministic in-memory snapshot backed by key-ordered stored values, avoiding any firmware adapter or ESP-IDF dependency.
- Reused `apply_settings_patch` as the update authority so future API handlers can call the same validation/update path instead of duplicating REST-to-NVS mapping.
- Kept CFG-001 at `implemented` because the row includes frequency, voltage, and fan defaults whose hardware use remains unverified; CFG-003 and CFG-004 are `verified` with pure unit/golden evidence.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] Imported StoredValueKind from its owning module**
- **Found during:** Task 1
- **Issue:** The initial persistence implementation imported `StoredValueKind` from the crate root, but `lib.rs` intentionally re-exports `StoredValue` and not the raw payload enum.
- **Fix:** Imported `StoredValueKind` from `crate::nvs` inside `persistence.rs`, preserving the public API surface.
- **Files modified:** `crates/bitaxe-config/src/persistence.rs`
- **Verification:** `bazel test //crates/bitaxe-config:tests --test_filter=persistence`, `cargo clippy --all-targets --all-features -- -D warnings`, and `cargo test --all-features` passed.
- **Committed in:** `3800fae`

### Process Adjustments

**AGENTS.md precedence over TDD RED commits**
- **Found during:** Task 1
- **Issue:** The plan requested TDD RED flow, but repo instructions require `cargo fmt --all`, `cargo clippy --all-targets --all-features -- -D warnings`, `cargo build --all-targets --all-features`, and `cargo test --all-features` to pass before every Rust-project commit.
- **Adjustment:** Ran the failing persistence RED test, then committed only the passing implementation after the required Rust pre-commit checks.
- **Impact:** No behavior change; preserves higher-priority repo policy.

**Total deviations:** 1 auto-fixed blocking issue, 1 process adjustment.
**Impact on plan:** No scope expansion. All changes remained within pure config persistence and parity evidence.

## Issues Encountered

- Existing parent-orchestrator planning artifacts remained untracked and were preserved.
- `main` is ahead of and behind `origin/main`; no rebase or push was attempted per the execution instructions.

## Verification

- `bazel test //crates/bitaxe-config:tests --test_filter=persistence` - passed
- `bazel test //crates/bitaxe-config:tests` - passed
- `just test` - passed
- `just parity` - passed with `validation_errors: none`
- `git status --short reference/esp-miner` - clean output
- `cargo fmt --all` - passed before both task commits
- `cargo clippy --all-targets --all-features -- -D warnings` - passed before both task commits
- `cargo build --all-targets --all-features` - passed before both task commits
- `cargo test --all-features` - passed before both task commits

## Known Stubs

None.

## Threat Flags

None. The new persistence surface is the planned in-memory trust boundary and introduces no network endpoint, auth path, file access pattern, schema migration at an external trust boundary, or hardware side effect.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

Phase 2 config/default/schema/validation/persistence behavior is complete for pure Rust evidence. Firmware ESP-IDF NVS adapters, API PATCH handlers, reboot evidence, and hardware-control effects remain deferred to their owning later phases.

## Self-Check: PASSED

- Created files exist: `crates/bitaxe-config/src/persistence.rs`, `docs/parity/evidence/phase-02-ultra-205-config-nvs-model.md`, and this summary.
- Task commits exist: `3800fae`, `4f38c25`.
- Reference implementation remains unmodified.

---
*Phase: 02-ultra-205-config-and-nvs-model*
*Completed: 2026-06-26*
