---
phase: 31-operator-claim-and-telemetry-contract
reviewed: 2026-07-13T21:45:22Z
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
  warning: 0
  info: 0
  total: 0
status: clean
generated_by: gsd-code-reviewer
lifecycle_mode: yolo
phase_lifecycle_id: 31-2026-07-13T19-47-51
generated_at: 2026-07-13T21:45:22Z
---

# Phase 31: Code Review Report

**Reviewed:** 2026-07-13T21:45:22Z
**Depth:** standard
**Files Reviewed:** 20
**Status:** clean

## Summary

Re-reviewed the Phase 31 observation truth core, safety migrations, API and firmware consumer boundary, hostname-only settings capability, and typed parity admission contract after fix commits `2463764` and `9f66dd9`. Both prior warnings are resolved. The retained Phase 27 compatibility path now publishes explicit unavailable truth instead of fixed placeholder stamps, and legacy reports can no longer expose unstamped live numerics or aggregate freshness through operator projections. No correctness, hidden freshness mutation, compatibility, fail-open, authority-widening, secret-leakage, claim-admission, security, or code-quality issue remains in the reviewed scope.

This review was materially informed by repo-local hardware and phase-boundary guidance in `AGENTS.md`, the Bright Builds workflow in `AGENTS.bright-builds.md`, `standards-overrides.md`, and the architecture, code-shape, verification, testing, and Rust standards under `standards/`.

## Resolved Findings

### WR-01: Retained Phase 27 samples were published with fixed compatibility stamps

**Resolution:** Fixed in `2463764`. `store_snapshot()` no longer projects the retained Phase 27 power and thermal compatibility objects into operator truth. It replaces the operator store with `TelemetryObservations::unavailable_from_unstamped_legacy_source()`, whose six facts are unavailable and carry no stamp. The new `unstamped_legacy_source_cannot_publish_fresh_operator_truth` regression proves that no fixed boot/session, sequence, or acquisition metadata escapes this path. Phase 32 remains the owner of real stamped acquisition.

### WR-02: Legacy report mapping created contradictory fresh and unavailable truth

**Resolution:** Fixed in `9f66dd9`. `SafeTelemetrySnapshot::from_report()` now maps a legacy fresh report to `Unavailable { reason: "legacy_telemetry_unstamped" }` and zero-compatible numeric values. Both system-info and statistics apply `operator_projection()`, which suppresses each Phase 31 compatibility numeric unless its matching fact is fresh and stamped. New regressions cover legacy report suppression, valid stamped values, and mixed unavailable/stale/fault truth even when the mutable aggregate field says `Fresh`.

## Verification

- `cargo test -p bitaxe-safety --all-features observation` passed: 12 tests.
- `cargo test -p bitaxe-api --all-features safety_telemetry` passed: 14 tests.
- `cargo test -p bitaxe-api --all-features unstamped` passed: 2 tests.
- `cargo test -p bitaxe-api --all-features system_info_wire_rejects_nonfresh` passed: 1 test.
- `cargo test -p bitaxe-api --all-features projection` passed: 19 tests.
- `cargo test -p bitaxe-api --all-features settings_v12` passed: 8 tests.
- `cargo test -p bitaxe-parity --all-features phase31` passed: 7 tests.
- Call-chain inspection confirmed request-side `collect_api_snapshot()` reads the stored observation snapshot, retained Phase 27 compatibility data stays unavailable to operator consumers, and operator system-info/statistics projections suppress nonfresh or unstamped numerics.
- Settings and admission inspection found no broadened persistence authority, eligible excluded-claim variant, or raw secret value in the reviewed Phase 31 surfaces.

***

_Reviewed: 2026-07-13T21:45:22Z_
_Reviewer: gsd-code-reviewer_
_Depth: standard_
