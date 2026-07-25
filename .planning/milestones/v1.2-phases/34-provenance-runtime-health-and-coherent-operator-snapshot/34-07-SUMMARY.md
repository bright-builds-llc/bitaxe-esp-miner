---
phase: 34-provenance-runtime-health-and-coherent-operator-snapshot
plan: "07"
subsystem: operator-snapshot-publication
tags: [operator-snapshot, concurrency, retained-chronology, http, websocket]
requires:
  - phase: 34-02-coherent-operator-snapshot
    provides: Typed boot-session/revision identity and correlated operator projections
provides:
  - Completion-ordered snapshot revision authority
  - Retained and externally issued monotonic chronology
  - Actual HTTP and WebSocket issuance inside one publication boundary
affects: [phase-34-gap-closure, phase-35-correlated-evidence]
tech-stack:
  added: []
  patterns: [collect-before-publish, serialized-retention-and-issuance, typed-stage-errors]
key-files:
  created:
    - crates/bitaxe-api/src/operator_snapshot_publication.rs
  modified:
    - crates/bitaxe-api/BUILD.bazel
    - crates/bitaxe-api/src/lib.rs
    - crates/bitaxe-api/src/operator_snapshot.rs
    - firmware/bitaxe/BUILD.bazel
    - firmware/bitaxe/src/runtime_snapshot.rs
    - firmware/bitaxe/src/http_api.rs
    - tools/parity/src/operator_snapshot_evidence.rs
    - tools/parity/src/phase34_source_guard.rs
    - tools/parity/src/phase33_source_guard.rs
key-decisions:
  - "Collection finishes before revision allocation; one mutex then owns identity attachment, retention, and the actual external issue call."
  - "A failed retention or issuance consumes its revision, while poison recovery preserves the existing sequence and reports degraded lock health."
  - "WebSocket send failures are recorded under publication authority, then stale-client cleanup and logging run only after the authority unlocks."
patterns-established:
  - "Operator snapshot publication: collect unnumbered candidate -> allocate and complete -> retain -> issue -> unlock."
  - "External side-effect cleanup follows publication return so no caller-side mutex can reverse-enter the ordering authority."
requirements-completed: []
generated_by: gsd-execute-plan
lifecycle_mode: interactive
phase_lifecycle_id: 34-2026-07-15T03-26-15
generated_at: 2026-07-16T00:49:18Z
duration: 21min
completed: 2026-07-15
---

# Phase 34 Plan 07: Ordered Operator Snapshot Publication Summary

**Snapshot revisions now follow completed publication order, with matching retained records and the real system-info/live-WebSocket issuance calls serialized by one production authority.**

## Performance

- **Duration:** 21 min
- **Started:** 2026-07-16T00:28:04Z
- **Completed:** 2026-07-16T00:49:18Z
- **Tasks:** 2
- **Implementation commits:** 2
- **Files:** 10

## Accomplishments

- Added a host-runnable `OperatorSnapshotPublisher` that collects before locking and then owns checked revision allocation, completion, retention, and issuance through return.
- Made poison, reentrancy, sequence exhaustion, retention failure, and issuance failure explicit typed outcomes without revision reset, reuse, retry, or deadlock.
- Replaced firmware's reservation-first path with one boot-lifetime publisher and unnumbered collection of runtime, platform, settings, Wi-Fi, safety, and health facts.
- Moved the actual system-info JSON write, live-cadence asynchronous broadcast, and live-connect synchronous WebSocket send into issuer closures under the publication authority.
- Added a deterministic reverse-completion regression that executes the production publisher and proves direct retained and combined HTTP/WebSocket revisions are `[1, 2]` without sorting.

## Task Commits

1. **Task 1: Add the host-runnable production publication transition** - `150495f1`
2. **Task 2: Wire retained records and real HTTP/WebSocket issuance through the transition** - `312d1273`

## Files Created/Modified

- `crates/bitaxe-api/src/operator_snapshot_publication.rs` - Production ordering transition, typed outcomes, RAII reentrancy protection, poison recovery, and focused concurrency/failure tests.
- `crates/bitaxe-api/src/operator_snapshot.rs` - Test-only exhausted-sequence constructor used to prove fail-closed overflow behavior.
- `crates/bitaxe-api/src/lib.rs` and `crates/bitaxe-api/BUILD.bazel` - Public exports and build graph ownership for the production publisher.
- `firmware/bitaxe/src/runtime_snapshot.rs` - Unnumbered candidate collection, one publisher static, retained records, and issuer-taking identity-bearing projections.
- `firmware/bitaxe/src/http_api.rs` - Real HTTP and live-WebSocket send calls inside publication issuance closures, with cleanup after unlock.
- `firmware/bitaxe/BUILD.bazel` - Focused host publication test suite.
- `tools/parity/src/operator_snapshot_evidence.rs` - Barrier/channel-controlled reverse-completion production regression and updated source assertions.
- `tools/parity/src/phase34_source_guard.rs` - Guards for one authority, collect-before-publish ordering, real send ownership, and prohibited legacy paths.
- `tools/parity/src/phase33_source_guard.rs` - Compatibility guard spellings updated for the new issuer-taking runtime API without weakening Phase 33 behavior.

