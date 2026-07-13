---
phase: 06-safety-controllers-and-self-test
plan: "08"
subsystem: firmware
tags: [rust, firmware, safety, telemetry, observe-only]

requires:
  - phase: 06-06
    provides: Safety evidence gate semantics for mining and ASIC initialization
  - phase: 06-07
    provides: Explicit API safety telemetry report/status projection
provides:
  - Observe-only firmware safety adapter facade
  - Ultra 205 DS4432U/INA260 and EMC2101 constants in firmware adapter modules
  - Safety telemetry population through the runtime API snapshot path
  - Firmware interpretation hooks for typed safety effects without hardware writes
affects: [phase-06, firmware-safety-adapter, api-telemetry]

tech-stack:
  added:
    - bitaxe-safety dependency for `bitaxe-firmware`
  patterns: [observe-only firmware adapter, explicit unavailable telemetry, fail-closed hardware boundary]

key-files:
  created:
    - firmware/bitaxe/src/safety_adapter.rs
    - firmware/bitaxe/src/safety_adapter/power.rs
    - firmware/bitaxe/src/safety_adapter/thermal.rs
  modified:
    - Cargo.lock
    - MODULE.bazel.lock
    - firmware/bitaxe/Cargo.toml
    - firmware/bitaxe/BUILD.bazel
    - firmware/bitaxe/src/main.rs
    - firmware/bitaxe/src/runtime_snapshot.rs

key-decisions:
  - "Keep firmware safety adapters observe-only by default; missing hardware evidence reports explicit unavailable telemetry."
  - "Wire bitaxe-safety into firmware through Cargo because the firmware Bazel target delegates Rust compilation to the Cargo/ESP-IDF wrapper."
  - "Expose effect interpretation hooks without DS4432U, PWM, reset, or mining hardware writes until later hardware-evidence phases own them."

patterns-established:
  - "Firmware snapshots source safety telemetry through `SafeTelemetrySnapshot::from_report` rather than zero placeholders."
  - "Hardware address/register facts live at the firmware adapter boundary, but effects stay suppressed without evidence."

requirements-completed: [SAFE-01, SAFE-02, SAFE-04, SAFE-07, SAFE-08]
generated_by: gsd-execute-plan
lifecycle_mode: yolo
phase_lifecycle_id: 6-2026-06-28T02-37-37
generated_at: 2026-06-28T04:55:52Z

duration: 7 min
completed: 2026-06-28
---

# Phase 06 Plan 08: Firmware Safety Adapter Summary

**Observe-only firmware safety adapters now feed explicit safety telemetry**

## Performance

- **Duration:** 7 min
- **Started:** 2026-06-28T04:49:00Z
- **Completed:** 2026-06-28T04:55:52Z
- **Tasks:** 1
- **Files modified:** 9

## Accomplishments

- Added `bitaxe-safety` to the firmware crate and documented the Cargo-owned dependency boundary in the firmware Bazel genrule.
- Added `safety_adapter` modules for the firmware facade, Ultra 205 power telemetry/effect interpretation, and thermal/fan telemetry/effect interpretation.
- Added DS4432U, INA260, and EMC2101 adapter constants without enabling voltage, fan, ASIC reset, or mining writes.
- Updated `runtime_snapshot::collect_api_snapshot()` to populate `snapshot.safe_telemetry` from the firmware safety adapter.
- Kept default firmware safety telemetry explicit as unavailable with `hardware_evidence_pending` or `thermal_hardware_evidence_pending`.

## Task Commits

1. **Task 1: Add observe-only power and thermal firmware safety adapters** - `ef5fed5`

## Files Created/Modified

- `firmware/bitaxe/src/safety_adapter.rs` - Firmware safety adapter facade and typed effect interpretation hooks.
- `firmware/bitaxe/src/safety_adapter/power.rs` - Observe-only DS4432U/INA260 constants, unavailable power report, and voltage write suppression.
- `firmware/bitaxe/src/safety_adapter/thermal.rs` - EMC2101 constant, unavailable thermal report, raw reading parser, and fan write suppression.
- `firmware/bitaxe/src/runtime_snapshot.rs` - API snapshot safety telemetry integration.
- `firmware/bitaxe/src/main.rs` - Safety adapter module declaration.
- `firmware/bitaxe/Cargo.toml`, `firmware/bitaxe/BUILD.bazel`, `Cargo.lock`, `MODULE.bazel.lock` - Firmware dependency wiring and Bazel lockfile fingerprints.

## Decisions Made

- Did not add a direct Bazel Rust dependency for `bitaxe-safety` because `//firmware/bitaxe:firmware` is a genrule that compiles through `//scripts:build_firmware` and Cargo.
- Left adapter interpretation functions locally allowed for `dead_code` because they are planned hooks for the next firmware supervision work and must not be forced into live effects prematurely.
- Included the Bazel module lockfile fingerprint refresh after `just test` reconciled the new firmware Cargo dependency.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] Firmware adapter hooks were intentionally unused in this observe-only plan**
- **Found during:** Firmware Xtensa build verification
- **Issue:** The build warned on future-use adapter hooks and constants that are intentionally not live-effect paths yet.
- **Fix:** Added local `dead_code` allowances only to the `safety_adapter` module files.
- **Files modified:** Firmware safety adapter files.
- **Verification:** Re-ran firmware build, Bazel firmware build, API telemetry tests, full Rust gate, `just test`, and `just parity`.
- **Committed in:** `ef5fed5`

**Total deviations:** 1 auto-fixed blocking issue
**Impact on plan:** Behavior stayed aligned with the plan; no hardware-control path was enabled.

## Issues Encountered

No blockers. The adapter reports unavailable telemetry until a later hardware phase supplies real sensor reads and hardware-evidence tokens.

## Verification

- `cargo fmt --all`
- `cargo build -p bitaxe-firmware --target xtensa-esp32s3-espidf`
- `bazel build //firmware/bitaxe:firmware`
- `cargo test -p bitaxe-api --all-features safety_telemetry`
- Acceptance `rg` checks for dependency wiring, module declarations, adapter entry points, hardware constants, unavailable reasons, and snapshot integration.
- `cargo clippy --all-targets --all-features -- -D warnings`
- `cargo build --all-targets --all-features`
- `cargo test --all-features`
- `just test`
- `just parity` (`validation_errors: none`)

## Known Stubs

- Firmware power and thermal reads remain unavailable/observe-only; no INA260, DS4432U, EMC2101, PWM, ASIC reset, or mining hardware effect is executed.
- `BITAXE_SAFETY_TELEMETRY=observe-only` is reserved as an explicit adapter path but currently returns the same unavailable report until real hardware evidence exists.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

Plan 06-09 can attach a firmware supervisor/watchdog surface to these adapter hooks and keep display/input runtime gaps explicit without bypassing evidence gates.

## Self-Check: PASSED

- Confirmed firmware-specific checks passed after the observe-only adapter integration.
- Confirmed full Rust gate and project-level `just test && just parity` passed.
- Confirmed task commit `ef5fed5` exists in git history.

---
*Phase: 06-safety-controllers-and-self-test*
*Completed: 2026-06-28*
