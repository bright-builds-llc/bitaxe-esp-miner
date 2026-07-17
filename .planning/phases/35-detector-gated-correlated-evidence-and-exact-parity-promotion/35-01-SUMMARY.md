---
phase: 35-detector-gated-correlated-evidence-and-exact-parity-promotion
plan: "01"
subsystem: parity-evidence
tags: [rust, serde, sha256, evidence-admission, redaction, fail-closed]
requires:
  - phase: 34-stable-runtime-build-identity-and-coherent-operator-snapshot
    provides: Coherent boot-local operator snapshots and stable runtime/package identity contracts
provides:
  - Typed exact-package, detector-run, inventory, event-chain, and two-epoch evidence admission
  - Closed redacted projection and read-only Phase 35 validation CLI
  - Synthetic eligible fixture and exhaustive checked-in invalid fixture taxonomy
affects: [phase-35-correlated-capture, phase-35-parity-promotion, EVD-11, EVD-12, EVD-14]
tech-stack:
  added: []
  patterns: [typed capability digests, content-addressed inventory, chained evidence events, closed redacted projection]
key-files:
  created:
    - tools/parity/src/phase35_evidence.rs
    - tools/parity/src/phase35_evidence/contract.rs
    - tools/parity/src/phase35_evidence/digests.rs
    - tools/parity/src/phase35_evidence/inventory.rs
    - tools/parity/src/phase35_evidence/projection.rs
    - tools/parity/fixtures/phase35/eligible.json
    - tools/parity/fixtures/phase35/invalid/
  modified:
    - tools/parity/src/main.rs
    - tools/parity/BUILD.bazel
    - tools/parity/Cargo.toml
key-decisions:
  - "Phase 35 admission is a pure typed core; the CLI shell performs only read-only live rechecks before serializing a closed redacted projection."
  - "Both boot epochs retain the existing Phase 34 coherent-snapshot validator unchanged, with Phase 35 adding only the exact N-to-N+1 cross-epoch join."
  - "The evidence core is split into contract, digest, inventory, and projection modules so every production file remains within repository code-shape guidance."
patterns-established:
  - "Capability binding: recompute package, detector, inventory, root, epoch, payload, and predecessor digests before eligibility."
  - "Fixture completeness: every invalid category is a checked-in descriptor included at compile time and bound to one exact typed-error test."
requirements-completed: [EVD-11, EVD-12, EVD-14]
generated_by: gsd-execute-plan
lifecycle_mode: yolo
phase_lifecycle_id: 35-2026-07-17T17-00-37
generated_at: 2026-07-17T21:35:46Z
duration: 37min
completed: 2026-07-17
---

# Phase 35 Plan 01: Typed Correlated Evidence Admission Summary

**Fail-closed Rust admission now binds one exact package and detector run to an exact content-addressed inventory, chained events, and two independently coherent boot epochs while exporting only a closed redacted projection.**

## Performance

- **Duration:** 37 min
- **Started:** 2026-07-17T20:59:20Z
- **Completed:** 2026-07-17T21:35:46Z
- **Tasks:** 2
- **Files modified:** 54

## Accomplishments

- Added the typed `phase35-evidence-v1` contract and evaluator for manifest-v3 package identity, detector capability, exact inventory, root provenance, ordered event linkage, N-to-N+1 boot correlation, storage durability, restoration, cleanup, and redaction.
- Reused the existing Phase 34 coherent-snapshot validator independently for Boot A and Boot B, then applied a narrow Phase 35 cross-epoch join without weakening boot-local same-session checks.
- Added the read-only `validate-phase35-evidence --root <path>` command with live current-HEAD, reference-cleanliness, lifecycle, package/runtime, and no-actuation rechecks and generic path-free errors.
- Added one synthetic eligible root and 39 checked-in invalid taxonomy fixtures, each bound to its own exact-error regression test plus a compile-time completeness and sensitive-pattern guard.

## Task Commits

1. **Task 1: Implement the typed two-epoch evidence root and invariant evaluator** - `7be06387` (feat)
2. **Task 2: Add eligible and exhaustive negative fixtures for evidence admission** - `308cf846` (test)

## Files Created/Modified

