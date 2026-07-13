---
phase: 23-redacted-operator-evidence-workflow
plan: 03
subsystem: scripts
tags: [just, bazel, operator-evidence, detector-gate, redaction]
requires:
  - phase: 23-redacted-operator-evidence-workflow
    provides: evidence-root contract and operator-evidence validator
provides:
  - `just phase23-evidence` command surface
  - Bazel `phase23_redacted_operator_evidence` shell binary
  - Bazel `phase23_redacted_operator_evidence_test` synthetic workflow test
  - Fail-closed hardware-mode detector behavior and category-only committed slot writing
affects: [operator-runbook, phase-24-bm1366-production-path, phase-25-live-stratum-runtime, parity-checklist]
tech-stack:
  added: []
  patterns: [thin shell orchestration, env-overridable test seams, blocked-mode static workflow proof]
key-files:
  created:
    - scripts/phase23-redacted-operator-evidence.sh
    - scripts/phase23-redacted-operator-evidence-test.sh
  modified:
    - Justfile
    - scripts/BUILD.bazel
key-decisions:
  - "Hardware mode starts with `just detect-ultra205` and exits non-zero with blocked detector and board-info slots when detection fails."
  - "Credential path arguments are treated as runtime-only presence signals; committed evidence records labels, never file paths or contents."
  - "The script delegates final evidence-root decisions to `tools/parity operator-evidence`."
requirements-completed: [EVD-07, STR-10, REL-09, CFG-07, EVD-09]
generated_by: gsd-execute-plan
lifecycle_mode: yolo
phase_lifecycle_id: 23-2026-07-04T22-53-37
generated_at: 2026-07-04T23:20:00Z
duration: 16min
completed: 2026-07-04
---

# Phase 23 Plan 03: Operator Workflow Shell Summary

**Bazel-owned `just phase23-evidence` workflow with detector-gated hardware mode and synthetic redaction tests.**

## Accomplishments

- Added `scripts/phase23-redacted-operator-evidence.sh` with `blocked` and `hardware` modes, required argument parsing, category-only credential labels, target-source blockers, slot writing, and validator invocation.
- Registered the wrapper through `Justfile` and `scripts/BUILD.bazel`.
- Added `scripts/phase23-redacted-operator-evidence-test.sh` covering blocked-mode slot creation, sentinel absence, and detector-failure fail-closed behavior without real credentials or hardware.

## Deviations from Plan

- The validator command is configurable through `PHASE23_PARITY_COMMAND` so the sh_test can avoid nested Bazel execution while the default operator path still invokes `bazel run //tools/parity:report --`.

## Verification

- `bazel test //scripts:phase23_redacted_operator_evidence_test //tools/parity:tests`
- `bazel run //tools/parity:report -- operator-evidence --evidence-root docs/parity/evidence/phase-23-redacted-operator-evidence-workflow --require-redaction-passed`

## User Setup Required

Optional hardware mode still requires a connected Ultra 205 and local untracked credential files when the operator chooses to supply them.
