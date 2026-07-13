---
phase: 10-route-manifest-and-api-compare-unification
plan: 02
subsystem: api
tags: [api-compare, route-manifest, axeos, parity, rust]
requires:
  - phase: 10-route-manifest-and-api-compare-unification
    provides: Plan 01 Phase 7 route reporting helper and firmware manifest-count alignment.
  - phase: 07-ota-filesystem-and-release-packaging
    provides: Phase 7 route manifest entries for firmware OTA, OTAWWW gap, recovery, and static wildcard ownership.
provides:
  - API compare route presence checks backed by `phase07_routes()`.
  - RouteKind policy checks for firmware OTA, OTAWWW, recovery, and static wildcard routes.
  - Regression tests for missing routes, RouteKind downgrades, and weak-evidence verified overclaims.
affects: [api-compare, parity-evidence, route-shell, release-gate]
tech-stack:
  added: []
  patterns:
    - Injectable route slices for API compare route-policy regression tests.
    - Optional typed fixture policy fields with default-absent parsing.
key-files:
  created:
    - .planning/phases/10-route-manifest-and-api-compare-unification/10-02-SUMMARY.md
  modified:
    - tools/parity/src/api_compare.rs
key-decisions:
  - "Use `phase07_routes()` for production API compare route presence and RouteKind policy while preserving Phase 5 schema and captured-response fixture checks."
  - "Reject release-sensitive `verified_claim` entries for OTA, OTAWWW, recovery, and static wildcard routes when their evidence is only weak tooling/package labels."
patterns-established:
  - "API compare route policy: production delegates to `run_api_compare_with_routes(..., phase07_routes())`; tests mutate injected `AxeosRoute` copies."
requirements-completed: [API-09, API-10, REL-01, REL-02, REL-03, EVD-01]
generated_by: gsd-execute-plan
lifecycle_mode: yolo
phase_lifecycle_id: 10-2026-06-29T15-52-48
generated_at: 2026-06-29T16:58:11Z
duration: "7 min"
completed: 2026-06-29
---

# Phase 10 Plan 02: API Compare Route Policy Summary

**API compare now uses the Phase 7 typed route manifest for static, recovery, firmware OTA, and OTAWWW route policy while keeping Phase 5 schema and captured-response checks active.**

## Performance

- **Duration:** 7 min
- **Started:** 2026-06-29T16:50:14Z
- **Completed:** 2026-06-29T16:58:11Z
- **Tasks:** 2
- **Files modified:** 1 code file

## Accomplishments

- Refactored production `run_api_compare` to delegate to `run_api_compare_with_routes(..., phase07_routes())`.
- Added exact Phase 7 policy checks for `POST /api/system/OTA`, `POST /api/system/OTAWWW`, `GET /recovery`, and `GET /*`.
- Preserved `schema_routes` and `captured_response_checks` parsing from `phase05-required-routes.json`.
- Added regression coverage for missing Phase 7 routes, RouteKind downgrades, and weak-evidence verified overclaims.

## Task Commits

Each task was committed atomically:

1. **Task 1: Make API compare consume Phase 7 typed routes** - `9ed3cbd` (feat)
2. **Task 2: Add missing-route, downgrade, and overclaim regressions** - `4e1445b` (feat)

**Plan metadata:** pending final metadata commit.

## Files Created/Modified

- `tools/parity/src/api_compare.rs` - Added injected route-slice API compare helper, Phase 7 route-kind policy, optional `verified_claim` parsing, weak-evidence overclaim validation, and focused regression tests.
- `.planning/phases/10-route-manifest-and-api-compare-unification/10-02-SUMMARY.md` - Records execution evidence and plan outcome.

## Decisions Made

- Used `phase07_routes()` as the production route-policy authority for API compare instead of adding a new JSON manifest or code generation path.
- Kept Phase 5 OpenAPI/schema and captured-response fixtures intact because those remain the active evidence source for representative Phase 5 API response compatibility.
- Treated release-sensitive verified claims as invalid when backed only by `unit`, `workflow`, `package`, `api-compare`, or `static-route` labels.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 2 - Missing Critical] Honored Rust pre-commit requirements during TDD**
- **Found during:** Tasks 1 and 2
- **Issue:** The generic TDD workflow calls for committing failing RED states, but repo `AGENTS.md` requires `cargo fmt --all`, `cargo clippy --all-targets --all-features -- -D warnings`, `cargo build --all-targets --all-features`, and `cargo test --all-features` to pass before every commit.
- **Fix:** Ran the RED checks and recorded the expected failures, then committed only passing implementation states after all required Rust checks passed.
- **Files modified:** `tools/parity/src/api_compare.rs`
- **Verification:** Task 1 RED failed on missing `run_api_compare_with_routes`; Task 2 RED failed on the weak-evidence verified overclaim assertion. Both task GREEN states passed targeted Bazel checks and the Rust pre-commit sequence.
- **Committed in:** `9ed3cbd`, `4e1445b`

**Total deviations:** 1 auto-fixed (1 missing critical)
**Impact on plan:** No behavior scope change. The adjustment preserves repo commit safety while still proving RED/GREEN behavior.

## Issues Encountered

None.

## Known Stubs

None. Stub scan found no incomplete-marker or empty-data stubs in `tools/parity/src/api_compare.rs`.

## User Setup Required

None - no external service configuration required.

## Verification

- `bazel test //tools/parity:tests` - failed during Task 1 RED on missing `run_api_compare_with_routes`, then passed.
- `bazel test //tools/parity:tests` - failed during Task 2 RED on weak-evidence verified overclaim, then passed.
- `bazel run //tools/parity:report -- api-compare` - passed with `validation_errors: none`.
- `cargo fmt --all` - passed before task commits.
- `cargo clippy --all-targets --all-features -- -D warnings` - passed before task commits.
- `cargo build --all-targets --all-features` - passed before task commits.
- `cargo test --all-features` - passed before task commits.
- `git status --short reference/esp-miner` - clean.

## Next Phase Readiness

Ready for Plan 10-03. API compare now has manifest-backed route policy and regressions, so the next plan can record checklist/evidence claim boundaries without needing additional route-policy code.

## Self-Check: PASSED

- Summary file exists.
- Task commits `9ed3cbd` and `4e1445b` are present in git history.
- Summary uses standalone `---` only for the opening and closing frontmatter delimiters.
