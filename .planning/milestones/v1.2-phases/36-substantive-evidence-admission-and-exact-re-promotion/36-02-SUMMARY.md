---
phase: 36-substantive-evidence-admission-and-exact-re-promotion
plan: "02"
subsystem: evidence-admission
tags:
  - rust
  - serde
  - provenance
  - device-session
  - effect-ledger
requires:
  - phase: 36-01
    provides: Versioned Phase 36 successor envelope and closed insufficiency vocabulary
  - phase: 35
    provides: Immutable correlated evidence root and device-session artifacts
provides:
  - Typed substantive sensor and runtime-health admission joined to one operator snapshot
  - Replayed observed runtime identity joined to exact package digests
  - Independent bounded effect-interval completeness classification
affects:
  - 36-03
  - 36-04
tech-stack:
  added:
    - bitaxe-device-session dependency in bitaxe-parity
  patterns:
    - Closed typed insufficiency instead of inferred eligibility
    - Explicit protected-root classification without repository or hardware discovery
    - JSONL event replay through the production device-session reducer
key-files:
  created:
    - tools/parity/src/phase36_evidence/substance.rs
    - tools/parity/src/phase36_evidence/substance/types.rs
    - tools/parity/src/phase36_evidence/runtime_identity.rs
    - tools/parity/src/phase36_evidence/runtime_identity/ledger.rs
    - tools/parity/src/phase36_evidence/effects.rs
  modified:
    - tools/parity/src/main.rs
    - tools/parity/BUILD.bazel
    - scripts/phase36-evidence-test.sh
key-decisions:
  - "Identity/revision membership remains unchanged and is composed with separate substantive fact validation."
  - "Package-produced runtime identity has zero observation authority; only replayed or independently complete device-session results can validate."
  - "Supervisor no-actuation booleans have zero authority; absent independent interval ownership remains typed insufficient."
  - "Canonical requirement completion remains unchanged for Plan 04 and independent verification."
requirements-completed: []
generated_by: gsd-execute-plan
lifecycle_mode: yolo
phase_lifecycle_id: 36-2026-07-23T15-20-53
generated_at: 2026-07-23T17:35:22Z
metrics:
  duration: 42m
  completed: 2026-07-23
---

# Phase 36 Plan 02: Substantive Evidence Admission Core Summary

Typed sensor, health, observed runtime-identity, and independent effect-interval admission with exact provenance joins and fail-closed insufficiency.

## Performance

- **Duration:** 42 minutes
- **Started:** 2026-07-23T16:53:52Z
- **Completed:** 2026-07-23T17:35:22Z
- **Tasks:** 3
- **Files changed:** 21 implementation and lock files

## Accomplishments

- Composed the unchanged operator snapshot identity validator with typed four-state sensor truth, atomic INA260 facts, runtime-health chronology, independent watchdog non-claims, and exact API/WebSocket/retained joins.
- Replayed immutable device-session request and JSONL events through `SessionState::apply`, compared private/public terminal artifacts, and bound Boot B source, reference, ELF, device, session, and package digests.
- Added a closed independent effect vocabulary with ordered authorization, invocation, result, and closure records for all eight allowed legacy effects.
- Added an explicit-root effect classifier whose missing-ledger result is exactly `independent_effect_observation_insufficient` and whose production path has no detector, credential, network, flash, monitor, serial-control, or hardware invocation.

## Task Commits

1. **Task 1: Admit substantive sensor and runtime-health facts with typed provenance** - `8a9d1e47`
2. **Task 2: Replay independently observed runtime identity and exact package correlation** - `5d18c4f3`
3. **Task 3: Classify independent no-actuation coverage for immutable legacy evidence** - `7fdd48cf`
4. **Overall review fix: Reject mixed producer boot sessions** - `52228071`

## Files Created and Modified

