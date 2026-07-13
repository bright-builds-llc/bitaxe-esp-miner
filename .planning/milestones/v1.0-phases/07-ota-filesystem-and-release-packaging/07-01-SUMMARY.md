---
phase: 07-ota-filesystem-and-release-packaging
plan: 01
generated_by: gsd-execute-plan
lifecycle_mode: yolo
phase_lifecycle_id: 7-2026-06-28T13-30-15
generated_at: 2026-06-28T15:21:05Z
subsystem: ota-static-api-contracts
tags: [rust, bitaxe-api, ota, otawww, spiffs, static-assets, recovery]
requires:
  - phase: 05-axeos-api-logs-and-telemetry
    provides: Phase 5 route shell and API compare contract
  - phase: 06-safety-controllers-and-self-test
    provides: Safety/evidence boundaries for release claims
provides:
  - Pure firmware OTA accept/reject/status planner
  - Explicit OTAWWW REL-03 gap decision
  - Pure static/recovery route resolver with traversal rejection
  - Phase 7 route ownership manifest
affects: [firmware-http-api, static-files, ota, parity-api-compare, phase-07-release]
tech-stack:
  added: []
  patterns:
    - Pure data-in/data-out planner modules for firmware route adapters
    - Catalog-driven static route resolution before SPIFFS file access
key-files:
  created:
    - crates/bitaxe-api/src/update_plan.rs
    - crates/bitaxe-api/src/static_plan.rs
  modified:
    - crates/bitaxe-api/src/route_shell.rs
    - crates/bitaxe-api/src/lib.rs
    - crates/bitaxe-api/BUILD.bazel
key-decisions:
  - Preserve phase05_routes() for Phase 5 API compare while adding phase07_routes() for release/static/update ownership.
  - Keep OTAWWW fail-closed as an explicit REL-03 gap until interruption and recovery evidence exists.
  - Reject static path traversal in the pure resolver before any firmware file adapter can open a path.
patterns-established:
  - "Update planners call the shared private-network/origin gate with AP bypass disabled, then apply OTA-specific AP/APSTA rejection."
  - "Static serving uses an adapter-supplied catalog and returns typed decisions for static, recovery, redirect, rejection, and recovery fallback."
requirements-completed: [REL-01, REL-02, REL-03, REL-08]
duration: 11 min
completed: 2026-06-28
---

# Phase 07 Plan 01: Pure Update And Static Route Contracts Summary

Pure OTA, OTAWWW, static-file, recovery, and Phase 7 route ownership decisions are now unit-tested in `bitaxe-api` before firmware OTA, SPIFFS, partition writes, or reboot effects exist.

## Performance

- **Duration:** 11 min
- **Started:** 2026-06-28T15:09:48Z
- **Completed:** 2026-06-28T15:21:05Z
- **Tasks:** 3
- **Files modified:** 5

## Accomplishments

- Added `plan_update_request()` with private-network/origin denial, OTA-specific AP/APSTA rejection, firmware OTA accept metadata, validation error copy, and an explicit OTAWWW REL-03 gap.
- Added `resolve_static_request()` with `/` to `/index.html`, embedded recovery, SPIFFS-unavailable recovery fallback, `.gz` preference, cache headers, captive-portal redirect, and traversal rejection.
- Added `phase07_routes()` while preserving `phase05_routes()` for the Phase 5 API compare contract.

## Task Commits

1. **Task 1: Add pure OTA and OTAWWW update planner** - `7ecc7f1` (feat)
2. **Task 2: Add pure static and recovery route resolver** - `dfa6155` (feat)
3. **Task 3: Expose Phase 7 route ownership without breaking Phase 5 API compare** - `49d0996` (feat)

## Files Created/Modified

