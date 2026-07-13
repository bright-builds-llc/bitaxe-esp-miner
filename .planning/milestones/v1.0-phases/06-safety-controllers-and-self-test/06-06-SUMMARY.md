---
phase: 06-safety-controllers-and-self-test
plan: "06"
subsystem: safety
tags: [rust, asic, stratum, mining-gate, evidence, fail-closed]

requires:
  - phase: 06-03
    provides: `PowerEvidenceToken`
  - phase: 06-04
    provides: `ThermalEvidenceToken`
  - phase: 06-05
    provides: safety and hardware evidence semantics for blocked self-test/mining work
provides:
  - BM1366 preflight constructors backed by safety-core power, thermal, and safety evidence
  - Stratum mining gate requiring power, thermal, safety, and hardware acknowledgment
  - Distinct fail-closed reasons for missing power, missing thermal, missing safety, and missing hardware acknowledgment
affects: [phase-06, asic-init, mining-gate, api-telemetry, firmware-safety-adapter, parity]

tech-stack:
  added:
    - bitaxe-safety dependency for `bitaxe-asic`
    - bitaxe-safety dependency for `bitaxe-stratum`
  patterns: [token-backed preflight, fail-closed gate ordering, hardware evidence acknowledgment]

key-files:
  modified:
    - Cargo.lock
    - MODULE.bazel.lock
    - crates/bitaxe-asic/Cargo.toml
    - crates/bitaxe-asic/BUILD.bazel
    - crates/bitaxe-asic/src/bm1366/init_plan.rs
    - crates/bitaxe-stratum/Cargo.toml
    - crates/bitaxe-stratum/BUILD.bazel
    - crates/bitaxe-stratum/src/v1/mining_loop.rs

key-decisions:
  - "Remove production no-context ASIC preflight evidence constructors and require safety-core tokens."
  - "Block Stratum work submission for missing power before safety, missing thermal before safety, faulted/missing safety, missing hardware acknowledgment, or missing ASIC initialization."
  - "Do not treat unit implementation evidence as hardware verification for mining readiness."

patterns-established:
  - "Downstream ASIC and Stratum crates depend on `bitaxe-safety` for safety evidence contracts."
  - "Mining readiness is a typed gate with stable reason strings for API and firmware telemetry."

requirements-completed: [SAFE-01, SAFE-02, SAFE-04, SAFE-08, SAFE-09]
generated_by: gsd-execute-plan
lifecycle_mode: yolo
phase_lifecycle_id: 6-2026-06-28T02-37-37
generated_at: 2026-06-28T04:42:05Z

duration: 28 min
completed: 2026-06-28
---

# Phase 06 Plan 06: ASIC And Mining Safety Gates Summary

**Power, thermal, safety, and hardware evidence gates for BM1366 init and Stratum work submission**

## Performance

- **Duration:** 28 min
- **Started:** 2026-06-28T04:14:00Z
- **Completed:** 2026-06-28T04:42:05Z
- **Tasks:** 1
- **Files modified:** 8

## Accomplishments

- Added `bitaxe-safety` as an internal dependency for ASIC and Stratum crates in Cargo and Bazel.
- Replaced marker ASIC preflight evidence constructors with `from_power_token`, `from_thermal_token`, and `from_safety_status`.
- Updated BM1366 full-init preflight tests for missing and faulted safety evidence.
- Replaced the Stratum boolean safety gate with power, thermal, safety, status, and hardware-ack fields.
- Added missing-power, missing-thermal, missing-safety, faulted-safety, missing-hardware-ack, and ready mining gate tests.

## Task Commits

1. **Task 1: Feed power, thermal, and safety evidence into ASIC and mining gates** - `c8fca73`

## Files Created/Modified

- `crates/bitaxe-asic/src/bm1366/init_plan.rs` - Token-backed BM1366 preflight evidence and safety-status checks.
- `crates/bitaxe-stratum/src/v1/mining_loop.rs` - D-18 mining gate shape and fail-closed ordering.
- `crates/bitaxe-asic/Cargo.toml`, `crates/bitaxe-stratum/Cargo.toml` - Internal safety crate dependencies.
- `crates/bitaxe-asic/BUILD.bazel`, `crates/bitaxe-stratum/BUILD.bazel` - Bazel safety crate dependencies.
- `Cargo.lock`, `MODULE.bazel.lock` - Dependency graph updates.

## Decisions Made

- Kept ASIC full-init as initialized-no-mining when pure tokens are present, while Stratum work submission remains stricter and requires hardware-verified safety evidence plus explicit hardware acknowledgment.
- Preserved existing `safety_preflight_evidence_missing` and `hardware_evidence_ack_missing` reason strings while adding power/thermal-specific reason strings.

## Deviations from Plan

None.

## Issues Encountered

No implementation blockers. Cargo and Bazel lockfiles updated as expected after adding internal crate dependencies.

## Verification

- `cargo test -p bitaxe-asic --all-features init_plan`
- `cargo test -p bitaxe-stratum --all-features mining_loop`
- `bazel test //crates/bitaxe-asic:tests //crates/bitaxe-stratum:tests --test_filter='init_plan|mining_loop'`
- `cargo fmt --all`
- `cargo clippy --all-targets --all-features -- -D warnings`
- `cargo build --all-targets --all-features`
- `cargo test --all-features`
- `just test`
- `just parity` (`validation_errors: none`)

## Known Stubs

The gates are pure decisions. Live ASIC init, live power/thermal telemetry, and live mining work submission remain blocked until hardware evidence is captured and firmware adapters are wired.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

The API and firmware phases can now read stable mining gate reasons and safety token states instead of boolean or zeroed placeholders.

## Self-Check: PASSED

- Confirmed scoped ASIC/Stratum tests passed under Cargo and Bazel.
- Confirmed full Rust gate and project-level `just test && just parity` passed.
- Confirmed task commit `c8fca73` exists in git history.

---
*Phase: 06-safety-controllers-and-self-test*
*Completed: 2026-06-28*
