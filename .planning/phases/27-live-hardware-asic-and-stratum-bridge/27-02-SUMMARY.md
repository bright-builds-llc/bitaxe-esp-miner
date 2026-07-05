---
phase: 27-live-hardware-asic-and-stratum-bridge
plan: 02
subsystem: stratum
tags: [rust, submit-intent, nonce-correlation, live-bridge]
requires:
  - phase: 27-live-hardware-asic-and-stratum-bridge
    provides: Plan 27-01 ASIC dispatch and generation stamping hook
provides:
  - apply_bridge_observation pure runtime API
  - Firmware read/correlate/submit/classify pump loop extension
affects: [STR-09, ASIC-11]
key-files:
  modified:
    - crates/bitaxe-stratum/src/v1/live_runtime.rs
    - firmware/bitaxe/src/live_stratum_runtime.rs
    - firmware/bitaxe/src/asic_adapter/production.rs
key-decisions:
  - "Submit queuing stays in pure LiveStratumRuntime; firmware stamps observed_generation."
  - "Phase 27 pump uses 32 iterations vs Phase 25 default 16."
requirements-completed: [STR-08, STR-09, ASIC-10, ASIC-11]
generated_by: gsd-execute-plan
lifecycle_mode: yolo
phase_lifecycle_id: 27-2026-07-05T14-51-50
completed: 2026-07-05
duration: 15min
---

# Phase 27 Plan 02: Observation-to-Submit Loop Summary

**Hardware nonce observations feed correlation and intent-tied mining.submit inside the live socket runtime**

## Task Commits

1. **Tasks 27-02-01 and 27-02-02** - `5e8461c` (feat)

## Deviations from Plan

None - plan executed as written.

## Self-Check: PASSED

- `apply_bridge_observation` in live_runtime.rs FOUND
- Commit `5e8461c` FOUND
