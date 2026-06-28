---
phase: 06-safety-controllers-and-self-test
plan: "05"
subsystem: safety
tags: [rust, self-test, watchdog, lifecycle, fail-closed]

requires:
  - phase: 06-02
    provides: Public `bitaxe_safety::self_test` and `bitaxe_safety::watchdog` module boundaries
provides:
  - Factory, boot-button, and manual self-test lifecycle states
  - Self-test result, cancel, restart, and factory-flag effect decisions
  - Diagnostic hardware evidence gates for self-test submodes
  - Bounded step supervision with watchdog yield/reset decisions
affects: [phase-06, asic-init, mining-gate, api-telemetry, firmware-safety-adapter, parity]

tech-stack:
  added: []
  patterns: [typed state machine, watchdog-friendly steps, fixture provenance]

key-files:
  created:
    - crates/bitaxe-safety/fixtures/safety/self-test-lifecycle-cases.json
    - crates/bitaxe-safety/fixtures/safety/watchdog-step-cases.json
  modified:
    - crates/bitaxe-safety/src/self_test.rs
    - crates/bitaxe-safety/src/watchdog.rs

key-decisions:
  - "Keep production mining blocked for all self-test lifecycle paths."
  - "Require hardware-verified safety evidence plus power, thermal, ASIC, and explicit hardware acknowledgment before diagnostic work is allowed."
  - "Represent long-running self-test and safety work as bounded pure steps, not blocking task loops."

patterns-established:
  - "Lifecycle commands return typed effects and optional `WatchdogDecision` values."
  - "Missing diagnostic evidence reports `self_test_hardware_evidence_missing` without enabling diagnostic or production work."

requirements-completed: [SAFE-05, SAFE-09]
generated_by: gsd-execute-plan
lifecycle_mode: yolo
phase_lifecycle_id: 6-2026-06-28T02-37-37
generated_at: 2026-06-28T04:35:35Z

duration: 20 min
completed: 2026-06-28
---

# Phase 06 Plan 05: Self-Test Lifecycle And Watchdog Summary

**Pure self-test state machine and bounded watchdog step supervision**

## Performance

- **Duration:** 20 min
- **Started:** 2026-06-28T04:15:00Z
- **Completed:** 2026-06-28T04:35:35Z
- **Tasks:** 2
- **Files modified:** 4

## Accomplishments

- Implemented self-test origins, commands, states, effects, decisions, and evidence gates.
- Added factory flag clearing behavior for pass/cancel while keeping fail paths blocked and result-reporting.
- Added explicit restart and cancel effects without enabling production work submission.
- Added watchdog step decisions for power, thermal, fan, self-test, and telemetry work.
- Added provenance fixtures for lifecycle and bounded-step cases.

## Task Commits

1. **Task 1: Model self-test factory/manual lifecycle and result effects** - `4a1703e` (feat, combined)
2. **Task 2: Add watchdog-friendly bounded step supervision** - `4a1703e` (feat, combined)

## Files Created/Modified

- `crates/bitaxe-safety/src/self_test.rs` - Self-test lifecycle, evidence gates, result/cancel/restart effects, and watchdog-linked step command.
- `crates/bitaxe-safety/src/watchdog.rs` - Step kinds, progress, supervisor constants, and yield/reset decisions.
- `crates/bitaxe-safety/fixtures/safety/self-test-lifecycle-cases.json` - Self-test lifecycle provenance and diagnostic-only cases.
- `crates/bitaxe-safety/fixtures/safety/watchdog-step-cases.json` - Bounded step provenance and watchdog decision cases.

## Decisions Made

- Used `SelfTestEvidence` booleans for power, thermal, ASIC, and hardware acknowledgment until Plan 06-06 wires concrete downstream tokens.
- Kept self-test pass as result/status data only; restart remains an explicit lifecycle command/effect.
- Treated watchdog reset/feed as a typed decision so firmware can decide how to service task responsiveness later.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] Combined coupled task commits**
- **Found during:** Task commit boundary
- **Issue:** Self-test `Step` depends directly on `WatchdogDecision`, so splitting the module commits would create avoidable partial behavior.
- **Fix:** Committed self-test and watchdog outcomes together after scoped checks and the full Rust gate passed.
- **Files modified:** `crates/bitaxe-safety/src/self_test.rs`, `crates/bitaxe-safety/src/watchdog.rs`, safety fixtures.
- **Verification:** Scoped self-test/watchdog tests plus full Rust gate.
- **Committed in:** `4a1703e`

**Total deviations:** 1 auto-fixed blocking issue
**Impact on plan:** Behavior and verification stayed aligned with the plan; commit granularity was coarser than the ideal task boundary.

## Issues Encountered

No implementation blockers. The existing module boundaries were intentionally empty, so the work stayed localized to the safety crate.

## Verification

- `cargo test -p bitaxe-safety --all-features self_test`
- `cargo test -p bitaxe-safety --all-features watchdog`
- `bazel test //crates/bitaxe-safety:tests --test_filter='watchdog|self_test'`
- `cargo fmt --all`
- `cargo clippy --all-targets --all-features -- -D warnings`
- `cargo build --all-targets --all-features`
- `cargo test --all-features`
- `just test`
- `just parity` (`validation_errors: none`)

## Known Stubs

This plan models lifecycle and watchdog decisions only. Firmware button handling, NVS writes, restart calls, mock Stratum injection, nonce capture, and hardware evidence capture remain gated adapter work.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

Self-test/watchdog contracts are ready for Plan 06-06 mining gates and Plan 06-08 firmware adapters. Wave 3 `just test && just parity` passed.

## Self-Check: PASSED

- Confirmed scoped self-test/watchdog tests passed under Cargo and Bazel.
- Confirmed task commit `4a1703e` exists in git history.

---
*Phase: 06-safety-controllers-and-self-test*
*Completed: 2026-06-28*
