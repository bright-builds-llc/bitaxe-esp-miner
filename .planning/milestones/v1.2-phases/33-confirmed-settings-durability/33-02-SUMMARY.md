---
phase: 33-confirmed-settings-durability
plan: 02
subsystem: firmware
tags: [settings, nvs, hostname, esp-idf, confirmation]
requires:
  - phase: 33-confirmed-settings-durability
    provides: Closed hostname authority and the serialized confirmation protocol from Plan 01
provides:
  - Serialized ESP-IDF NVS hostname transactions through atomic confirmed publication
  - Exact v1.2 route authority with effect-free compatibility and generic error paths
  - Host and Bazel guards against optimistic overlays, broad writes, and pre-response effects
affects: [33-03-durability-evidence, phase-34-operator-snapshot]
tech-stack:
  added: []
  patterns: [independent read-only confirmation, atomic confirmed snapshot, source boundary guard]
key-files:
  created:
    - tools/parity/src/phase33_source_guard.rs
  modified:
    - firmware/bitaxe/BUILD.bazel
    - firmware/bitaxe/src/http_api.rs
    - firmware/bitaxe/src/runtime_snapshot.rs
    - firmware/bitaxe/src/settings_adapter.rs
    - tools/parity/BUILD.bazel
    - tools/parity/src/main.rs
key-decisions:
  - "Writable NVS opens only after exact hostname authority and while the process-lifetime transaction mutex is held."
  - "Authoritative reload is strict and non-publishing; only the reconciled candidate can atomically replace confirmed runtime truth."
  - "Compatibility requests and invalid inputs never construct the adapter, and the only post-response live effect is hostname application."
patterns-established:
  - "Confirmed adapter: lock, writable open, write, commit, independent read-only reload, typed reconcile, atomic publish."
  - "Immediate projection: runtime system info reads only the atomically published confirmed settings snapshot."
requirements-completed: [CFG-09, CFG-10, CFG-11, CFG-13]
generated_by: gsd-execute-plan
lifecycle_mode: yolo
phase_lifecycle_id: 33-2026-07-14T01-50-49
generated_at: 2026-07-14T02:48:33Z
duration: 9min
completed: 2026-07-14
---

# Phase 33 Plan 02: Firmware Confirmation Integration Summary

**ESP-IDF hostname PATCH success now follows strict NVS reload and atomic confirmed publication, with immediate system-info projection and no optimistic overlay.**

## Performance

- **Duration:** 9 min
- **Started:** 2026-07-14T02:39:00Z
- **Completed:** 2026-07-14T02:48:33Z
- **Tasks:** 2
- **Files modified:** 7

## Accomplishments

- Holds one process-lifetime mutex from writable NVS open through exact reconciliation and confirmed snapshot publication.
- Preserves broad AxeOS response compatibility while restricting NVS construction and live effects to exact validated hostname authority.
- Guards route ordering, strict candidate loading, confirmed immediate readback, generic failures, and hostname-only post-response effects through Cargo and Bazel.

## Task Commits

Each task was committed atomically:

1. **Task 1: Build a serialized fallible NVS confirmation adapter** - `6e53c8a` (feat)
2. **Task 2: Route v1.2 PATCH and immediate readback through confirmed truth** - `441b90e` (test)

## Files Created/Modified

- `firmware/bitaxe/src/settings_adapter.rs` - Owns transaction serialization, strict independent reload, and atomic candidate publication.
- `firmware/bitaxe/src/http_api.rs` - Routes closed hostname authority and preserves inert compatibility and generic error behavior.
- `firmware/bitaxe/src/runtime_snapshot.rs` - Names and projects the confirmed settings snapshot into immediate system info.
- `tools/parity/src/phase33_source_guard.rs` - Protects the firmware boundary from broad writes, overlays, and effect-order regressions.
- `firmware/bitaxe/BUILD.bazel` and `tools/parity/BUILD.bazel` - Expose the guarded firmware sources to Bazel tests.
- `tools/parity/src/main.rs` - Registers the Phase 33 source guard in host tests.

## Decisions Made

- Kept startup loading best-effort but made PATCH confirmation independently strict and fallible, so startup compatibility cannot authenticate a PATCH success.
- Retained the last confirmed snapshot until a fully loaded, typed exact match is ready for one atomic replacement.
- Removed legacy mining/settings refresh calls from hostname effects because compatibility-only fields have no Phase 33 authority and hostname is the only eligible live effect.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] Adapted the route during Task 1 so the narrowed Plan 01 trait could compile**

- **Found during:** Task 1 (serialized fallible NVS confirmation adapter)
- **Issue:** Plan 01 removed broad adapter methods and plan constructors, so the previous broad firmware route could not compile while Task 1 was verified with `just build`.
- **Fix:** Moved the exact-authority route adaptation into the first commit, then strengthened its behavior guards in Task 2.
- **Files modified:** `firmware/bitaxe/src/http_api.rs`
- **Verification:** Targeted Cargo tests, Phase 33 Cargo/Bazel source guards, and `just build` passed.
- **Committed in:** `6e53c8a`

**2. [Rule 3 - Blocking] Added a Bazel firmware source filegroup for the host guard**

- **Found during:** Task 1 (serialized fallible NVS confirmation adapter)
- **Issue:** Bazel sandboxing cannot resolve cross-package `include_str!` firmware inputs without declared data ownership.
- **Fix:** Added the narrow `phase33_settings_sources` filegroup and wired it as parity test compile data.
- **Files modified:** `firmware/bitaxe/BUILD.bazel`, `tools/parity/BUILD.bazel`
- **Verification:** `bazel test //tools/parity:tests --test_filter=phase33_settings_source_guard` passed.
- **Committed in:** `6e53c8a`

**Total deviations:** 2 auto-fixed blocking issues.
**Impact on plan:** Both changes were necessary to compile and verify the planned boundary; no authority, hardware, revision, or evidence scope was expanded.

## Issues Encountered

None.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

- Ready for Plan 33-03 to build the detector-gated simulation/evidence wrapper and attempt one approved normal-reboot durability proof.
- Software and firmware build evidence is green; real-device reboot durability remains intentionally unclaimed until Plan 03.

## Self-Check: PASSED

- `33-02-SUMMARY.md` exists and records both task commits.
- Targeted v1.2 authority/settings tests, Phase 33 Cargo/Bazel source guards, and canonical `just build` passed.
- The exact ordered Rust format, lint, build, and test gate passed before each task commit.
- `git diff --check` passed and the source guards reject optimistic overlays, broad route writes, and mining refresh effects.

***

*Phase: 33-confirmed-settings-durability*
*Completed: 2026-07-14*
