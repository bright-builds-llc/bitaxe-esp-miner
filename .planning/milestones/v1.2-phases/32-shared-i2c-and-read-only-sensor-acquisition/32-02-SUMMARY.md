---
phase: 32-shared-i2c-and-read-only-sensor-acquisition
plan: "02"
subsystem: firmware-i2c
tags: [rust, esp-idf, embedded-hal, i2c, ssd1306, sensors]
requires:
  - phase: 32-01
    provides: Pure typed sensor decoders and failure-isolated observation reducer
provides:
  - One finite-timeout I2C0 owner with a borrowed startup-display adapter
  - Closed read-only INA260 and EMC2101 register capabilities
  - Token-gated legacy active-write capability unreachable from normal runtime
  - Cargo and Bazel source guards for timeout, ownership, register, and no-actuation boundaries
affects: [32-03-sensor-producer, phase-34-operator-snapshot, phase-35-hardware-evidence]
tech-stack:
  added: []
  patterns: [borrowed bounded embedded-hal adapter, closed register enum, unconstructable active token]
key-files:
  created:
    - tools/parity/src/phase32_source_guard.rs
  modified:
    - firmware/bitaxe/src/safety_adapter/i2c_bus.rs
    - firmware/bitaxe/src/display_adapter.rs
    - firmware/bitaxe/src/safety_adapter/ina260.rs
    - firmware/bitaxe/src/safety_adapter/emc2101.rs
    - firmware/bitaxe/src/safety_adapter/phase27_bring_up.rs
key-decisions:
  - "All I2C operations convert a named 50 ms bound with TickType before calling ESP-IDF."
  - "The display borrows the shared driver through an embedded-hal adapter and cannot select the HAL's BLOCK implementation."
  - "Normal sensor code can name only seven allowlisted read registers; active writes require a Phase 27-only token."
requirements-completed: [OBS-02, OBS-03, OBS-04]
generated_by: gsd-execute-plan
lifecycle_mode: yolo
phase_lifecycle_id: 32-2026-07-13T23-12-34
generated_at: 2026-07-14T00:24:47Z
duration: 21 min
completed: 2026-07-13
---

# Phase 32 Plan 02: Bounded Shared I2C and Read-Only Adapters Summary

**One finite-timeout I2C0 owner now serves startup display and closed sensor-read capabilities while legacy writes require an unconstructable normal-runtime token.**

## Performance

- **Duration:** 21 min
- **Started:** 2026-07-14T00:03:40Z
- **Completed:** 2026-07-14T00:24:47Z
- **Tasks:** 2
- **Files modified:** 17

## Accomplishments

- Replaced raw millisecond-as-tick calls and split pointer/read operations with real `TickType` conversion and bounded repeated-start transactions.
- Preserved SSD1306 startup rendering through a borrowed embedded-hal adapter whose read, write, write-read, and transaction methods all use finite timeouts.
- Added closed INA260 and EMC2101 read-register enums and typed normal acquisitions consumed directly by the Plan 32-01 core.
- Preserved dormant Phase 27 active helpers while making their write-capable bus wrapper require a token constructible only inside that gated module.
- Added Cargo/Bazel source guards for one driver, finite timeouts, display borrowing, exact register allowlists, and no active-effect identifiers.

## Task Commits

The two adapter-boundary tasks were committed together after their shared firmware build passed:

1. **Task 1: Make the shared I2C0 transaction boundary finite and capability-scoped** - `53f3afc` (feat)
2. **Task 2: Convert legacy readers into closed read-only typed acquisitions** - `53f3afc` (feat)

**Plan metadata:** This commit

## Files Created/Modified

