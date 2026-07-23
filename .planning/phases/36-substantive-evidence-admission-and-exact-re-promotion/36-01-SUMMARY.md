---
phase: 36-substantive-evidence-admission-and-exact-re-promotion
plan: "01"
subsystem: evidence-admission
tags: [typed-evidence, immutable-provenance, privacy, fail-closed, bazel]
requires:
  - phase: 35-detector-gated-correlated-evidence-and-exact-parity-promotion
    provides: Immutable Phase 35 evidence root and generation identities
provides:
  - Versioned Phase 36 successor envelope with typed shareable facts and closed insufficiency categories
  - Read-only explicit-root classifier that preserves Phase 35 bytes and avoids workspace or target discovery
  - Synthetic mutation catalog and real-process protected-root non-leakage guard
affects: [36-02-substantive-classification, 36-03-promotion, evidence-policy]
tech-stack:
  added: []
  patterns: [typed successor envelope, pure classification with thin filesystem adapter, executable mutation catalog, protected-root process guard]
key-files:
  created:
    - tools/parity/src/phase36_evidence.rs
    - tools/parity/src/phase36_evidence/contract.rs
    - tools/parity/src/phase36_evidence/tests/mutations.rs
    - tools/parity/fixtures/phase36/eligible.json
    - tools/parity/fixtures/phase36/mutation-catalog.json
    - scripts/phase36-evidence-test.sh
  modified:
    - tools/parity/src/main.rs
    - tools/parity/src/phase36_evidence/tests.rs
    - tools/parity/BUILD.bazel
    - scripts/BUILD.bazel
key-decisions:
  - "Phase 36 uses a distinct successor schema that references immutable Phase 35 root and generation digests without rewriting Phase 35 evidence."
  - "Incomplete but well-formed evidence produces ordered typed component insufficiencies; contradictory, ambiguous, or privacy-unsafe input remains a closed error."
  - "The explicit-root classifier dispatches before Git workspace detection, so classification depends only on its protected input root."
patterns-established:
  - "Successor evidence: validate identity and immutable references -> validate typed facts -> derive component sufficiency -> publish one aggregate assessment."
  - "Protected-root classification: explicit mode-0700 root + mode-0600 input -> read-only adapter -> pure classifier -> shareable typed output."
requirements-completed: [EVD-11, EVD-12, EVD-14]
generated_by: gsd-execute-plan
lifecycle_mode: yolo
phase_lifecycle_id: 36-2026-07-23T15-20-53
generated_at: 2026-07-23T16:47:28Z
duration: 34min
completed: 2026-07-23
---

# Phase 36 Plan 01: Synthetic Evidence Successor and Admission Guards Summary

**A versioned typed successor now classifies immutable Phase 35 evidence into exact sufficiency outcomes through a hermetic, read-only protected-root boundary.**

## Performance

- **Duration:** 34 min
- **Started:** 2026-07-23T16:14:06Z
- **Completed:** 2026-07-23T16:47:28Z
- **Tasks:** 2
- **Implementation commits:** 2
- **Files:** 10

## Accomplishments

- Added `phase36-evidence-v1` with typed Phase 35 root/generation references, evaluator identity, six ordered immutable artifact roles, sensor and runtime-health facts, provenance joins, observed runtime identity, independent effect evidence, and claim digests.
- Derived one `immutable_artifacts_sufficient` or `immutable_artifacts_insufficient` assessment while preserving exact component categories for snapshot substance, runtime health, runtime identity observation, and independent effect observation.
- Added a thin read-only filesystem adapter that rejects missing, symlinked, non-private, or malformed protected roots and never rewrites Phase 35 bytes.
- Added a canonical synthetic eligible fixture and an executable one-field mutation catalog covering schema, provenance, evaluator, role, path, symlink, permission, source, observation, effect, contradiction, join, and partial-output boundaries.
- Added a real-process Bazel guard proving the production CLI emits neither protected canaries nor input paths and contains no target discovery, credentials, flash, monitor, serial-control, mutation request, archived-lineage, or hardware-run invocation.

## Task Commits

1. **Task 1: Define the Phase 36 successor envelope and closed insufficiency types** - `72cd3347`
2. **Task 2: Add synthetic mutation fixtures and protected-root real-process guards** - `830cdec5`

## Files Created/Modified

