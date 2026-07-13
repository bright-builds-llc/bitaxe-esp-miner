---
phase: 29
slug: evidence-workflow-automation-closure
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-07-12
---

# Phase 29 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

***

## Test Infrastructure

| Property | Value |
| --- | --- |
| **Framework** | Rust built-in tests plus repo-owned Bash regression harnesses under Bazel |
| **Config file** | `Cargo.toml`, `tools/parity/BUILD.bazel`, `scripts/BUILD.bazel` |
| **Quick run command** | `cargo test -p bitaxe-parity --all-features operator_evidence` |
| **Full suite command** | `bazel test //tools/parity:tests //scripts:phase23_redacted_operator_evidence_test //scripts:phase25_live_stratum_evidence_test //scripts:phase27_live_hardware_bridge_evidence_test //scripts:phase28_evidence_test` |
| **Estimated runtime** | Quick checks under 30 seconds; full relevant local suite under 10 minutes |

***

## Sampling Rate

- **After every task commit:** Run the narrowest affected Rust or script regression target.
- **After every plan wave:** Run all Phase 29 parity and wrapper targets plus `just parity`.
- **Before phase verification:** Run the full Rust pre-commit sequence, affected Bazel targets, `just parity`, and `just verify-reference`.
- **Max feedback latency:** 10 minutes for the full local gate; task-local feedback should remain under 60 seconds.

***

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Threat Ref | Secure Behavior | Test Type | Automated Command | File Exists | Status |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| 29-01-01 | 01 | 1 | EVD-07, EVD-09 | T-29-01 | Phase identity and slot dispositions fail closed | unit | `cargo test -p bitaxe-parity --all-features operator_evidence` | ✅ | ⬜ pending |
| 29-01-02 | 01 | 1 | EVD-08, EVD-09 | T-29-02 | Cross-link generation cannot infer success or copy secrets | unit/CLI | `cargo test -p bitaxe-parity --all-features` | ✅ | ⬜ pending |
| 29-02-01 | 02 | 2 | EVD-07, REL-09 | T-29-03 | Phase 25/27 validate exactly once and preserve failures | integration | `bazel test //scripts:phase23_redacted_operator_evidence_test //scripts:phase25_live_stratum_evidence_test //scripts:phase27_live_hardware_bridge_evidence_test` | ✅ | ⬜ pending |
| 29-02-02 | 02 | 2 | EVD-07, EVD-08, EVD-09, REL-09 | T-29-04 | Invalid consolidation never replaces a valid destination | integration | `bazel test //scripts:phase28_evidence_test //tools/parity:tests` | ❌ W0 | ⬜ pending |
| 29-03-01 | 03 | 3 | EVD-07, EVD-09, REL-09 | T-29-05 | New guide lines and Phase 29 evidence reject secret/local/network identifiers | regression/static | `bazel test //scripts:phase29_doc_redaction_check_test` then `bazel run //scripts:phase29_doc_redaction_check -- --baseline-ref "$(git log -1 --format=%H -- .planning/phases/29-evidence-workflow-automation-closure/29-02-SUMMARY.md)" --evidence-root docs/parity/evidence/phase-29-evidence-workflow-automation-closure` | ❌ W0 | ⬜ pending |
| 29-03-02 | 03 | 3 | EVD-08, EVD-09 | T-29-06 | Overclaim and prohibited-token guards remain fail closed | regression | `just parity` | ✅ | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

***

## Wave 0 Requirements

- [ ] Plan 29-01 Tasks 1 and 2 write profile/generation tests first, run and record the expected red result locally, then implement to green without committing the red state.
- [ ] Plan 29-02 Task 1 extends existing wrapper tests before wrapper production edits; Task 2 creates the Phase 28 test and Bazel target before the wrapper implementation.
- [ ] Plan 29-03 Task 1 creates the diff-aware documentation redaction test before the scanner and documentation edits.
- [x] Exact Cargo package (`bitaxe-parity`) and existing/new Bazel target names are recorded in the verification map.

***

## Manual-Only Verifications

All Phase 29 workflow-automation behavior has automated verification. Real hardware is not required unless implementation unexpectedly changes the hardware path. Any optional hardware run remains detector-gated, board-205-only, credential-local, 360-second minimum, and redacted per `AGENTS.md`.

***

## Validation Sign-Off

- [x] All tasks have an automated verify command or a Wave 0 dependency.
- [x] Sampling continuity: no three consecutive tasks without automated verification.
- [ ] Wave 0 test-first steps have been executed and every expected red test was driven to green.
- [x] No watch-mode flags.
- [x] Feedback latency remains below 10 minutes.
- [ ] Set `nyquist_compliant: true` and `wave_0_complete: true` only during Plan 03 after test-first steps and every mapped command pass.

**Approval:** pending
