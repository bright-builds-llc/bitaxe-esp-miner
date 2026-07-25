---
phase: 34-provenance-runtime-health-and-coherent-operator-snapshot
plan: 04
subsystem: runtime-observability
tags: [runtime-health, supervisor-checkpoint, monotonic-age, operator-snapshot, source-guard]
requires:
  - phase: 34-02-coherent-operator-snapshot
    provides: One boot-session and monotonic revision shared by every projection of a completed capture
  - phase: 34-03-running-platform-identity
    provides: Typed availability and one read-only running-device snapshot adapter
provides:
  - Pure passive self-test, supervisor, checkpoint-health, and task-watchdog vocabulary
  - Producer-owned monotonic supervisor checkpoints with deterministic stale and unhealthy derivation
  - One captured runtimeHealth value shared by system-info, live WebSocket, and retained records
  - Source guards proving the health read path cannot trigger self-test, watchdog, load, fault, or hardware effects
affects: [phase-35-correlated-evidence]
tech-stack:
  added: []
  patterns: [functional-core-imperative-shell, passive observation, checked monotonic arithmetic, shared correlated projection]
key-files:
  created:
    - crates/bitaxe-core/src/runtime_health.rs
    - firmware/bitaxe/src/runtime_health_adapter.rs
  modified:
    - firmware/bitaxe/src/safety_adapter/watchdog.rs
    - firmware/bitaxe/src/runtime_snapshot.rs
    - crates/bitaxe-api/src/snapshot.rs
    - crates/bitaxe-api/src/wire.rs
    - crates/bitaxe-api/src/runtime_projection.rs
    - tools/parity/src/phase34_source_guard.rs
key-decisions:
  - "Runtime health is an immutable captured value: a pure evaluator receives only producer-owned observations and monotonic time, and every public or retained surface projects that same value."
  - "Supervisor checkpoint visibility and ESP task-watchdog participation are independent; Phase 34 reports watchdog participation unavailable with reason unproved."
patterns-established:
  - "Passive health: producer checkpoint -> read-only adapter -> pure age evaluation -> one coherent operator snapshot -> shared projections."
  - "Fail-closed progress: missing, regressed, mutated, or arithmetically invalid checkpoints never project healthy."
requirements-completed: [HLT-01, HLT-02, HLT-03, HLT-04]
generated_by: gsd-execute-plan
lifecycle_mode: interactive
phase_lifecycle_id: 34-2026-07-15T03-26-15
generated_at: 2026-07-15T16:29:19Z
duration: 19min
completed: 2026-07-15
---

# Phase 34 Plan 04: Passive Runtime Health Summary

**One coherent operator snapshot now reports bounded passive self-test state, monotonic supervisor progress and age-derived health, and explicitly unproved task-watchdog participation across system-info, live WebSocket, and redacted retained records.**

## Performance

- **Duration:** 19 min
- **Started:** 2026-07-15T11:10:17-05:00
- **Completed:** 2026-07-15T11:29:19-05:00
- **Tasks:** 1
- **Implementation commits:** 1
- **Files:** 20

## Accomplishments

- Added a dependency-free functional core with all exact lifecycle and health spellings, bounded checkpoint categories, transition validation, checked age arithmetic, and deterministic healthy/stale/unhealthy thresholds.
- Added a producer-owned monotonic checkpoint to the existing safety supervisor and a read-only firmware adapter that copies checkpoint history into the pure evaluator without starting work or touching hardware.
- Captured runtime health once per completed operator snapshot and projected the same value through additive `runtimeHealth` system-info/live telemetry fields and one correlated redacted retained record.
- Kept supervisor visibility structurally separate from ESP task-watchdog participation, which remains `unavailable` with reason `unproved` without a direct proof source.
- Added focused boundary, regression, recovery, serialization, fixture, correlation, retained-redaction, and source-guard coverage, including the exact plan-owned Bazel targets.

## Task Commits

1. **Task 1: Implement and project passive self-test and supervisor/checkpoint health** - `c22cd53`

## Files Created/Modified

- `crates/bitaxe-core/src/runtime_health.rs` - Pure passive-health vocabulary, validated checkpoint history, checked age derivation, and deterministic unit tests.
- `firmware/bitaxe/src/runtime_health_adapter.rs` - Read-only bridge from existing producer observations into the pure evaluator.
- `firmware/bitaxe/src/safety_adapter/watchdog.rs` - Producer-owned bounded monotonic checkpoint sequence recorded after existing supervisor steps.
- `firmware/bitaxe/src/runtime_snapshot.rs` - One health capture and one correlated retained record per completed operator snapshot.
- `crates/bitaxe-api/src/snapshot.rs`, `wire.rs`, `runtime_projection.rs`, fixture, and dependency metadata - Shared additive `runtimeHealth` model with HTTP, WebSocket, retained, redaction, and compatibility coverage.
- `firmware/bitaxe/BUILD.bazel`, core/API/parity Bazel files, and `tools/parity/src/phase34_source_guard.rs` - Exact focused targets and automated passive/effect-free path enforcement.
- `Cargo.lock` and `MODULE.bazel.lock` - Cargo/Bazel dependency graph synchronization for the API-to-core dependency.

