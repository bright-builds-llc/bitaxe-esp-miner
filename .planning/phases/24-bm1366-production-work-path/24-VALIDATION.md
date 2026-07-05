---
phase: 24
slug: bm1366-production-work-path
status: draft
nyquist_compliant: true
wave_0_complete: false
created: 2026-07-05
---

# Phase 24 - Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | Bazel-driven Rust tests and repo-owned shell/parity checks |
| **Config file** | `MODULE.bazel`, crate `BUILD.bazel` files, and `Justfile` |
| **Quick run command** | `bazel test //crates/bitaxe-asic/... //crates/bitaxe-stratum/...` |
| **Full suite command** | `just test && just parity && just verify-reference` |
| **Estimated runtime** | ~120 seconds for quick crate tests; full suite depends on firmware/tool cache state |

## Sampling Rate

- **After every task commit:** Run `bazel test //crates/bitaxe-asic/... //crates/bitaxe-stratum/...` when Rust core files changed, or the task-specific command from its `<verify>` block.
- **After every plan wave:** Run the wave's combined automated verification plus `just parity` when checklist/evidence files changed.
- **Before `/gsd-verify-work`:** `just test`, `just parity`, `just verify-reference`, redaction review, lifecycle validation, and any detector-gated hardware commands actually used must be green or explicitly blocked with exact non-claims.
- **Max feedback latency:** 180 seconds for pure crate/task-level checks; hardware-capable checks may exceed this only when the plan records the detector gate and evidence path.

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Threat Ref | Secure Behavior | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|------------|-----------------|-----------|-------------------|-------------|--------|
| 24-01-01 | 01 | 1 | ASIC-09 | T-24-01 / T-24-04 | Diagnostic and production BM1366 modes are distinct and raw frames stay outside firmware | unit/static | `bazel test //crates/bitaxe-asic/... && ! rg -n "0x55|0xaa|crc5|crc16|raw frame" firmware/bitaxe/src` | W0 | pending |
| 24-02-01 | 02 | 1 | ASIC-10 | T-24-02 / T-24-05 | Pool-derived work binds job, extranonce, difficulty/target, and session generation before dispatch | unit | `bazel test //crates/bitaxe-stratum/...` | W0 | pending |
| 24-02-02 | 02 | 1 | ASIC-10 | T-24-02 | Clean-jobs and reconnect invalidate queued and active BM1366 production work | unit | `bazel test //crates/bitaxe-stratum/...` | W0 | pending |
| 24-03-01 | 03 | 2 | ASIC-11 | T-24-03 / T-24-05 | Nonce/result observations produce submit intent only after active-work correlation | unit/fixture | `bazel test //crates/bitaxe-asic/... //crates/bitaxe-stratum/...` | W0 | pending |
| 24-03-02 | 03 | 2 | ASIC-12 | T-24-04 / T-24-06 | Malformed, stale, duplicate, wrong-session, or uncorrelated results fail closed with redaction-safe reasons | unit/redaction | `bazel test //crates/bitaxe-asic/... //crates/bitaxe-stratum/...` | W0 | pending |
| 24-04-01 | 04 | 3 | ASIC-09, ASIC-10, ASIC-11, ASIC-12 | T-24-06 / T-24-07 | Checklist and evidence updates promote only exact implemented/evidenced claims and preserve Phase 25 non-claims | parity/docs | `just parity && just verify-reference` | W0 | pending |

## Wave 0 Requirements

- [ ] Existing `bazel test` infrastructure remains usable for `crates/bitaxe-asic` and `crates/bitaxe-stratum`.
- [ ] New or changed Rust core modules include focused unit tests with Arrange, Act, Assert structure.
- [ ] If new shell or Node helpers are added, task plans include syntax/unit checks such as `bash -n` or the existing repo-owned script test pattern.
- [ ] If hardware-capable evidence is attempted, `just detect-ultra205` succeeds first and the plan records the exact evidence directory and redaction review path.

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| Optional detector-gated Ultra 205 production work evidence | ASIC-09, ASIC-10, ASIC-11, ASIC-12 | Hardware availability and safe prerequisite evidence cannot be assumed | Run only if the active plan documents the recovery path, `just detect-ultra205` finds exactly one board `205`, and evidence is captured through repo-owned redacted commands. Otherwise record `hardware evidence pending` and preserve non-claims. |

## Validation Sign-Off

- [x] All planned requirement areas have automated verification targets or explicit hardware-gated manual evidence.
- [x] Sampling continuity: no 3 consecutive tasks should proceed without automated verification.
- [x] Wave 0 covers known missing validation references.
- [x] No watch-mode flags.
- [x] Feedback latency target documented.
- [x] `nyquist_compliant: true` set in frontmatter.

**Approval:** approved 2026-07-05
