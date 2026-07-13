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
| **Quick run command** | `cargo test -p bitaxe-parity --all-features` if that package name is confirmed; otherwise the targeted parity package command from `tools/parity/Cargo.toml` |
| **Full suite command** | `cargo test --all-features` plus affected `bazel test` targets in `//tools/parity` and `//scripts` |
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
| 29-01-01 | 01 | 1 | EVD-07, EVD-09 | T-29-01 | Phase identity and slot dispositions fail closed | unit | Targeted `tools/parity` Rust tests | ✅ | ⬜ pending |
| 29-01-02 | 01 | 1 | EVD-08, EVD-09 | T-29-02 | Cross-link generation cannot infer success or copy secrets | unit/CLI | Targeted operator-evidence CLI tests | ✅ | ⬜ pending |
| 29-02-01 | 02 | 2 | EVD-07, REL-09 | T-29-03 | Phase 25/27 validate exactly once and preserve failures | integration | Phase 25 and Phase 27 script regression targets | ✅ | ⬜ pending |
| 29-02-02 | 02 | 2 | EVD-07, EVD-09, REL-09 | T-29-04 | Invalid consolidation never replaces a valid destination | integration | New Phase 28 script regression target | ❌ W0 | ⬜ pending |
| 29-03-01 | 03 | 3 | REL-09 | — | Operator docs expose only repo-owned redaction-safe commands | static | Targeted docs/command checks and `git diff --check` | ✅ | ⬜ pending |
| 29-03-02 | 03 | 3 | EVD-08, EVD-09 | T-29-05 | Overclaim and prohibited-token guards remain fail closed | regression | `just parity` and affected negative fixtures | ✅ | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

***

## Wave 0 Requirements

- [ ] Add the Phase 28 wrapper regression script and Bazel test target before implementing consolidation behavior.
- [ ] Add targeted Rust fixtures for explicit Phase 25, Phase 27, and Phase 28 profiles before changing validator policy.
- [ ] Confirm the exact Cargo package and Bazel target names used by the quick commands; plans must replace provisional names with repo-valid commands.

***

## Manual-Only Verifications

All Phase 29 workflow-automation behavior has automated verification. Real hardware is not required unless implementation unexpectedly changes the hardware path. Any optional hardware run remains detector-gated, board-205-only, credential-local, 360-second minimum, and redacted per `AGENTS.md`.

***

## Validation Sign-Off

- [ ] All tasks have an automated verify command or a Wave 0 dependency.
- [ ] Sampling continuity: no three consecutive tasks without automated verification.
- [ ] Wave 0 covers every missing test target or fixture.
- [ ] No watch-mode flags.
- [ ] Feedback latency remains below 10 minutes.
- [ ] `nyquist_compliant: true` and `wave_0_complete: true` are set after plan verification confirms exact commands and test coverage.

**Approval:** pending
