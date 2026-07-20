---
phase: quick-260719-tfu
plan: "01"
subsystem: hardware-evidence-workflow
tags: [rust, bash, curl, http, redaction, phase35]
requires:
  - phase: 35-detector-gated-correlated-evidence-and-exact-parity-promotion
    provides: Correlated evidence supervisor and conservative non-promotion contract
provides:
  - Strict pure Rust classification for Phase 35 HTTP setting-read boundaries
  - One-request private curl adapter shared by original, immediate, and restoration reads
  - Capture-once primary failure precedence with secondary-only finalization categories
affects: [phase35, hardware-evidence, parity, operator-diagnostics]
tech-stack:
  added: []
  patterns: [functional-core-imperative-shell, strict-redacted-projection, capture-once-primary-failure]
key-files:
  created:
    - tools/parity/src/phase35_http.rs
    - tools/parity/src/phase35_http/tests.rs
    - scripts/phase35-http-boundary-read.sh
    - scripts/phase35-http-boundary-read-test.sh
  modified:
    - tools/parity/src/main.rs
    - tools/parity/BUILD.bazel
    - scripts/phase35-correlated-evidence.sh
    - scripts/phase35-correlated-evidence-effects.sh
    - scripts/phase35-correlated-evidence-fixture.sh
    - scripts/phase35-correlated-evidence-test.sh
    - scripts/BUILD.bazel
    - .codex/tasks/todo.md
key-decisions:
  - "Malformed or inconsistent shell observations persist a minimal http_diagnostic_invalid projection and never enter the ordered HTTP classifier."
  - "The first HTTP or supervisor failure remains authoritative; restoration and cleanup categories are secondary-only."
patterns-established:
  - "Phase 35 setting reads use one direct bounded HTTP/1.1 GET and classify the captured artifacts afterward."
  - "Only the ready result exposes a hostname, and only through a private mode-0600 file."
requirements-completed: [QUICK-TFU-01, QUICK-TFU-02, QUICK-TFU-03, QUICK-TFU-04]
generated_by: gsd-execute-plan
lifecycle_mode: direct-fallback
phase_lifecycle_id: quick-260719-tfu
generated_at: 2026-07-20T03:16:43Z
duration: 58min
completed: 2026-07-19
---

# Quick 260719-TFU: Phase 35 HTTP Boundary Instrumentation Summary

**Strict redacted HTTP-boundary classification now instruments all three Phase 35 setting reads with one bounded request while preserving the earliest primary failure through finalization.**

## Performance

- **Duration:** 58 min
- **Started:** 2026-07-20T02:18:44Z
- **Completed:** 2026-07-20T03:16:43Z
- **Tasks:** 3
- **Files modified:** 12

## Accomplishments

- Added a deny-unknown Rust observation parser, exact ordered terminal classifier, redacted projection, private-hostname separation, and `classify-phase35-http` CLI.
- Added a private mode-0700/0600 shell adapter that performs exactly one bounded HTTP/1.1 GET and validates process, metrics, timing, and artifact consistency before classification.
- Integrated original, immediate, and restoration reads through the adapter while retaining first-primary failure precedence and secondary-only restoration/cleanup outcomes.
- Closed the scoped todo only after the complete software gate and effect-free Phase 35 preflight passed.

## Task Commits

1. **Task 1: Add the strict typed HTTP classifier and redacted projection** - `c43a42a5`
2. **Task 2: Integrate one bounded curl request and preserve primary failure precedence** - `fbe6b8ff`
3. **Task 3: Run the complete software gate and close only the diagnostic todo** - `5a6e9dd0`

The quick PLAN and this SUMMARY remain uncommitted for the parent orchestrator.

## Verification

- Exact Rust pre-commit sequence passed before every commit: format, Clippy with warnings denied, all-target/all-feature build, and all-feature tests.
- Bash syntax, `shfmt`, and `shellcheck` passed for all six affected scripts.
- Direct real-process fake-curl and supervisor suites passed without production curl or network access.
- Fresh Bazel parity, adapter, supervisor, Phase 35 promotion-contract, and Phase 30 non-promotion tests passed; the correlated-evidence target built successfully.
- `just verify-reference`, `just parity`, exact Phase 35 lifecycle verification, diff checks, and added-line redaction scans passed.
- `just phase35-evidence preflight-only=true` passed without detector, target, credentials, curl, HTTP, device, or evidence effects.

## Decisions Made

- Invalid setup, write-out, process-status, bounds, and artifact facts use the separate `http_diagnostic_invalid` fallback with a persisted minimal projection.
- Original readiness is checkpointed before mutation can begin.
- Restoration and cleanup always run as finalization responsibilities, but cannot replace an existing primary category; finalization-only failure maps to `supervisor_finalization_failed`.

## Deviations from Plan

None - the plan was executed within its software-only scope. A final review added explicit regression coverage for unauthorized test-executable overrides persisting an invalid projection without invoking curl.

## Known Stubs

None.

## External and Evidence Boundaries

No hardware, detector, credentials, real network request, production curl, flash, monitor, PATCH, reboot, evidence admission or promotion, parity-row change, lifecycle-truth change, attempt documentation, push, direct UART, or pin work occurred. Attempts 1 through 10 remain sealed and immutable. Phase 35 Plan 04 Task 2 remains incomplete, and this work does not authorize another attempt or change evidence truth.

## Self-Check: PASSED

- All 12 implementation/documentation files exist.
- Commits `c43a42a5`, `fbe6b8ff`, and `5a6e9dd0` exist with the intended task scopes.
- The quick PLAN and SUMMARY are the only untracked workflow artifacts.
