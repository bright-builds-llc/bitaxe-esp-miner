---
phase: 08-parity-evidence-and-ultra-205-release-gate
plan: 01
subsystem: parity-release-gate
tags: [parity, release-gate, evidence-governance, manifest-validation]
requires:
  - phase: 07-ota-filesystem-and-release-packaging
    provides: Phase 7 package, OTA, filesystem, provenance, and deferred live-evidence records.
provides:
  - Checklist verified-claim guards for REL-08, firmware OTA, filesystem/static recovery, and deferred-scope evidence overclaims.
  - Manifest-backed release-gate validation for supplied Ultra 205 package manifests.
  - Focused regression tests proving package/workflow evidence cannot stand in for live Phase 8 evidence.
affects: [phase-08-release-gate, parity-checklist, release-packaging, release-provenance]
tech-stack:
  added: []
  patterns:
    - Pure evidence-policy helpers inside tools/parity with filesystem access kept in the CLI adapter.
    - Manifest JSON parsed with serde_json::Value before release-readiness decisions.
key-files:
  created:
    - .planning/phases/08-parity-evidence-and-ultra-205-release-gate/08-01-SUMMARY.md
  modified:
    - tools/parity/src/main.rs
    - tools/parity/src/release_gate.rs
key-decisions:
  - "Keep Phase 8 evidence hardening inside the existing tools/parity path instead of adding a second release-readiness tool."
  - "Do not promote checklist rows or mark Phase 8 requirements complete from this guard-only plan; later evidence plans own live hardware closure."
patterns-established:
  - "Verified release-sensitive rows must cite the exact evidence class terms they depend on."
  - "Supplying --manifest turns on package-manifest content validation while the default release gate remains document-only."
requirements-completed: [REL-08, EVD-01, EVD-02, EVD-03, EVD-05]
generated_by: gsd-execute-plan
lifecycle_mode: yolo
phase_lifecycle_id: 8-2026-06-28T21-51-32
generated_at: 2026-06-28T23:19:32Z
duration: 9 min
completed: 2026-06-28
---

# Phase 08 Plan 01: Parity Evidence Guard Summary

**Evidence-policy guards now prevent Phase 8 checklist and release-gate output from treating package or Ultra 205-only evidence as broader verified parity.**

## Performance

- **Duration:** 9 min
- **Started:** 2026-06-28T23:10:27Z
- **Completed:** 2026-06-28T23:19:32Z
- **Tasks:** 3
- **Files modified:** 3

## Accomplishments

- Tightened `tools/parity` checklist validation so verified `REL-003`, `OTA-001`, and `FS-001` rows require the specific Phase 8 evidence terms they claim.
- Added deferred-scope protection so non-205, deferred ASIC/protocol/BAP, all-board, and Angular rows fail if they reuse Ultra 205 evidence as verified proof.
- Added manifest-backed release-gate validation for schema version, key metadata, named Ultra 205 artifacts, lowercase SHA-256 values, manifest filename evidence, and artifact-review closure.

## Task Commits

Each code task was committed atomically:

1. **Task 1: Tighten checklist verified-claim guards** - `d8b54e6` (fix)
2. **Task 2: Validate manifest-backed release-gate evidence** - `e29273d` (fix)
3. **Task 3: Prove canonical guard commands still work** - verification-only task, no file changes to commit

## Files Created/Modified

- `tools/parity/src/main.rs` - Adds required evidence-term checks and deferred-scope Ultra 205 reuse guards.
- `tools/parity/src/release_gate.rs` - Parses supplied manifest JSON and validates package/review evidence.
- `.planning/phases/08-parity-evidence-and-ultra-205-release-gate/08-01-SUMMARY.md` - Records plan outcome and verification evidence.

## Decisions Made

- Used `serde_json::Value` for release manifest checks instead of ad hoc string parsing.
- Kept default `release-gate` behavior optional for manifests; manifest-backed checks run only when `--manifest` is supplied.
- Left `.planning/STATE.md`, `.planning/ROADMAP.md`, and `.planning/REQUIREMENTS.md` unstaged because `.planning/STATE.md` was already dirty with orchestrator-owned execution-start changes, and marking Phase 8 requirements complete here would overstate evidence closure.

