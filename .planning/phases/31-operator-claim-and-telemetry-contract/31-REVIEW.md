---
phase: 31-operator-claim-and-telemetry-contract
reviewed: 2026-07-13T21:33:59Z
depth: standard
files_reviewed: 20
files_reviewed_list:
  - crates/bitaxe-safety/src/observation.rs
  - crates/bitaxe-safety/src/lib.rs
  - crates/bitaxe-safety/BUILD.bazel
  - crates/bitaxe-safety/src/power.rs
  - crates/bitaxe-safety/src/thermal.rs
  - crates/bitaxe-safety/src/mining_preconditions.rs
  - crates/bitaxe-api/src/observation.rs
  - crates/bitaxe-api/src/snapshot.rs
  - crates/bitaxe-api/src/wire.rs
  - crates/bitaxe-api/src/telemetry.rs
  - firmware/bitaxe/src/safety_adapter/observation_store.rs
  - firmware/bitaxe/src/runtime_snapshot.rs
  - firmware/bitaxe/src/safety_adapter/phase27_bring_up.rs
  - crates/bitaxe-api/src/v12_settings.rs
  - crates/bitaxe-api/src/settings.rs
  - crates/bitaxe-api/src/lib.rs
  - crates/bitaxe-api/BUILD.bazel
  - tools/parity/src/v12_admission.rs
  - tools/parity/src/main.rs
  - tools/parity/BUILD.bazel
findings:
  critical: 0
  warning: 2
  info: 0
  total: 2
status: issues_found
generated_by: gsd-code-reviewer
lifecycle_mode: yolo
phase_lifecycle_id: 31-2026-07-13T19-47-51
generated_at: 2026-07-13T21:33:59Z
---

# Phase 31: Code Review Report

**Reviewed:** 2026-07-13T21:33:59Z
**Depth:** standard
**Files Reviewed:** 20
**Status:** issues_found

## Summary

Reviewed the Phase 31 observation truth core, safety migrations, API and firmware consumer boundary, hostname-only settings capability, and typed parity admission contract. The state-carrying observation enum, read-only request path, hostname classifier, and excluded-claim vocabulary are generally conservative, and the focused test suites pass. Two actionable gaps remain: the retained Phase 27 publication path emits fixed compatibility stamps as if they were producer provenance, and the public legacy report mapper can still construct a contradictory telemetry snapshot whose aggregate state is fresh while every new fact-level truth field is unavailable.

This review was materially informed by repo-local hardware and phase-boundary guidance in `AGENTS.md`, the Bright Builds workflow in `AGENTS.bright-builds.md`, `standards-overrides.md`, and the architecture, code-shape, verification, testing, and Rust standards under `standards/`.

## Warnings

### WR-01: Retained Phase 27 samples are published with fixed compatibility stamps

**Files:** `firmware/bitaxe/src/safety_adapter/phase27_bring_up.rs:138-165`, `firmware/bitaxe/src/safety_adapter/phase27_bring_up.rs:209-228`, `crates/bitaxe-safety/src/power.rs:85-99`, `crates/bitaxe-safety/src/thermal.rs:79-87`

**Issue:** `store_snapshot()` publishes Phase 27 power and temperature observations into the new operator store. Those observations are created through `PowerObservation::from_ina260_sample()` and `ThermalObservation::from_reading()`, whose compatibility constructors hard-code boot session `0`, prior sequence `0`, and acquisition time `0`. Every successful live sample therefore appears as `fresh` with the same public stamp `{ bootSession: 0, sequence: 1, acquiredAtMs: 0 }`, regardless of the actual boot, acquisition time, or prior producer success. Repeated consumer reads do preserve that stamp, but the stamp itself is not producer provenance and cannot support the Phase 31 claim that successful samples are bound to a boot/session identity and monotonic acquisition metadata.

**Fix:** Do not publish these compatibility-constructed observations as fresh operator truth. Either keep the retained Phase 27 facts unavailable until Phase 32 installs the sole stamped producer, or pass a real boot-session identity, source-owned sequence, and acquisition-time value through `from_stamped_ina260_sample()` and `from_stamped_reading()` at the acquisition boundary. Add a firmware-hostable regression proving that successive producer successes advance the source sequence, carry their actual monotonic acquisition times, and do not reuse the fixed compatibility stamp.

### WR-02: Legacy report mapping creates contradictory fresh and unavailable truth

**Files:** `crates/bitaxe-api/src/snapshot.rs:217-245`, `crates/bitaxe-api/src/wire.rs:417-453`

**Issue:** `SafeTelemetrySnapshot::from_report()` accepts a hardware-verified `SafetyTelemetryReport` with `SafetyTelemetryStatus::Fresh`, preserves all nonzero numeric values, and sets the aggregate snapshot status to `Fresh`, but fills each of the six Phase 31 fact-level truth fields with `legacy_unavailable_truth()`. The existing wire regression exercises this exact path and asserts the numeric values without checking the truth fields. The public API can therefore serialize power, voltage, current, temperature, VR temperature, and fan RPM as live values while simultaneously labeling each fact unavailable, leaving two incompatible truth authorities in one response. This violates the Phase 31 requirement that observation truth be independent from compatibility numerics and that legacy DTOs not remain an alternate source of truth.

**Fix:** Make operator-facing telemetry snapshots constructible only from `TelemetryObservations`, or change the legacy mapper so it cannot advertise aggregate freshness or live numeric facts without matching stamped per-fact observations. Add a regression over the serialized `SystemInfoWire` that rejects any snapshot with aggregate `Fresh` and unavailable/fault/stale fact truth, and ensure every nonzero live numeric emitted by this path has a matching fresh stamped fact.

## Verification

- `cargo test -p bitaxe-safety --all-features observation` passed: 12 tests.
- `cargo test -p bitaxe-api --all-features safety_telemetry` passed: 14 tests.
- `cargo test -p bitaxe-api --all-features settings_v12` passed: 8 tests.
- `cargo test -p bitaxe-parity --all-features phase31` passed: 7 tests.
- Call-chain inspection confirmed request-side `collect_api_snapshot()` reads the stored observation snapshot and does not invoke sensor acquisition.
- Settings and admission inspection found no broadened persistence authority, eligible excluded-claim variant, or raw secret value in the reviewed Phase 31 surfaces.

***

_Reviewed: 2026-07-13T21:33:59Z_
_Reviewer: gsd-code-reviewer_
_Depth: standard_
