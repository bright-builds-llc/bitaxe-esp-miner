---
phase: 34-provenance-runtime-health-and-coherent-operator-snapshot
plan: "09"
subsystem: operator-snapshot-retention
tags: [atomic-retention, typed-errors, operator-snapshot, esp32-s3, observability]
requires:
  - phase: 34-07-coherent-operator-snapshot-publication
    provides: Completion-ordered operator snapshot publisher and production issuance paths
  - phase: 34-08-exact-immutable-package-admission
    provides: Final preceding Phase 34 gap-closure wave
provides:
  - Validated atomic marker/runtime-health retained-pair transaction
  - Independent retention and issuance error channels in the publication authority
  - Production-path proof that retention failure prevents external issuance
affects: [phase-34-review, phase-34-verification, phase-35-correlated-evidence]
tech-stack:
  added: []
  patterns: [validated-retained-pair, one-lock-transaction, heterogeneous-stage-errors]
key-files:
  created:
    - firmware/bitaxe/src/operator_snapshot_retention.rs
    - firmware/bitaxe/src/operator_snapshot_retention_production_tests.rs
  modified:
    - crates/bitaxe-api/src/logs.rs
    - crates/bitaxe-api/src/operator_snapshot_publication.rs
    - firmware/bitaxe/src/log_buffer.rs
    - firmware/bitaxe/src/runtime_snapshot.rs
    - firmware/bitaxe/src/main.rs
    - firmware/bitaxe/BUILD.bazel
    - tools/parity/src/operator_snapshot_evidence.rs
    - tools/parity/src/phase34_source_guard.rs
key-decisions:
  - "Retention validates and preflights the complete normalized pair before mutation, then appends both records while one production mutex guard is held."
  - "OperatorSnapshotPublishError preserves independent RetentionError and IssueError sources rather than coercing heterogeneous firmware failures."
  - "Production retention failure consumes its allocated revision and releases publication ownership while skipping the external issue closure."
patterns-established:
  - "Correlation retention: validate pair -> acquire singleton once -> preflight complete capacity/counter state -> append marker then health -> issue externally."
  - "Stage-local publication errors remain concrete and carry the ordering-lock health observed by the attempt."
requirements-completed: []
generated_by: gsd-execute-plan
lifecycle_mode: interactive
phase_lifecycle_id: 34-2026-07-15T03-26-15
generated_at: 2026-07-16T05:00:07Z
duration: 1h 20m
completed: 2026-07-15
---

# Phase 34 Plan 09: Transactional Operator Snapshot Retention Summary

**Every externally issued operator snapshot now depends on one successful, typed, one-lock transaction that retains its complete marker/runtime-health correlation pair.**

## Performance

- **Duration:** 1h 20m
- **Started:** 2026-07-16T03:40:00Z
- **Completed:** 2026-07-16T05:00:07Z
- **Tasks:** 2
- **Implementation commits:** 2
- **Files:** 11

## Accomplishments

- Added a validated `RetainedPair` that rejects empty or multiline records, normalizes exactly one trailing newline per record, and checks aggregate byte arithmetic before it can reach retained storage.
- Added an atomic `RetainedLogBuffer` pair append that rejects unavailable or undersized storage and counter overflow without changing existing bytes or counters.
- Added a closed redaction-safe firmware storage error and one production mutex acquisition covering the complete pair transaction.
- Introduced a named production retention adapter that returns the concrete storage result unchanged and emits the two existing log records only after retention succeeds.
- Split the publication contract into independent `RetentionError` and `IssueError` generic channels while preserving the exact source and ordering-lock health for each failure stage.
- Wired the production publisher directly to the transactional adapter before HTTP or WebSocket issuance, removing the prior unconditional-success closure and independent retained appends.
- Behaviorally compiled the exact production adapter and log-buffer modules with the production publisher, proving unavailable, poisoned, and undersized storage skip issuance, retain no partial pair, consume the failed revision, release the lock, and allow the next revision to succeed.
- Strengthened supplementary parity guards around the named adapter, one-lock API, heterogeneous publisher contract, and prohibited unconditional or two-append retention paths.

## Task Commits

1. **Task 1: Add one typed atomic retained-pair transaction** - `39c19c3b`
2. **Task 2: Propagate production retention failure before external issuance** - `e8fdd0d6`

## Files Created/Modified

