---
phase: quick
plan: "01"
quick_task: 260719-g3g-add-deterministic-lesson-loading-and-mai
subsystem: agent-instructions
tags: [lessons, audit, deterministic-loading, guardrails]
requires: []
provides:
  - Deterministic global and repository active-lesson loading policy
  - Append-only global and repository lesson-audit baselines
affects: [agent-startup, lesson-maintenance, lesson-audits]
tech-stack:
  added: []
  patterns: [bounded complete-block loading, explicit audit triggers, loss-averse archival]
key-files:
  created:
    - .codex/tasks/lesson-audits.md
    - /Users/peterryszkiewicz/.codex/tasks/lesson-audits.md
  modified:
    - AGENTS.md
    - /Users/peterryszkiewicz/.codex/AGENTS.md
key-decisions:
  - "Complete loading requires both the 24,000-byte and per-file-summed 8,000-token-estimate limits to pass."
  - "Age and size alone never justify lesson consolidation or archival."
requirements-completed: [QUICK-G3G-01, QUICK-G3G-02, QUICK-G3G-03]
generated_by: gsd-execute-plan
lifecycle_mode: direct-fallback
phase_lifecycle_id: quick-260719-g3g
generated_at: 2026-07-19T17:02:34Z
duration: 8 min
completed: 2026-07-19
---

# Quick 260719-g3g: Deterministic Lesson Loading and Audit Baselines

**Bounded, block-safe lesson loading with explicit non-recursive audit triggers and loss-averse lifecycle rules across global and repository instructions**

## Performance

- **Duration:** 8 min
- **Started:** 2026-07-19T16:54:17Z
- **Completed:** 2026-07-19T17:02:34Z
- **Tasks:** 3
- **Files modified:** 5, including this uncommitted summary

## Accomplishments

- Added the same deterministic active-lesson resolution, byte measurement, per-file ceiling estimate, complete-read, and over-budget block-selection contract to global and repository instructions.
- Defined five explicit audit triggers, prevented recursive 75% audit churn, and protected safety, privacy, security, authorization, and evidence guardrails from loss.
- Recorded separate append-only audit baselines retaining all 17 repository and 4 global lesson IDs at 18,167 combined bytes and 6,056 summed estimated tokens, with no consolidation, archival, or archive files.
- Verified below-budget, byte-over-budget, token-over-budget, missing-repository, global-only, heading-inventory, priority-order, and complete-block selection cases.

## Task Commit

All three plan tasks were delivered in the single atomic repository implementation commit required by the plan:

- `713fe593` — `feat(quick-260719-g3g): add deterministic lesson loading`

The commit contains exactly `AGENTS.md` and `.codex/tasks/lesson-audits.md`. The global instruction and audit files remain outside Git, and this summary remains uncommitted for the orchestrator.

## Files Created/Modified

- `AGENTS.md` — Repository-local deterministic lesson loading and lifecycle policy outside the managed Bright Builds block.
- `.codex/tasks/lesson-audits.md` — Repository audit baseline retaining all 21 active lesson IDs.
- `/Users/peterryszkiewicz/.codex/AGENTS.md` — Matching global lesson loading and lifecycle policy.
- `/Users/peterryszkiewicz/.codex/tasks/lesson-audits.md` — Separate global audit baseline.
- `.planning/quick/260719-g3g-add-deterministic-lesson-loading-and-mai/260719-g3g-SUMMARY.md` — Uncommitted execution handoff.

## Decisions Made

- Missing active files count as zero; canonical paths are de-duplicated and archives are excluded.
- Complete startup loading occurs if and only if both limits pass.
- Over-budget loading inventories every heading before selecting unsplit lesson blocks by the prescribed priority order.
- Audit baselines record hashes as well as counts, sizes, estimates, timestamps, and threshold state so every future trigger can be evaluated deterministically.

## Verification

- Lesson integrity: exactly 17 repository and 4 global unique blocks, each with date, what went wrong, preventive rule, and trigger signal.
- Policy and audit calculation checks: passed.
- Managed Bright Builds block equality: passed.
- PLAN standalone frontmatter delimiter count and plan-schema validation: passed.
- `git diff --check`: passed.
- `just verify-reference`: passed at pinned reference `c1915b0a63bfabebdb95a515cedfee05146c1d50`.
- `just parity`: passed with `validation_errors: none`.
- Phase 35 lifecycle validation: passed.
- Redaction, separate global/repository diff, backup-mirror cleanliness, no-archive, and disabled global-learnings reviews: passed.
- Ordered Rust sequence passed: `cargo fmt --all`; `cargo clippy --all-targets --all-features -- -D warnings`; `cargo build --all-targets --all-features`; `cargo test --all-features`.

## Deviations from Plan

None - plan executed exactly as written.

## Issues Encountered

- The initial quick PLAN frontmatter lacked the generic plan-schema `phase` field. The orchestrator corrected the planning artifact without changing implementation scope; schema validation then passed.

## Authentication Gates

None.

## Known Stubs

None.

## User Setup Required

None.

## Next Phase Readiness

- Future agents can load both current active lesson files completely under the recorded baseline.
- Any later audit can compare the baseline timestamp, active IDs, counts, source hashes, byte totals, summed estimate, and consumed 75% crossing without rereading archive content.

## Self-Check

PASSED

- All four implementation files and this summary exist.
- Commit `713fe593` exists and contains exactly the two authorized repository implementation files.
- The global files and this summary remain outside the repository commit.
