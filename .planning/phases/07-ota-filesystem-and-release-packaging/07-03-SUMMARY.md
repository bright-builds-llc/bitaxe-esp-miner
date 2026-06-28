---
phase: 07-ota-filesystem-and-release-packaging
plan: 03
title: Release Documentation, License, Provenance, And Evidence Contracts
generated_by: gsd-execute-plan
executor_model: gpt-5-codex
lifecycle_mode: yolo
phase_lifecycle_id: 7-2026-06-28T13-30-15
generated_at: 2026-06-28T16:11:06Z
status: complete
started_at: 2026-06-28T15:59:45Z
completed_at: 2026-06-28T16:11:06Z
duration: 11m21s
requirements-completed: [REL-05, REL-07, REL-08, REL-03]
subsystem: release-compliance-docs
tags:
  - cargo-about
  - release-docs
  - provenance
  - license-inventory
  - parity-evidence
dependency_graph:
  requires:
    - 07-01
    - 07-02
  provides:
    - cargo-about-policy-and-report
    - ultra205-operator-guide-contract
    - release-license-provenance-evidence-records
  affects:
    - phase-07-release-gate
    - phase-07-static-assets
    - phase-07-otawww-gap
tech_stack:
  added:
    - cargo-about 0.9.0 local CLI tool
  patterns:
    - Cargo-only license report paired with separate non-Cargo release inventory
    - Evidence docs separate package, live firmware, hardware, and gap conclusions
key_files:
  created:
    - about.toml
    - about.hbs
    - docs/release/cargo-about.html
    - docs/release/ultra-205.md
    - docs/release/license-inventory.md
    - docs/release/provenance-manifest.md
    - docs/parity/evidence/phase-07-ota-filesystem-release.md
  modified:
    - .planning/phases/07-ota-filesystem-and-release-packaging/07-02-SUMMARY.md
key-decisions:
  - Keep cargo-about scoped to Cargo dependencies and require separate non-Cargo inventory for Bazel, ESP-IDF, tools, static assets, and artifacts.
  - Preserve OTAWWW as an explicit REL-03 evidence gap until D-16 recovery/interruption evidence exists.
  - Initialize Phase 7 evidence with not-run/live-hardware-pending conclusions instead of release parity claims.
metrics:
  tasks_completed: 3
  task_commits: 3
  files_created: 7
  files_modified: 1
---

# Phase 07 Plan 03: Release Documentation, License, Provenance, And Evidence Contracts Summary

Wave 0 release documentation now has cargo-about policy/report output, an Ultra 205 operator guide contract, and separated license, provenance, and evidence records that later package/OTA plans can fill without overclaiming parity.

## Performance

- **Duration:** 11m21s
- **Started:** 2026-06-28T15:59:45Z
- **Completed:** 2026-06-28T16:11:06Z
- **Tasks:** 3
- **Files modified:** 8

## Accomplishments

- Added `about.toml`, `about.hbs`, and generated `docs/release/cargo-about.html` with a Cargo-only dependency license scope and explicit PROVENANCE/GPL caveats.
- Added `docs/release/ultra-205.md` covering package, flash, monitor, firmware OTA, OTAWWW gap status, recovery, large erase, failed update, interrupted update, rollback, evidence, and release caveats.
- Added `docs/release/license-inventory.md`, `docs/release/provenance-manifest.md`, and `docs/parity/evidence/phase-07-ota-filesystem-release.md` with separated package/live/hardware/gap/compliance conclusions.

## Task Commits

1. **Task 1: Add cargo-about release inventory config** - `9127943` (docs)
2. **Task 2: Create release operator guide structure** - `2befb58` (docs)
3. **Task 3: Create provenance, license, and Phase 7 evidence records** - `a619d8d` (docs)

## Files Created/Modified

- `about.toml` - cargo-about 0.9.0 accepted-license policy for the current Cargo graph.
- `about.hbs` - release report template with Cargo-only, PROVENANCE, and GPL scope notes.
- `docs/release/cargo-about.html` - generated Cargo dependency license report.
- `docs/release/ultra-205.md` - Ultra 205 build, package, flash, OTA, recovery, rollback, and caveat guide structure.
- `docs/release/license-inventory.md` - separated Cargo, Bazel, ESP-IDF, tooling, static asset, and artifact inventory.
- `docs/release/provenance-manifest.md` - source/reference/static/recovery/GPL/release review manifest.
- `docs/parity/evidence/phase-07-ota-filesystem-release.md` - initial Phase 7 release evidence record.
- `.planning/phases/07-ota-filesystem-and-release-packaging/07-02-SUMMARY.md` - lifecycle metadata repair required before execution.

