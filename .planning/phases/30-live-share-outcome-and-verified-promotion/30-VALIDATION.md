---
phase: 30
slug: live-share-outcome-and-verified-promotion
status: complete
nyquist_compliant: true
wave_0_complete: true
created: 2026-07-13
---

# Phase 30 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

***

## Test Infrastructure

| Property | Value |
| --- | --- |
| **Framework** | Rust built-in tests plus repository-local Markdown/content checks under Bazel where needed |
| **Config file** | `Cargo.toml`, `tools/parity/BUILD.bazel`, `docs/parity/checklist.md` |
| **Quick run command** | `cargo test -p bitaxe-parity --all-features phase30_` |
| **Full suite command** | `bazel test //tools/parity:tests && just parity && just verify-reference` |
| **Estimated runtime** | Quick checks under 30 seconds; affected local suite under 5 minutes |

***

## Sampling Rate

- **After every task commit:** Run the narrowest affected `phase30_` Rust test or exact repository content check.
- **After every plan wave:** Run `bazel test //tools/parity:tests` and `just parity`.
- **Before phase verification:** Run the full Rust pre-commit sequence in required order, the affected Bazel tests, `just parity`, `just verify-reference`, lifecycle validation, and `git diff --check`.
- **Max feedback latency:** 5 minutes for the affected local suite; task-local feedback should remain under 60 seconds.

***

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Threat Ref | Secure Behavior | Test Type | Automated Command | File Exists | Status |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| 30-01-01 | 01 | 1 | STR-09, CFG-07, ASIC-11 | T-30-01 | No-input disposition cannot be interpreted as promotion or live proof | static/regression | `rg -n "phase30_disposition: no_promotion_no_eligible_evidence|new_evidence_input: none|eligible_share_outcome: none" docs/parity/evidence/phase-30-live-share-outcome-and-verified-promotion/disposition.md` | ✅ | ✅ green |
| 30-01-02 | 01 | 1 | STR-09, CFG-07, ASIC-11 | T-30-02 | Checklist and Nyquist closure preserve implemented/pending/gaps-found truth | static/regression | `rg -n "STR-09|CFG-07|ASIC-11|implemented|closed_wont_do_unresolved|gaps_found" docs/parity/checklist.md .planning/phases/28.1-live-mining-blocker-fix-h4-w13-orchestration-parity-discrimi/28.1-VALIDATION.md` | ✅ | ✅ green |
| 30-02-01 | 02 | 2 | STR-09, CFG-07, ASIC-11 | T-30-03 | Verified rows require explicit row-specific eligible Phase 30 evidence | unit/regression | `cargo test -p bitaxe-parity --all-features phase30_ && bazel test //tools/parity:tests` | ✅ | ✅ green |
| 30-02-02 | 02 | 2 | STR-09, CFG-07, ASIC-11 | T-30-04 | Final closure retains pending traceability and every explicit non-claim | repository gate | `just parity && just verify-reference && git diff --check` | ✅ | ✅ green |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

***

## Wave 0 Requirements

- [x] Add focused `phase30_` regression fixtures in `tools/parity/src/main.rs` before changing the guard: conservative rows pass; fabricated verified STR-09, CFG-07, and ASIC-11 rows fail without exact row-specific admission.
- [x] Add a failing artifact-contract check for the absent `docs/parity/evidence/phase-30-live-share-outcome-and-verified-promotion/disposition.md` before creating it.
- [x] Preserve the existing Phase 28 promotion tests as regression coverage; do not replace or weaken them.
- [x] Record the exact Cargo package (`bitaxe-parity`) and Bazel target (`//tools/parity:tests`) in each implementation task.

## Manual-Only Verifications

All Phase 30 behavior is deterministic and repository-local. Hardware, USB, flashing, monitoring, credentials, ignored local evidence, direct UART, and pin manipulation are prohibited for this phase.

***

## Validation Sign-Off

- [x] Every planned behavior has an automated verify command or explicit Wave 0 dependency.
- [x] Sampling continuity: no three consecutive tasks lack automated verification.
- [x] Wave 0 names every missing test/artifact reference.
- [x] No watch-mode flags.
- [x] Expected feedback latency is below 5 minutes.
- [x] `nyquist_compliant: true` reflects a complete strategy only; it does not claim the phase or requirements passed.
- [x] Set `wave_0_complete: true` only after the test-first red/green steps execute.

**Approval:** approved 2026-07-13

## Final Gate Results

- `cargo test -p bitaxe-parity --all-features phase30_` passed eight focused admission tests.
- `bazel test //scripts:phase30_no_promotion_contract_test //tools/parity:tests` passed both repository contracts.
- `just parity` passed with `validation_errors: none`.
- `just verify-reference` passed for the pinned clean reference.
- The targeted requirements/checklist/archive truth review passed with all three requirements pending, all three checklist rows implemented, archived verification `gaps_found`, and Phase 28.1 `closed_wont_do_unresolved`.
- `git diff --check` passed.

`nyquist_compliant: true` records complete deterministic sampling for Phase 30 only. It does not promote or verify STR-09, CFG-07, or ASIC-11.
