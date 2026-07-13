---
phase: 31-operator-claim-and-telemetry-contract
plan: "03"
subsystem: api-parity
tags: [rust, settings, hostname, parity, admission]
requires:
  - phase: 31-01
    provides: Producer-owned observation truth contract
  - phase: 31-02
    provides: Observation-aware API projections and immutable consumer reads
provides:
  - Effect-free hostname-only v1.2 settings authority
  - Closed, row-scoped Phase 31 claim admission
affects: [phase-33-settings-persistence, phase-35-parity-claims]
tech-stack:
  added: []
  patterns: [closed capability enum, typed fail-closed admission]
key-files:
  created:
    - crates/bitaxe-api/src/v12_settings.rs
    - tools/parity/src/v12_admission.rs
  modified:
    - crates/bitaxe-api/src/settings.rs
    - crates/bitaxe-api/src/lib.rs
    - crates/bitaxe-api/BUILD.bazel
    - tools/parity/src/main.rs
    - tools/parity/BUILD.bazel
key-decisions:
  - "Only an exact validated hostname field set can construct V12SettingsChange; compatibility parsing never grants broader authority."
  - "Phase 31 claim eligibility is closed over two typed claims and bound to OBS-01 or CFG-08 evidence respectively."
requirements-completed: [CFG-08]
generated_by: gsd-execute-plan
lifecycle_mode: yolo
phase_lifecycle_id: 31-2026-07-13T19-47-51
generated_at: 2026-07-13T21:26:47Z
duration: 14 min
completed: 2026-07-13
---

# Phase 31 Plan 03: Settings Authority and Claim Admission Summary

**Hostname is now the sole constructible v1.2 settings capability, while exact Phase 31 claims are typed, evidence-gated, row-scoped, and fail closed.**

## Performance

- **Duration:** 14 min
- **Started:** 2026-07-13T21:12:53Z
- **Completed:** 2026-07-13T21:26:47Z
- **Tasks:** 2
- **Files modified:** 7

## Accomplishments

- Added a redaction-safe hostname domain type and effect-free v1.2 settings classifier that authorizes only an exact validated hostname payload.
- Preserved existing public compatibility responses while excluding every broader, mixed, credential, hardware-control, mining, self-test, display, unknown, and empty payload from v1.2 authority.
- Added a closed parity admission model in which OBS-01 can support only observation truth and CFG-08 can support only the hostname allowlist.
- Kept every excluded category typed and ineligible, prevented arbitrary strings or schema growth from constructing claims, and preserved all checklist statuses.

## Task Commits

Each task was committed atomically:

1. **Task 1: Add the effect-free hostname-only v1.2 settings capability** - `fca14e8` (feat)
2. **Task 2: Add typed exact-claim admission and final Phase 31 regression gates** - `0d7c3d4` (feat)

**Plan metadata:** This commit

## Files Created/Modified

- `crates/bitaxe-api/src/v12_settings.rs` - Closed hostname-only authority and compatibility-only exclusion decisions.
- `crates/bitaxe-api/src/settings.rs` - Shared pure parser details with the v1.2 classifier without granting persistence authority.
- `crates/bitaxe-api/src/lib.rs` - Exposes the v1.2 settings module.
- `crates/bitaxe-api/BUILD.bazel` - Includes the new API module.
- `tools/parity/src/v12_admission.rs` - Typed Phase 31 claim admission, exclusions, and regression tests.
- `tools/parity/src/main.rs` - Runs the closed Phase 31 admission contract during parity validation.
- `tools/parity/BUILD.bazel` - Includes admission code and declares checklist compile data.

## Decisions Made

- The compatibility parser remains broad for response stability, but only `V12SettingsChange::Hostname` crosses the v1.2 authority boundary.
- Eligible parity claims are enum values with explicit requirement bindings; strings, broad production language, and one claim's evidence cannot authenticate another claim.
- Live persistence, commit/reload behavior, and firmware handler integration remain deferred to Phase 33.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] Declared checklist compile data for the parity test target**

- **Found during:** Task 2 focused Bazel verification
- **Issue:** `include_str!` correctly made checklist status preservation compile-time visible, but Bazel initially rejected the undeclared input.
- **Fix:** Added `//:docs/parity/checklist.md` to `tools/parity` compile data.
- **Files modified:** `tools/parity/BUILD.bazel`
- **Verification:** `bazel test //tools/parity:tests` passed, followed by the complete repository verification gate.
- **Committed in:** `0d7c3d4`

***

**Total deviations:** 1 auto-fixed (1 blocking)
**Impact on plan:** The declaration was required for hermetic Bazel compilation and did not broaden scope or behavior.

## Issues Encountered

None beyond the resolved Bazel compile-data declaration.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

- Phase 31's three plans are implemented and ready for phase verification.
- Phase 33 can consume the hostname capability when it adds persistence, commit/reload, durability, and live handler integration.
- Hardware, credentials, mining, OTA, archived Phase 28.1.1 work, other boards, and broad checklist promotion remain explicitly unauthorized.

## Self-Check: PASSED

- Created files exist and are included in Cargo/Bazel coverage.
- Task commits `fca14e8` and `0d7c3d4` exist in history.
- Mandatory Rust gates, focused Cargo/Bazel tests, repository tests, parity validation, reference cleanliness, lifecycle validation, traceability checks, and diff checks passed.