- `firmware/bitaxe/src/safety_adapter/i2c_bus.rs` - Shared owner, bounded display adapter, closed sensor registers, and Phase 27-token-gated active wrapper.
- `firmware/bitaxe/src/display_adapter.rs` - Borrowed bounded-bus SSD1306 rendering.
- `firmware/bitaxe/src/safety_adapter/ina260.rs` - Complete three-register normal acquisition.
- `firmware/bitaxe/src/safety_adapter/emc2101.rs` - Independent temperature and tachometer normal acquisitions.
- `firmware/bitaxe/src/safety_adapter/phase27_bring_up.rs` and `ds4432u.rs` - Preserved legacy active path behind the private token.
- `tools/parity/src/phase32_source_guard.rs` - Host-runnable source boundary regressions.
- Cargo/Bazel manifests and locks - Declared the already-resolved embedded-hal trait dependency and hermetic source inputs.

## Decisions Made

- The initial transaction bound is 50 ms, materially below the later 500 ms producer cadence, with no same-sweep retry loop.
- INA260 attempts all three registers before deciding the one logical power outcome.
- EMC2101 temperature and tachometer remain separate reader calls so the producer can continue after either failure.
- A public generic write method is not present on the normal bus; dormant active helpers accept only the token-created active wrapper.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] Declared embedded-hal as a direct firmware dependency**

- **Found during:** Task 1 bounded display adapter implementation
- **Issue:** The exact SSD1306 interface requires implementing the embedded-hal 1.0 I2C trait, but the firmware previously received it only transitively.
- **Fix:** Declared the already-resolved `embedded-hal 1.0.0` dependency directly and refreshed Cargo/Bazel lock metadata.
- **Files modified:** `Cargo.toml`, `Cargo.lock`, `firmware/bitaxe/Cargo.toml`, `MODULE.bazel.lock`
- **Verification:** The ESP-IDF firmware target and all Cargo/Bazel guards compiled and passed.
- **Committed in:** `53f3afc`

**2. [Rule 2 - Missing Critical] Added the Plan 32-01 module to the Bazel safety target**

- **Found during:** Hermetic Phase 32 source-guard test
- **Issue:** Cargo tests passed, but Bazel could not compile `bitaxe-safety` because `sensor_acquisition.rs` was absent from its explicit source list.
- **Fix:** Added the module to `crates/bitaxe-safety/BUILD.bazel`.
- **Files modified:** `crates/bitaxe-safety/BUILD.bazel`
- **Verification:** `bazel test //tools/parity:tests` passed with the Phase 32 source filter.
- **Committed in:** `53f3afc`

**3. [Rule 2 - Missing Critical] Wired one normal driver before producer integration**

- **Found during:** Task 1 firmware compile integration
- **Issue:** Changing display rendering to borrow the shared owner required the normal boot call site to construct that owner before Plan 32-03 moves it into the producer.
- **Fix:** Updated `main.rs` to construct one `BitaxeI2cBus` and borrow it for best-effort startup rendering; producer handoff remains Plan 32-03-owned.
- **Files modified:** `firmware/bitaxe/src/main.rs`
- **Verification:** `just build` passed and the source guard proves one normal construction with no display-owned driver.
- **Committed in:** `53f3afc`

***

**Total deviations:** 3 auto-fixed (2 missing critical, 1 blocking)
**Impact on plan:** All changes were necessary to compile or hermetically enforce the planned boundary; no active behavior or hardware scope was added.

## Issues Encountered

- A first negative source token matched the legitimate name `bus_voltage`; the guard was narrowed to actual active-effect identifiers before final verification.
- System load made one fresh parity test link slow, but the process remained active and completed without intervention.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

- Plan 32-03 can move the exact post-display bus into the 500 ms producer and publish complete observation snapshots.
- Hardware remains pending by plan: no detector, target, credential, flash, monitor, active control, mining, OTA, UART/pin, or archived-lineage action occurred.

## Self-Check: PASSED

- Commit `53f3afc` and all created/modified files exist.
- Focused Cargo source guards, Bazel source guards, safety acquisition tests, and `just build` passed.
- The mandatory format, Clippy, all-target build, and all-feature test gate passed in order.
- `git diff --check` passed and `.planning/config.json` remained outside the implementation commit.
