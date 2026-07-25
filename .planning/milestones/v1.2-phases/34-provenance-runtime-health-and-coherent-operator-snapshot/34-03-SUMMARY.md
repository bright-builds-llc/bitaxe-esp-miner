---
phase: 34-provenance-runtime-health-and-coherent-operator-snapshot
plan: 03
subsystem: runtime-observability
tags: [platform-identity, esp-idf, availability, operator-snapshot, source-guard]
requires:
  - phase: 34-02-coherent-operator-snapshot
    provides: One boot-session and monotonic revision shared by all projections of a completed capture
provides:
  - Typed per-fact availability with closed Ultra 205, BM1366, and ESP-IDF reset vocabularies
  - One read-only firmware adapter for embedded and current running-platform identity
  - Coherent system-info and retained compatibility projections sourced from one captured platform candidate
  - Source guards against host substitution, fixtures, synthetic claims, and active hardware effects
affects: [34-04-passive-health, phase-35-correlated-evidence]
tech-stack:
  added: []
  patterns: [typed availability, fail-closed compatibility projection, single read-only platform adapter, coherent candidate capture]
key-files:
  created:
    - crates/bitaxe-api/src/platform_identity.rs
    - firmware/bitaxe/src/platform_identity.rs
  modified:
    - crates/bitaxe-api/src/snapshot.rs
    - crates/bitaxe-api/src/wire.rs
    - firmware/bitaxe/src/runtime_snapshot.rs
    - firmware/bitaxe/src/main.rs
    - tools/parity/src/phase34_source_guard.rs
key-decisions:
  - "Every platform fact carries its own available or unavailable state; zero and compatibility defaults never authenticate proof."
  - "The embedded static release asset and current ESP-IDF reads are the only production sources for running-platform identity."
  - "Existing compatibility scalars are projected conservatively from the same typed candidate captured under one Plan 02 session and revision."
patterns-established:
  - "Platform truth: capture one read-only candidate -> retain typed per-field proof -> project all public compatibility views from that candidate."
  - "Independent failure: one unavailable platform fact does not discard or downgrade unrelated proved facts."
requirements-completed: [SYS-03, SYS-04, SYS-05]
generated_by: gsd-execute-plan
lifecycle_mode: interactive
phase_lifecycle_id: 34-2026-07-15T03-26-15
generated_at: 2026-07-15T16:04:37Z
duration: 18min
completed: 2026-07-15
---

# Phase 34 Plan 03: Truthful Running-Platform Identity Summary

**System-info and retained startup projections now expose independently provable ESP-IDF, static-asset, board, ASIC, partition, reset, uptime, heap, and PSRAM facts from one coherent running-device snapshot.**

## Performance

- **Duration:** 18 min
- **Started:** 2026-07-15T10:47:07-05:00
- **Completed:** 2026-07-15T11:04:37-05:00
- **Tasks:** 2
- **Implementation commits:** 2
- **Files:** 12

## Accomplishments

- Added a generic tagged availability contract, closed board/ASIC vocabularies, and a closed reset-reason decoder that preserves software CPU reset and fails unknown codes closed.
- Added all required platform facts to `ApiSnapshot` and the additive nested system-info wire object while leaving upstream-compatible fields present.
- Embedded the shipped static release asset and collected current ESP-IDF version, running partition, reset reason, uptime, internal heap facts, and PSRAM availability through one read-only firmware adapter.
- Captured the platform candidate exactly once inside the Plan 02 completed-snapshot transaction and derived compatibility fields and retained startup markers from that same candidate.
- Guarded production identity paths against fixtures, host Git/process reads, synthetic placeholders, alternate hardware claims, request-time mutation, and reset/watchdog/OTA/UART/GPIO/credential capabilities.

## Task Commits

1. **Task 1: Define closed platform identity and availability contracts** - `fab2378`
2. **Task 2: Collect running ESP-IDF facts once and project them coherently** - `a735a7c`

## Files Created/Modified