## Validation Results

| Command | Result |
| --- | --- |
| RED: `cargo test -p bitaxe-parity --all-features` | Failed as expected with 4 new checklist guard tests failing before implementation. |
| RED: `cargo test -p bitaxe-parity --all-features release_gate_manifest` | Failed as expected with 4 new manifest tests failing before implementation. |
| `rg` acceptance checks for new test names and guard tokens | Passed. |
| `cargo test -p bitaxe-parity --all-features` | Passed, 37 tests. |
| `cargo test -p bitaxe-parity --all-features release_gate_manifest` | Passed, 4 focused tests. |
| `cargo fmt --all` | Passed before task commits. |
| `cargo clippy --all-targets --all-features -- -D warnings` | Passed before task commits. |
| `cargo build --all-targets --all-features` | Passed before task commits. |
| `cargo test --all-features` | Passed before task commits. |
| `bazel test //tools/parity:tests` | Passed. |
| `just parity` | Passed and printed `validation_errors: none`. |
| `bazel run //tools/parity:report -- release-gate` | Passed and printed `release_gate: passed`. |

## Deviations from Plan

No Rule 1-3 auto-fixes were needed.

### Process Adjustments

**1. TDD RED state was not committed**
- **Found during:** Tasks 1 and 2
- **Issue:** The plan requested TDD, but repo-local Rust rules require format, clippy, build, and tests to pass before every commit.
- **Adjustment:** Ran RED tests locally, confirmed expected failures, then committed only passing GREEN states.
- **Files modified:** `tools/parity/src/main.rs`, `tools/parity/src/release_gate.rs`
- **Verification:** RED failures and later GREEN passes are recorded in Validation Results.
- **Committed in:** `d8b54e6`, `e29273d`

**2. GSD requirement/state mutation was not staged**
- **Found during:** Metadata finalization
- **Issue:** `.planning/STATE.md` was dirty before this executor modified any files, and the Phase 8 requirements in the plan frontmatter are not globally complete after this guard-only plan.
- **Adjustment:** Created the required plan summary and left pre-existing STATE changes unstaged to avoid capturing unrelated orchestrator edits or overstating evidence closure.
- **Files modified:** `.planning/phases/08-parity-evidence-and-ultra-205-release-gate/08-01-SUMMARY.md`
- **Verification:** `git diff -- .planning/STATE.md` showed only execution-start changes predating this plan's file edits.
- **Committed in:** final metadata commit

## Issues Encountered

- None in code implementation.
- Metadata update was intentionally narrowed as described above to preserve the user's main-worktree constraint and the plan's no-overclaiming goal.

## Known Stubs

None. Stub scan found only an existing report-format string in `tools/parity/src/main.rs`; it is not a UI/data placeholder.

## Threat Flags

None beyond planned trust boundaries. The manifest validator expands the existing local `manifest-to-release-gate` trust boundary and introduces no network endpoint, auth path, firmware effect, schema migration, or filesystem write path.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

Plan 08-02 can use the hardened `tools/parity` path as a guardrail before collecting live Ultra 205 HTTP/OTA/recovery evidence. Manifest-backed release-gate checks are ready for use once package artifact review evidence is closed in later Phase 8 work.

## Self-Check: PASSED

- Found summary file: `.planning/phases/08-parity-evidence-and-ultra-205-release-gate/08-01-SUMMARY.md`
- Found key files: `tools/parity/src/main.rs` and `tools/parity/src/release_gate.rs`
- Found task commits: `d8b54e6` and `e29273d`
- Stub scan found no blocking stubs; the only hit was an existing formatting placeholder in report rendering.
- Threat surface scan found only the planned local checklist and manifest validation surfaces.

***
*Phase: 08-parity-evidence-and-ultra-205-release-gate*
*Completed: 2026-06-28*