- `tools/parity/src/phase36_evidence/substance.rs` and `substance/types.rs` - Typed three-surface sensor and health parser, state legality, provenance joins, and claim digests.
- `tools/parity/src/phase36_evidence/runtime_identity.rs` and `runtime_identity/ledger.rs` - Exact-package document validation, device-session replay, terminal-pair admission, and ordered ledger coverage.
- `tools/parity/src/phase36_evidence/effects.rs` - Independent effect ledger schema, completeness reducer, protected-root loader, and closed projection.
- `tools/parity/src/phase36_evidence/tests/` and `tools/parity/fixtures/phase36/` - Mutation matrices and synthetic closed-value fixtures.
- `tools/parity/src/main.rs` - Read-only `classify-phase36-effects --root` command.
- `scripts/phase36-evidence-test.sh` - Real-process eligible/insufficient classification, protected-sink checks, and zero-effect invocation source guard.
- `Cargo.lock`, `MODULE.bazel.lock`, `tools/parity/Cargo.toml`, and Bazel files - Device-session dependency and source/test graph updates.

## Decisions Made

- Sensor and health eligibility requires exact substantive documents; the old Phase 35 identity-only shape is explicitly insufficient.
- Fresh compatibility zero is not observation truth. Stale, unavailable, and fault states retain only the values and stamps their typed state permits.
- All available producer stamps must share one producer boot session, while every document also shares one operator boot session and revision.
- A narrower terminal-result/public-projection identity path is accepted only when every required public/private fact is independently present and mutually consistent.
- Legacy effect records require exactly one ordered record for package probe, package flash, passive observation, hostname read, hostname patch, approved reboot, hostname restoration, and cleanup.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] Resolved stale read-first source paths**

- **Found during:** Tasks 1 and 2
- **Issue:** `crates/bitaxe-api/src/runtime_health.rs` and `tools/device-session/src/reducer.rs` do not exist.
- **Fix:** Loaded the actual runtime-health sources and reused the production reducer implemented by `SessionState::apply`.
- **Files modified:** No source-path compatibility shim was added.
- **Commit:** `8a9d1e47`, `5d18c4f3`

**2. [Rule 1 - Bug] Rejected mixed producer boot sessions**

- **Found during:** Overall provenance review
- **Issue:** Distinct, non-duplicate power/temperature/tachometer stamps could carry different producer boot sessions while still changing only the digest.
- **Fix:** Added a shared producer-session invariant, bound it into `SubstantiveSnapshotJoin`, and added a regression.
- **Files modified:** `substance.rs`, `substance/types.rs`, `tests/substance.rs`
- **Commit:** `52228071`

**3. [Rule 1 - Test] Narrowed the zero-effect invocation guard**

- **Found during:** Task 3 real-process verification
- **Issue:** The original source regex treated the required closed vocabulary token `package_flash` as a command invocation.
- **Fix:** Kept the existing classifier guard and added an invocation-API-specific guard for the effect vocabulary module.
- **Files modified:** `scripts/phase36-evidence-test.sh`
- **Commit:** `7fdd48cf`

## Verification

- `cargo test -p bitaxe-parity phase36_substance --all-features`: 12 passed.
- `cargo test -p bitaxe-parity phase36_runtime_identity --all-features`: 11 passed.
- `cargo test -p bitaxe-device-session --all-features`: 18 passed across unit and CLI tests.
- `cargo test -p bitaxe-parity phase36_effects --all-features`: 12 passed.
- `bazel test //tools/parity:phase36_evidence_tests //scripts:phase36_evidence_test`: passed.
- Required Rust sequence passed in order: format, clippy with warnings denied, all-target build, and all-feature tests.
- `just parity`, `just verify-reference`, `just verify-redaction`, and `git diff --check`: passed.
- `just test` completed all relevant Phase 36 work but reported the preexisting archived `//scripts:phase28_1_1_lifecycle_frame_test` socket refusal. Repo guidance makes that lineage terminal and forbids repair; it is recorded in `deferred-items.md`.
- Canonical `.planning/REQUIREMENTS.md` remained unchanged.

## Known Stubs

None.

## Deferred Issues

- The repository-wide archived Phase 28.1.1 lifecycle-frame test remains an out-of-scope installed-GSD/archive exception. It does not affect any Phase 36 target or admission path.

## Next Phase Readiness

- Plan 36-03 can consume separate substantive, runtime-identity, and independent-effect claim types.
- Plan 36-04 retains ownership of canonical requirement reconciliation after independent verification.

## Self-Check: PASSED

- All key implementation files and the summary exist.
- All four implementation/fix commits are present in Git history.
- Frontmatter uses exactly one opening and one closing delimiter.
