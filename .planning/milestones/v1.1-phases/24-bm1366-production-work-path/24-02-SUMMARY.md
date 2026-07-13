---
phase: 24-bm1366-production-work-path
plan: 02
subsystem: stratum
tags: [rust, stratum-v1, bm1366, production-work, redaction, bazel]
requires:
  - phase: 24-bm1366-production-work-path
    provides: BM1366 production-only payload and command primitives from Plan 24-01
provides:
  - Session-generation registry for pool-derived BM1366 production work
  - Clean-jobs and reconnect invalidation across queued, active, and valid-job state
  - Redaction-safe formatting for production work records, dispatches, target context, and registry state
affects: [phase-24, phase-25, bitaxe-stratum, firmware-production-dispatch, parity-evidence]
tech-stack:
  added: []
  patterns:
    - functional-core active work registry
    - typed pool-session generation invalidation
    - redaction-safe Debug implementations for raw-bearing Stratum production work
key-files:
  created:
    - crates/bitaxe-stratum/src/v1/production_work.rs
  modified:
    - crates/bitaxe-stratum/src/v1.rs
    - crates/bitaxe-stratum/BUILD.bazel
key-decisions:
  - "Production BM1366 work is bound to PoolSessionGeneration before dispatch."
  - "Clean-jobs, reconnect, authorization reset, and session replacement all advance generation and clear queued, active, and valid-job state."
  - "Raw-bearing registry surfaces use custom Debug output with category labels instead of derived raw field rendering."
patterns-established:
  - "ProductionWorkRegistry reuses BoundedWorkQueue and Bm1366ValidJobIds instead of duplicating queue or valid-job logic."
  - "ProductionDispatch preserves original MiningWork context for later correlation while rendering only redacted category labels."
requirements-completed: [ASIC-10]
generated_by: gsd-execute-plan
lifecycle_mode: yolo
phase_lifecycle_id: 24-2026-07-05T00-27-27
generated_at: 2026-07-05T00:57:28Z
duration: 3min 22s
completed: 2026-07-05
---

# Phase 24 Plan 02: Session-Generation Production Work Registry Summary

**Pool-derived BM1366 work registry with generation-aware invalidation and redaction-safe active dispatch records**

## Performance

- **Duration:** 3min 22s
- **Started:** 2026-07-05T00:54:06Z
- **Completed:** 2026-07-05T00:57:28Z
- **Tasks:** 3
- **Files modified:** 3

## Accomplishments

- Added `PoolSessionGeneration`, `ProductionWorkRegistry`, `ProductionWorkRecord`, `ProductionDispatch`, and `ProductionTargetContext` in `crates/bitaxe-stratum`.
- Bound queued and dispatched `MiningWork` to the active pool generation, original job/extranonce/time/difficulty context, and BM1366 `ProductionWorkPayload`.
- Proved clean-jobs and reconnect invalidation clear queued work, active work, and `Bm1366ValidJobIds` before stale work can be returned.
- Exported the production work registry through the Stratum v1 module root and Bazel crate sources.

## Task Commits

Each task was committed atomically:

1. **Task 24-02-01 RED: Add failing production work registry tests** - `a4068c8` (test)
2. **Task 24-02-01 GREEN: Implement production work registry** - `bfbd75d` (feat)
3. **Task 24-02-02: Prove clean-jobs and reconnect invalidation** - `34cd1a4` (test)
4. **Task 24-02-03: Export production registry through module roots** - `9b94356` (feat)

## Files Created/Modified

- `crates/bitaxe-stratum/src/v1/production_work.rs` - Production active-work registry, session generation, invalidation methods, redacted Debug output, and unit tests.
- `crates/bitaxe-stratum/src/v1.rs` - Public `production_work` module export.
- `crates/bitaxe-stratum/BUILD.bazel` - Bazel source registration for the new registry module.

## Decisions Made