- `tools/parity/src/phase36_evidence.rs` - Pure validation, sufficiency derivation, claim digest binding, and protected-root read adapter.
- `tools/parity/src/phase36_evidence/contract.rs` - Versioned successor schema, typed facts, aggregate assessment, and stable closed errors.
- `tools/parity/src/phase36_evidence/tests.rs` - Contract, digest-binding, insufficiency, round-trip, and immutable-byte regressions.
- `tools/parity/src/phase36_evidence/tests/mutations.rs` - Executable mutation table plus symlink, permission, and checked-in fixture tests.
- `tools/parity/fixtures/phase36/eligible.json` - Canonical synthetic eligible envelope.
- `tools/parity/fixtures/phase36/mutation-catalog.json` - Exact expected error or insufficiency for every reopened admission boundary.
- `tools/parity/src/main.rs` - Hermetic `classify-phase36-evidence --root` dispatch and shareable output rendering.
- `scripts/phase36-evidence-test.sh` - Real-process permissions, non-leakage, and no-effect structural guard.
- `tools/parity/BUILD.bazel` and `scripts/BUILD.bazel` - Named Phase 36 Rust and shell Bazel targets.

## Decisions Made

- Kept incomplete evidence distinct from invalid evidence: missing substantive observation authority yields typed insufficiency, while contradictions, malformed digests, ambiguous artifact roles, unsafe paths, and incomplete public claim binding fail closed.
- Bound every shareable claim field to one of four typed claim digests and verified that each one-field mutation changes the corresponding digest or classification outcome.
- Made the Phase 36 command independent of repository discovery. Absolute protected roots classify inside Bazel sandboxes without Git metadata and without searching for alternate evidence roots.
- Kept fixtures entirely synthetic and committed only shareable facts and public provenance-shaped digests.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Removed Git workspace discovery from the explicit-root classifier path**

- **Found during:** Task 2 real-process Bazel verification.
- **Issue:** The CLI constructed `LocalEnvironment` before command dispatch, so the otherwise read-only explicit-root classifier failed in a Bazel sandbox with no `.git` directory.
- **Fix:** Dispatched `classify-phase36-evidence` before workspace-aware command initialization and passed its explicit root directly to the protected-root adapter.
- **Files modified:** `tools/parity/src/main.rs`
- **Verification:** `//scripts:phase36_evidence_test` now passes hermetically, and the full Rust suite remains green.
- **Committed in:** `830cdec5`

**Total deviations:** 1 auto-fixed bug.
**Impact on plan:** The fix narrows dependencies and enforces the planned no-discovery contract; it adds no new behavior or authority.

## Issues Encountered

- Initial Bazel wiring needed explicit source visibility and the nested mutation module added to the Rust source graph. Both were corrected within Task 2 before its verification and commit.

## Verification

- The mandatory Rust sequence passed before both task commits: `cargo fmt --all`, all-target/all-feature Clippy with warnings denied, all-target/all-feature build, and all-feature tests.
- Focused Cargo verification passed all 8 `phase36_contract` tests; the expanded Phase 36 suite passed 13 tests.
- Bazel verification passed `//tools/parity:phase36_evidence_tests` and `//scripts:phase36_evidence_test`.
- `just verify-redaction`, `just verify-reference`, shell syntax validation, fixture denylist checks, and `git diff --check` passed.
- Stub scanning found no TODO, FIXME, placeholder, coming-soon, or UI-flow empty-value stub in created or modified Phase 36 files.
- No hardware, USB, device discovery, credentials, network access, flash, monitor, serial control, direct UART/pins, archived Phase 28.1.1 action, or Phase 35 evidence mutation occurred.

## User Setup Required

None.

## Next Phase Readiness

- Plan 36-02 can implement substantive snapshot, runtime-health, runtime-identity, and independent-effect replay against the closed successor contract.
- The mutation catalog and hermetic process guard are ready to fail shallow, privacy-unsafe, self-attested, or hardware-fallback implementations before promotion logic is introduced.

## Known Stubs

None.

## Self-Check: PASSED

- All six primary created artifacts exist, and implementation commits `72cd3347` and `830cdec5` resolve as commits.
- The summary has matching lifecycle provenance, exactly two standalone frontmatter delimiters, and a clean Markdown diff.
- The pre-existing `.planning/STATE.md` and `.planning/config.json` changes remain preserved; no hardware, credentials, network, archived-lineage action, or push occurred.

***

*Phase: 36-substantive-evidence-admission-and-exact-re-promotion*
*Completed: 2026-07-23*
