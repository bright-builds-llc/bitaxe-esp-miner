---
phase: 29-evidence-workflow-automation-closure
plan: "02"
subsystem: evidence-workflow
tags:
  - bash
  - bazel
  - operator-evidence
  - atomic-consolidation
requires:
  - phase: 29-evidence-workflow-automation-closure
    plan: "01"
    provides: typed Phase 23/25/27/28 profiles, completion, and atomic consolidation core
provides:
  - explicit profile validation across Phase 23, 25, and 27 wrappers
  - single terminal finalizers preserving workflow and validator failures
  - Bazel-owned Phase 28 consolidation and strict validation command
  - deterministic wrapper regressions for all closed Phase 28 outcomes
affects:
  - 29-03 operator documentation and regression closure
  - Phase 30 evidence promotion inputs
tech-stack:
  added: []
  patterns:
    - one terminal evidence finalizer after workflow setup
    - redaction-safe category-only command traces
    - thin shell wrapper delegating atomic promotion to typed Rust
key-files:
  created:
    - scripts/phase28-evidence.sh
    - scripts/phase28-evidence-test.sh
  modified:
    - scripts/phase23-redacted-operator-evidence.sh
    - scripts/phase23-redacted-operator-evidence-test.sh
    - scripts/phase25-live-stratum-evidence.sh
    - scripts/phase25-live-stratum-evidence-test.sh
    - scripts/phase27-live-hardware-bridge-evidence.sh
    - scripts/phase27-live-hardware-bridge-evidence-test.sh
    - scripts/BUILD.bazel
    - Justfile
key-decisions:
  - Blocked, prerequisite-missing, detector-failed, and capture-failed workflows retain a non-passing workflow status even when terminal validators pass.
  - Phase 25 and Phase 27 run completion first, applicable mining-allow second, and strict operator validation exactly once and last.
  - The Phase 28 shell surface accepts only two normalized repo-relative roots and delegates relationship checks, rendering, staging, and atomic replacement to the typed Rust core.
patterns-established:
  - "Terminal validation: accumulate workflow state, complete the canonical root, run applicable policy validation, then run strict operator validation last."
  - "Wrapper traces: record only command/profile/category state and never raw paths, credentials, endpoints, or device identifiers."
requirements-completed:
  - EVD-07
  - EVD-08
  - EVD-09
  - REL-09
duration: 14 min
completed: 2026-07-13
generated_by: gsd-execute-plan
lifecycle_mode: yolo
phase_lifecycle_id: 29-2026-07-13T00-19-45
generated_at: 2026-07-13T01:38:15Z
---

# Phase 29 Plan 02: Evidence Workflow Finalizers and Phase 28 Command Summary

**Single-finalizer Phase 25/27 workflows and a deterministic Bazel-owned Phase 28 consolidation command with strict terminal validation**

## Performance

- **Duration:** 14 min
- **Started:** 2026-07-13T01:23:54Z
- **Completed:** 2026-07-13T01:38:15Z
- **Tasks:** 2
- **Files modified:** 10

## Accomplishments

- Added explicit `phase23`, `phase25`, `phase27`, and `phase28` profile arguments at every wrapper validation boundary.
- Replaced Phase 25 and Phase 27 branch-local validator exits with one finalizer that always attempts completion, applicable mining allow, and strict operator validation while preserving every prior failure.
- Ensured detector failures, missing prerequisites, blocked mode, and prior capture failures leave a complete eleven-slot root before returning nonzero.
- Added `scripts/phase28-evidence.sh`, its Bazel binary/test targets, and `just phase28-evidence` without exposing credentials, ports, device URLs, raw logs, or promotion logic in shell.
- Added deterministic regression coverage for accepted, rejected, and blocked Phase 28 outcomes; equal/nested roots; missing or contradictory source categories; unknown destination files; source-only sentinels; command failures; and unchanged prior destinations.

## Task Commits

1. **Task 1: Route Phase 23, Phase 25, and Phase 27 through explicit profiles and terminal validation** - `295b155` (`feat`)
2. **Task 2: Add the atomic Phase 28 wrapper, Bazel targets, and Just command** - `c0f1a27` (`feat`)

## Tests and Verification

