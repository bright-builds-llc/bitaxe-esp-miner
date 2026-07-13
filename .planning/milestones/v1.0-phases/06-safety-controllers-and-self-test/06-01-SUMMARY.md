---
phase: 06-safety-controllers-and-self-test
plan: "01"
subsystem: safety
tags: [rust, bazel, safety, evidence, fail-closed]

requires:
  - phase: 05-axeos-api-logs-and-telemetry
    provides: AxeOS API/log/telemetry surfaces that Phase 6 safety status will later feed
provides:
  - Host-testable `bitaxe-safety` crate wired into Cargo and Bazel
  - Shared `SafetyStatus`, `SafetyCriticalEvidence`, `SafetyEffect`, and `SafetyEffectPlan` contracts
  - Hardware evidence labels that distinguish implementation from hardware verification
  - Fail-closed effect plan contract for downstream power, thermal, ASIC, mining, API, and firmware adapters
affects: [phase-06, power, thermal, self-test, watchdog, api-telemetry, asic-init, mining-gate, parity]

tech-stack:
  added: [bitaxe-safety crate]
  patterns: [pure safety contract crate, typed fail-closed effect plans, explicit hardware evidence labels]

key-files:
  created:
    - crates/bitaxe-safety/Cargo.toml
    - crates/bitaxe-safety/BUILD.bazel
    - crates/bitaxe-safety/src/lib.rs
    - crates/bitaxe-safety/src/effects.rs
    - crates/bitaxe-safety/src/evidence.rs
    - crates/bitaxe-safety/src/status.rs
  modified:
    - Cargo.toml
    - Cargo.lock
    - MODULE.bazel.lock

key-decisions:
  - "Use a focused pure `bitaxe-safety` crate for Phase 6 contracts before firmware hardware effects are touched."
  - "Only `hardware-smoke` and `hardware-regression` evidence satisfy safety-critical hardware verification."
  - "Fail-closed safety plans explicitly hold reset low, disable ASIC enable, suppress voltage writes, block work submission, and publish visible status."

patterns-established:
  - "Safety contracts are typed data plans, not firmware actions."
  - "Evidence labels must not let implemented/unit proof masquerade as hardware verification."

requirements-completed: [SAFE-01, SAFE-02, SAFE-03, SAFE-04, SAFE-05, SAFE-07, SAFE-08, SAFE-09]
generated_by: gsd-execute-plan
lifecycle_mode: yolo
phase_lifecycle_id: 6-2026-06-28T02-37-37
generated_at: 2026-06-28T04:09:06Z

duration: 9 min
completed: 2026-06-28
---

# Phase 06 Plan 01: Safety Contracts Summary

**Pure Rust safety contract crate with fail-closed effect plans and hardware-verification evidence labels**

## Performance

- **Duration:** 9 min
- **Started:** 2026-06-28T04:00:36Z
- **Completed:** 2026-06-28T04:09:06Z
- **Tasks:** 2
- **Files modified:** 9

## Accomplishments

- Created `crates/bitaxe-safety` as a host-testable pure Rust crate with Cargo workspace and Bazel target wiring.
- Added shared public modules `effects`, `evidence`, and `status` behind the Phase 6 safety facade.
- Defined serializable contracts for visible safety statuses, safety-critical evidence labels, and fail-closed effect plans.

## Task Commits

1. **Task 1: Create the bitaxe-safety crate and shared contract facade** - `01ca11d` (feat)
2. **Task 2: Define shared safety status, effect, and evidence contracts** - `e160eca` (feat)

## Files Created/Modified

- `Cargo.toml` - Added `crates/bitaxe-safety` to workspace members and default members.
- `Cargo.lock` - Refreshed workspace package metadata for `bitaxe-safety`.
- `MODULE.bazel.lock` - Refreshed crate universe metadata for the new crate manifest.
- `crates/bitaxe-safety/Cargo.toml` - Declared the pure safety crate and dependencies.
- `crates/bitaxe-safety/BUILD.bazel` - Added public `rust_library` and test target.
- `crates/bitaxe-safety/src/lib.rs` - Exposed the Phase 6 contract facade.
- `crates/bitaxe-safety/src/effects.rs` - Added fail-closed safety effect plans.
- `crates/bitaxe-safety/src/evidence.rs` - Added safety-critical evidence classification.
- `crates/bitaxe-safety/src/status.rs` - Added user-visible safety status contract.

## Decisions Made

- Created a focused `bitaxe-safety` crate instead of extending `bitaxe-core`, matching Phase 6's need for a named shared contract surface.
- Modeled `ImplementedNotVerified` as `unit` evidence and kept it invalid for hardware verification.
- Included `DisableAsicEnable` in `SafetyEffectPlan::fail_closed` alongside the plan-required reset, voltage suppression, work blocking, and status publication.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] Recorded TDD RED failures without failing commits**
- **Found during:** Tasks 1 and 2
- **Issue:** The generic TDD flow asks for failing RED commits, but `AGENTS.md` requires `cargo fmt`, Clippy, build, and tests to pass before every Rust commit.
- **Fix:** Ran RED tests and confirmed expected failures, then committed only the passing task states after the full Rust gate.
- **Files modified:** None beyond planned task files
- **Verification:** RED failures observed before implementation; final task gates passed before each commit.
- **Committed in:** `01ca11d`, `e160eca`

**Total deviations:** 1 process adjustment
**Impact on plan:** Code scope stayed aligned with the plan while satisfying repo-local commit rules.

## Issues Encountered

None. The expected TDD RED failures were resolved by the planned implementations.

## Verification

- `node "$HOME/.codex/get-shit-done/bin/gsd-tools.cjs" verify lifecycle "06" --require-plans --raw` -> `valid`
- `cargo test -p bitaxe-safety --all-features safety_contract`
- `cargo test -p bitaxe-safety --all-features evidence`
- `bazel test //crates/bitaxe-safety:tests`
- `cargo fmt --all`
- `cargo clippy --all-targets --all-features -- -D warnings`
- `cargo build --all-targets --all-features`
- `cargo test --all-features`
- `cargo test -p bitaxe-safety --all-features`
- `just test`
- `just parity`

## Known Stubs

None.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

Ready for 06-02. Downstream plans can now build module-specific power, thermal, self-test, watchdog, API, firmware, and parity work against named status, evidence, and effect contracts.

## Self-Check: PASSED

- Confirmed created safety crate files and summary exist.
- Confirmed task commits `01ca11d` and `e160eca` exist in git history.

---
*Phase: 06-safety-controllers-and-self-test*
*Completed: 2026-06-28*