## Decisions Made

- Kept collection and all device-facing reads outside the publisher mutex; ordered work is limited to checked identity allocation, bounded projection/serialization, two retained writes, and bounded HTTP/WebSocket issuance.
- Preserved revisions across poison by recovering the existing inner sequence and clearing only the mutex poison flag; callers receive `RecoveredPoison` for observability.
- Preserved existing public response/frame shapes and sample-marker drain policy. Identity-bearing DTOs are no longer returned to HTTP code for a later, unordered send.
- Returned bounded WebSocket send failures from the issuance adapter and deferred stale-session unregister/log operations until after publication unlock.

## Deviations from Plan

### Test-only sequence exhaustion seam

- **Found during:** Task 1
- **Issue:** The planned production publisher exhaustion regression could not construct the otherwise private maximum sequence state.
- **Resolution:** Added a `cfg(test)`-only `OperatorSnapshotSequence::with_last_revision_for_test` constructor in `operator_snapshot.rs`. It is unavailable to production code and preserves the fail-closed public surface.
- **Files:** `crates/bitaxe-api/src/operator_snapshot.rs`

### Phase 33 source-guard compatibility update

- **Found during:** Task 2 mandatory repository tests
- **Issue:** Phase 33 guards asserted obsolete function names and a moved hostname assignment after Plan 07 replaced return-then-send APIs.
- **Resolution:** Updated only the structural spellings to follow `publish_projected_system_info` and the moved assignment while retaining the same compatibility, non-publication, and no-effect assertions.
- **Files:** `tools/parity/src/phase33_source_guard.rs`

## Issues Encountered

- Task 1 RED failed with the expected missing production module before implementation.
- Task 2 RED executed the new reverse-completion behavioral regression successfully but failed the source guard because no production publisher static existed yet, isolating the missing firmware wiring.
- The first Task 2 mandatory test pass exposed the two obsolete Phase 33 source-guard assumptions above. After the narrow compatibility update, the exact mandatory sequence was restarted from formatting and passed.
- The ESP32-S3 firmware build retained 14 pre-existing dead-code warnings; host Clippy passed with warnings denied and this plan introduced no new firmware warning.

## Verification

- The exact pre-commit Rust sequence passed for both task commits, and the final plan-wide rerun passed in order: `cargo fmt --all`, all-target/all-feature Clippy with warnings denied, all-target/all-feature build, and all-feature tests.
- Focused Cargo verification passed all 7 publisher tests, 6 operator-evidence tests, 5 Phase 34 source guards, and 11 Phase 33 compatibility source guards.
- Focused Bazel verification passed the API and parity suites; `just build` produced the ESP32-S3 firmware successfully.
- Repository-wide `bazel test //...` passed all 59 test targets; `just build`, `just package`, `just verify-reference`, and `git diff --check` also passed.
- The final source audit found exactly one `OnceLock<OperatorSnapshotPublisher>`, no reservation-only sequence, no identity-bearing return-then-send API, and no sorting in the adversarial chronology test.
- No hardware, USB, serial, credentials, network access, OTA execution, direct UART/pins, mining, Phase 35, or archived Phase 28.1.1 operation was used.

## User Setup Required

None.

## Next Phase Readiness

- The OBS-06 production defect is implemented and regression-covered, but the requirement remains deliberately pending until the orchestrator runs a fresh Phase 34 verifier across Plans 34-05 through 34-07.
- Phase 35 remains blocked until that verifier decides Phase 34 status. No hardware qualification was attempted or promoted.

## Self-Check: PASSED

- Implementation commits `150495f1` and `312d1273` exist and contain the two planned task changes plus only the documented test/compatibility adaptations.
- All focused and repository-wide software gates pass, the summary has matching lifecycle provenance, and no requirement or Phase 34 completion status was promoted.
- The orchestrator-owned `.planning/STATE.md` and `.planning/ROADMAP.md` modifications remain preserved outside this plan's commits; no push occurred.

***

*Phase: 34-provenance-runtime-health-and-coherent-operator-snapshot*
*Completed: 2026-07-15*
