---
phase: 34-provenance-runtime-health-and-coherent-operator-snapshot
plan: 02
subsystem: runtime-observability
tags: [operator-snapshot, boot-session, correlation, websocket, retained-logs, evidence]
requires:
  - phase: 34-01-canonical-build-identity
    provides: Canonical build provenance and shared API/runtime identity projections
provides:
  - Strict opaque boot-session and nonzero monotonic operator-snapshot revision domain types
  - One firmware capture authority that assigns and retains a correlation identity before projection
  - Opt-in OBS-06 evidence validation across API, WebSocket, redacted documents, and retained logs
affects: [34-03-platform-identity, 34-04-passive-health, phase-35-correlated-evidence]
tech-stack:
  added: []
  patterns: [typed capture identity, checked boot-local sequencing, retained correlation marker, opt-in evidence admission]
key-files:
  created:
    - crates/bitaxe-api/src/operator_snapshot.rs
    - tools/parity/src/operator_snapshot_evidence.rs
  modified:
    - crates/bitaxe-api/src/runtime_projection.rs
    - crates/bitaxe-api/src/wire.rs
    - firmware/bitaxe/src/boot_evidence.rs
    - firmware/bitaxe/src/runtime_snapshot.rs
    - tools/parity/src/operator_evidence.rs
key-decisions:
  - "The existing hardware-RNG boot observer supplies the only session; firmware owns one checked revision sequence for all public captures."
  - "A retained marker is emitted only after one complete ApiSnapshot is assembled, and all projections copy the attached identity."
  - "Historical operator-evidence profiles remain unchanged; OBS-06 coherence is enabled explicitly with a fail-closed validation flag."
patterns-established:
  - "Coherent capture: reserve identity -> collect immutable facts -> retain exact marker -> project the same snapshot."
  - "Evidence correlation: strict typed JSON fields + exact retained marker + one-session chronology + matching redacted projection."
requirements-completed: [OBS-06]
generated_by: gsd-execute-plan
lifecycle_mode: interactive
phase_lifecycle_id: 34-2026-07-15T03-26-15
generated_at: 2026-07-15T15:42:00Z
duration: 30min
completed: 2026-07-15
---

# Phase 34 Plan 02: Coherent Operator Snapshot Identity Summary

**System-info, live WebSocket, retained logs, and opt-in evidence validation now share one typed per-boot session and checked monotonic snapshot revision.**

## Performance

- **Duration:** 30 min
- **Started:** 2026-07-15T10:12:25-05:00
- **Completed:** 2026-07-15T10:42:00-05:00
- **Tasks:** 3
- **Implementation commits:** 3

## Accomplishments

- Added strict dependency-free types for 32-character lowercase boot sessions, nonzero revisions, capture identities, checked sequencing, and exact retained markers.
- Propagated one identity from `ApiSnapshot` into system-info and live telemetry without projection-time generation, host inference, clocks, or fixture fallback.
- Reused the existing hardware-RNG boot nonce and added the sole boot-lifetime firmware revision owner, with unique concurrent revisions and one retained marker after each completed capture.
- Added a pure operator-evidence validator for exact JSON fields, redacted document fields, retained-marker membership, one-session chronology, duplicate/partial/malformed input, fixture identity, and host-checkout substitution.
- Preserved Phase 23/25/27/28 behavior behind an explicit `--require-operator-snapshot-coherence` opt-in.

## Task Commits

1. **Task 1: Define the typed operator-snapshot identity and additive wire contract** - `4f54da9`
2. **Task 2: Assign one runtime pair and retain its correlation marker** - `11404bf`
3. **Task 3: Validate coherent evidence projections without hardware** - `0e718f5`

## Files Created/Modified

- `crates/bitaxe-api/src/operator_snapshot.rs` - Strict session/revision identity, checked allocator, exact marker, and concurrency coverage.
- `crates/bitaxe-api/src/snapshot.rs`, `runtime_projection.rs`, and `wire.rs` - One captured identity copied to the exact public JSON fields.
- `firmware/bitaxe/src/boot_evidence.rs` and `runtime_snapshot.rs` - Existing boot nonce adapter, sole revision sequence, complete-capture assembly, and retained marker.
- `tools/parity/src/operator_snapshot_evidence.rs` - Typed JSON/marker/document parsing with session, chronology, membership, duplicate, and substitution checks.
- `tools/parity/src/operator_evidence.rs` and `main.rs` - Explicit OBS-06 opt-in while keeping historical validation unchanged.

## Decisions Made

- Kept boot-session semantics opaque: it is a random per-boot identifier, not a reset ordinal, timestamp, MAC, build label, or source commit.
- Reserved revisions before fact collection, but retained the marker only after the snapshot was fully assembled; abandoned partial captures leave no correlation breadcrumb.
- Allowed equal revisions across API and WebSocket only when the complete typed identity is identical, representing two projections of one capture.
- Rejected the all-zero fixture session and commit-shaped session substitutions in evidence rather than admitting deterministic host values as runtime truth.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Updated the Phase 33 single-materialization source guard for the coherent capture boundary**

- **Found during:** Task 2 mandatory full Rust verification.
- **Issue:** The historical guard required the old inline call text `project_system_info(collect_api_snapshot(), &projection)`, which no longer represented the required identity-first, one-state-read capture flow.
- **Fix:** Kept the original invariant but asserted the new ordered boundary: identity reservation, one runtime-state projection read, one completed snapshot materialization, then system-info projection.
- **Verification:** The Phase 33 guard and full Rust/Bazel suites pass.
- **Committed in:** `11404bf`

**Total deviations:** 1 auto-fixed regression-guard update.
**Impact on plan:** No runtime behavior or scope expanded; the guard now describes the same anti-duplication property through the Plan 02 capture authority.

## Issues Encountered

- The first Task 2 full test run exposed the stale Phase 33 source-text assertion; the invariant-preserving guard update resolved it.
- Clippy rejected one test-only conditional expression as obscured control flow; it was simplified to an explicit `if` before the successful mandatory sequence.

## Verification

- The mandatory Rust sequence passed in order: `cargo fmt --all`, all-target/all-feature clippy with warnings denied, all-target/all-feature build, and all-feature tests.
- Focused API identity/projection, concurrency, runtime source-guard, operator-snapshot evidence, and historical operator-evidence tests passed.
- `bazel test //...` passed all 57 test targets.
- `just build`, `just package`, `just verify-reference`, and `git diff --check` passed.
- Full implementation diff review found only the declared Plan 02 surfaces plus the invariant-preserving Phase 33 guard update; no secrets, network identities, device paths, or hardware commands were added.

## User Setup Required

None. No device detection, board access, credentials, network discovery, flash, reset, monitor, direct UART, pin manipulation, OTA, mining, or hardware evidence command was used.

## Next Phase Readiness

- Plan 34-03 can attach remaining read-only platform identity facts to the established coherent snapshot boundary.
- Plan 34-04 can project passive supervisor health through the same capture identity.
- Phase 35 remains the sole owner of correlated current-package hardware qualification.

## Self-Check: PASSED

- All three implementation commits exist and all plan-wide software verification gates are green.
- OBS-06 is implemented without changing historical evidence acceptance unless explicitly enabled.
- The reference tree is clean, no hardware or credentials were accessed, and no push occurred.

***

*Phase: 34-provenance-runtime-health-and-coherent-operator-snapshot*
*Completed: 2026-07-15*
