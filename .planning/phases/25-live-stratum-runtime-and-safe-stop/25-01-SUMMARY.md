---
phase: 25-live-stratum-runtime-and-safe-stop
plan: 01
subsystem: stratum
tags: [rust, stratum-v1, live-runtime, fake-pool, safe-stop, redaction, bazel]
requires:
  - phase: 24-bm1366-production-work-path
    provides: BM1366 production work registry, session generation, and SubmitIntent correlation boundary
provides:
  - Pure live Stratum v1 runtime state machine for subscribe, authorize, difficulty, notify, reconnect, invalidation, and safe stop
  - SubmitIntent-tied submit response classifier with redaction-safe accepted, rejected, blocked, timeout, reconnect, malformed, stopped, and no-observed-share labels
  - Deterministic fake-pool coverage that drives the new runtime/classifier without socket I/O or live STR-09 claims
affects: [phase-25-live-stratum-runtime, phase-26-telemetry-and-parity, bitaxe-stratum, parity-evidence]
tech-stack:
  added: []
  patterns:
    - pure live-runtime state machine
    - SubmitIntent-gated response classification
    - deterministic fake-pool transcript runner
key-files:
  created:
    - crates/bitaxe-stratum/src/v1/live_runtime.rs
    - crates/bitaxe-stratum/src/v1/submit_response.rs
    - .planning/phases/25-live-stratum-runtime-and-safe-stop/25-01-SUMMARY.md
  modified:
    - crates/bitaxe-stratum/src/v1/fake_pool.rs
    - crates/bitaxe-stratum/src/v1.rs
    - crates/bitaxe-stratum/BUILD.bazel
key-decisions:
  - "Kept live Stratum lifecycle, submit response classification, and fake-pool behavior in crates/bitaxe-stratum with no socket, ESP-IDF, or credential-file ownership."
  - "Classified accepted/rejected shares only from a SubmitIntent plus matching request id and typed StratumResponse."
  - "Preserved fake-pool coverage as deterministic STR-11 evidence only; it does not promote live STR-09 accepted/rejected proof."
patterns-established:
  - "LiveStratumRuntime emits redaction-safe outbound actions and owns pure lifecycle state while firmware remains responsible for socket I/O."
  - "SubmitResponseObservation carries typed response categories and renders raw pool responses as redacted debug output."
  - "FakePoolTranscript::run_live_runtime drives the runtime and classifier directly for deterministic coverage."
requirements-completed: [STR-08, STR-09, STR-11, SAFE-12]
generated_by: gsd-execute-plan
lifecycle_mode: yolo
phase_lifecycle_id: 25-2026-07-05T01-55-45
generated_at: 2026-07-05T02:20:16Z
duration: 6min 52s
completed: 2026-07-05
---

# Phase 25 Plan 01: Pure Live Stratum Runtime and Fake-Pool Coverage Summary

**Pure Stratum v1 runtime, SubmitIntent-gated response classifier, and deterministic fake-pool lifecycle coverage without socket I/O or live share-outcome claims**

## Performance

- **Duration:** 6min 52s
- **Started:** 2026-07-05T02:13:24Z
- **Completed:** 2026-07-05T02:20:16Z
- **Tasks:** 3 completed
- **Files modified:** 6

## Accomplishments

- Added `LiveStratumRuntime` with redaction-safe `Debug`, outbound action queueing, subscribe/authorize sequencing, difficulty/extranonce tracking, notify-derived production work enqueueing, clean-jobs/reconnect/session invalidation, fallback/blocking hooks, and safe-stop postconditions.
- Added `submit_response.rs` so accepted/rejected classification requires a live `SubmitIntent`, matching request id, and typed `StratumResponse`; stale, fake-only, absent, timeout, malformed, reconnect, blocked, and stopped cases fail closed.
- Extended the existing `FakePoolTranscript` with a live-runtime runner that covers subscribe, authorize, set-difficulty, notify, accepted/rejected deterministic responses, clean-jobs, reconnect, fallback, timeout, malformed, blocked, and no-response paths.

## Task Commits

Each TDD task was committed atomically:

1. **Task 25-01-01 RED: Add failing live runtime state machine tests** - `78994d4` (`test`)
2. **Task 25-01-01 GREEN: Implement pure live runtime state machine** - `5240996` (`feat`)
3. **Task 25-01-02 RED: Add failing submit response classifier tests** - `0599e53` (`test`)
4. **Task 25-01-02 GREEN: Implement submit intent response classifier** - `487110a` (`feat`)
5. **Task 25-01-03 RED: Add failing fake-pool live runtime coverage** - `44cb0e2` (`test`)
6. **Task 25-01-03 GREEN: Drive fake-pool coverage through live runtime** - `24a0ce8` (`feat`)

