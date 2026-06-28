---
phase: 07-ota-filesystem-and-release-packaging
plan: 06
subsystem: release-compliance-gate
tags: [release-gate, license-inventory, provenance, cargo-about, parity-tooling]
requires:
  - phase: 07-03
    provides: Cargo license report, release inventory, provenance manifest, and operator documentation structure.
  - phase: 07-04
    provides: Rust-owned static and recovery assets with source paths for provenance review.
provides:
  - Release-gate CLI validation for license and provenance release records.
  - Required non-Cargo license inventory rows for Bazel, ESP-IDF, flashing tools, static assets, and release artifacts.
  - Source/reference/static/recovery/GPL/artifact provenance records for Phase 7 release review.
affects: [phase-07-release-packaging, phase-07-ota-evidence, release-publication]
tech-stack:
  added: []
  patterns:
    - Section-based Markdown release validation in a pure parity-tool module.
    - Cargo license report treated as required input, not full release compliance.
key-files:
  created:
    - tools/parity/src/release_gate.rs
  modified:
    - tools/parity/src/main.rs
    - tools/parity/BUILD.bazel
    - docs/release/license-inventory.md
    - docs/release/provenance-manifest.md
key-decisions:
  - "Keep release-gate validation in tools/parity with filesystem access isolated to the CLI adapter."
  - "Require non-Cargo license/provenance sections so docs/release/cargo-about.html cannot satisfy REL-05 alone."
  - "Record source commit provenance as the release-time git command while pinning the reference commit explicitly."
patterns-established:
  - "Release compliance docs should include Owner and Follow-up lines when unresolved review language is unavoidable."
requirements-completed: [REL-05]
generated_by: gsd-execute-plan
lifecycle_mode: yolo
phase_lifecycle_id: 7-2026-06-28T13-30-15
generated_at: 2026-06-28T16:55:35Z
duration: 10m17s
completed: 2026-06-28
---

# Phase 07 Plan 06: Release License And Provenance Validation Summary

**Parity-tool release gate plus populated non-Cargo license and provenance records for REL-05.**

## Performance

- **Duration:** 10m17s
- **Started:** 2026-06-28T16:45:18Z
- **Completed:** 2026-06-28T16:55:35Z
- **Tasks:** 2
- **Files modified:** 5

## Accomplishments

- Added `tools/parity release-gate` with deterministic `release_gate: passed` output and validation errors for missing required sections, missing/empty Cargo reports, missing Cargo report references, and unresolved `unknown` language without owner/follow-up.
- Populated release inventory rows for Cargo, Bazel/rules, ESP-IDF/esp-rs, flashing tools, Rust-owned static assets, and release artifacts.
- Populated provenance records for source commit command, pinned reference commit, static/recovery asset sources, GPL review status, and release artifact review requirements.

## Task Commits

Each task was committed atomically:

1. **Task 1: Add release-gate validator to parity tooling** - `05eebee` (feat)
2. **Task 2: Populate non-Cargo license and provenance inventory** - `45c9cf4` (docs)

## Files Created/Modified

- `tools/parity/src/release_gate.rs` - Pure section-based release gate validator plus unit tests.
- `tools/parity/src/main.rs` - `release-gate` CLI subcommand and filesystem adapter.
- `tools/parity/BUILD.bazel` - Includes the new release gate module in the parity binary.
- `docs/release/license-inventory.md` - Concrete Cargo and non-Cargo release inventory rows.
- `docs/release/provenance-manifest.md` - Source, reference, static, recovery, GPL, and artifact review records.

## Decisions Made

- `release-gate` validates Markdown sections directly instead of shelling out to grep, keeping release checks deterministic and testable.
- The optional `--manifest` argument is accepted and validated for missing/empty content only when provided; Phase 7 package manifest v2 content remains owned by package-generation plans.
- Source commit provenance is recorded as the release-time git command to avoid creating a stale self-referential commit hash in a tracked document.

## Validation Results

| Command | Result |
| --- | --- |
| `node /Users/peterryszkiewicz/.codex/get-shit-done/bin/gsd-tools.cjs verify lifecycle 07 --require-plans` | Passed before implementation. |
| RED: `cargo test -p bitaxe-parity --all-features release_gate` | Failed as expected after tests were added; 6 release-gate behavior tests failed against the stub validator. |
| `rg -n "release-gate\|release_gate: passed\|cargo-about.html\|Cargo crates\|Bazel and rules\|GPL review status\|Release artifact review" tools/parity/src/main.rs tools/parity/src/release_gate.rs` | Passed. |
| `cargo test -p bitaxe-parity --all-features release_gate` | Passed, 7 targeted tests. |
| `bazel run //tools/parity:report -- release-gate --cargo-about docs/release/cargo-about.html` | Passed with `release_gate: passed`. |
| Task 2 inventory/provenance acceptance `rg` | Passed for required headings, `app.css.gz`, pinned reference commit, and no-upstream-generated-assets wording. |
| `just parity` | Passed with `validation_errors: none`. |
| `git diff --check` | Passed. |
| Rust pre-commit sequence before each task commit | `cargo fmt --all`, `cargo clippy --all-targets --all-features -- -D warnings`, `cargo build --all-targets --all-features`, and `cargo test --all-features` passed before both task commits. |

## Deviations from Plan

No Rule 1-3 auto-fixes were needed.

### Process Adjustments

**1. TDD RED state was not committed**
- **Found during:** Task 1 (Add release-gate validator to parity tooling)
- **Issue:** The plan's TDD flow calls for a failing RED commit, but `AGENTS.md` requires the full Rust pre-commit sequence to pass before every commit.
- **Adjustment:** Ran the failing RED test state, then implemented the validator and committed only the passing task state.
- **Files modified:** `tools/parity/src/release_gate.rs`, `tools/parity/src/main.rs`, `tools/parity/BUILD.bazel`
- **Verification:** RED failed as expected; GREEN and all pre-commit checks passed.
- **Committed in:** `05eebee`

## Issues Encountered

- The first RED run exposed a test authoring error (`str::replace` with a count argument). That was corrected with `replacen` before the intended RED behavior failures were recorded.

## Known Stubs

None. The only `unknown` strings are validator logic and unit-test fixtures proving unresolved unknowns fail the release gate.

## Threat Flags

None beyond the planned release input trust boundary. The new CLI reads local release documents and optional report/manifest files; it introduces no network endpoint, auth path, firmware effect, schema change, or runtime filesystem write.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

`release-gate` is ready for package and OTA release plans to call. Phase 7 still has remaining package, OTA, evidence, and verification plans to complete before public release claims.

## Self-Check: PASSED

- Found summary file: `.planning/phases/07-ota-filesystem-and-release-packaging/07-06-SUMMARY.md`
- Found key files: `tools/parity/src/release_gate.rs`, `tools/parity/src/main.rs`, `tools/parity/BUILD.bazel`, `docs/release/license-inventory.md`, and `docs/release/provenance-manifest.md`
- Found task commits: `05eebee` and `45c9cf4`
- Stub scan found no release-doc or UI/data stubs. The only pattern hit was an existing format string in `tools/parity/src/main.rs`.
- Threat surface scan found only the planned local release-document read path in the CLI; no unplanned network, auth, firmware, schema, or write surface was introduced.

***
*Phase: 07-ota-filesystem-and-release-packaging*
*Completed: 2026-06-28*
