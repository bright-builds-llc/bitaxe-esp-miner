---
phase: quick-260720-wfa
plan: "260720-wfa"
subsystem: hardware-policy
tags: [hardware, progress-gates, policy, bazel, phase35]
requires:
  - phase: 35
    provides: "Sealed attempts 1-12 and the software-only Phase 35 preflight contract"
provides:
  - "Canonical repository-wide progress-gated hardware attempt policy"
  - "Hermetic contract coverage for policy, guidance, authority, and safety invariants"
  - "Verified inert exact-current-HEAD Phase 35 preflight"
affects: [hardware-workflows, phase35, evidence-policy, agent-guidance]
tech-stack:
  added: []
  patterns:
    - "Hardware attempts require new information and exactly one typed outcome"
    - "Policy prose is enforced through a hermetic Bazel contract test"
key-files:
  created:
    - docs/hardware/hardware-attempt-policy.md
    - scripts/hardware-attempt-policy-contract-test.sh
  modified:
    - AGENTS.md
    - .codex/tasks/lessons.md
    - BUILD.bazel
    - scripts/BUILD.bazel
    - .planning/STATE.md
    - .planning/phases/35-detector-gated-correlated-evidence-and-exact-parity-promotion/35-CONTEXT.md
    - .planning/phases/35-detector-gated-correlated-evidence-and-exact-parity-promotion/35-04-PLAN.md
    - .planning/phases/35-detector-gated-correlated-evidence-and-exact-parity-promotion/35-VALIDATION.md
key-decisions:
  - "Use progress gates instead of a fixed attempt cap or unchanged blind retries."
  - "Preserve the established public preflight interface while relying on its private seal and regression test for the no-effects contract."
patterns-established:
  - "Fresh hardware attempt ordinals require a verified fix, manual remediation, or other material new information."
  - "Repeated typed boundaries after a targeted verified fix stop further autonomous attempts."
requirements-completed: []
generated_by: gsd-execute-plan
lifecycle_mode: quick-full
phase_lifecycle_id: quick-260720-wfa
generated_at: 2026-07-21T05:00:21Z
duration: 17m
completed: 2026-07-21
---

# Quick Task 260720-wfa: Progress-Gated Hardware Attempt Policy Summary

Repository-wide hardware attempts now require demonstrable progress, immutable evidence roots, exact preflight identity, and one stable terminal outcome before another attempt can be considered.

## Performance

- **Duration:** 17m
- **Started:** 2026-07-21T04:43:51Z
- **Completed:** 2026-07-21T05:00:21Z
- **Tasks:** 3
- **Implementation files:** 6

## Accomplishments

- Defined the canonical policy for progress-gated hardware attempts, including all seven stable outcomes and explicit repeated-boundary stop behavior.
- Added concise repository guidance and an append-only lesson while preserving the managed Bright Builds block byte-for-byte.
- Added a non-echoing hermetic shell contract test and Bazel wiring covering progress, evidence, authority, privacy, and fault-test safety requirements.
- Passed the full targeted software suite and an inert exact-current-HEAD Phase 35 preflight without detector, credential, port, device, or hardware arguments.
- Synchronized Phase 35 and project state so attempts 1-12 remain immutable, attempt 13 is authorized after a fresh exact-head gate, and later ordinals require a positive canonical progress decision.

## Task Commits

1. **Task 1: Define the repository-wide policy and guidance** - `f481cf4a18b3eb5683434109885949b536d6a049`
1. **Task 2: Enforce the policy contract in Bazel** - `fc5c2f8d600af6819bb3ef8f6215a456d94f7b48`
1. **Task 3: Verify the integrated contract and inert preflight** - validation-only task; no implementation commit

The quick-task summary is intentionally uncommitted for orchestrator handoff.

## Decisions Made

- Hardware progress is governed by evidence and typed outcomes, not a fixed numeric attempt limit.
- An unchanged retry is prohibited; a fresh ordinal requires a verified fix, manual remediation, or other material new information.
- The established preflight public output remains unchanged. Its private seal contract and regression test prove that effects remain disabled.
- Phase and command owners retain authority over recovery, evidence, and safe fault-test limits; this policy does not expand hardware permissions.

## Deviations from Plan

- Scoped bare Markdown formatting to the new policy because formatting the unchanged AGENTS/lesson baselines would rewrite managed and historical content; byte-preservation, append-only, structure, redaction, and diff checks covered those localized edits instead.
- Used the established public preflight markers `status=preflight_passed` and `current_head_equal=true`; the existing regression and private seal continue to prove the no-effects boundary without a production change.

## Issues Encountered

- Bare Markdown formatting checks would have rewritten unchanged managed or historical text. Verification was scoped to the new policy plus byte-preservation and append-only checks for existing files.
- Phase 35 preflight exposes `status=preflight_passed` publicly while keeping no-effects markers in its private seal. The accepted verification used the established interface and its regression contract without changing production code.

## Verification

- `mdformat --check docs/hardware/hardware-attempt-policy.md`
- `bash -n`, `shfmt -d`, and `shellcheck` for `scripts/hardware-attempt-policy-contract-test.sh`
- Direct contract script execution and Bazel contract target
- Targeted Bazel suite covering settings durability, Phase 35 boundary reads and correlation/promotion contracts, Phase 30 no-promotion, redaction, flash tooling, and parity tooling: 9/9 passed
- `just verify-redaction`, `just verify-reference`, and `just parity`
- Lifecycle ID/mode validation and `git diff --check`
- Required Rust sequence: format, clippy with warnings denied, all-target/all-feature build, and all-feature tests
- Inert `just phase35-evidence preflight-only=true` against the exact current HEAD, with private mode-0600 capture and no hardware-related inputs

## Known Stubs

None.

## Next Phase Readiness

- The policy, enforcement tooling, Phase 35 planning, and project state are synchronized for the fresh attempt-13 continuation.
- Sealed attempts 1-12 remain unchanged. Attempt 13 is authorized after a fresh exact-current-HEAD gate, but no attempt 13 hardware execution occurred in this quick task.
- Project state and Phase 35 planning now carry the progress-gated authority. ROADMAP, hardware evidence, admitted evidence, checklist truth, and `35-04-SUMMARY.md` remain untouched.

## Self-Check: PASSED

- All six implementation files exist.
- Both required implementation commits exist and contain the exact planned scopes.
- Exactly two implementation commits were created.
- This summary exists only as an uncommitted orchestrator handoff artifact.
