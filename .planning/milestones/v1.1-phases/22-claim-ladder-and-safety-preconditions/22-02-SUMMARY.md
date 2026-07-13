---
phase: 22-claim-ladder-and-safety-preconditions
plan: 02
subsystem: safety
tags: [production-mining, safety-preconditions, blocker-reasons, stratum, api, rust, bazel]
requires:
  - phase: 22-claim-ladder-and-safety-preconditions
    provides: operator-visible claim ladder and controlled no-share overclaim guard
provides:
  - Typed production-mining prerequisite contract for power, thermal, fan, voltage, and safety readiness
  - Bounded Ultra 205 observation evidence validation with stable blocker reasons
  - Mining-loop exact blocker reason propagation into runtime state and API mining output
affects: [phase-23-evidence-workflow, phase-24-bm1366-production-path, phase-25-live-stratum-runtime, phase-26-telemetry-and-parity]
tech-stack:
  added: []
  patterns: [typed prerequisite contract, fail-closed decision projection, exact blocker reason storage]
key-files:
  created:
    - crates/bitaxe-safety/src/mining_preconditions.rs
    - .planning/phases/22-claim-ladder-and-safety-preconditions/22-02-SUMMARY.md
  modified:
    - crates/bitaxe-safety/BUILD.bazel
    - crates/bitaxe-safety/src/lib.rs
    - crates/bitaxe-stratum/src/v1/mining_loop.rs
    - crates/bitaxe-stratum/src/v1/state.rs
    - crates/bitaxe-api/src/mining.rs
    - crates/bitaxe-stratum/src/v1/controlled_runtime.rs
    - crates/bitaxe-stratum/src/v1/controlled_runtime/tests.rs
    - firmware/bitaxe/src/controlled_mining_runtime.rs
key-decisions:
  - "Modeled production mining prerequisites as typed Fresh, Bounded, or Blocked inputs instead of accepting shell-owned readiness strings."
  - "Kept existing power, thermal, safety, hardware ack, and ASIC initialization checks after the typed precondition decision as defense in depth."
  - "Preserved controlled-runtime default blocker behavior by making controlled gate builders pass an explicit typed Ready decision."
patterns-established:
  - "Blocked production-mining prerequisite decisions carry both the stable reason and SafetyEffectPlan::fail_closed(reason)."
  - "MiningRuntimeState is the source of truth for API-visible blockedReason when work submission is safe-blocked."
requirements-completed: [SAFE-10, SAFE-11]
generated_by: gsd-execute-plan
lifecycle_mode: yolo
phase_lifecycle_id: 22-2026-07-04T20-10-36
generated_at: 2026-07-04T20:37:04Z
duration: 4min 52s
completed: 2026-07-04
---

# Phase 22 Plan 02: Safety Preconditions And Blocker Reasons Summary

**Typed Ultra 205 production-mining prerequisites with fail-closed blocker reasons carried from safety decisions through mining runtime state into API output.**

## Performance

- **Duration:** 4min 52s
- **Started:** 2026-07-04T20:32:12Z
- **Completed:** 2026-07-04T20:37:04Z
- **Tasks:** 2 completed
- **Files modified:** 9

## Accomplishments

- Added `crates/bitaxe-safety/src/mining_preconditions.rs` with `ProductionMiningPreconditions`, `ProductionMiningPrerequisite`, `BoundedObservationEvidence`, and `ProductionMiningPreconditionDecision`.
- Covered ready, unavailable, stale, unsafe, undocumented, ambiguous, and board-mismatched prerequisite cases with focused safety tests.
- Added `MiningRuntimeState::block_work_submission(reason)` and `maybe_blocked_reason`, then projected that exact reason through `blockedReason` in `crates/bitaxe-api/src/mining.rs`.
- Updated `MiningLoopGate` to evaluate typed production preconditions before BM1366 dispatch while preserving the existing defense-in-depth checks.

## Task Commits

Each task was committed atomically:

1. **Task 1 RED: Add typed production-mining prerequisite tests** - `40f7fa5` (`test`)
2. **Task 1 GREEN: Implement typed production-mining prerequisites** - `4d294fc` (`feat`)
3. **Task 2 RED: Add exact blocker propagation tests** - `006c694` (`test`)
4. **Task 2 GREEN: Propagate exact mining blocker reasons** - `ddf4cf3` (`feat`)

**Plan metadata:** included in the final docs commit for this execution

## Files Created/Modified

