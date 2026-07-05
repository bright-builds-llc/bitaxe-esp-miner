---
phase: 27-live-hardware-asic-and-stratum-bridge
plan: 01
subsystem: firmware
tags: [rust, bm1366, live-stratum, phase27, uart, mining-evidence-mode]
requires:
  - phase: 25-live-stratum-runtime-and-safe-stop
    provides: Live socket shell, prerequisite gate, safe-stop postconditions
  - phase: 24-bm1366-production-work-path
    provides: Bm1366ProductionCommand, guarded mining-loop dispatch plan
provides:
  - Phase 27 compile-time mode/ack pair
  - Retained production UART executor
  - Live socket ASIC bridge dispatch from WorkQueued notify
affects: [phase-27-live-hardware-bridge, STR-08, ASIC-10]
key-files:
  created:
    - firmware/bitaxe/src/asic_adapter/production.rs
  modified:
    - firmware/bitaxe/src/mining_evidence_mode.rs
    - firmware/bitaxe/src/asic_adapter.rs
    - firmware/bitaxe/src/live_stratum_runtime.rs
    - firmware/bitaxe/src/main.rs
key-decisions:
  - "Phase 27 uses distinct mode/ack; mutually exclusive with Phase 25 starter."
  - "Production UART retained in OnceLock after work-result diagnostic success."
requirements-completed: [STR-08, ASIC-10]
generated_by: gsd-execute-plan
lifecycle_mode: yolo
phase_lifecycle_id: 27-2026-07-05T14-51-50
completed: 2026-07-05
duration: 25min
---

# Phase 27 Plan 01: Mode Gate and Live ASIC Dispatch Summary

**Distinct Phase 27 opt-in, retained production UART executor, and notify-driven BM1366 dispatch inside the Phase 25 socket shell**

## Task Commits

1. **Tasks 27-01-01 and 27-01-02** - `af53528` (feat)

## Deviations from Plan

None - plan executed as written.

## Self-Check: PASSED

- `firmware/bitaxe/src/asic_adapter/production.rs` FOUND
- Commit `af53528` FOUND
