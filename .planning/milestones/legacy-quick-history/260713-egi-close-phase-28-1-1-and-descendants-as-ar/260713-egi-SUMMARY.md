---
quick_id: 260713-egi
status: complete
completed: 2026-07-13
commit: 1be80d1
repair_commit: 2285ebe
---

# Quick Task 260713-egi Summary

## Outcome

Phase 28.1.1 and descendants 28.1.1.1 through 28.1.1.7 are terminal archived unresolved history. All eight lifecycles validate from `.planning/milestones/v1.1-phases/`, every verification remains `gaps_found`, and STR-09, CFG-07, and ASIC-11 remain pending. Phase 30 is the only active continuation and carries a conservative no-promotion contract unless explicitly supplied new eligible evidence passes the existing gates.

## Changes

- Restored the root lifecycle ID, refreshed truthful closure verification metadata, fixed a pre-existing Phase 28.1.1.5 frontmatter-body separator defect, and validated exact plan/summary counts before and after archival.
- Archived all eight lineage directories and moved six related debug records into the archived root; the two open sessions now use `closed_wont_do_unresolved`.
- Updated ROADMAP, STATE, config, todos, Phase 29's canonical reference, Phase 29 re-verification metadata, the Phase 30 immutability helper, and repository guidance.
- Added one shared exit-64 guard to all twenty historical hardware/capture surfaces and both public Just recipes, plus direct and Bazel-owned regression coverage. Pure classifiers, state helpers, comparators, and protected-evidence logic remain executable.
- Extended `.gitignore` for archived local `hardware-runs/` so preserved ignored user data remains uninspected and untracked after directory archival.

## Verification

- All eight archived lifecycle validations and Phase 29 lifecycle validation passed with exact lifecycle IDs and historical counts.
- GSD yolo target `discuss`, `chain`, and `push` select Phase 30; roadmap/progress select Phase 30; audit-uat reports no active lineage work.
- The twenty-surface/two-Just guard regression, twelve terminal launcher tests, immutability tests, bash syntax, shfmt, shellcheck, and targeted Bazel tests passed.
- `just verify-reference` passed at pinned reference `c1915b0a63bfabebdb95a515cedfee05146c1d50`.
- Mandatory Rust sequence passed in order: fmt, Clippy with denied warnings, all-target/all-feature build, and all-feature tests.
- Frontmatter, archived reference resolution, non-promotion, diff, and protected-evidence archive-path checks passed.

## Deviation

The approved plan expected eight W006 warnings. The installed GSD introduced no W006, so health remains degraded only by the pre-existing Phase 01-21 W007 warnings. Its archive behavior is split: canonical `find-phase 28.1.1*` returns `found: false`, and `init phase-op 28.1.1*` reports `phase_found: true` with `phase_dir: null`, while `verify lifecycle` resolves all eight milestone archives and passes. Guidance records both this installed behavior and the W006 cross-version exception without recreating active directories, promoting verification, patching global GSD core, or permitting explicit lineage operations; Phase 30 remains the sole continuation.

## Repair Follow-Up

Forward commit `2285ebe` repaired body separator frontmatter hazards in twenty archived summaries, added ShellCheck source metadata to all twenty terminally guarded entrypoints, closed stale todo retry language, and documented the installed-GSD archive lookup split. The full shell, Bazel, routing, lifecycle, reference, non-promotion, diff, and mandatory Rust gates passed after the repair.

## Residual Risk

Nonce production, hashing-capable state, a correlated BM1366 result, and an accepted/rejected live share remain unverified by design. Phase 30 must preserve that no-promotion outcome unless new evidence is explicitly supplied and validated.
