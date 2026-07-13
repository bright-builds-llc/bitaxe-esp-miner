---
phase: 31-operator-claim-and-telemetry-contract
plan: "01"
subsystem: safety
tags: [rust, observation, telemetry, power, thermal]
requires:
  - phase: 30-live-share-outcome-and-verified-promotion
    provides: Closed exact-claim and fail-closed admission precedent
provides:
  - Mutually exclusive fresh, stale, unavailable, and fault observation truth
  - Producer-owned boot session, sequence, and monotonic acquisition stamps
  - Power and independent temperature/tachometer safety adapters
affects: [31-02-api-consumer-boundary, 32-read-only-i2c-producer, 34-operator-snapshot]
tech-stack:
  added: []
  patterns: [state-carrying observation enum, producer-owned stamps, compatibility projection outside truth]
key-files:
  created:
    - crates/bitaxe-safety/src/observation.rs
  modified:
    - crates/bitaxe-safety/src/lib.rs
    - crates/bitaxe-safety/BUILD.bazel
    - crates/bitaxe-safety/src/power.rs
    - crates/bitaxe-safety/src/thermal.rs
    - crates/bitaxe-safety/src/mining_preconditions.rs
key-decisions:
  - "Power current, voltage, and wattage share one stamped acquisition because INA260 supplies them atomically."
  - "Temperature and tachometer truth remain independent so one producer failure cannot erase the other fact."
  - "Legacy numeric fallbacks are accessor projections and never data stored inside Observation<T>."
patterns-established:
  - "Observation truth: state variants own only the data legal for that state."
  - "Producer provenance: only validated success advances a boot-scoped source sequence."
requirements-completed: [OBS-01]
generated_by: gsd-execute-plan
lifecycle_mode: yolo
phase_lifecycle_id: 31-2026-07-13T19-47-51
generated_at: 2026-07-13T20:45:59Z
duration: 12 min
completed: 2026-07-13
---

# Phase 31 Plan 01: Observation Truth Core Summary

**Typed four-state observation truth with producer-owned provenance now drives fail-closed power and independent temperature/tachometer safety decisions.**

## Performance

- **Duration:** 12 min
- **Started:** 2026-07-13T20:33:06Z
- **Completed:** 2026-07-13T20:45:59Z
- **Tasks:** 2
- **Files modified:** 6

## Accomplishments

- Added a generic observation contract that makes contradictory state/value combinations unconstructible and scopes sequence ordering to one boot session.
- Migrated power safety to one validated stamped INA260 reading while keeping compatibility numerics outside observation truth.
- Split temperature and tachometer truth so their freshness and failures are independent while existing safety decisions remain fail closed.

## Task Commits

Each task was committed atomically:

1. **Task 1: Add the typed observation and producer-transition core** — `d5c6155` (feat)
2. **Task 2: Migrate power and thermal decisions to the authoritative truth core** — `decc841` (feat)

**Plan metadata:** this commit

## Files Created/Modified

- `crates/bitaxe-safety/src/observation.rs` — Four-state truth, typed reasons, stamped samples, transitions, and ordering tests.
- `crates/bitaxe-safety/src/power.rs` — Stamped aggregate power truth, compatibility accessors, and fail-closed regressions.
- `crates/bitaxe-safety/src/thermal.rs` — Independent temperature/tachometer truth and safety regressions.
- `crates/bitaxe-safety/src/mining_preconditions.rs` — Updated thermal fixture construction for the optional-field naming contract.
- `crates/bitaxe-safety/src/lib.rs` — Public observation module export.
- `crates/bitaxe-safety/BUILD.bazel` — Observation module included in Bazel sources.

## Decisions Made

- Kept INA260 current, bus voltage, and power in one sample because one producer acquisition owns those values together.
- Modeled temperature and tachometer as separate observations; missing tachometer data does not invalidate a fresh temperature.
- Kept legacy numeric zero as a wrapper projection only. It cannot create or authenticate a fresh observation.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] Resolved stale fixture paths in the task read-first list**

- **Found during:** Task 2
- **Issue:** `fixtures/power-observations.json` and `fixtures/thermal-observations.json` do not exist.
- **Fix:** Read and preserved the canonical existing `fixtures/safety/power-telemetry-cases.json`, `thermal-fault-cases.json`, and `fan-pid-cases.json` contracts.
- **Files modified:** None for this deviation.
- **Verification:** Full safety Cargo and Bazel suites passed with the canonical fixtures.
- **Committed in:** `decc841`

**2. [Rule 3 - Blocking] Updated dependent mining-precondition test construction**

- **Found during:** Task 2
- **Issue:** Optional thermal fields adopted the required `maybe_` naming, so dependent safety tests needed matching fixture field names.
- **Fix:** Updated only `ThermalReading` fixture construction in `mining_preconditions.rs`.
- **Files modified:** `crates/bitaxe-safety/src/mining_preconditions.rs`
- **Verification:** All mining-precondition and complete safety tests passed.
- **Committed in:** `decc841`

**Total deviations:** 2 auto-fixed (2 blocking). **Impact:** Both changes were narrow prerequisites for executing the planned migration; no firmware, hardware, reference, or evidence scope was added.

## Issues Encountered

- The first full Clippy gate reported `double_must_use` and copy-vs-clone lints. The code was simplified and the complete mandated gate was restarted and passed.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

- Ready for Plan 31-02 to project stored observation truth through API and firmware consumer boundaries.
- Firmware callers still use the old field-shaped adapter contract and are intentionally migrated by Plan 31-02 before its canonical firmware build gate.
- No hardware or network evidence was used or claimed.

## Self-Check: PASSED

- Created file exists: `crates/bitaxe-safety/src/observation.rs`.
- Task commits exist: `d5c6155`, `decc841`.
- Focused Cargo/Bazel verification and the full Rust commit gates passed.

***

*Phase: 31-operator-claim-and-telemetry-contract*
*Completed: 2026-07-13*
