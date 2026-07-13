---
phase: 06-safety-controllers-and-self-test
plan: "03"
subsystem: safety
tags: [rust, power, voltage, ina260, ds4432u, fail-closed]

requires:
  - phase: 06-02
    provides: Public `bitaxe_safety::power` module boundary
provides:
  - Typed INA260 power observation classification
  - `PowerEvidenceToken` for downstream ASIC and mining preflight gates
  - Observe-only DS4432U voltage effect planning with hardware-evidence gates
  - Provenance-rich power telemetry and voltage effect fixtures
affects: [phase-06, asic-init, mining-gate, api-telemetry, firmware-safety-adapter, parity]

tech-stack:
  added: []
  patterns: [typed safety observations, observe-only hardware effect plans, fixture provenance]

key-files:
  created:
    - crates/bitaxe-safety/fixtures/safety/power-telemetry-cases.json
    - crates/bitaxe-safety/fixtures/safety/voltage-effect-cases.json
  modified:
    - crates/bitaxe-safety/src/power.rs
    - crates/bitaxe-safety/BUILD.bazel

key-decisions:
  - "Treat stale, missing, failed, non-finite, unsafe-voltage, and over-power INA260 readings as fail-closed before firmware effects."
  - "Keep DS4432U voltage writes as pure plans; actual writes require hardware-smoke or hardware-regression evidence plus an armed mode."
  - "Expose `PowerEvidenceToken` only from fresh safe observations so downstream gates cannot use stale or faulted readings."

patterns-established:
  - "Safety observations produce typed evidence tokens only on fresh safe data."
  - "Hardware actuation planning separates voltage plans from safety effect plans."

requirements-completed: [SAFE-01, SAFE-04, SAFE-07, SAFE-08]
generated_by: gsd-execute-plan
lifecycle_mode: yolo
phase_lifecycle_id: 6-2026-06-28T02-37-37
generated_at: 2026-06-28T04:24:26Z

duration: 8 min
completed: 2026-06-28
---

# Phase 06 Plan 03: Power Safety Decisions Summary

**INA260 power observation classification and DS4432U observe-only voltage effect planning**

## Performance

- **Duration:** 8 min
- **Started:** 2026-06-28T04:16:00Z
- **Completed:** 2026-06-28T04:24:26Z
- **Tasks:** 2
- **Files modified:** 4

## Accomplishments

- Implemented typed INA260 observations with fresh, stale, fault, and unavailable states.
- Added fail-closed `PowerSafetyDecision` behavior and `PowerEvidenceToken` production for fresh safe observations.
- Added DS4432U voltage planning that suppresses writes unless valid voltage, board capabilities, fresh power, hardware evidence, and armed mode all agree.
- Added provenance fixtures for power telemetry and voltage effect cases.

## Task Commits

1. **Task 1: Classify INA260 telemetry and unsafe power observations** - `2b1e72f` (feat, combined)
2. **Task 2: Plan DS4432U voltage effects with observe-only defaults** - `2b1e72f` (feat, combined)

## Files Created/Modified

- `crates/bitaxe-safety/src/power.rs` - Power telemetry classification, evidence token, and voltage effect planning.
- `crates/bitaxe-safety/fixtures/safety/power-telemetry-cases.json` - Power telemetry provenance and fault cases.
- `crates/bitaxe-safety/fixtures/safety/voltage-effect-cases.json` - Voltage effect provenance and hardware-evidence cases.
- `crates/bitaxe-safety/BUILD.bazel` - Adds `serde_json` test dependency for fixture parsing tests.

## Decisions Made

- Used explicit reason strings for every fail-closed power path so API, firmware, and parity docs can reference stable behavior.
- Modeled observe-only voltage planning as `NoWrite` with fail-closed safety effects instead of silently accepting implementation evidence.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] Added Bazel test dependency for fixture parsing**
- **Found during:** Task 1 (Classify INA260 telemetry)
- **Issue:** Cargo could use the crate dev-dependency `serde_json`, but Bazel `rust_test` did not link `@crates//:serde_json`.
- **Fix:** Added `serde_json` to `crates/bitaxe-safety/BUILD.bazel` test deps.
- **Files modified:** `crates/bitaxe-safety/BUILD.bazel`
- **Verification:** `bazel test //crates/bitaxe-safety:tests --test_filter='safety_power|voltage_effect'`
- **Committed in:** `2b1e72f`

**2. [Rule 3 - Blocking] Combined coupled task commits**
- **Found during:** Task commit boundary
- **Issue:** Both tasks change the same `power.rs` module and the repo requires the full Rust gate before commit; splitting after implementation would create unnecessary partial states.
- **Fix:** Committed both task outcomes together after scoped checks and the full Rust gate passed.
- **Files modified:** `crates/bitaxe-safety/src/power.rs`, safety fixtures, `crates/bitaxe-safety/BUILD.bazel`
- **Verification:** Scoped power/voltage tests plus full Rust gate.
- **Committed in:** `2b1e72f`

**Total deviations:** 2 auto-fixed blocking issues
**Impact on plan:** Behavior and verification stayed aligned with the plan; commit granularity was coarser than the ideal task boundary.

## Issues Encountered

Bazel initially failed to build the fixture parsing tests because `serde_json` was not listed on the `rust_test` target. The dependency was added and the scoped Bazel test passed.

## Verification

- `cargo test -p bitaxe-safety --all-features safety_power`
- `cargo test -p bitaxe-safety --all-features voltage_effect`
- `bazel test //crates/bitaxe-safety:tests --test_filter='safety_power|voltage_effect'`
- `cargo fmt --all`
- `cargo clippy --all-targets --all-features -- -D warnings`
- `cargo build --all-targets --all-features`
- `cargo test --all-features`

## Known Stubs

This plan produces pure effect plans only. Firmware I2C writes, DS4432U transfer math, and hardware evidence capture remain later hardware-gated work.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

Power evidence can now feed Plan 06-06 ASIC and mining gates. Firmware adapters in Plan 06-08 can publish unavailable/observe-only power telemetry without enabling hardware writes.

## Self-Check: PASSED

- Confirmed scoped power and voltage tests passed under Cargo and Bazel.
- Confirmed task commit `2b1e72f` exists in git history.

---
*Phase: 06-safety-controllers-and-self-test*
*Completed: 2026-06-28*
