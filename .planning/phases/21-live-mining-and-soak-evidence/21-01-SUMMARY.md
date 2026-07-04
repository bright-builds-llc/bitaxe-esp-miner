---
phase: 21-live-mining-and-soak-evidence
plan: "01"
subsystem: phase21-live-mining-validation-foundation
tags:
  - mining-allow
  - phase21
  - evidence
  - redaction
  - wrapper
dependency_graph:
  requires:
    - phase-15-controlled-mining-evidence
    - phase-20-active-safety-telemetry-boundary
  provides:
    - phase21-command-shape-validation
    - phase21-evidence-wrapper
    - phase21-evidence-contract
    - phase21-redaction-scaffold
    - phase21-readiness-audit
  affects:
    - tools/parity/src/mining_allow.rs
    - scripts/phase21-live-mining-evidence.sh
    - docs/parity/evidence/phase-21-live-mining-and-soak-evidence
tech_stack:
  added: []
  patterns:
    - allow-manifest command-shape validation
    - gated shell evidence wrapper
    - redacted committed evidence scaffolding
key_files:
  created:
    - scripts/phase21-live-mining-evidence.sh
    - scripts/phase21-live-mining-evidence-test.sh
    - docs/parity/evidence/phase-21-live-mining-and-soak-evidence/evidence-contract.md
    - docs/parity/evidence/phase-21-live-mining-and-soak-evidence/redaction-review.md
    - docs/parity/evidence/phase-21-live-mining-and-soak-evidence/readiness-audit.md
    - .planning/phases/21-live-mining-and-soak-evidence/21-01-SUMMARY.md
  modified:
    - tools/parity/src/mining_allow.rs
    - scripts/BUILD.bazel
    - .planning/phases/21-live-mining-and-soak-evidence/21-VALIDATION.md
decisions:
  - Phase 21 mining commands are accepted only when the allow manifest matches the phase-owned wrapper shape.
  - Missing live prerequisites produce blocked or pending evidence, not controlled-no-share claims.
  - Live mining smoke and bounded soak require both controlled package readiness and controlled runtime harness readiness.
  - Plan 21-01 records firmware live mining as blocked by default and leaves runtime enablement to later plans.
metrics:
  tasks_completed: 3
  task_commits: 3
  verification_status: passed
  completed_at: 2026-07-04T04:01:19Z
  duration: about 30 minutes
requirements_completed:
  - ASIC-07
  - STR-06
  - STR-07
  - SAFE-09
  - EVD-05
generated_by: gsd-execute-plan
lifecycle_mode: yolo
phase_lifecycle_id: 21-2026-07-04T01-35-47
generated_at: 2026-07-04T04:01:19Z
---

# Phase 21 Plan 01: Live Mining Validation Foundation Summary

Phase 21 now has command-shape gates, a phase-owned evidence wrapper, redaction scaffolding, and a blocked-by-default readiness audit before any live mining or bounded soak execution.

## Tasks Completed

| Task | Result | Commit |
|------|--------|--------|
| 1. Extend mining allow validation | Added Phase 21 wrapper command acceptance, bounded duration matching, required schema fields, unsafe-token rejection, and tests. | `a89c6ef` |
| 2. Add Phase 21 evidence wrapper | Added `scripts/phase21-live-mining-evidence.sh`, its Bazel shell test, explicit target handling, redaction, missing-prerequisite blocking, and default bounded-soak behavior. | `e6ea0b3` |
| 3. Add evidence scaffolds | Added the evidence contract, redaction review scaffold, readiness audit, and green Wave 0 validation rows without claiming live mining enablement. | `6f2613b` |

## Verification

