---
phase: 32-shared-i2c-and-read-only-sensor-acquisition
plan: "01"
subsystem: safety-core
tags: [rust, i2c, ina260, emc2101, observations]
requires:
  - phase: 31-01
    provides: Producer-owned observation truth and provenance contract
provides:
  - Pure INA260 and EMC2101 decoding with checked sensor semantics
  - Failure-isolated producer sweep reduction into stamped observations
  - Producer-time stale transitions with exact last-good preservation
affects: [32-02-firmware-readers, 32-03-sensor-producer, phase-34-operator-snapshot]
tech-stack:
  added: []
  patterns: [pure acquisition reducer, source-local observation sequence, atomic power sample]
key-files:
  created:
    - crates/bitaxe-safety/src/sensor_acquisition.rs
  modified:
    - crates/bitaxe-safety/src/lib.rs
    - crates/bitaxe-safety/src/power.rs
key-decisions:
  - "One validated INA260 triple advances one shared power observation sequence; a partial or invalid attempt advances nothing."
  - "Temperature and tachometer retain separate outcomes, sequences, and last-good stamps."
  - "Only producer-supplied monotonic time may transition fresh observations to stale."
requirements-completed: [OBS-03, OBS-04, OBS-05]
generated_by: gsd-execute-plan
lifecycle_mode: yolo
phase_lifecycle_id: 32-2026-07-13T23-12-34
generated_at: 2026-07-14T00:01:53Z
duration: 17 min
completed: 2026-07-13
---

# Phase 32 Plan 01: Pure Sensor Acquisition Core Summary

**Checked INA260 and EMC2101 decoders now feed one failure-isolated, producer-stamped observation reducer without ESP-IDF or active-effect dependencies.**

## Performance

- **Duration:** 17 min
- **Started:** 2026-07-13T23:45:16Z
- **Completed:** 2026-07-14T00:01:53Z
- **Tasks:** 2
- **Files modified:** 3

## Accomplishments

- Added exact signed-current, voltage, power, signed external-temperature, sensor-fault, tachometer-zero, no-spin, and overflow-safe decoding.
- Added one pure producer reducer that advances only successful source-local sequences and preserves exact last-good provenance on read or validation failure.
- Preserved atomic INA260 publication while allowing EMC2101 temperature and tachometer facts to succeed or fail independently.
- Added producer-owned staleness transitions with no internal clock or consumer-side metadata mutation.

## Task Commits

The two tightly coupled pure-core tasks were committed together:

1. **Task 1: Define typed sensor decoders and read-only acquisition results** - `310b838` (feat)
2. **Task 2: Reduce one failure-isolated producer sweep into stamped observations** - `310b838` (feat)

**Plan metadata:** This commit

## Files Created/Modified

- `crates/bitaxe-safety/src/sensor_acquisition.rs` - Pure decoders, typed acquisition outcomes, reducer, stale transition, and regression tests.
- `crates/bitaxe-safety/src/power.rs` - Prior-aware complete INA260 recording that preserves last-good truth on failed attempts.
- `crates/bitaxe-safety/src/lib.rs` - Exposes the pure sensor acquisition module.

## Decisions Made

- INA260 success is admitted only after all three register values are decoded and validated as one sample.
- A failed read and a failed validation both retain the prior stamped sample and prior sequence while changing only the observation state.
- EMC2101 temperature and tachometer observations never borrow each other's success, sequence, or acquisition time.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] Consolidated the two coupled core tasks after the delegated executor stalled**

- **Found during:** Plan execution startup
- **Issue:** The delegated executor produced no artifacts and left the focused Cargo run waiting on the build lock.
- **Fix:** Interrupted the stalled executor, completed both pure-core tasks inline, and committed their inseparable reducer/API change as one atomic unit.
- **Files modified:** `crates/bitaxe-safety/src/lib.rs`, `crates/bitaxe-safety/src/power.rs`, `crates/bitaxe-safety/src/sensor_acquisition.rs`
- **Verification:** Focused safety and API observation tests passed, followed by the complete mandatory Rust gate.
- **Committed in:** `310b838`

***

**Total deviations:** 1 auto-fixed (1 blocking)
**Impact on plan:** Task commit granularity changed, but scope, requirements, behavior, and verification remained exactly within Plan 32-01.

## Issues Encountered

- The first focused test exposed two fixture-only issues: an integer cast needed explicit grouping and the tachometer no-spin sentinel is represented by `u16::MAX`. Both were corrected before the full verification gate.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

- Plan 32-02 can bind these pure outcomes to one bounded read-only firmware bus owner.
- No hardware, credentials, fan/voltage/reset effects, ASIC/mining paths, OTA, direct UART/pins, or archived Phase 28.1.1 work were invoked.

## Self-Check: PASSED

- Created files exist and commit `310b838` is present in history.
- `cargo test -p bitaxe-safety sensor_acquisition` and `cargo test -p bitaxe-api observation` passed.
- The mandatory `cargo fmt`, Clippy, all-target build, and all-feature test gate passed in order.
- `git diff --check` passed and only the intended safety-core files entered the task commit.
