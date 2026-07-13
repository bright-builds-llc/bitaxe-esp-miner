---
phase: 29-evidence-workflow-automation-closure
plan: "01"
subsystem: evidence-workflow
tags:
  - rust
  - operator-evidence
  - atomic-promotion
  - redaction
requires:
  - phase: 23-redacted-operator-evidence-workflow
    provides: canonical eleven-slot operator evidence contract
  - phase: 27-live-hardware-asic-and-stratum-bridge
    provides: closed share-outcome categories and committed source evidence
provides:
  - explicit typed Phase 23, 25, 27, and 28 evidence profiles
  - shared eleven-slot validation descriptor and closed share outcomes
  - deterministic Phase 25/27 completion and Phase 28 consolidation commands
  - atomic managed-root exchange with rollback and typed recovery errors
affects:
  - 29-02 evidence wrapper finalizers and Phase 28 command integration
  - 29-03 documentation and regression closure
tech-stack:
  added:
    - direct libc 0.2 workspace dependency
  patterns:
    - functional rendering core with thin filesystem promotion adapter
    - explicit profile parsing at the CLI boundary
    - sibling staging plus OS-native atomic directory exchange
key-files:
  created:
    - tools/parity/src/operator_evidence/profile.rs
    - tools/parity/src/operator_evidence/generation.rs
    - tools/parity/src/operator_evidence/generation/rendering.rs
    - tools/parity/src/operator_evidence/generation/filesystem.rs
    - tools/parity/src/operator_evidence/generation/tests.rs
  modified:
    - Cargo.toml
    - Cargo.lock
    - MODULE.bazel.lock
    - tools/parity/Cargo.toml
    - tools/parity/BUILD.bazel
    - tools/parity/src/main.rs
    - tools/parity/src/operator_evidence.rs
key-decisions:
  - Phase identity is selected only through OperatorEvidenceProfile, never evidence-root spelling.
  - Generated blocked or deferred slots remain typed non-claims and cannot satisfy required observed evidence.
  - Existing Phase 28 destinations require a generator manifest and reject unknown files before promotion.
  - macOS uses renamex_np RENAME_SWAP and Linux uses renameat2 RENAME_EXCHANGE for atomic replacement.
requirements-completed:
  - EVD-07
  - EVD-08
  - EVD-09
duration: 18 min
completed: 2026-07-13
generated_by: gsd-execute-plan
lifecycle_mode: yolo
phase_lifecycle_id: 29-2026-07-13T00-19-45
generated_at: 2026-07-13T01:20:59Z
---

# Phase 29 Plan 01: Typed Evidence Contract and Generation Core Summary

**Explicit evidence profiles, a shared eleven-slot contract, deterministic completion, and fail-closed atomic Phase 28 consolidation**

## Performance

- **Duration:** 18 min
- **Started:** 2026-07-13T01:02:54Z
- **Completed:** 2026-07-13T01:20:59Z
- **Tasks:** 2
- **Files modified:** 12

## Accomplishments

- Replaced Phase 28 path/content inference with explicit `OperatorEvidenceProfile` CLI selection and a single typed descriptor for all eleven evidence slots.
- Added closed evidence disposition and share-outcome types with validation that rejects contradictory metadata, cross-linked observation claims, and unsupported accepted/rejected outcomes.
- Added deterministic Phase 25/27 missing-slot completion while preserving existing observed slot bytes.
- Added cross-link-only Phase 28 consolidation with repo-relative normalized breadcrumbs, allowlisted source categories, redaction-safe generated content, and byte-identical reruns.
- Added mode-0700 sibling staging, per-file and directory synchronization, macOS/Linux atomic directory exchange, rollback, retained-root recovery errors, and failure injection at every planned promotion boundary.
- Added typed CLI surfaces for validation, completion, and consolidation with constrained profile/status values.

## Task Commits

1. **Task 1: Define typed workflow profiles and the shared eleven-slot schema** - `a8d4634` (`feat`)
2. **Task 2: Implement deterministic completion and atomic Phase 28 consolidation** - `8a87000` (`feat`)

## Tests and Verification

- Task 1 RED: focused test failed because `operator_evidence/profile.rs` did not exist.
- Task 1 GREEN: `cargo test -p bitaxe-parity --all-features operator_evidence` passed 14 tests.
- Task 1 ordered gate passed: `cargo fmt --all`; `cargo clippy --all-targets --all-features -- -D warnings`; `cargo build --all-targets --all-features`; `cargo test --all-features`.
- Task 2 RED: focused test failed because `operator_evidence/generation.rs` did not exist.
- Task 2 GREEN: `cargo test -p bitaxe-parity --all-features` passed 152 tests.
- Task 2 ordered gate passed in the required order: format, Clippy with denied warnings, all-target/all-feature build, and all-feature workspace tests.
- CLI help checks passed for required `operator-evidence`, constrained `complete-operator-evidence`, and required-root `consolidate-phase28-evidence` arguments.
- Plan-wide Rust gate passed again after both commits.
- `bazel test //tools/parity:tests` passed 1/1 target on macOS and exercised `RENAME_SWAP` through the atomic replacement tests.
- `git diff --check` passed.

