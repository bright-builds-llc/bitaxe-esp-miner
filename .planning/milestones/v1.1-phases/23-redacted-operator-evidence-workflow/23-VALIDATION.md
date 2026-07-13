---
phase: 23
slug: redacted-operator-evidence-workflow
status: complete
nyquist_compliant: true
wave_0_complete: true
created: 2026-07-04
---

# Phase 23 - Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | Bazel `rust_test` for Rust tools/crates and Bazel `sh_test` for script workflow tests |
| **Config file** | `MODULE.bazel`, `Cargo.toml`, `scripts/BUILD.bazel`, `tools/parity/BUILD.bazel`, `tools/flash/BUILD.bazel` |
| **Quick run command** | `bazel test //scripts:phase23_redacted_operator_evidence_test //tools/parity:tests` |
| **Full suite command** | `just test` |
| **Estimated runtime** | ~60-180 seconds for targeted checks; full suite depends on ESP/Bazel cache state |

## Sampling Rate

- **After every task commit:** Run the task-specific targeted command from the plan.
- **After every plan wave:** Run Phase 23 targeted tests plus `just parity` and `just verify-reference` when evidence/checklist docs changed.
- **Before verification:** Run targeted tests for changed helpers, `just parity`, `just verify-reference`, deterministic redaction scan/review, and lifecycle validation.
- **Max feedback latency:** Prefer under 180 seconds for targeted checks; record any cache/tooling delay in the summary.

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Threat Ref | Secure Behavior | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|------------|-----------------|-----------|-------------------|-------------|--------|
| 23-01-01 | 01 | 1 | EVD-07, REL-09 | T-23-01 | Evidence root slot contract exists and every slot can be represented as passed, blocked, pending, or deferred without overclaiming later phases. | doc/fixture | `rg -n "package|detector|board-info|command|log|api|websocket|share-outcome|safe-stop|redaction-review|conclusion" docs/parity/evidence/phase-23-redacted-operator-evidence-workflow` | present | green |
| 23-01-02 | 01 | 1 | STR-10, CFG-07, EVD-09 | T-23-02 | Synthetic pool, owner, target, extranonce, share, socket, NVS, device, IP, MAC, Wi-Fi, and token values do not survive committed redaction output. | shell fixture | `bazel test //scripts:phase23_redacted_operator_evidence_test //tools/parity:tests` | present | green |
| 23-02-01 | 02 | 1 | EVD-07, EVD-09 | T-23-03 | Validator or script inventory rejects missing required root slots and missing redaction review. | Rust or shell unit | `bazel test //tools/parity:tests //scripts:phase23_redacted_operator_evidence_test` | present | green |
| 23-03-01 | 03 | 2 | REL-09, CFG-07 | T-23-04 | Operator runbook and command surface document detector gate, runtime-only credential category labels, blocked hardware slot behavior, and no raw secrets. | doc/static | `rg -n "just detect-ultra205|pool_config: local-owner-supplied|raw_artifacts_committed: no|redaction_status" docs/parity/evidence/phase-23-redacted-operator-evidence-workflow docs/release/ultra-205.md` | present | green |
| 23-04-01 | 04 | 2 | EVD-07, STR-10, REL-09, CFG-07, EVD-09 | T-23-05 | Final summary and parity updates promote only exact Phase 23 evidence workflow/redaction claims and preserve Phase 24/25/26 non-claims. | repo gate | `bazel test //scripts:phase23_redacted_operator_evidence_test //tools/parity:tests && bazel run //tools/parity:report -- operator-evidence --evidence-root docs/parity/evidence/phase-23-redacted-operator-evidence-workflow --require-redaction-passed && just parity && just verify-reference && node "$HOME/.cursor/get-shit-done/bin/gsd-tools.cjs" verify lifecycle 23 --expect-id 23-2026-07-04T22-53-37 --expect-mode yolo --require-plans` | present | green |

*Status: pending, green, red, flaky*

## Wave 0 Requirements

- [x] `scripts/phase23-redacted-operator-evidence.sh` or equivalent repo-owned command surface.
- [x] `scripts/phase23-redacted-operator-evidence-test.sh` registered in `scripts/BUILD.bazel` as `phase23_redacted_operator_evidence_test`.
- [x] Evidence-root contract files under `docs/parity/evidence/phase-23-redacted-operator-evidence-workflow/`.
- [x] Validator or deterministic inventory check for required evidence root slots.
- [x] Synthetic redaction fixture coverage for Phase 23 forbidden categories.

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| Optional detector-gated hardware run with local credentials | REL-09, CFG-07 | Requires connected board state and optional local credential files that must not be read or exposed by agents. | Run only after `just detect-ultra205` succeeds exactly once; use repo-owned command with redacted evidence enabled; commit only redacted/category artifacts. |
| Review of raw local artifacts before deletion or promotion | STR-10, EVD-09 | Raw local artifacts, if any, may contain secrets and cannot be safely inspected or summarized in chat. | Operator or agent reviews locally without printing values; committed root records only category labels and `raw_artifacts_committed: no`. |

## Validation Sign-Off

- [x] All tasks have automated verification or Wave 0 dependencies.
- [x] Sampling continuity: no 3 consecutive tasks without automated verify.
- [x] Wave 0 covers all missing references.
- [x] No watch-mode flags.
- [x] Feedback latency target recorded for targeted checks.
- [x] `nyquist_compliant: true` set in frontmatter after implementation proves the validation map is satisfied.

**Approval:** green