- Kept production active-work ownership in `bitaxe-stratum`, with BM1366 payload construction delegated to `bitaxe-asic::bm1366::production::ProductionWorkPayload`.
- Stored `stratum_job_id`, `extranonce2`, `ntime`, compact nbits, pool difficulty, original `MiningWork`, dispatch state, and result-seen state on active records for later Phase 24/25 correlation.
- Implemented explicit invalidation entrypoints for clean-jobs, reconnect, authorization reset, and session replacement even though all currently share the same generation-advance-and-clear behavior.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] Registered the module during RED test setup**
- **Found during:** Task 24-02-01 (Add session-generation production work registry)
- **Issue:** The new inline tests would not compile or run under Bazel unless `production_work.rs` was present in `BUILD.bazel` and reachable from `v1.rs`.
- **Fix:** Added the Bazel source entry and a temporary `#[cfg(test)] pub mod production_work;` export in the RED commit, then promoted it to a normal public export in Task 24-02-03.
- **Files modified:** `crates/bitaxe-stratum/src/v1.rs`, `crates/bitaxe-stratum/BUILD.bazel`
- **Verification:** `bazel test //crates/bitaxe-stratum:tests` failed in RED on missing production registry types, then passed after implementation.
- **Committed in:** `a4068c8`, finalized by `9b94356`

**2. [Rule 2 - Missing Critical] Redacted registry-level Debug output**
- **Found during:** Task 24-02-02 (Prove clean-jobs and reconnect invalidation)
- **Issue:** `ProductionWorkRegistry` itself owns queued and active `MiningWork`; deriving `Debug` would expose raw job and extranonce context even though record and dispatch Debug output was redacted.
- **Fix:** Replaced derived `Debug` with a custom formatter that renders only generation and redacted category labels, plus a unit test proving queued and active context is omitted.
- **Files modified:** `crates/bitaxe-stratum/src/v1/production_work.rs`
- **Verification:** `bazel test //crates/bitaxe-stratum:tests`
- **Committed in:** `34cd1a4`

**Total deviations:** 2 auto-fixed (1 blocking, 1 missing critical)
**Impact on plan:** Both changes were required to make the planned tests executable and to preserve the plan's redaction invariants. No Phase 25 share-response behavior was added.

## Issues Encountered

- RED verification failed as intended on unresolved `ProductionWorkRegistry` and `PoolSessionGeneration` types before implementation.
- Task 24-02-02's additional tests passed immediately after being added because Task 24-02-01 had already implemented the invalidation behavior; no extra behavior code was needed beyond the registry Debug redaction hardening.
- No stubs were found in created or modified files.
- No unplanned threat flags were introduced beyond the plan's production work registry trust boundary.

## Verification

- `bazel test //crates/bitaxe-stratum:tests`
- `rg "pub struct PoolSessionGeneration|pub struct ProductionWorkRegistry|pub struct ProductionDispatch" crates/bitaxe-stratum/src/v1/production_work.rs`
- `rg "invalidate_for_clean_jobs|invalidate_for_reconnect|invalidate_for_authorization_reset|invalidate_for_session_replacement" crates/bitaxe-stratum/src/v1/production_work.rs`
- `rg "ProductionTargetContext|compact_nbits|maybe_pool_difficulty" crates/bitaxe-stratum/src/v1/production_work.rs`
- `rg "production_work_record_debug_redacts_raw_context|production_dispatch_debug_redacts_raw_context|no_debug_for_raw_production_records" crates/bitaxe-stratum/src/v1/production_work.rs`
- `rg "production_work_clean_jobs_invalidates_queued_active_and_valid_jobs|production_work_reconnect_advances_generation_and_clears_work|production_work_records_pool_context" crates/bitaxe-stratum/src/v1/production_work.rs`
- `rg "assert_eq!\(registry\.generation\(\)\.raw\(\), 1\)|assert_eq!\(registry\.generation\(\)\.raw\(\), 2\)" crates/bitaxe-stratum/src/v1/production_work.rs`
- `rg "pub mod production_work;" crates/bitaxe-stratum/src/v1.rs`
- `rg "\"src/v1/production_work.rs\"" crates/bitaxe-stratum/BUILD.bazel`
- Stub scan: no matches for TODO/FIXME/placeholder/empty-value patterns in `production_work.rs`.
- Threat-surface scan: no network, credential, raw-frame, device URL, or share-payload patterns in `production_work.rs`.

## Auth Gates

None.

## Known Stubs

None.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

Plan 24-03 can consume `ProductionWorkRegistry` to correlate parsed BM1366 nonce/results against current-generation active work and prepare redaction-safe submit intent data. Phase 25 accepted/rejected pool response classification remains explicitly out of scope.

## Self-Check: PASSED

Confirmed the summary and modified source files exist, and task commits `a4068c8`, `bfbd75d`, `34cd1a4`, and `9b94356` are present in git history.

*Phase: 24-bm1366-production-work-path*
*Completed: 2026-07-05*