- `tools/parity/src/phase35_evidence.rs` - Orchestrates static facts, exact capabilities, epoch validation, cross-epoch joins, and event admission.
- `tools/parity/src/phase35_evidence/contract.rs` - Defines serde inputs, private validated outputs, closed projection types, and exact error vocabulary.
- `tools/parity/src/phase35_evidence/digests.rs` - Provides domain-separated SHA-256 capability, root, epoch, event, inventory, and storage digests.
- `tools/parity/src/phase35_evidence/inventory.rs` - Loads and validates exact-role, normalized relative-path, regular-file, content-addressed evidence.
- `tools/parity/src/phase35_evidence/projection.rs` - Enforces a closed projection key set and raw-value canary rejection.
- `tools/parity/src/phase35_evidence/tests/` - Covers focused admission invariants and one exact test per checked-in invalid category.
- `tools/parity/fixtures/phase35/eligible.json` - Supplies the minimal synthetic eligible two-epoch root.
- `tools/parity/fixtures/phase35/invalid/` - Supplies 39 synthetic negative-category descriptors.
- `tools/parity/src/main.rs` - Exposes the read-only Phase 35 validation command and live rechecks.
- `tools/parity/BUILD.bazel`, `tools/parity/Cargo.toml`, `Cargo.lock`, and `MODULE.bazel.lock` - Wire existing workspace SHA-256/error dependencies and all fixture-backed tests into Cargo and Bazel.

## Decisions Made

- Kept filesystem and repository inspection in the CLI shell while all evidence eligibility remains deterministic and side-effect free in the functional core.
- Modeled redaction as construction from validated facts rather than filtering an input document, then rejected both forbidden keys and raw input canaries before serialization.
- Split the initial implementation into focused modules after the repository code-shape review; the production entry module is 425 lines and every supporting production module is smaller.
- Used checked-in invalid descriptors plus compile-time `include_str!` bindings so deleting a required fixture breaks the test target rather than silently reducing coverage.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Corrected malformed GSD state and roadmap updates**

- **Found during:** Plan metadata closeout.
- **Issue:** `state advance-plan` could not parse the repository's current-position format, decision insertion could not find the phase-scoped decision sections, and `roadmap update-plan-progress` replaced the Phase 35 row with shifted columns.
- **Fix:** Applied the intended Plan 1-of-4 position, Phase 35 decisions, next-step text, requirement total, and four-column roadmap row directly after the standard commands updated progress, metrics, session, and requirements.
- **Files modified:** `.planning/STATE.md` and `.planning/ROADMAP.md`.
- **Verification:** Frontmatter progress is 21/24 plans, the Phase 35 row is `In Progress (1/4 plans)`, EVD-11/EVD-12/EVD-14 are complete, and lifecycle validation is included in final closeout.
- **Committed in:** Final plan metadata commit.

**Total deviations:** 1 auto-fixed bug.
**Impact on plan:** Metadata truth was restored without changing implementation, evidence, promotion, or hardware scope. The internal module split also preserves the planned `phase35_evidence.rs` entrypoint while applying repository code-shape guidance.

## Issues Encountered

- The plan referenced a non-existent `tools/parity/fixtures/operator-snapshot/` directory. The existing Phase 34 snapshot validator and its established inline synthetic document shape remained the authoritative reusable contract, so no validator relaxation or substitute behavior was needed.
- Initial strict Clippy checks identified test-helper argument and table type complexity. Small fixture-spec and mutation type aliases resolved the warnings without suppressions.

## Known Stubs

None.

## User Setup Required

None - this plan is software-only and requires no credentials, hardware, or external service configuration.

## Next Phase Readiness

- Plan 35-02 can build the detector-gated capture shell against this exact typed admission boundary.
- Later Phase 35 promotion work can consume only the validated redacted projection and exact root digest.
- No hardware, detector, USB, serial, flash, monitor, HTTP, credential, checklist, parity-promotion, or archived-lineage action occurred in this plan.

## Self-Check: PASSED

- All key implementation, fixture, and summary files exist.
- Task commits `7be06387` and `308cf846` exist in repository history.
- Exactly 39 invalid fixture descriptors are present and compile-time bound to the completeness suite.
- Ordered Rust gates, 52 focused Phase 35 tests, Bazel parity tests, reference cleanliness, fixture safety scan, and `git diff --check` passed.

***

*Phase: 35-detector-gated-correlated-evidence-and-exact-parity-promotion*
*Completed: 2026-07-17*