- `crates/bitaxe-safety/src/mining_preconditions.rs` - Pure prerequisite contract, bounded evidence validation, stable blocker constants, and unit tests.
- `crates/bitaxe-safety/src/lib.rs` and `crates/bitaxe-safety/BUILD.bazel` - Public module export and Bazel source wiring.
- `crates/bitaxe-stratum/src/v1/state.rs` - Exact blocked-reason storage and clearing when work submission becomes ready.
- `crates/bitaxe-stratum/src/v1/mining_loop.rs` - Typed precondition gate before legacy dispatch checks and exact reason state application.
- `crates/bitaxe-api/src/mining.rs` - API-visible `blockedReason` projection from runtime state.
- `crates/bitaxe-stratum/src/v1/controlled_runtime.rs`, `crates/bitaxe-stratum/src/v1/controlled_runtime/tests.rs`, and `firmware/bitaxe/src/controlled_mining_runtime.rs` - Compile-required explicit typed Ready decisions for controlled runtime gate builders.

## Decisions Made

- Chose a small enum-based prerequisite model instead of a new service layer, preserving functional-core safety decisions.
- Kept `MiningLoopGate::default()` behavior compatible with existing controlled runtime tests by defaulting the typed precondition to `Ready`; the default gate still fails closed on missing power evidence.
- Required controlled runtime helpers to pass `ProductionMiningPreconditionDecision::Ready` explicitly, making the controlled evidence exception visible at each constructor.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] Updated controlled-runtime constructors after `MiningLoopGate` became non-`Copy`**
- **Found during:** Task 2 GREEN
- **Issue:** Adding `ProductionMiningPreconditionDecision` to `MiningLoopGate` made the gate non-`Copy`, and existing controlled runtime code moved the gate twice while building dispatch and nonce-result plans.
- **Fix:** Cloned the gate for the first controlled-runtime plan and added explicit typed Ready decisions to controlled runtime gate builders.
- **Files modified:** `crates/bitaxe-stratum/src/v1/controlled_runtime.rs`, `crates/bitaxe-stratum/src/v1/controlled_runtime/tests.rs`, `firmware/bitaxe/src/controlled_mining_runtime.rs`
- **Verification:** `bazel test //crates/bitaxe-safety:tests //crates/bitaxe-stratum:tests //crates/bitaxe-api:tests`
- **Committed in:** `ddf4cf3`

**Total deviations:** 1 auto-fixed (1 blocking)
**Impact on plan:** The fix was necessary to keep existing controlled-runtime behavior compiling without adding bypass flags or broad architecture changes.

## Issues Encountered

- The first Task 2 GREEN test run failed because `MiningLoopGate` was no longer `Copy`. The controlled runtime now clones the gate for the first guarded plan.
- No authentication gates occurred.

## Known Stubs

None.

## Threat Flags

None - the changed trust boundaries are covered by T-22-04 through T-22-07, and no new network endpoint, auth path, file access pattern, or schema boundary was added.

## Verification

- `bazel test //crates/bitaxe-safety:tests`
- `bazel test //crates/bitaxe-safety:tests //crates/bitaxe-stratum:tests //crates/bitaxe-api:tests`
- `rg "maybe_blocked_reason|block_work_submission|clear_blocked_reason" crates/bitaxe-stratum/src/v1/state.rs`
- `rg "ProductionMiningPreconditionDecision|block_work_submission\\(reason\\)" crates/bitaxe-stratum/src/v1/mining_loop.rs`
- `rg "maybe_blocked_reason|unwrap_or\\(\"\"\\)" crates/bitaxe-api/src/mining.rs`
- `rg "force|unsafe_override|allow_unsafe|BYPASS|OVERRIDE" crates/bitaxe-safety/src/mining_preconditions.rs crates/bitaxe-stratum/src/v1/mining_loop.rs crates/bitaxe-stratum/src/v1/state.rs crates/bitaxe-api/src/mining.rs` returned no matches
- `just parity` passed with `validation_errors: none`
- `just verify-reference` passed with `reference clean: c1915b0a63bfabebdb95a515cedfee05146c1d50`
- `git diff --check`

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

Plan 22-03 can document the prerequisite contract and checklist implications with SAFE-10 and SAFE-11 backed by pure unit tests plus runtime/API blocker propagation. Later production-mining phases still need detector-gated hardware evidence before promoting live mining behavior or active voltage/fan/thermal parity.

## Self-Check: PASSED

- Found created files: `crates/bitaxe-safety/src/mining_preconditions.rs` and `22-02-SUMMARY.md`.
- Found task commits: `40f7fa5`, `4d294fc`, `006c694`, and `ddf4cf3`.

*Phase: 22-claim-ladder-and-safety-preconditions*
*Completed: 2026-07-04*