## Files Created/Modified

- `tools/parity/src/operator_evidence/profile.rs` - Profile, slot, disposition, outcome, and descriptor domain types.
- `tools/parity/src/operator_evidence/generation.rs` - Completion/consolidation entrypoint and typed errors.
- `tools/parity/src/operator_evidence/generation/rendering.rs` - Allowlisted source parsing and deterministic slot rendering.
- `tools/parity/src/operator_evidence/generation/filesystem.rs` - Path guards, durable staging, atomic exchange, rollback, and cleanup.
- `tools/parity/src/operator_evidence/generation/tests.rs` - Determinism, traversal, symlink, outcome, unknown-file, and failure-injection regressions.
- `tools/parity/src/operator_evidence.rs` - Explicit-profile validation consuming the shared descriptor.
- `tools/parity/src/main.rs` - Typed validation, completion, and consolidation subcommands.
- `Cargo.toml`, `Cargo.lock`, `tools/parity/Cargo.toml` - Direct `libc` dependency wiring.
- `tools/parity/BUILD.bazel`, `MODULE.bazel.lock` - Bazel source/dependency graph and generated crate-universe lock update.

## Decisions Made

- Completion creates only missing slots; existing observed files are never rewritten.
- Generated completion slots use stable blocked/deferred non-claims based on explicit workflow status.
- Phase 28 reads only a closed set of category fields from mandatory source files and renders new files instead of copying source contents.
- Existing destinations are replaced only when generator-owned and fully known; otherwise consolidation fails before staging promotion.
- A failed rollback or cleanup retains both complete generations and returns a typed recovery error instead of deleting either root.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 2 - Missing Critical] Split the generation prototype into focused modules**

- **Found during:** Task 2 simplification pass
- **Issue:** The initial generation implementation exceeded the repository's 628-line refactor trigger.
- **Fix:** Split rendering, filesystem effects, and tests into `generation/` children while retaining `generation.rs` as the module entrypoint.
- **Files modified:** `tools/parity/src/operator_evidence/generation.rs`, `tools/parity/src/operator_evidence/generation/rendering.rs`, `tools/parity/src/operator_evidence/generation/filesystem.rs`, `tools/parity/src/operator_evidence/generation/tests.rs`, `tools/parity/BUILD.bazel`
- **Verification:** All generation files are below 628 lines; Clippy, Cargo tests, and Bazel tests pass.
- **Committed in:** `8a87000`

**2. [Rule 3 - Blocking] Updated the generated Bazel module lock for direct libc use**

- **Found during:** Plan-wide Bazel verification
- **Issue:** Crate-universe regenerated `MODULE.bazel.lock` after `libc` became a direct parity-tool dependency.
- **Fix:** Included the generated lock update so clean Bazel resolution exposes `@crates//:libc` deterministically.
- **Files modified:** `MODULE.bazel.lock`
- **Verification:** `bazel test //tools/parity:tests` passes with the checked-in lock.
- **Committed in:** `8a87000`

**Total deviations:** 2 auto-fixed (1 missing critical, 1 blocking)

**Impact on plan:** Both adjustments preserve the planned architecture and reproducible build graph without adding workflow scope.

## Issues Encountered

- The first Task 1 Clippy gate identified a test-only profile constant as dead code; it was correctly scoped with `cfg(test)` and the full ordered gate was restarted.
- The first Task 2 Clippy gate identified target-specific needless returns; both platform branches were simplified and the full ordered gate was restarted.

## Known Stubs

None.

## Residual Risks

- The macOS host exercised `renamex_np(..., RENAME_SWAP)` at runtime. The same cfg-gated regression exercises Linux `renameat2(..., RENAME_EXCHANGE)` when run on Linux, but that platform branch was not runtime-executed on this host.
- `STATE.md` and `ROADMAP.md` were intentionally not updated because the orchestrator owns the existing planning-state change in the shared worktree.

## Next Phase Readiness

- Plan 29-02 can consume the typed completion/consolidation commands from Phase 25/27 finalizers and the new Phase 28 wrapper.
- Plan 29-03 can document the tested operator surfaces and close full workflow regressions.
- No hardware, credentials, raw private evidence, reference modifications, direct UART, or pin interaction occurred.

## Self-Check: PASSED

- Confirmed all five created Rust module files exist.
- Confirmed task commits `a8d4634` and `8a87000` exist in repository history.
- Confirmed the summary lifecycle matches `lifecycle_mode: yolo` and `phase_lifecycle_id: 29-2026-07-13T00-19-45`.
- Confirmed `.planning/STATE.md` and `.planning/ROADMAP.md` are absent from staged plan files.

***

*Phase: 29-evidence-workflow-automation-closure*
*Completed: 2026-07-13*
