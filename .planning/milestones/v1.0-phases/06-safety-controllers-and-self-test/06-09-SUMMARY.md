---
phase: 06-safety-controllers-and-self-test
plan: "09"
subsystem: firmware
tags: [rust, firmware, watchdog, display, safety]

requires:
  - phase: 06-08
    provides: Observe-only firmware safety adapter facade
provides:
  - Bounded firmware safety supervisor shell
  - Watchdog-cadence yield log for supervisor work
  - Runtime display/input parity gap publication
  - Main firmware wiring before the API route shell starts
affects: [phase-06, firmware-safety-supervisor, display-input-boundary]

tech-stack:
  added:
    - firmware safety adapter watchdog child module
  patterns: [named firmware thread, bounded safety cadence, explicit runtime parity gap]

key-files:
  created:
    - firmware/bitaxe/src/safety_adapter/watchdog.rs
  modified:
    - firmware/bitaxe/src/main.rs
    - firmware/bitaxe/src/display_adapter.rs
    - firmware/bitaxe/src/safety_adapter.rs

key-decisions:
  - "Start the safety supervisor as a named firmware thread before the HTTP/API shell without blocking API/log/WebSocket administration."
  - "Use the safety-core watchdog cadence and a single first-yield log instead of a noisy continuous status log."
  - "Publish runtime display/input as an explicit gap while preserving the prior startup OLED evidence as startup-only."

patterns-established:
  - "Firmware long-running safety work is represented by a small shell around pure watchdog decisions."
  - "Runtime display/input status is logged separately from startup SSD1306 rendering so evidence is not overclaimed."

requirements-completed: [SAFE-06, SAFE-09]
generated_by: gsd-execute-plan
lifecycle_mode: yolo
phase_lifecycle_id: 6-2026-06-28T02-37-37
generated_at: 2026-06-28T05:00:49Z

duration: 5 min
completed: 2026-06-28
---

# Phase 06 Plan 09: Firmware Supervisor And Display/Input Boundary Summary

**Watchdog-friendly safety supervisor shell and explicit runtime display/input gap**

## Performance

- **Duration:** 5 min
- **Started:** 2026-06-28T04:55:52Z
- **Completed:** 2026-06-28T05:00:49Z
- **Tasks:** 1
- **Files modified:** 4

## Accomplishments

- Added `firmware/bitaxe/src/safety_adapter/watchdog.rs` with a named `bitaxe-safety-supervisor` thread.
- Wrapped `StepSupervisor` decisions in a firmware loop that sleeps with `Duration::from_millis(100)`.
- Logged the first `safety_supervisor_step=yield reason=yield_interval_reached` decision without spamming logs on every loop.
- Exposed `start_safety_supervisor()` from the safety adapter facade and called it before `http_api::start_http_api()`.
- Added `publish_runtime_display_input_boundary()` and called it after startup display render attempts to log `display_input_status=runtime_gap reason=hardware_evidence_pending startup_only=true`.

## Task Commits

1. **Task 1: Start bounded safety supervisor and preserve display/input gap status** - `a465a1d`

## Files Created/Modified

- `firmware/bitaxe/src/safety_adapter/watchdog.rs` - Watchdog-cadence firmware supervisor shell.
- `firmware/bitaxe/src/safety_adapter.rs` - Supervisor facade export and spawn-failure status log.
- `firmware/bitaxe/src/display_adapter.rs` - Runtime display/input boundary publication.
- `firmware/bitaxe/src/main.rs` - Supervisor and display/input boundary startup wiring.

## Decisions Made

- Did not implement full runtime display flow, input routing, or UI pages; the log explicitly records the runtime gap.
- Used a detached Rust thread, matching the existing firmware HTTP live telemetry cadence pattern.
- Kept the supervisor low-noise by logging the first yielded step and continuing the 100 ms sleep cadence.

## Deviations from Plan

No deviations. The plan remained an observe-only supervisor shell and did not expand into hardware display/input parity.

## Issues Encountered

No blockers.

## Verification

- Acceptance `rg` checks for supervisor thread name, yield reason, spawn-failure log, 100 ms sleep, watchdog child module, display/input gap log, and API startup ordering.
- Negative `rg` check confirmed no full runtime display/input parity terms were introduced into firmware display/main/safety files.
- `cargo fmt --all`
- `cargo build -p bitaxe-firmware --target xtensa-esp32s3-espidf`
- `bazel build //firmware/bitaxe:firmware`
- `just test`
- `cargo clippy --all-targets --all-features -- -D warnings`
- `cargo build --all-targets --all-features`
- `cargo test --all-features`

## Known Stubs

- The supervisor shell does not execute real power, thermal, fan, display, input, or self-test hardware work yet.
- Runtime display/input parity remains an explicit V1 gap until a later evidence-backed implementation closes it.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

Plan 06-10 can enforce parity evidence gates and record Phase 6 evidence status with the firmware supervisor and runtime display/input gap now visible.

## Self-Check: PASSED

- Confirmed 06-09 acceptance anchors passed.
- Confirmed firmware-specific checks, `just test`, and the full Rust gate passed.
- Confirmed task commit `a465a1d` exists in git history.

---
*Phase: 06-safety-controllers-and-self-test*
*Completed: 2026-06-28*