## Validation Results

| Command | Result |
| --- | --- |
| `node ~/.codex/get-shit-done/bin/gsd-tools.cjs verify lifecycle 07 --require-plans` | Passed after metadata repair |
| Task 1 fixed-string checks and `cargo about generate about.hbs --output-file docs/release/cargo-about.html` | Passed |
| Task 2 fixed-string operator-guide checks | Passed |
| Task 3 fixed-string license/provenance/evidence checks | Passed |
| `rg -n --fixed-strings "verified" docs/parity/evidence/phase-07-ota-filesystem-release.md` | No matches |
| Combined plan-level fixed-string checks plus `git diff --check` | Passed |
| `cargo fmt --all` | Passed before each task commit |
| `cargo clippy --all-targets --all-features -- -D warnings` | Passed before each task commit |
| `cargo build --all-targets --all-features` | Passed before each task commit |
| `cargo test --all-features` | Passed before each task commit |

## Decisions Made

- `cargo-about` remains a Cargo dependency report only. Release compliance still requires non-Cargo inventory rows for Bazel, ESP-IDF, flashing tools, static assets, and final artifacts.
- `about.toml` uses only the license identifiers observed in the current graph and compatible with the project’s MIT-first/GPL-guardrail posture.
- The evidence record uses explicit gap/not-run language for OTAWWW, rollback/recovery, and interrupted-update cases so later plans must add evidence before release claims.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] Repaired prior summary lifecycle metadata**

- **Found during:** Pre-execution lifecycle validation
- **Issue:** `verify lifecycle 07 --require-plans` failed because `07-02-SUMMARY.md` was missing `generated_at`.
- **Fix:** Added `generated_at: 2026-06-28T15:51:40Z` matching the prior summary completion timestamp.
- **Files modified:** `.planning/phases/07-ota-filesystem-and-release-packaging/07-02-SUMMARY.md`
- **Verification:** Lifecycle validation passed before any 07-03 deliverable edits.
- **Committed in:** final metadata commit for this plan

**2. [Rule 3 - Blocking] Installed cargo-about with required CLI feature**

- **Found during:** Task 1
- **Issue:** The plan-specified `cargo install cargo-about --locked --version 0.9.0` completed without installing a binary because `cargo-about 0.9.0` requires the `cli` feature for `cargo-about`.
- **Fix:** Re-ran installation as `cargo install cargo-about --locked --version 0.9.0 --features cli`.
- **Files modified:** None; local tool installation only.
- **Verification:** `cargo about --version` returned `cargo-about 0.9.0`, and report generation passed.
- **Committed in:** Not applicable - local tool installation

### Process Adjustments

- Ran the full Rust pre-commit sequence before each docs/config task commit because `AGENTS.md` requires it before any commit in this Rust repo.

***

**Total deviations:** 2 auto-fixed (2 blocking)
**Impact on plan:** Both fixes were required to satisfy the lifecycle gate and REL-05 report generation. No runtime firmware, API, or release behavior was expanded.

## Issues Encountered

None unresolved.

## Known Stubs

None. The new evidence and release documents intentionally record pending evidence states; they are release evidence contracts, not placeholder UI or unwired code.

## Threat Flags

None. The added files document planned release trust boundaries and do not introduce new network endpoints, auth paths, file-access code, schema changes, partition writes, or firmware effects.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

Ready for `07-04-PLAN.md`. Later Phase 7 plans can append package, static/SPIFFS, OTA, OTAWWW gap, rollback/recovery, interrupted-update, license, and provenance evidence into the fixed sections created here.

## Self-Check: PASSED

- Found summary file: `.planning/phases/07-ota-filesystem-and-release-packaging/07-03-SUMMARY.md`
- Found created files: `about.toml`, `about.hbs`, `docs/release/cargo-about.html`, `docs/release/ultra-205.md`, `docs/release/license-inventory.md`, `docs/release/provenance-manifest.md`, `docs/parity/evidence/phase-07-ota-filesystem-release.md`
- Found modified lifecycle repair file: `.planning/phases/07-ota-filesystem-and-release-packaging/07-02-SUMMARY.md`
- Found task commits: `9127943`, `2befb58`, `a619d8d`
- Focused stub scan found only required operator-guide wording that OTAWWW is "not available" for parity until D-16 evidence closes; this is intentional gap documentation, not a placeholder or unwired UI stub.
- Threat surface scan found documentation references to routes, partitions, erases, and release trust boundaries only; no new runtime endpoint, auth path, filesystem implementation, schema, partition write, or firmware effect was introduced.

---

*Phase: 07-ota-filesystem-and-release-packaging*
*Completed: 2026-06-28*
