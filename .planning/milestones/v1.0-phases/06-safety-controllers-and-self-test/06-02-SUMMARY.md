---
phase: 06-safety-controllers-and-self-test
plan: "02"
subsystem: safety
tags: [rust, bazel, safety, module-graph, provenance]

requires:
  - phase: 06-01
    provides: Pure `bitaxe-safety` crate with shared status, evidence, and effect contracts
provides:
  - Public `bitaxe_safety::{power, thermal, fault, self_test, watchdog}` module paths
  - Breadcrumbed pure module boundary files for Phase 6 feature slices
  - Import and purity tests that keep feature boundaries free of firmware effects
affects: [phase-06, power, thermal, self-test, watchdog, asic-init, mining-gate]

tech-stack:
  added: []
  patterns: [pure module graph, reference-breadcrumbed safety boundaries]

key-files:
  created:
    - crates/bitaxe-safety/src/power.rs
    - crates/bitaxe-safety/src/thermal.rs
    - crates/bitaxe-safety/src/fault.rs
    - crates/bitaxe-safety/src/self_test.rs
    - crates/bitaxe-safety/src/watchdog.rs
  modified:
    - crates/bitaxe-safety/src/lib.rs
    - crates/bitaxe-safety/BUILD.bazel

key-decisions:
  - "Keep Phase 6 feature modules as pure breadcrumbed boundaries before adding behavior."
  - "Include new module files in the Bazel `bitaxe_safety` target immediately so Bazel tests cover the same module graph as Cargo."

patterns-established:
  - "Feature plans can now add power, thermal, fault, self-test, and watchdog behavior without parallel edits to `lib.rs`."
  - "Module boundary tests use source includes to enforce provenance breadcrumbs and forbidden firmware-effect imports."

requirements-completed: [SAFE-01, SAFE-02, SAFE-03, SAFE-04, SAFE-05, SAFE-07, SAFE-08, SAFE-09]
generated_by: gsd-execute-plan
lifecycle_mode: yolo
phase_lifecycle_id: 6-2026-06-28T02-37-37
generated_at: 2026-06-28T04:19:14Z

duration: 4 min
completed: 2026-06-28
---

# Phase 06 Plan 02: Safety Module Graph Summary

**Pure safety module graph for power, thermal, fault, self-test, and watchdog feature slices**

## Performance

- **Duration:** 4 min
- **Started:** 2026-06-28T04:15:00Z
- **Completed:** 2026-06-28T04:19:14Z
- **Tasks:** 1
- **Files modified:** 7

## Accomplishments

- Added public `power`, `thermal`, `fault`, `self_test`, and `watchdog` module exports to `bitaxe-safety`.
- Created pure module boundary files with upstream breadcrumbs for each Phase 6 feature slice.
- Added `safety_module_graph` tests proving imports, breadcrumbs, and absence of firmware-effect terms.

## Task Commits

1. **Task 1: Add Phase 6 feature module boundaries** - `acd3126` (feat)

## Files Created/Modified

- `crates/bitaxe-safety/src/lib.rs` - Exports the Phase 6 feature modules and tests the module graph.
- `crates/bitaxe-safety/src/power.rs` - Breadcrumbed power, voltage, and current safety boundary.
- `crates/bitaxe-safety/src/thermal.rs` - Breadcrumbed thermal, fan, and PID safety boundary.
- `crates/bitaxe-safety/src/fault.rs` - Breadcrumbed fault-policy boundary.
- `crates/bitaxe-safety/src/self_test.rs` - Breadcrumbed self-test lifecycle boundary.
- `crates/bitaxe-safety/src/watchdog.rs` - Breadcrumbed watchdog supervision boundary.
- `crates/bitaxe-safety/BUILD.bazel` - Includes the new module files in the Bazel library/test target.

## Decisions Made

- Kept these modules as boundary-only files so Plans 06-03 through 06-05 can add domain behavior without editing the crate facade.
- Included the new module files in Bazel now; otherwise the Bazel test target would not mirror Cargo module coverage.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] Added new module files to Bazel source list**
- **Found during:** Task 1 (Add Phase 6 feature module boundaries)
- **Issue:** The plan listed Rust module files but not `crates/bitaxe-safety/BUILD.bazel`; without updating Bazel `srcs`, `bazel test //crates/bitaxe-safety:tests` would not compile the new module graph consistently.
- **Fix:** Added `power.rs`, `thermal.rs`, `fault.rs`, `self_test.rs`, and `watchdog.rs` to the Bazel `rust_library` source list.
- **Files modified:** `crates/bitaxe-safety/BUILD.bazel`
- **Verification:** `bazel test //crates/bitaxe-safety:tests --test_filter=safety_module_graph`
- **Committed in:** `acd3126`

**Total deviations:** 1 auto-fixed blocking issue
**Impact on plan:** Scope stayed within the safety module graph and made Cargo/Bazel verification consistent.

## Issues Encountered

The first delegated executor stalled after writing files but before committing or summarizing. The orchestrator verified the partial work, ran the required checks, committed the source files, and completed the metadata path.

## Verification

- `node "$HOME/.codex/get-shit-done/bin/gsd-tools.cjs" verify lifecycle "06" --require-plans --raw` -> `valid`
- `cargo test -p bitaxe-safety --all-features safety_module_graph`
- `bazel test //crates/bitaxe-safety:tests --test_filter=safety_module_graph`
- `cargo fmt --all`
- `cargo clippy --all-targets --all-features -- -D warnings`
- `cargo build --all-targets --all-features`
- `cargo test --all-features`

## Known Stubs

Power, thermal, fault, self-test, and watchdog modules are intentionally boundary-only until Plans 06-03 through 06-05 add behavior.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

Ready for Wave 3. Plans 06-03, 06-04, and 06-05 can now add behavior in separate module files without sharing `lib.rs`.

## Self-Check: PASSED

- Confirmed created module files exist.
- Confirmed `pub mod power`, `pub mod thermal`, `pub mod fault`, `pub mod self_test`, and `pub mod watchdog` exports exist.
- Confirmed task commit `acd3126` exists in git history.

---
*Phase: 06-safety-controllers-and-self-test*
*Completed: 2026-06-28*