- `crates/bitaxe-api/src/platform_identity.rs` - Per-field availability, closed platform/reset vocabularies, fixture-safe defaults, and focused proof/unavailability tests.
- `crates/bitaxe-api/src/snapshot.rs`, `wire.rs`, `lib.rs`, API Bazel metadata, and the system-info fixture - Complete additive platform identity carried through the coherent API snapshot.
- `firmware/bitaxe/src/platform_identity.rs` - Sole read-only embedded/ESP-IDF adapter for current running-platform facts.
- `firmware/bitaxe/src/runtime_snapshot.rs` and `main.rs` - One candidate capture and conservative compatibility/retained-log projection.
- `firmware/bitaxe/BUILD.bazel` and `runtime_uptime.rs` - Firmware source ownership plus removal of a superseded duplicate uptime projection helper.
- `tools/parity/src/phase34_source_guard.rs` - Static enforcement of one capture and prohibited-source/effect boundaries.

## Decisions Made

- Kept availability independent per field so a null partition pointer or unknown reset code cannot erase a proved ESP-IDF, board, ASIC, heap, or static-asset fact.
- Treated host fixtures as explicitly unauthenticated for every platform fact, including otherwise fixed board and ASIC values.
- Read the AxeOS/static identity from the exact asset embedded in the firmware binary instead of the checkout, package directory, or host process.
- Preserved old scalar fields only as conservative compatibility projections; typed availability remains authoritative and zero never changes unavailable to available.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] Registered the new firmware adapter and removed its superseded uptime helper**

- **Found during:** Task 2 firmware build verification.
- **Issue:** The Phase 34 firmware source filegroup needed the new adapter for Bazel tracking, and moving the sole uptime read into that adapter left the older seconds helper unused.
- **Fix:** Added `platform_identity.rs` to the existing Phase 34 filegroup and removed only the now-unreferenced `runtime_uptime::seconds` function.
- **Verification:** The real ESP32-S3 `just build` succeeds with no new warning; the remaining 14 dead-code warnings predate Plan 03.
- **Committed in:** `a735a7c`

**2. [Rule 3 - Blocking] Repaired repository-specific GSD metadata after generic updater mismatch**

- **Found during:** Plan completion metadata update.
- **Issue:** The GSD roadmap updater interpreted this repository's four-column `Phase / Name / Requirements / Status` table as `Phase / Plans / Status / Date`, and its decision appender did not recognize phase-qualified decision headings.
- **Fix:** Preserved the updater's valid plan/requirement counts, restored the repository table schema, and added the Plan 03 decisions under the established phase-qualified state heading.
- **Verification:** ROADMAP reports 3/4 Phase 34 plans and 16/27 completed requirements; STATE points to Plan 04 and records the Plan 03 decisions.
- **Committed in:** Plan metadata commit.

**Total deviations:** 2 auto-fixed integration/metadata cleanups.
**Impact on plan:** No runtime scope or behavior expansion; the adapter is tracked by Bazel, uptime has one production read path, and GSD artifacts retain their repository-defined schemas.

## Issues Encountered

- All contract, source-guard, Rust, Bazel, and firmware-build verification passed without a failing test.
- The generic GSD metadata helpers required the narrow repository-schema repair described above; no implementation artifact was affected.

## Verification

- The mandatory Rust sequence passed in order before each implementation commit: `cargo fmt --all`, all-target/all-feature clippy with warnings denied, all-target/all-feature build, and all-feature tests.
- Focused `platform_identity`, `operator_snapshot`, `wire`, and Phase 34 source-guard tests passed.
- `bazel test //crates/bitaxe-api:tests //tools/parity:tests` passed both affected test targets.
- `just build` produced the ESP32-S3 firmware ELF using the pinned ESP-IDF `v5.5.4` toolchain.
- `git diff --check` and the full two-commit implementation diff review passed with no unintended production source, host substitution, secret, network, device, or hardware-action changes.

## User Setup Required

None. No device detection, board access, credentials, network discovery, flash, reset, monitor, direct UART, pin manipulation, OTA, mining, or hardware evidence command was used.

## Next Phase Readiness

- Plan 34-04 can attach passive supervisor health to the same coherent capture identity.
- Phase 35 remains the sole owner of correlated current-package hardware qualification.

## Self-Check: PASSED

- Both implementation commits exist and every required software verification gate is green.
- SYS-03, SYS-04, and SYS-05 are implemented with explicit per-field unavailability and no fixture or host substitution.
- The orchestrator-owned todo file remains untouched by this plan, no hardware or credentials were accessed, and no push occurred.

***

*Phase: 34-provenance-runtime-health-and-coherent-operator-snapshot*
*Completed: 2026-07-15*
