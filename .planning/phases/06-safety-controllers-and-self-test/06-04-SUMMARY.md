---
phase: 06-safety-controllers-and-self-test
plan: "04"
subsystem: safety
tags: [rust, thermal, fan, pid, fault, fail-closed]

requires:
  - phase: 06-02
    provides: Public `bitaxe_safety::thermal` and `bitaxe_safety::fault` module boundaries
provides:
  - Typed thermal observation parsing for sentinel, invalid, unavailable, and fresh-safe states
  - `ThermalEvidenceToken` for downstream ASIC and mining preflight gates
  - Pure fan mode and PID duty decisions with fixture-backed constants
  - Overheat, fan, power, thermal, and ASIC fault decisions with visible fail-closed effects
affects: [phase-06, asic-init, mining-gate, api-telemetry, firmware-safety-adapter, parity]

tech-stack:
  added: []
  patterns: [typed safety observations, pure safety decisions, fixture provenance]

key-files:
  created:
    - crates/bitaxe-safety/fixtures/safety/fan-pid-cases.json
    - crates/bitaxe-safety/fixtures/safety/thermal-fault-cases.json
    - crates/bitaxe-safety/fixtures/safety/overheat-state-cases.json
  modified:
    - crates/bitaxe-safety/src/thermal.rs
    - crates/bitaxe-safety/src/fault.rs

key-decisions:
  - "Classify thermal sentinel, non-finite, and implausible values before PID or fan decisions can run."
  - "Expose `ThermalEvidenceToken` only for fresh safe thermal observations plus non-missing safety evidence."
  - "Model overheat restart as `RestartCandidate`, not mining-ready; later gates must prove ASIC, power, thermal, safety, and hardware evidence."

patterns-established:
  - "Fan, overheat, and fault decisions remain pure data until firmware adapters consume explicit effect plans."
  - "Sustained fault paths publish visible status and block work submission with stable reason strings."

requirements-completed: [SAFE-02, SAFE-03, SAFE-04, SAFE-07, SAFE-08]
generated_by: gsd-execute-plan
lifecycle_mode: yolo
phase_lifecycle_id: 6-2026-06-28T02-37-37
generated_at: 2026-06-28T04:30:39Z

duration: 18 min
completed: 2026-06-28
---

# Phase 06 Plan 04: Thermal And Fault Decisions Summary

**Pure thermal, fan, PID, overheat, and fault policy for Ultra 205 safety control**

## Performance

- **Duration:** 18 min
- **Started:** 2026-06-28T04:12:00Z
- **Completed:** 2026-06-28T04:30:39Z
- **Tasks:** 2
- **Files modified:** 5

## Accomplishments

- Implemented typed thermal readings, observations, fault/unavailable classification, and thermal evidence tokens.
- Added pure fan decisions for overheat, startup, paused/no-pool, manual, and auto PID modes.
- Added overheat safe-stop, cooling, and restart-candidate states with fail-closed effect plans.
- Added sustained fault classification for fan zero RPM, fan set failure, thermal sensor, power, and ASIC faults.
- Added provenance fixtures for fan/PID, thermal fault, and overheat/fault cases.

## Task Commits

1. **Task 1: Implement thermal observation, PID, and fan duty decisions** - `cb61c26` (feat, combined)
2. **Task 2: Implement overheat and sustained fault state decisions** - `cb61c26` (feat, combined)

## Files Created/Modified

- `crates/bitaxe-safety/src/thermal.rs` - Thermal observation parsing, PID/fan decisions, thermal evidence, and overheat state decisions.
- `crates/bitaxe-safety/src/fault.rs` - Safety fault classification and fail-closed fault plans.
- `crates/bitaxe-safety/fixtures/safety/fan-pid-cases.json` - Fan/PID provenance and duty cases.
- `crates/bitaxe-safety/fixtures/safety/thermal-fault-cases.json` - Thermal sentinel and invalid reading cases.
- `crates/bitaxe-safety/fixtures/safety/overheat-state-cases.json` - Overheat, restart, and sustained fault cases.

## Decisions Made

- Kept fan and thermal controls as pure decisions; no firmware PWM, sensor, ASIC, voltage, or mining effect is executed here.
- Treated restart after overheat as a blocked candidate state so later preflight gates must prove safety before mining resumes.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] Combined coupled task commits**
- **Found during:** Task commit boundary
- **Issue:** Both tasks changed the same thermal/fault module set and the repo requires the full Rust gate before commit.
- **Fix:** Committed both task outcomes together after scoped Cargo/Bazel checks and the full Rust gate passed.
- **Files modified:** `crates/bitaxe-safety/src/thermal.rs`, `crates/bitaxe-safety/src/fault.rs`, safety fixtures.
- **Verification:** Scoped thermal/fault tests plus full Rust gate.
- **Committed in:** `cb61c26`

**Total deviations:** 1 auto-fixed blocking issue
**Impact on plan:** Behavior and verification stayed aligned with the plan; commit granularity was coarser than the ideal task boundary.

## Issues Encountered

No implementation blockers. A stale carryover acceptance note listed older PID constants, but the actual 06-04 plan required the upstream-derived constants already present in the code.

## Verification

- `cargo test -p bitaxe-safety --all-features safety_thermal`
- `cargo test -p bitaxe-safety --all-features safety_fault`
- `bazel test //crates/bitaxe-safety:tests --test_filter='safety_thermal|safety_fault'`
- `cargo fmt --all`
- `cargo clippy --all-targets --all-features -- -D warnings`
- `cargo build --all-targets --all-features`
- `cargo test --all-features`

## Known Stubs

This plan produces pure safety decisions only. Firmware sensor reads, fan PWM, voltage writes, ASIC reset lines, and hardware evidence capture remain later gated work.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

Thermal and fault evidence can now feed Plan 06-06 ASIC/mining gates, and firmware adapters in Plan 06-08 can publish safe unavailable or fail-closed telemetry without enabling hardware effects.

## Self-Check: PASSED

- Confirmed scoped thermal/fault tests passed under Cargo and Bazel.
- Confirmed task commit `cb61c26` exists in git history.

---
*Phase: 06-safety-controllers-and-self-test*
*Completed: 2026-06-28*