- `crates/bitaxe-api/src/update_plan.rs` - Pure update planner, status labels, firmware OTA accept plan, and OTAWWW gap decision tests.
- `crates/bitaxe-api/src/static_plan.rs` - Pure static/recovery resolver, catalog contract, traversal rejection, redirect, gzip, and recovery fallback tests.
- `crates/bitaxe-api/src/route_shell.rs` - `phase07_routes()` and Phase 7 route ownership variants while keeping `phase05_routes()` unchanged.
- `crates/bitaxe-api/src/lib.rs` - Public exports for update, static, and Phase 7 route contracts.
- `crates/bitaxe-api/BUILD.bazel` - Bazel source list entries for the new modules.

## Verification

| Command | Result |
| --- | --- |
| `cargo test -p bitaxe-api --all-features update_plan` | Passed |
| `cargo test -p bitaxe-api --all-features static_plan` | Passed |
| `cargo test -p bitaxe-api --all-features route_shell` | Passed |
| `git diff --check` | Passed |
| `cargo fmt --all` | Passed before each task commit |
| `cargo clippy --all-targets --all-features -- -D warnings` | Passed before each task commit |
| `cargo build --all-targets --all-features` | Passed before each task commit |
| `cargo test --all-features` | Passed before each task commit |

## Decisions Made

- `phase07_routes()` is a separate manifest instead of changing `phase05_routes()`, because Phase 5 parity tooling still depends on the old route list and `SafeUnsupportedUpdate` meaning.
- OTAWWW returns a typed `OtaWwwGapDecision` with owner, release impact, and follow-up instead of silently sharing the generic unsupported route behavior.
- Static route resolution stays catalog-driven and side-effect-free, leaving SPIFFS mounts and file opens to later firmware adapters.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] Fixed clippy `manual_contains` warning**

- **Found during:** Task 2 (Add pure static and recovery route resolver)
- **Issue:** The first static catalog lookup used `iter().any()` and failed the required clippy gate with `manual_contains`.
- **Fix:** Switched catalog membership to `self.files.contains(&path)`.
- **Files modified:** `crates/bitaxe-api/src/static_plan.rs`
- **Verification:** `cargo clippy --all-targets --all-features -- -D warnings` passed, followed by build and tests.
- **Committed in:** `dfa6155`

### Process Adjustments

- TDD RED failures were run for all three tasks but not committed separately because `AGENTS.md` requires the full Rust pre-commit sequence to pass before every commit.
- A route manifest simplification pass added a local `axeos_route!` macro after the duplicated Phase 7 manifest pushed `route_shell.rs` above the advisory file-size trigger.
- `.planning/STATE.md` was updated in the worktree but left out of the final metadata commit because it already contained orchestrator-owned uncommitted Phase 7 setup edits before this executor started.

***

**Total deviations:** 1 auto-fixed (1 blocking)
**Impact on plan:** The auto-fix and simplification preserved the planned behavior and did not add runtime side effects or expand scope.

## Issues Encountered

None unresolved.

## Known Stubs

None. Empty catalogs in tests are intentional inputs for recovery and missing-file cases, not UI placeholders or unwired data sources.

## Threat Flags

None. The new code introduces pure decision surfaces only; no new runtime network endpoint, firmware file access, partition write, authentication path, or schema trust boundary was added.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

Ready for `07-02-PLAN.md`. Firmware, package, and parity plans can now import typed update/static decisions without reaching into ESP-IDF OTA, SPIFFS, partition, or reboot effects.

## Self-Check: PASSED

- Found summary file: `.planning/phases/07-ota-filesystem-and-release-packaging/07-01-SUMMARY.md`
- Found created files: `crates/bitaxe-api/src/update_plan.rs`, `crates/bitaxe-api/src/static_plan.rs`
- Found modified files: `crates/bitaxe-api/src/route_shell.rs`, `crates/bitaxe-api/src/lib.rs`, `crates/bitaxe-api/BUILD.bazel`
- Found task commits: `7ecc7f1`, `dfa6155`, `49d0996`
- Focused stub scan over touched source/build files returned no matches.

***
*Phase: 07-ota-filesystem-and-release-packaging*
*Completed: 2026-06-28*
