---
phase: 24
slug: bm1366-production-work-path
status: passed
nyquist_compliant: true
wave_0_complete: true
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
- **Before `/gsd-verify-work`:** `just test`, `just parity`, `just verify-reference`, redaction review, and lifecycle validation must be green or explicitly blocked with exact non-claims.
- **Max feedback latency:** 180 seconds for pure crate/task-level checks. Phase 24 does not include a hardware-promotion validation branch.

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Threat Ref | Secure Behavior | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|------------|-----------------|-----------|-------------------|-------------|--------|
| 24-01-01 | 01 | 1 | ASIC-09 | T-24-01 / T-24-04 | Diagnostic and production BM1366 modes are distinct and raw frames stay outside firmware | unit/static | `bazel test //crates/bitaxe-asic/... && ! rg -n "0x55|0xaa|crc5|crc16|raw frame" firmware/bitaxe/src` | W0 | passed - observed `bazel test //crates/bitaxe-asic:tests` in `24-01-SUMMARY.md` and full Phase 24 suite in Plan 04 |
| 24-02-01 | 02 | 1 | ASIC-10 | T-24-02 / T-24-05 | Pool-derived work binds job, extranonce, difficulty/target, and session generation before dispatch | unit | `bazel test //crates/bitaxe-stratum/...` | W0 | passed - observed `bazel test //crates/bitaxe-stratum:tests` in `24-02-SUMMARY.md` and full Phase 24 suite in Plan 04 |
| 24-02-02 | 02 | 1 | ASIC-10 | T-24-02 | Clean-jobs and reconnect invalidate queued and active BM1366 production work | unit | `bazel test //crates/bitaxe-stratum/...` | W0 | passed - observed `bazel test //crates/bitaxe-stratum:tests` in `24-02-SUMMARY.md` and full Phase 24 suite in Plan 04 |
| 24-03-01 | 03 | 3 | ASIC-11 | T-24-09 / T-24-10 | Nonce/result observations produce submit intent only after generation-stamped active-work correlation and redacted submit formatting | unit/fixture/redaction | `bazel test //crates/bitaxe-asic/... //crates/bitaxe-stratum/...` | W0 | passed - observed `bazel test //crates/bitaxe-stratum:tests //crates/bitaxe-asic:tests` in `24-03-SUMMARY.md` and full Phase 24 suite in Plan 04 |
| 24-03-02 | 03 | 3 | ASIC-11, ASIC-12 | T-24-09 / T-24-11 / T-24-13 | Malformed, stale, duplicate, wrong-session, or uncorrelated results fail closed with redaction-safe reasons | unit/redaction | `bazel test //crates/bitaxe-asic/... //crates/bitaxe-stratum/... //crates/bitaxe-safety:tests` | W0 | passed - observed `bazel test //crates/bitaxe-stratum:tests //crates/bitaxe-asic:tests //crates/bitaxe-safety:tests` in `24-03-SUMMARY.md` and full Phase 24 suite in Plan 04 |
| 24-03-03 | 03 | 3 | ASIC-09, ASIC-11, ASIC-12 | T-24-12 / T-24-19 | Controlled firmware runtime consumes production dispatch/result outputs and keeps diagnostic work out of the production route | build/static | `bazel build //firmware/bitaxe:firmware` | W0 | passed - observed `bazel build //firmware/bitaxe:firmware` in `24-03-SUMMARY.md` |
| 24-04-01 | 04 | 4 | ASIC-09, ASIC-10, ASIC-11, ASIC-12 | T-24-14 / T-24-15 / T-24-17 | Checklist and evidence updates promote only exact implemented/evidenced claims and preserve Phase 25 non-claims | parity/docs | `just parity && just verify-reference` | W0 | passed - observed evidence metadata scan, forbidden-value scan, `just parity`, and `just verify-reference` in Plan 04 |
| 24-04-02 | 04 | 4 | ASIC-09, ASIC-10, ASIC-11, ASIC-12 | T-24-15 / T-24-16 / T-24-17 | Validation metadata and checklist rows remain implemented/unit,workflow with no Phase 24 hardware promotion | lifecycle/parity | `node "$HOME/.cursor/get-shit-done/bin/gsd-tools.cjs" verify lifecycle 24 --expect-id 24-2026-07-05T00-27-27 --expect-mode yolo --require-plans` | W0 | passed - observed lifecycle validation, `just parity`, and `just verify-reference` in Plan 04 |

## Wave 0 Requirements

- [x] Existing `bazel test` infrastructure remains usable for `crates/bitaxe-asic` and `crates/bitaxe-stratum`.
- [x] New or changed Rust core modules include focused unit tests with Arrange, Act, Assert structure.
- [x] If new shell or Node helpers are added, task plans include syntax/unit checks such as `bash -n` or the existing repo-owned script test pattern.
- [x] Hardware checklist promotion remains out of Phase 24; any later hardware-capable evidence plan must start with `just detect-ultra205` and record selected port, board-info, package identity, exact commands, evidence directory, redaction workflow, observed behavior, and conclusion.

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| Detector-gated Ultra 205 production work evidence | ASIC-09, ASIC-10, ASIC-11, ASIC-12 | Hardware availability and safe prerequisite evidence cannot be assumed, and Phase 24 has no hardware-promotion task | Do not promote Phase 24 rows beyond `implemented | unit,workflow`. Record `hardware evidence pending` and preserve non-claims unless a later plan documents the recovery path, `just detect-ultra205` finds exactly one board `205`, and evidence is captured through repo-owned redacted commands with the required AGENTS.md fields. |

## Validation Sign-Off

- [x] All planned requirement areas have automated verification targets; hardware checklist promotion remains pending outside Phase 24.
- [x] Sampling continuity: no 3 consecutive tasks should proceed without automated verification.
- [x] Wave 0 covers known missing validation references.
- [x] No watch-mode flags.
- [x] Feedback latency target documented.
- [x] `nyquist_compliant: true` set in frontmatter.

**Approval:** approved 2026-07-05