- Task 1 RED: all three wrapper Bazel targets failed after the tests began requiring explicit profiles, complete roots, and terminal validation on detector/blocked paths.
- Task 1 trace assertions require `complete-operator-evidence`, applicable `mining-allow`, then exactly one `operator-evidence` call last, with `slots=complete` at every validation event.
- Task 1 failure matrices independently force completion, mining-allow, operator, and prior-workflow failures for both Phase 25 and Phase 27; every case remains nonzero.
- Task 2 RED: `//scripts:phase28_evidence_test` failed to build because `phase28-evidence.sh` was intentionally absent.
- Task 2 traces require consolidation first and exactly one strict `operator-evidence --profile phase28` invocation last.
- Task 2 fixtures prove accepted, rejected, and `blocked_safe_prerequisite` outcomes remain unchanged, deterministic reruns retain the same digest, and source-only sentinel files never enter the destination.
- Each task passed, in order, `cargo fmt --all`, Clippy with denied warnings, all-target/all-feature build, and all-feature workspace tests before its commit.
- Plan-wide verification passed the ordered Rust gate, all five Phase 29 parity/wrapper Bazel targets, `just parity`, `just verify-reference`, and `git diff --check`.

## Files Created/Modified

- `scripts/phase28-evidence.sh` - Thin repo-relative consolidation and strict Phase 28 validation wrapper.
- `scripts/phase28-evidence-test.sh` - Category-only command trace, outcome, determinism, root relation, redaction, and destination-preservation regressions.
- `scripts/phase23-redacted-operator-evidence.sh` and paired test - Explicit Phase 23 profile plus detector-failure terminal validation.
- `scripts/phase25-live-stratum-evidence.sh` and paired test - Complete-root single finalizer and four-way failure precedence matrix.
- `scripts/phase27-live-hardware-bridge-evidence.sh` and paired test - Complete-root single finalizer and four-way failure precedence matrix.
- `scripts/BUILD.bazel` - Phase 28 shell binary and local test targets.
- `Justfile` - Human-facing `phase28-evidence` Bazel route.

## Decisions Made

- Treat blocked and missing-prerequisite outcomes as truthful non-passing workflow statuses; a successful validator cannot convert them into command success.
- Continue to attempt strict operator validation after completion or mining validation fails so the terminal evidence state and every independent failure remain observable.
- Keep the Phase 28 shell implementation at two parity calls and two input paths; all filesystem and evidence-domain behavior remains in the Plan 01 Rust core.

## Deviations from Plan

None - plan executed exactly as written.

## Issues Encountered

- The Phase 25 finalizer initially pushed its wrapper above the repository's 628-line refactor trigger. A simplification pass reduced the file to the trigger boundary without changing behavior or adding another orchestration layer.
- Bazel test size warnings remain informational for the existing shell suites; all affected targets pass.

## Known Stubs

None.

## Residual Risks

- The Phase 28 shell test uses a deterministic fake parity executable for orchestration and failure injection; the actual filesystem exchange and rollback implementation is independently exercised by `//tools/parity:tests` on this macOS host.
- Linux `renameat2(..., RENAME_EXCHANGE)` remains cfg-gated and was not runtime-exercised on this host; the matching Rust regression runs when the suite executes on Linux.
- No committed Phase 27 evidence root was regenerated during this plan, avoiding mutation of historical evidence. Plan 03 documentation/regression closure remains responsible for the final operator-facing integration proof.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

- Plan 29-03 can document `just phase28-evidence`, automatic Phase 25/27 finalization, and the tested failure/ordering contract.
- Phase 30 receives explicit, validated evidence roots without any new hardware or parity promotion claim from this plan.
- No hardware, credentials, raw private evidence, reference modifications, direct UART, or pin interaction occurred.

## Self-Check: PASSED

- Confirmed both created Phase 28 scripts exist and are executable.
- Confirmed task commits `295b155` and `c0f1a27` exist in repository history.
- Confirmed the summary lifecycle matches `lifecycle_mode: yolo` and `phase_lifecycle_id: 29-2026-07-13T00-19-45`.
- Confirmed `.planning/STATE.md` and `.planning/ROADMAP.md` remain unstaged and outside Plan 02 commits.

***

*Phase: 29-evidence-workflow-automation-closure*
*Completed: 2026-07-13*
