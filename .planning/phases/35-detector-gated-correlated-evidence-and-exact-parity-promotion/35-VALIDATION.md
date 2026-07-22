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
| **Quick run command** | `bazel test //scripts:hardware_attempt_policy_contract_test //tools/parity:tests //scripts:phase33_confirmed_settings_durability_test` |
| **Full suite command** | `cargo fmt --all && cargo clippy --all-targets --all-features -- -D warnings && cargo build --all-targets --all-features && cargo test --all-features` |
| **Estimated runtime** | ~180 seconds for software; hardware capture ≥360 seconds |

## Sampling Rate

- **After every task commit:** Run the narrowest affected target plus `bazel test //tools/parity:tests`.
- **After every plan wave:** Run `//scripts:hardware_attempt_policy_contract_test`, all Phase 35 targets, `//scripts:phase30_no_promotion_contract_test`, and `just parity`.
- **Before `/gsd-verify-work`:** The ordered Rust gate and all Phase 35 Bazel targets must be green.
- **Max software feedback latency:** 240 seconds.
- **Hardware timeout floor:** 360 seconds capture and 420 seconds wall clock.

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Threat Ref | Secure Behavior | Test Type | Automated Command | File Exists | Status |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| 35-01-01 | 01 | 1 | EVD-11, EVD-12, EVD-14 | T-35-01 through T-35-04 | Reject malformed exact-package/detector/root provenance, mixed epochs, event-chain drift, inventory drift, or raw projection fields | Rust fixture | `bazel test //tools/parity:tests` | ✅ | ⬜ pending |
| 35-01-02 | 01 | 1 | EVD-11, EVD-12, EVD-14 | T-35-01 through T-35-04 | Every evidence, chronology, cleanup, lifecycle, no-actuation, and redaction boundary has an exact negative fixture | Rust fixture | `bazel test //tools/parity:tests` | ✅ | ⬜ pending |
| 35-02-01 | 02 | 2 | CFG-12, EVD-10–EVD-14 | T-35-09 through T-35-12 | Gate 1 freezes exact package, Gate 2 invokes the detector once, and Gate 3 alone may perform the bounded capture/recovery sequence | Shell build/static | `bazel build //scripts:phase35_correlated_evidence` | ✅ | ⬜ pending |
| 35-02-02 | 02 | 2 | CFG-12, EVD-10–EVD-14 | T-35-09 through T-35-12 | Zero/multiple/wrong detector and every post-mutation failure prove short-circuit, restoration, cleanup, and sealing | Shell simulation | `bazel test //scripts:phase35_correlated_evidence_test` | ✅ | ⬜ pending |
| 35-03-01 | 03 | 3 | EVD-14, EVD-15 | T-35-13 through T-35-16 | Every claim scope has one exact decision and live admission rechecks precede promotion | Rust integration | `bazel test //tools/parity:tests //scripts:phase30_no_promotion_contract_test` | ✅ | ⬜ pending |
| 35-03-02 | 03 | 3 | EVD-14, EVD-15 | T-35-13 through T-35-16 | Atomic failures preserve the previous generation and every non-allowlisted row remains unchanged | Shell + Rust integration | `bazel test //tools/parity:tests //scripts:phase35_promotion_contract_test //scripts:phase30_no_promotion_contract_test` | ✅ | ⬜ pending |
| 35-04-01 | 04 | 4 | CFG-12, EVD-10–EVD-15 | T-35-17 through T-35-20 | Hardware policy plus ordered Rust/Bazel/reference/parity/lifecycle/exact-package preflight passes before each fresh attempt | Software preflight | `bazel test //scripts:hardware_attempt_policy_contract_test && just phase35-evidence preflight-only=true` | ✅ | ⬜ pending |
| 35-04-02 | 04 | 4 | CFG-12, EVD-10–EVD-15 | T-35-17 through T-35-20 | Each fresh command invocation owns the sole detector call for its root; the first eligible two-epoch root admits, while sealed roots follow the canonical progress decision | Hardware + admission | `just phase35-evidence capture-timeout-seconds=360` | ✅ | ⬜ pending |
| 35-04-03 | 04 | 4 | CFG-12, EVD-10–EVD-15 | T-35-17 through T-35-20 | Committed digests, redaction, exact row diff, exhaustive non-claims, parity, and lifecycle revalidate | Admission audit | `bazel test //tools/parity:tests //scripts:phase35_promotion_contract_test //scripts:phase30_no_promotion_contract_test` | ✅ | ⬜ pending |

## Wave 0 Requirements

- [x] Existing Rust and Bazel test infrastructure covers typed evaluators and parity admission.
- [x] Existing Phase 33 fixtures cover detector, restart, cleanup, and restoration patterns to reuse.
- [x] `//scripts:phase35_correlated_evidence_test` — simulation/failure-injection coverage exists before hardware execution.
- [x] `just phase35-evidence` — the repo-owned bounded evidence entrypoint exists before hardware execution.
- [x] `//scripts:hardware_attempt_policy_contract_test` — the progress-gated fresh-attempt contract is hermetically enforced.

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
| --- | --- | --- | --- |
| Exact-current-package Ultra 205 evidence root | CFG-12, EVD-10–EVD-15 | Requires the physically connected board and one approved normal reboot | Attempt 18 invoked the command once at exact source `065240279c4657945ffce70d2baa501b4da7ceae` after both gates passed. Boot A pre-capture and PATCH succeeded, then the post-PATCH retained-log response exposed malformed chunk framing after a coherent WebSocket capture. Restoration and cleanup passed. The missing WebSocket close-handshake boundary is regression-backed and selects `continue_after_verified_fix`; attempt 19 is authorized only after the complete exact-current-HEAD gate and fresh preflight. |

## Validation Sign-Off

- [x] Every planned task has a narrow automated verification target or an explicit Wave 0 dependency.
- [x] Sampling continuity has no three consecutive tasks without automated verification.
- [x] Wave 0 Phase 35 entrypoint, failure-injection, and progress-policy targets exist.
- [x] No watch-mode flags are used.
- [x] Software feedback latency target is below 240 seconds.
- [x] Hardware capture and wall-clock floors match repo policy.
- [x] `nyquist_compliant: true` is set.

**Approval:** approved 2026-07-17
