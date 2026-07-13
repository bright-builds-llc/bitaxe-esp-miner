---
phase: 29
slug: evidence-workflow-automation-closure
status: complete
nyquist_compliant: true
wave_0_complete: true
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
| **Full suite command** | `bazel test //tools/parity:tests //scripts:phase23_redacted_operator_evidence_test //scripts:phase25_live_stratum_evidence_test //scripts:phase27_live_hardware_bridge_evidence_test //scripts:phase28_evidence_test //scripts:phase29_doc_redaction_check_test` |
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
| 29-01-01 | 01 | 1 | EVD-07, EVD-09 | T-29-01 | Phase identity and slot dispositions fail closed | unit | `cargo test -p bitaxe-parity --all-features operator_evidence` | ✅ | ✅ green |
| 29-01-02 | 01 | 1 | EVD-08, EVD-09 | T-29-02 | Cross-link generation cannot infer success or copy secrets | unit/CLI | `cargo test -p bitaxe-parity --all-features` | ✅ | ✅ green |
| 29-02-01 | 02 | 2 | EVD-07, REL-09 | T-29-03 | Phase 25/27 validate exactly once and preserve failures | integration | `bazel test //scripts:phase23_redacted_operator_evidence_test //scripts:phase25_live_stratum_evidence_test //scripts:phase27_live_hardware_bridge_evidence_test` | ✅ | ✅ green |
| 29-02-02 | 02 | 2 | EVD-07, EVD-08, EVD-09, REL-09 | T-29-04 | Invalid consolidation never replaces a valid destination | integration | `bazel test //scripts:phase28_evidence_test //tools/parity:tests` | ✅ | ✅ green |
| 29-03-01 | 03 | 3 | EVD-07, EVD-09, REL-09 | T-29-05 | New guide lines and Phase 29 evidence reject secret/local/network identifiers | regression/static | `bazel test //scripts:phase29_doc_redaction_check_test` then `bazel run //scripts:phase29_doc_redaction_check -- --baseline-ref "$(git log -1 --format=%H -- .planning/phases/29-evidence-workflow-automation-closure/29-02-SUMMARY.md)" --evidence-root docs/parity/evidence/phase-29-evidence-workflow-automation-closure` | ✅ | ✅ green |
| 29-03-02 | 03 | 3 | EVD-08, EVD-09 | T-29-06 | Overclaim and prohibited-token guards remain fail closed | regression | `just parity` | ✅ | ✅ green |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

***

## Wave 0 Requirements

- [x] Plan 29-01 Tasks 1 and 2 wrote profile/generation tests first, recorded the expected red result locally, and drove them green without committing the red state.
- [x] Plan 29-02 Task 1 extended wrapper tests before wrapper production edits; Task 2 created the Phase 28 test and Bazel target before the wrapper implementation.
- [x] Plan 29-03 Task 1 created the diff-aware documentation redaction test before the scanner and documentation edits.
- [x] Exact Cargo package (`bitaxe-parity`) and existing/new Bazel target names are recorded in the verification map.

## Final Gate Results

- `cargo fmt --all`, Clippy with denied warnings, the all-target/all-feature build, and all-feature workspace tests passed in the required order.
- Both mapped `bitaxe-parity` Cargo commands passed.
- All mapped Phase 23, Phase 25, Phase 27, Phase 28, Phase 29, and parity Bazel targets passed.
- The live Plan 02 baseline document scan passed without printing matched content.
- The whole-file checklist comparison against the Plan 02 summary commit passed byte-for-byte.
- `just parity`, `just verify-reference`, `git diff --check`, and lifecycle validation passed.
- Shell formatter/lint checks and Markdown check mode passed for the changed non-frontmatter files; this file retained exactly two top frontmatter delimiters and was not rewritten by `mdformat`.

***

## Manual-Only Verifications

All Phase 29 workflow-automation behavior has automated verification. Real hardware is not required unless implementation unexpectedly changes the hardware path. Any optional hardware run remains detector-gated, board-205-only, credential-local, 360-second minimum, and redacted per `AGENTS.md`.

***

## Validation Sign-Off

- [x] All tasks have an automated verify command or a Wave 0 dependency.
- [x] Sampling continuity: no three consecutive tasks without automated verification.
- [x] Wave 0 test-first steps have been executed and every expected red test was driven to green.
- [x] No watch-mode flags.
- [x] Feedback latency remains below 10 minutes.
- [x] `nyquist_compliant: true` and `wave_0_complete: true` were set during Plan 03 after test-first steps and every mapped command passed.

**Approval:** approved 2026-07-12
