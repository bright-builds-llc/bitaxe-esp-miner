---
phase: 32-shared-i2c-and-read-only-sensor-acquisition
plan: "03"
subsystem: firmware-telemetry
tags: [rust, esp-idf, i2c, sensors, provenance, telemetry]
requires:
  - phase: 32-01
    provides: Pure typed sensor decoders and failure-isolated observation reducer
  - phase: 32-02
    provides: Bounded shared I2C owner and closed read-only sensor capabilities
provides:
  - One deadline-based 500 ms producer owning the post-display I2C0 bus
  - Complete snapshot publication after independent power, temperature, and tachometer attempts
  - Clone-only API consumer regression coverage with exact stamp preservation
  - Software build/package evidence and an explicit hardware-pending non-claim
affects: [phase-34-operator-snapshot, phase-35-hardware-evidence]
tech-stack:
  added: []
  patterns: [single producer ownership, deadline skip without catch-up, complete snapshot replacement]
key-files:
  created:
    - firmware/bitaxe/src/operator_sensor_runtime.rs
    - docs/evidence/phase-32/README.md
  modified:
    - firmware/bitaxe/src/main.rs
    - firmware/bitaxe/src/safety_adapter.rs
    - crates/bitaxe-api/src/observation.rs
    - tools/parity/src/phase32_source_guard.rs
key-decisions:
  - "The normal producer attempts power, temperature, and tachometer once per sweep before replacing one complete observation snapshot."
  - "Missed 500 ms deadlines skip to the next future slot, preventing retry loops and catch-up bursts."
  - "Phase 32 admits software evidence only; hardware remains pending until a wrapper records the complete private session trace and separate sanitized summary."
requirements-completed: [OBS-02, OBS-03, OBS-04, OBS-05]
generated_by: gsd-execute-plan
lifecycle_mode: yolo
phase_lifecycle_id: 32-2026-07-13T23-12-34
generated_at: 2026-07-14T00:40:00Z
duration: 15 min
completed: 2026-07-13
---

# Phase 32 Plan 03: Sole Sensor Producer and Software Evidence Summary

**One post-display I2C0 owner now drives a failure-isolated 500 ms read-only producer whose complete stamped snapshots remain unchanged across API reads.**

## Performance

- **Duration:** 15 min
- **Started:** 2026-07-14T00:25:00Z
- **Completed:** 2026-07-14T00:40:00Z
- **Tasks:** 3
- **Files modified:** 7

## Accomplishments

- Moved the normal-path shared bus into one named producer after best-effort startup display rendering, including producer start after display failure.
- Attempted the atomic INA260 triple, external temperature, and tachometer independently on each deadline-based sweep, then published one complete snapshot outside all I2C calls.
- Projected the one stamped power observation into watts, voltage, and current while keeping temperature, tachometer, and unavailable VR temperature independent and explicit.
- Added source and API regressions proving sole ownership, no active-effect reachability, source-failure isolation, and repeated clone-only reads with byte-identical provenance.
- Passed the ordered Rust gate, production ESP firmware build, package construction, and reference-clean check without hardware access or parity promotion.

## Task Commits

The three tightly coupled runtime/evidence tasks were committed together after the complete gate passed:

1. **Task 1: Start one deadline-based producer and publish complete snapshots** - `8f84c5b` (feat)
2. **Task 2: Prove API consumers remain clone-only and unaffected by sensor failures** - `8f84c5b` (test)
3. **Task 3: Build, package, and record hardware verification pending** - `8f84c5b` (docs)

**Plan metadata:** This commit

## Files Created/Modified

- `firmware/bitaxe/src/operator_sensor_runtime.rs` - Sole normal producer, 500 ms deadline schedule, independent acquisitions, and complete projection.
- `firmware/bitaxe/src/main.rs` - Explicit display outcome followed by ownership transfer into the producer.
- `firmware/bitaxe/src/safety_adapter.rs` - Closed facade functions for normal power, temperature, and tachometer reads.
- `crates/bitaxe-api/src/observation.rs` - Repeated-read and failed-source projection regression.
- `tools/parity/src/phase32_source_guard.rs` - Runtime ownership, ordering, no-actuation, and clone-only consumer guards.
- `firmware/bitaxe/BUILD.bazel` - Hermetic source inputs for Phase 32 guards.
- `docs/evidence/phase-32/README.md` - Software evidence inventory, exact exclusions, and hardware-pending blocker.

## Decisions Made

- The producer publishes even after a failed sweep reduction, preserving the last known complete state while logging sequence overflow as a redaction-safe category.
- Each acquisition runs once per sweep with no blocking retry; overdue execution skips missed slots to preserve bounded load.
- Hardware proof is not attempted through an incomplete wrapper. Phase 35 remains the only promotion owner.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] Used the repository's actual evidence root while preserving the planned artifact path**

- **Found during:** Task 3 evidence-path inspection
- **Issue:** The plan required reading `docs/evidence/README.md`, but neither that file nor the `docs/evidence` directory existed; established evidence contracts live under `docs/parity/evidence`.
- **Fix:** Read the current Phase 23 and Phase 29 evidence contracts, fully inspected the flash/parity tools, and created the exact planned `docs/evidence/phase-32/README.md` software-only record.
- **Files modified:** `docs/evidence/phase-32/README.md`
- **Verification:** The documentation records the actual wrapper gap, exact exclusions, no hardware access, and Phase 35-only promotion.
- **Committed in:** `8f84c5b`

***

**Total deviations:** 1 auto-fixed (1 blocking)
**Impact on plan:** The missing index was handled conservatively without widening evidence scope, changing checklist status, or invoking hardware.

## Issues Encountered

- Existing firmware-only dead-code warnings remain in the ESP cross-build, while the required host Clippy gate passes with `-D warnings`; Phase 32 introduced no new host lint warning.

## User Setup Required

None - no external service or hardware configuration was used.

## Next Phase Readiness

- Phase 32 software implementation is complete and ready for phase verification and lifecycle review.
- Phase 33 can rely on one functioning read-only producer and clone-only request boundary.
- Physical display/sensor observations remain explicitly pending until a later compliant detector-gated wrapper exists; Phase 32 makes no hardware or parity-verification claim.

## Self-Check: PASSED

- Commit `8f84c5b` and all created/modified files exist.
- Focused Cargo and Bazel Phase 32 guards passed.
- `cargo fmt --all`, Clippy with warnings denied, all-target build, and all-feature tests passed in order.
- `just build`, `just package`, and `just verify-reference` passed.
- No detector, flash, monitor, device API, credentials, reset, hardware, checklist-promotion, reference, or archived-lineage command/change occurred.
