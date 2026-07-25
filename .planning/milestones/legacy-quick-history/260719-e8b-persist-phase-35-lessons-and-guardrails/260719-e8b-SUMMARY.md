---
quick_id: 260719-e8b
status: complete
completed: 2026-07-19
task_commits:
  - c48252991ca986d6854489beac8b7357bc55409e
  - 9eb1aca55810085f9c3475a774d5def6e7664008
requirements_completed: []
---

# Quick Task 260719-e8b Summary

## Outcome

Phase 35 retrospective lessons and protected-root guardrails are now durable, the hermetic suite proves pre-effect existing-child rejection and private caller/supervisor ownership ordering, and two cross-project lessons remain confined to the external global ledger.

## Performance

- **Duration:** 50 minutes
- **Started:** 2026-07-19T15:18:23Z
- **Completed:** 2026-07-19T16:07:55Z
- **Tasks:** 3
- **Implementation files modified:** 5

## Task Commits

1. **Task 1: Persist repository lessons, guidance, and the diagnostic todo** — `c4825299`
2. **Task 2: Enforce protected-root ownership through the hermetic shell boundary** — `9eb1aca5`
3. **Task 3: Append and separately review the two global-only lessons** — external file only; intentionally not committed

## Changes

- Extended the existing real-process lesson for Bazel/runfiles execution and added four repository lessons for protected-root ownership, earliest typed-failure precedence, ESP-IDF main-task runtime capacity, and HTTP response readiness.
- Added repo-local guidance requiring a mode-0700 protected parent, an absent supervisor-owned child, mode-0600 sibling wrapper outputs, pre-admission existing-child rejection, and no ambiguous same-boundary Phase 35 retry.
- Added a pending, non-authorizing HTTP boundary diagnostic todo that separates connection, request, response, body, and schema outcomes.
- Added fresh-process hermetic regressions proving existing-child rejection before package admission or later commands, sentinel preservation, caller-before-supervisor ordering, and recursive 0700/0600 permissions.
- Added global-only lessons for diagnostic completeness before costly external attempts and zsh lowercase `path` mutation of `PATH`.

## Verification

- Before each repository commit, the mandatory Rust sequence passed: `cargo fmt --all`, Clippy with denied warnings, all-target/all-feature build, and all-feature tests.
- Bash syntax, shfmt, ShellCheck, reference cleanliness, parity validation, exact Phase 35 lifecycle validation, diff checks, and mode-0600 added-line denylist scans passed.
- The complete `//scripts:phase35_correlated_evidence_test` Bazel target passed in 34.4 seconds after the stable checked-in fixture dispatcher replaced generated executable stubs.
- Task 1 preserved the managed Bright Builds block byte-for-byte.
- Task 2 changed only `scripts/phase35-correlated-evidence-test.sh`; production Phase 35 helpers and all evidence/checklist/summary/verification truth remained unchanged.
- The two global blocks each contain exactly one numbered Date, What went wrong, Preventive rule, and Trigger signal field; their separate mode-0600 extraction passed the existing denylist and remains outside Git.
- Git history contains exactly the two approved implementation commits with the validated messages.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] Replaced generated executable test stubs with a stable checked-in dispatcher**

- **Found during:** Task 2 authoritative hermetic verification
- **Issue:** Newly generated shell executables stalled at the macOS launch boundary until Bazel's 300-second timeout, even though the ownership assertions had passed.
- **Fix:** Kept fresh processes and deployed runfiles behavior, used symlinks to the stable checked-in test entrypoint, and added a dedicated environment-gated basename dispatcher reproducing the prior flash, parity, and blocked-tool semantics.
- **Files modified:** `scripts/phase35-correlated-evidence-test.sh`
- **Verification:** Bounded dispatcher sanity passed; the complete hermetic Bazel target then passed in 34.4 seconds.
- **Committed in:** `9eb1aca5`

**Total deviations:** 1 auto-fixed blocking issue.

## Issues Encountered

- Two pre-fix authoritative Bazel runs reached the established 300-second timeout at generated-script launch boundaries without assertion failures. Read-only process inspection isolated the boundary, and bounded sanity checks disproved the first direct-shebang hypothesis before the stable-dispatcher repair was implemented.

## Scope Confirmation

No hardware, detector, HTTP request, credential access, PATCH, reboot, evidence promotion, checklist edit, Phase 35 summary or verification-truth edit, archived-lineage action, direct UART/pin action, `.planning/LEARNINGS.md` creation, GSD global-learning change, push, ROADMAP update, or STATE update occurred.

## User Setup Required

None.

## Self-Check: PASSED

The summary, both repository commits, both global lesson blocks, absence of `.planning/LEARNINGS.md`, exact two-commit history, and final diff checks were verified.