- `cargo test -p bitaxe-parity --all-features mining_allow` passed.
- `bazel test //tools/parity:tests --test_filter=mining_allow` passed.
- `bash -n scripts/phase21-live-mining-evidence.sh scripts/phase21-live-mining-evidence-test.sh` passed.
- `bazel test //scripts:phase21_live_mining_evidence_test` passed.
- Task 3 acceptance `rg` checks passed, including the Phase 21 ladder, readiness markers, redaction scaffold, validation rows, and no standalone body `---` in new evidence docs.
- Required Rust pre-commit sequence passed before every task commit: `cargo fmt --all`, `cargo clippy --all-targets --all-features -- -D warnings`, `cargo build --all-targets --all-features`, `cargo test --all-features`.
- Plan-level checks passed: `bazel test //tools/parity:tests //scripts:phase21_live_mining_evidence_test --test_filter=mining_allow`, `just test`, `just parity`, `just verify-reference`, and `git diff -- reference/esp-miner --exit-code`.

## Deviations from Plan

### AGENTS.md-Driven Adjustment

**1. TDD RED commits were not created as separate commits**
- **Found during:** Tasks 1 and 2
- **Issue:** The GSD TDD flow asks for RED commits, but repo-local Rust rules require the full pre-commit sequence to pass before any commit. Failing RED commits would violate the higher-priority repo rule.
- **Fix:** Wrote and observed failing tests first, then committed only passing task states after the required Rust verification sequence.
- **Files affected:** `tools/parity/src/mining_allow.rs`, `scripts/phase21-live-mining-evidence-test.sh`, `scripts/BUILD.bazel`
- **Commits:** `a89c6ef`, `e6ea0b3`

### Auto-Fixed Issues

**1. [Rule 2 - Missing Critical Functionality] Required runtime harness readiness**
- **Found during:** Task 2
- **Issue:** Live mining wrapper execution must not proceed on package readiness alone; the Phase 21 plan requires a controlled runtime/harness enablement contract.
- **Fix:** Required `controlled_runtime_harness_status: ready` alongside `controlled_live_mining_package_status: ready`.
- **Files modified:** `scripts/phase21-live-mining-evidence.sh`, `scripts/phase21-live-mining-evidence-test.sh`, `docs/parity/evidence/phase-21-live-mining-and-soak-evidence/evidence-contract.md`, `docs/parity/evidence/phase-21-live-mining-and-soak-evidence/readiness-audit.md`
- **Commits:** `e6ea0b3`, `6f2613b`

## Auth Gates

None.

## Known Stubs

None. The stub scan found only shell argument-initialization variables such as `manifest=""` and test-local `out_file=""`; these do not flow to UI rendering or mock data sources.

## Threat Flags

None. The new explicit-target `curl` and WebSocket helper paths are the planned Phase 21 surface and remain behind allow-manifest, prerequisite, redaction, explicit `DEVICE_URL`, and safe-state gates.

## Blockers

None for Plan 21-01. Live mining, accepted/rejected shares, bounded soak, live API/WebSocket freshness, and watchdog behavior remain unclaimed until later Phase 21 plans produce redaction-reviewed hardware/runtime evidence.

## Changed Files

- `tools/parity/src/mining_allow.rs`
- `scripts/BUILD.bazel`
- `scripts/phase21-live-mining-evidence.sh`
- `scripts/phase21-live-mining-evidence-test.sh`
- `docs/parity/evidence/phase-21-live-mining-and-soak-evidence/evidence-contract.md`
- `docs/parity/evidence/phase-21-live-mining-and-soak-evidence/redaction-review.md`
- `docs/parity/evidence/phase-21-live-mining-and-soak-evidence/readiness-audit.md`
- `.planning/phases/21-live-mining-and-soak-evidence/21-VALIDATION.md`

## Next Step

Proceed to Plan 21-02 to produce the controlled live-mining package/runtime enablement pack required before any live mining smoke or bounded soak attempt.

## Self-Check: PASSED

- Created files exist: wrapper, wrapper test, evidence contract, redaction review, readiness audit, and summary.
- Task commits exist: `a89c6ef`, `e6ea0b3`, `6f2613b`.
- Summary frontmatter uses only the opening and closing `---` delimiters.
