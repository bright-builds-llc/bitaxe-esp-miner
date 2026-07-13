---
phase: 26-telemetry-and-parity-closure
plan: 01
subsystem: telemetry
tags: [rust, stratum, telemetry, projection, parity]
requires:
  - phase: 24-bm1366-production-work-path
    provides: PoolSessionGeneration and SubmitIntent boundaries for production-visible share context.
  - phase: 25-live-stratum-runtime-and-safe-stop
    provides: SubmitClassification and safe-stop semantics for live Stratum outcomes.
provides:
  - Shared Phase 26 RuntimeTelemetryProjection for mining telemetry state.
  - RuntimeProjectionSampleMarker drain contract for bounded statistics samples.
  - Current-generation SubmitClassification gate for accepted and rejected counters.
affects: [api-projection, websocket-telemetry, statistics, scoreboard, parity-evidence]
tech-stack:
  added: []
  patterns: [functional-core-event-projection, sequence-guarded-runtime-fold, current-generation-counter-gate]
key-files:
  created:
    - crates/bitaxe-stratum/src/v1/telemetry_projection.rs
  modified:
    - crates/bitaxe-stratum/src/v1.rs
    - crates/bitaxe-stratum/BUILD.bazel
key-decisions:
  - "Keep Phase 26 telemetry projection as a pure stratum v1 core module exported through Rust and Bazel."
  - "Advance accepted and rejected counters only for current-generation SubmitClassification Accepted or Rejected events."
  - "Drain statistics sample markers at most once per runtime event boundary to prevent request-time sample fabrication."
patterns-established:
  - "Runtime telemetry event folds use monotonic RuntimeTelemetrySequence guards before mutating projected state."
  - "Safe-stop folds advance the projection generation and leave mining state disconnected, blocked, and safe-blocked."
requirements-completed: [API-11, API-13]
generated_by: gsd-execute-plan
lifecycle_mode: yolo
phase_lifecycle_id: 26-2026-07-05T03-48-38
generated_at: 2026-07-05T04:14:14Z
duration: 4min
completed: 2026-07-05
---

# Phase 26 Plan 01: Telemetry Projection Contract Summary

**Typed runtime-event projection for Phase 26 mining state, bounded statistics sample markers, and current-generation share counter gating**

## Performance

- **Duration:** 4min
- **Started:** 2026-07-05T04:10:46Z
- **Completed:** 2026-07-05T04:14:14Z
- **Tasks:** 2
- **Files modified:** 3

## Accomplishments

- Added `RuntimeTelemetryProjection`, `RuntimeTelemetryEvent`, `RuntimeTelemetrySequence`, `RuntimeProjectionSampleMarker`, `RuntimeProjectionSampleSource`, and `ProjectionShareOutcome` in `crates/bitaxe-stratum`.
- Exported the projection through `crates/bitaxe-stratum/src/v1.rs` and registered it in `crates/bitaxe-stratum/BUILD.bazel`.
- Proved lifecycle, hashrate, blocked, bounded sample marker, safe-stop, stale-sequence, stale-generation, accepted-counter, rejected-counter, and redaction-safe rendering invariants with focused unit tests.

## Task Commits

Each TDD task was committed atomically:

1. **Task 26-01-01 RED:** `eac81be` test(26-01): add failing tests for telemetry projection contracts
2. **Task 26-01-01 GREEN:** `b6d00cd` feat(26-01): implement telemetry projection contracts
3. **Task 26-01-02 RED:** `5b9337b` test(26-01): add failing tests for submit counter gate
4. **Task 26-01-02 GREEN:** `b1e49dd` feat(26-01): gate projection share counters
5. **Task 26-01-02 REFACTOR:** `668f8c6` refactor(26-01): format telemetry projection tests

## Files Created/Modified

- `crates/bitaxe-stratum/src/v1/telemetry_projection.rs` - Pure Phase 26 event projection, sample marker drain contract, counter gate, safe-stop reset, and unit tests.
- `crates/bitaxe-stratum/src/v1.rs` - Public module export for the new projection.
- `crates/bitaxe-stratum/BUILD.bazel` - Bazel source registration for the projection module.

## Decisions Made

- Kept statistics source labeling internal through `RuntimeProjectionSampleMarker` and `RuntimeProjectionSampleSource`; no public API field was added in this plan.
- Treated `SubmitClassification::Accepted` with no explicit difficulty as an accepted parsed response with `ShareDifficulty::new(0.0)`, preserving counter legitimacy without inventing best-difficulty evidence.
- Advanced the projection generation on `SafeStopped` so stale post-stop submit classifications cannot reactivate mining or advance counters.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Reverted premature milestone requirement completion**
- **Found during:** Final metadata review
- **Issue:** The generic GSD `requirements mark-complete` command marked `API-11` and `API-13` fully complete, but Plan 26-01 only delivers the shared projection slice; API route wiring, WebSocket projection, evidence, and checklist closure remain in later Phase 26 plans.
- **Fix:** Restored `API-11` and `API-13` to pending in `.planning/REQUIREMENTS.md` while keeping this plan summary tied to the requirement IDs from plan frontmatter.
- **Files modified:** `.planning/REQUIREMENTS.md`, `.planning/phases/26-telemetry-and-parity-closure/26-01-SUMMARY.md`
- **Verification:** Final metadata diff shows `API-11` and `API-13` remain pending.
- **Committed in:** final metadata commit

**Total deviations:** 1 auto-fixed (Rule 1 bug)
**Impact on plan:** Prevented requirement overclaim; no code scope change.

## Issues Encountered

- Package-wide `cargo fmt --package bitaxe-stratum --check` reported pre-existing formatting diffs in unrelated stratum files. The changed file was verified with `rustfmt --edition 2021 --check crates/bitaxe-stratum/src/v1/telemetry_projection.rs` instead, and unrelated files were left untouched.

## Known Stubs

None found.

## Threat Flags

None - the plan added a pure in-memory projection module and did not introduce new network endpoints, auth paths, file access, or schema boundaries.

## Verification

- `bazel test //crates/bitaxe-stratum:tests` passed.
- `rustfmt --edition 2021 --check crates/bitaxe-stratum/src/v1/telemetry_projection.rs` passed.
- `node "$HOME/.cursor/get-shit-done/bin/gsd-tools.cjs" verify lifecycle 26 --expect-id 26-2026-07-05T03-48-38 --expect-mode yolo --require-plans` passed.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

Plan 26-02 can consume `RuntimeTelemetryProjection` and drain `RuntimeProjectionSampleMarker` exactly once per runtime boundary when deriving API, statistics, scoreboard, and WebSocket views.

## Self-Check: PASSED

- Found summary file at `.planning/phases/26-telemetry-and-parity-closure/26-01-SUMMARY.md`.
- Found task commits `eac81be`, `b6d00cd`, `5b9337b`, `b1e49dd`, and `668f8c6`.

*Phase: 26-telemetry-and-parity-closure*
*Completed: 2026-07-05*