- `crates/bitaxe-api/src/logs.rs` - Validated pair type, closed errors, atomic bounded append, and pure behavior tests.
- `crates/bitaxe-api/src/operator_snapshot_publication.rs` - Independent retention/issuance error generics and source-preservation tests.
- `crates/bitaxe-api/BUILD.bazel` - Phase 34 source visibility for retained-log contracts.
- `firmware/bitaxe/src/log_buffer.rs` - Concrete one-lock retained-pair transaction and serialized host-test seam.
- `firmware/bitaxe/src/operator_snapshot_retention.rs` - Named production adapter over the real log-buffer transaction.
- `firmware/bitaxe/src/operator_snapshot_retention_production_tests.rs` - Exact-module behavioral publisher and retention regressions.
- `firmware/bitaxe/src/runtime_snapshot.rs` - Direct fallible retention wiring before external issuance.
- `firmware/bitaxe/src/main.rs` - Production module registration.
- `firmware/bitaxe/BUILD.bazel` - Explicit exact-source host test and Phase 34 source coverage.
- `tools/parity/src/operator_snapshot_evidence.rs` - Supplementary production-wiring guards.
- `tools/parity/src/phase34_source_guard.rs` - Phase-level retention and heterogeneous-error guards.

## Decisions Made

- Kept pair validation in the pure API crate and singleton/mutex ownership in the firmware adapter, preserving the functional-core/imperative-shell boundary.
- Kept retained-pair errors closed to stable categories. Debug and Display output contain no marker text, boot session, revision, path, network value, or credential material.
- Preserved the existing public API and WebSocket behavior: internal projections use `Infallible`, system-info issuance uses `anyhow::Error`, cadence issuance uses `LiveCadenceIssueError`, and live-connect issuance uses `esp_err_t` alongside the same concrete retention error.
- Left the existing public re-export intact because it already exports the generic publication types without encoding their arity; no compatibility-only `lib.rs` edit was necessary.
- Left OBS-06 pending for fresh independent Phase 34 review and verification, as required by the gap-plan contract.

## Deviations from Plan

None - plan executed within its specified architecture and safety scope.

## Issues Encountered

- The resumed Task 1 normalization test contained an incorrect hard-coded expected size. Focused RED showed 59 bytes rather than 61, so the assertion was corrected to derive the byte requirement from the two normalized records.
- The Task 2 heterogeneous-error RED failed exactly because the former publisher forced both closures to return one error type. The two-error implementation made the same test compile and pass without coercion.
- The first production publisher test build required `Display` on its distinct issuance sentinel so the two-error enum could be checked for redaction; adding the closed sentinel formatter resolved the test-only compile failure.
- The ESP32-S3 firmware build retained 14 pre-existing dead-code warnings. Host Clippy passed with warnings denied, and this plan introduced no new firmware warning.

## Verification

- The exact pre-commit Rust sequence passed before each task commit: `cargo fmt --all`, all-target/all-feature Clippy with warnings denied, all-target/all-feature build, and all-feature tests.
- Focused Cargo tests passed for retained pairs, heterogeneous publication errors, operator-snapshot evidence, and Phase 34 source guards.
- Focused Bazel tests passed for the exact production retained-pair target, operator-snapshot publication suite, parity suite, and API suite.
- `just build` compiled the real ESP32-S3 firmware with all four heterogeneous issuance error shapes and the concrete fallible storage error.
- Repository-wide `bazel test //...` passed all 60 test targets; `just build`, `just package`, `just verify-reference`, and `git diff --check` passed.
- Bazel target inspection confirmed `retained_pair_production_tests` explicitly owns the exact production `operator_snapshot_retention.rs`, `log_buffer.rs`, and behavioral crate root. The packaged schema-v3 manifest records clean source commit `e8fdd0d6e97d68993afb370d5c791cefca6f1401` with matching dev build identity.
- Final review found no shared error coercion, partial retained pair, poison recovery reporting success, revision retry/reuse, issue after retention failure, raw diagnostic content, public response change, or prohibited hardware/effect path.
- No hardware, USB, serial, credentials, network discovery, flashing, OTA execution, direct UART/pins, Phase 35, or archived-lineage operation was used.

## User Setup Required

None.

## Next Phase Readiness

- Both Phase 34 gap plans now have implementation summaries and all software gates pass.
- OBS-06 remains deliberately pending. A fresh Phase 34 code review, regression gate, schema-drift check, and independent goal verifier are the only authorized next steps; Phase 35 remains blocked until that authority passes.

## Self-Check: PASSED

- Implementation commits `39c19c3b` and `e8fdd0d6` exist and contain the two planned task changes.
- The summary has lifecycle `34-2026-07-15T03-26-15` and exactly two standalone frontmatter delimiters.
- All focused and repository-wide software gates pass, OBS-06 remains pending, and no push or prohibited hardware/Phase 35 activity occurred.

***

*Phase: 34-provenance-runtime-health-and-coherent-operator-snapshot*
*Completed: 2026-07-15*