## Decisions Made

- Used the existing safety supervisor as the sole checkpoint producer; snapshot reads only clone its accepted previous/latest observations.
- Derived staleness from the existing 500 ms publisher cadence on every snapshot: healthy through three intervals, stale through ten intervals, and unhealthy afterward.
- Rejected same-sequence mutation as synthetic recovery while allowing repeated publication of one unchanged observation to age naturally.
- Kept task-watchdog participation explicitly unavailable and unproved even when the supervisor checkpoint is available and healthy.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] Added the minimal producer and dependency wiring required by the planned projections**

- **Found during:** Task implementation and focused Bazel verification.
- **Issue:** The plan named the pure model and reader but no existing firmware checkpoint store could supply producer-owned sequence/time history, and the API crate did not yet depend on the core model. The plan-owned test suites also needed package visibility.
- **Fix:** Added a passive checkpoint store to the already-running safety supervisor, synchronized Cargo/Bazel dependency locks, and exposed only the test targets needed by the exact verification command.
- **Verification:** The firmware build, focused runtime-health targets, full Bazel graph, and effect-free source guard all pass.
- **Committed in:** `c22cd53`

**2. [Rule 3 - Blocking] Repaired repository-specific GSD metadata after generic updater mismatch**

- **Found during:** Plan completion metadata update.
- **Issue:** The generic roadmap updater again interpreted the repository's four-column `Phase / Name / Requirements / Status` table as a different schema, and the generic decision appender did not recognize phase-qualified decision headings.
- **Fix:** Preserved the valid completion counts while restoring the established roadmap schema and recording the Plan 04 decisions under a phase-qualified state heading.
- **Verification:** ROADMAP reports Phase 34 complete at 4/4 plans and 20/27 completed requirements; STATE records Plan 04 completion and the two passive-health decisions.
- **Committed in:** Plan metadata commit.

**Total deviations:** 2 auto-fixed producer/dependency and metadata integration issues.
**Impact on plan:** The additions are the minimum wiring needed to make the planned observation truthful and testable; no active health control, hardware action, evidence promotion, or Phase 35 behavior was added.

## Issues Encountered

- The first focused Bazel run exposed target visibility needed by the exact plan-owned test suite; the visibility was narrowed to the firmware package and the rerun passed.
- The first firmware build reported one new unused re-export from the checkpoint reader; removing the redundant export eliminated the new warning. The remaining 14 firmware dead-code warnings predate Plan 04.
- No implementation, contract, source-guard, Rust, Bazel, firmware-build, or reference-clean verification remained failing.

## Verification

- Before the implementation commit, the mandatory sequence passed in exact order: `cargo fmt --all`, all-target/all-feature clippy with warnings denied, all-target/all-feature build, and all-feature tests.
- The exact focused plan command passed: core/API Cargo tests, all core/API Bazel targets, `runtime_health_tests`, `runtime_health_no_effects_test`, and `git diff --check`.
- `bazel test //...` passed all 58 repository test targets and rebuilt the ESP32-S3 firmware with pinned ESP-IDF `v5.5.4`.
- `just build` and `just verify-reference` passed; the pinned reference remained clean at `c1915b0a63bfabebdb95a515cedfee05146c1d50`.
- The complete diff and source guards contain no self-test execution, ESP task-watchdog mutation, delay/load/fault injection, hardware actuation, mining, credentials, network/device discovery, UART/pin work, OTA, other-board claim, Phase 35 behavior, or archived-lineage work.

## User Setup Required

None. No device detection, board access, credentials, network discovery, flash, reset, monitor, direct UART, pin manipulation, OTA, mining, or hardware evidence command was used.

## Next Phase Readiness

- Phase 34's coherent operator snapshot is complete across telemetry, settings, provenance, running-platform identity, and passive runtime health.
- Phase 35 remains the sole owner of detector-gated current-package evidence qualification and exact parity promotion.

## Self-Check: PASSED

- Implementation commit `c22cd53` exists and all plan-specific plus repository-wide software verification gates are green.
- HLT-01 through HLT-04 are implemented and recorded complete without any active self-test, watchdog intervention, or hardware effect.
- The orchestrator-owned todo file remains unstaged and uncommitted, no hardware or credentials were accessed, and no push occurred.

***

*Phase: 34-provenance-runtime-health-and-coherent-operator-snapshot*
*Completed: 2026-07-15*
