---
phase: "35"
slug: detector-gated-correlated-evidence-and-exact-parity-promotion
status: approved
nyquist_compliant: true
wave_0_complete: true
created: "2026-07-17"
phase_lifecycle_id: 35-2026-07-17T17-00-37
---

# Phase 35 — Validation Strategy

> Per-phase validation contract for detector gating, two-epoch correlation, and exact parity admission.

## Test Infrastructure

| Property | Value |
| --- | --- |
| **Framework** | Rust unit tests, Bazel `rust_test`/`sh_test`, repo-owned shell fixtures |
| **Config file** | `Cargo.toml`, `MODULE.bazel`, `tools/parity/BUILD.bazel`, `scripts/BUILD.bazel` |
| **Quick run command** | `bazel test //tools/parity:tests //scripts:phase33_confirmed_settings_durability_test` |
| **Full suite command** | `cargo fmt --all && cargo clippy --all-targets --all-features -- -D warnings && cargo build --all-targets --all-features && cargo test --all-features` |
| **Estimated runtime** | ~180 seconds for software; hardware capture ≥360 seconds |

## Sampling Rate

- **After every task commit:** Run the narrowest affected target plus `bazel test //tools/parity:tests`.
- **After every plan wave:** Run all new Phase 35 targets, `//scripts:phase30_no_promotion_contract_test`, and `just parity`.
- **Before `/gsd-verify-work`:** The ordered Rust gate and all Phase 35 Bazel targets must be green.
- **Max software feedback latency:** 240 seconds.
- **Hardware timeout floor:** 360 seconds capture and 420 seconds wall clock.

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Threat Ref | Secure Behavior | Test Type | Automated Command | File Exists | Status |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| 35-01-01 | 01 | 1 | EVD-11, EVD-12 | T-35-01 through T-35-04 | Reject malformed provenance, mixed epochs, chronology drift, and inventory drift | Rust fixture | `bazel test //tools/parity:tests` | ✅ | ⬜ pending |
| 35-01-02 | 01 | 1 | EVD-14, EVD-15 | T-35-05 through T-35-08 | Redaction is allowlisted and every claim scope receives a typed decision | Rust fixture | `bazel test //tools/parity:tests` | ✅ | ⬜ pending |
| 35-02-01 | 02 | 2 | CFG-12, EVD-10, EVD-13 | T-35-09 through T-35-12 | No target/effect before detector; one approved reboot; restoration and cleanup on every exit | Shell simulation | `bazel test //scripts:phase35_correlated_evidence_test` | ❌ W0 | ⬜ pending |
| 35-03-01 | 03 | 3 | EVD-14, EVD-15 | T-35-13 through T-35-16 | Admission is atomic; excluded rows and Phase 30 non-promotions are unchanged | Integration | `bazel test //tools/parity:tests //scripts:phase30_no_promotion_contract_test` | ✅ | ⬜ pending |
| 35-04-01 | 04 | 4 | CFG-12, EVD-10–EVD-15 | T-35-17 through T-35-20 | One eligible root passes all gates or deterministically seals non-promotion | Hardware + admission | `just phase35-evidence capture-timeout-seconds=360` | ❌ W0 | ⬜ pending |

## Wave 0 Requirements

- [x] Existing Rust and Bazel test infrastructure covers typed evaluators and parity admission.
- [x] Existing Phase 33 fixtures cover detector, restart, cleanup, and restoration patterns to reuse.
- [ ] `//scripts:phase35_correlated_evidence_test` — add simulation/failure-injection coverage before hardware execution.
- [ ] `just phase35-evidence` — add the repo-owned bounded evidence entrypoint before hardware execution.

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
| --- | --- | --- | --- |
| Exact-current-package Ultra 205 evidence root | CFG-12, EVD-10–EVD-15 | Requires the physically connected board and one approved normal reboot | Run `just detect-ultra205`; only on exactly one board-205 candidate and successful board-info run the planned Phase 35 command with ≥360-second capture, ignored credential paths, protected local root, restoration, cleanup, redaction, and admission gates. |

## Validation Sign-Off

- [x] Every planned task has a narrow automated verification target or an explicit Wave 0 dependency.
- [x] Sampling continuity has no three consecutive tasks without automated verification.
- [x] Wave 0 names all missing Phase 35 targets.
- [x] No watch-mode flags are used.
- [x] Software feedback latency target is below 240 seconds.
- [x] Hardware capture and wall-clock floors match repo policy.
- [x] `nyquist_compliant: true` is set.

**Approval:** approved 2026-07-17