## Files Created/Modified

- `crates/bitaxe-stratum/src/v1/live_runtime.rs` - Pure live lifecycle state machine, outbound action queue, production registry integration, invalidation paths, and safe-stop postconditions.
- `crates/bitaxe-stratum/src/v1/submit_response.rs` - SubmitIntent-tied response classifier and redaction-safe classification labels.
- `crates/bitaxe-stratum/src/v1/fake_pool.rs` - Existing transcript harness extended for Phase 25 deterministic runtime/classifier coverage.
- `crates/bitaxe-stratum/src/v1.rs` - Public exports for `live_runtime` and `submit_response`.
- `crates/bitaxe-stratum/BUILD.bazel` - Bazel source registration for new modules.

## Decisions Made

- Used narrow pure runtime accessors for fake-pool correlation instead of adding socket, daemon, or simulator ownership to `crates/bitaxe-stratum`.
- Generated BM1366 job IDs in `JOB_ID_STEP` increments to match the existing lookup-key invariant.
- Kept fake-pool accepted/rejected outcomes explicitly deterministic; live accepted/rejected STR-09 evidence remains deferred to detector-gated socket/runtime proof.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] Added narrow runtime accessors for fake-pool correlation**
- **Found during:** Task 25-01-03 (Expand deterministic fake-pool coverage)
- **Issue:** The fake-pool runner needed to dispatch queued runtime work and correlate a deterministic nonce into a `SubmitIntent`, but `LiveStratumRuntime` exposed only immutable registry/state views.
- **Fix:** Added `production_registry_mut`, `activate_fallback`, and `block_work_submission` methods to support deterministic fake-pool driving without exposing socket or firmware effects.
- **Files modified:** `crates/bitaxe-stratum/src/v1/live_runtime.rs`, `crates/bitaxe-stratum/src/v1/fake_pool.rs`
- **Verification:** `bazel test //crates/bitaxe-stratum:tests`
- **Committed in:** `24a0ce8`

**Total deviations:** 1 auto-fixed (1 blocking)
**Impact on plan:** The accessors are pure and scoped to the planned fake-pool/runtime integration; no new socket I/O, credential file access, hardware control, or live evidence claim was added.

## Issues Encountered

- The first live-runtime GREEN test run exposed the BM1366 job-ID lookup invariant: adjacent IDs alias under the lookup mask. The runtime now advances generated ASIC job IDs by eight, matching `Bm1366JobId::advance`.
- No authentication gates occurred.

## Known Stubs

None.

## Threat Flags

None - the changed trust boundaries are covered by T-25-01 through T-25-05, and no new network endpoint, auth path, file access pattern, schema boundary, socket ownership, firmware effect, or credential-file access was introduced.

## Verification

- `bazel test //crates/bitaxe-stratum:tests`
- `rg "pub mod live_runtime;|pub mod submit_response;" crates/bitaxe-stratum/src/v1.rs`
- `rg "\"src/v1/live_runtime.rs\"|\"src/v1/submit_response.rs\"" crates/bitaxe-stratum/BUILD.bazel`
- `rg "pub struct LiveStratumRuntime|pub enum LiveRuntimeAction|pub struct SafeStopPostconditions" crates/bitaxe-stratum/src/v1/live_runtime.rs`
- `rg "pub enum SubmitResponseObservation|pub enum SubmitClassification|pub enum RedactedSubmitRejectReason|classify_submit_response" crates/bitaxe-stratum/src/v1/submit_response.rs`
- `rg "classify_submit_response|SubmitClassification|LiveStratumRuntime" crates/bitaxe-stratum/src/v1/fake_pool.rs`
- Forbidden-pattern scan returned no matches for socket/secret/raw sentinels across `live_runtime.rs`, `submit_response.rs`, and `fake_pool.rs`.
- Stub scan returned no TODO/FIXME/placeholder or empty-rendering stubs in the changed modules.
- `git diff --check`

## Auth Gates

None.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

Plan 25-02 can build the firmware socket/safe-stop shell on top of the pure runtime and classifier. The remaining STR-09 live accepted/rejected proof still requires detector-gated Ultra 205 socket evidence with runtime-only pool credentials and redaction review.

## Self-Check: PASSED

- Found created files: `crates/bitaxe-stratum/src/v1/live_runtime.rs`, `crates/bitaxe-stratum/src/v1/submit_response.rs`, and `25-01-SUMMARY.md`.
- Found task commits: `78994d4`, `5240996`, `0599e53`, `487110a`, `44cb0e2`, and `24a0ce8`.

*Phase: 25-live-stratum-runtime-and-safe-stop*
*Completed: 2026-07-05*
