---
phase: 32-shared-i2c-and-read-only-sensor-acquisition
reviewed: 2026-07-14T00:49:40Z
depth: standard
files_reviewed: 14
files_reviewed_list:
  - crates/bitaxe-api/src/observation.rs
  - crates/bitaxe-safety/src/lib.rs
  - crates/bitaxe-safety/src/power.rs
  - crates/bitaxe-safety/src/sensor_acquisition.rs
  - docs/evidence/phase-32/README.md
  - firmware/bitaxe/src/display_adapter.rs
  - firmware/bitaxe/src/main.rs
  - firmware/bitaxe/src/operator_sensor_runtime.rs
  - firmware/bitaxe/src/safety_adapter.rs
  - firmware/bitaxe/src/safety_adapter/emc2101.rs
  - firmware/bitaxe/src/safety_adapter/i2c_bus.rs
  - firmware/bitaxe/src/safety_adapter/ina260.rs
  - firmware/bitaxe/src/safety_adapter/phase27_bring_up.rs
  - tools/parity/src/phase32_source_guard.rs
findings:
  critical: 0
  warning: 0
  info: 0
  total: 0
status: clean
---

# Phase 32: Code Review Report

**Reviewed:** 2026-07-14T00:49:40Z
**Depth:** standard
**Files Reviewed:** 14
**Status:** clean

## Summary

Re-reviewed the Phase 32 bounded I2C ownership, INA260/EMC2101 decoding and reduction, periodic producer, observation projection/store boundary, source guards, and software-only evidence record after the three initial findings were addressed. The review was materially informed by `AGENTS.md`, `AGENTS.bright-builds.md`, `standards-overrides.md`, `standards/index.md`, `standards/core/architecture.md`, `standards/core/code-shape.md`, `standards/core/operability.md`, `standards/core/testing.md`, `standards/core/verification.md`, `standards/languages/rust.md`, and the Phase 32 context, plan, validation, and research artifacts.

The post-display producer now owns a distinct read-only type-state, sustained failed acquisitions age retained last-good samples to stale under producer-supplied time, and consumer failure isolation is exercised for power, temperature, and tachometer sources. No remaining critical, warning, or info findings were found. The evidence record remains conservative: no credential exposure, direct-UART/pin work, archived-lineage reopening, hardware overclaim, or normal-path actuator call was found.

## Critical Issues

None.

## Resolved Findings

### WR-01: Post-display producer owner could express arbitrary I2C writes — resolved

`BitaxeI2cBus::into_read_only` now consumes the display-capable owner and returns `ReadOnlySensorOwner`, whose private driver is exposed only through the closed typed `ReadOnlySensorBus`. `operator_sensor_runtime::start` and its loop accept only `ReadOnlySensorOwner`; they can no longer mint `StartupDisplayBus` or access the Phase 27 active capability. The runtime source guard now checks the consuming handoff, read-only producer signature, and absence of `BitaxeI2cBus` and `startup_display` from the producer module.

### WR-02: Sustained failures could not age retained last-good data to stale — resolved

Elapsed-time evaluation now examines any retained last-good stamp rather than fresh observations only. A focused regression test proves that power, temperature, and tachometer remain faulted through the exact threshold, transition to stale after it, preserve their exact last-good stamps, and do not advance source sequences during failed attempts.

### IN-01: Consumer integration covered only temperature failure — resolved

The API/store regression suite now iterates over failed power, temperature, and tachometer sources. Each case performs repeated clone-only reads and verifies fault projection for the affected field or atomic power group while all unaffected source fields remain fresh.

## Remaining Findings

None.

## Verification

- `cargo fmt --all -- --check` passed.
- `cargo test -p bitaxe-safety sensor_acquisition` passed: 10 tests, 0 failures.
- `cargo test -p bitaxe-api phase32_consumer` passed: 2 tests, 0 failures.
- `cargo test -p bitaxe-parity phase32_` passed: 8 tests, 0 failures.
- `just build` passed; `//firmware/bitaxe:firmware` is up to date and successfully analyzed.
- The reviewed evidence document remains software-only, records hardware evidence pending, and makes no parity promotion claim.
- The only pre-existing unrelated worktree modification observed before this report was `.planning/config.json`; no implementation file was edited by this review.

***

_Reviewed: 2026-07-14T00:49:40Z_
_Reviewer: gsd-code-reviewer_
_Depth: standard_
