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
| 35-01-01 | 01 | 1 | EVD-11, EVD-12, EVD-14 | T-35-01 through T-35-04 | Reject malformed exact-package/detector/root provenance, mixed epochs, event-chain drift, inventory drift, or raw projection fields | Rust fixture | `bazel test //tools/parity:tests` | ✅ | ⬜ pending |
| 35-01-02 | 01 | 1 | EVD-11, EVD-12, EVD-14 | T-35-01 through T-35-04 | Every evidence, chronology, cleanup, lifecycle, no-actuation, and redaction boundary has an exact negative fixture | Rust fixture | `bazel test //tools/parity:tests` | ✅ | ⬜ pending |
| 35-02-01 | 02 | 2 | CFG-12, EVD-10–EVD-14 | T-35-09 through T-35-12 | Gate 1 freezes exact package, Gate 2 invokes the detector once, and Gate 3 alone may perform the bounded capture/recovery sequence | Shell build/static | `bazel build //scripts:phase35_correlated_evidence` | ❌ W0 | ⬜ pending |
| 35-02-02 | 02 | 2 | CFG-12, EVD-10–EVD-14 | T-35-09 through T-35-12 | Zero/multiple/wrong detector and every post-mutation failure prove short-circuit, restoration, cleanup, and sealing | Shell simulation | `bazel test //scripts:phase35_correlated_evidence_test` | ❌ W0 | ⬜ pending |
| 35-03-01 | 03 | 3 | EVD-14, EVD-15 | T-35-13 through T-35-16 | Every claim scope has one exact decision and live admission rechecks precede promotion | Rust integration | `bazel test //tools/parity:tests //scripts:phase30_no_promotion_contract_test` | ✅ | ⬜ pending |
| 35-03-02 | 03 | 3 | EVD-14, EVD-15 | T-35-13 through T-35-16 | Atomic failures preserve the previous generation and every non-allowlisted row remains unchanged | Shell + Rust integration | `bazel test //tools/parity:tests //scripts:phase35_promotion_contract_test //scripts:phase30_no_promotion_contract_test` | ❌ W0 | ⬜ pending |
| 35-04-01 | 04 | 4 | CFG-12, EVD-10–EVD-15 | T-35-17 through T-35-20 | Ordered Rust/Bazel/reference/parity/lifecycle/exact-package preflight passes before detector use | Software preflight | `just phase35-evidence preflight-only=true` | ❌ W0 | ⬜ pending |
| 35-04-02 | 04 | 4 | CFG-12, EVD-10–EVD-15 | T-35-17 through T-35-20 | The command performs the sole detector call and admits one eligible two-epoch root; safe non-promotion remains phase failure | Hardware + admission | `just phase35-evidence capture-timeout-seconds=360` | ❌ W0 | ⬜ pending |
| 35-04-03 | 04 | 4 | CFG-12, EVD-10–EVD-15 | T-35-17 through T-35-20 | Committed digests, redaction, exact row diff, exhaustive non-claims, parity, and lifecycle revalidate | Admission audit | `bazel test //tools/parity:tests //scripts:phase35_promotion_contract_test //scripts:phase30_no_promotion_contract_test` | ❌ W0 | ⬜ pending |

## Wave 0 Requirements

- [x] Existing Rust and Bazel test infrastructure covers typed evaluators and parity admission.
- [x] Existing Phase 33 fixtures cover detector, restart, cleanup, and restoration patterns to reuse.
- [ ] `//scripts:phase35_correlated_evidence_test` — add simulation/failure-injection coverage before hardware execution.
- [ ] `just phase35-evidence` — add the repo-owned bounded evidence entrypoint before hardware execution.

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
| --- | --- | --- | --- |
| Exact-current-package Ultra 205 evidence root | CFG-12, EVD-10–EVD-15 | Requires the physically connected board and one approved normal reboot | Invoke `just phase35-evidence capture-timeout-seconds=360 wifi-credentials=wifi-credentials.json` once with ≥420-second caller wall clock. The command owns the sole detector call; the caller must not check/open/stat the opaque credential path, and the command may validate/access it only after detector capability succeeds. Require protected root, restoration, cleanup, redaction, exact admission, and one eligible atomically admitted result. |

## Validation Sign-Off

- [x] Every planned task has a narrow automated verification target or an explicit Wave 0 dependency.
- [x] Sampling continuity has no three consecutive tasks without automated verification.
- [x] Wave 0 names all missing Phase 35 targets.
- [x] No watch-mode flags are used.
- [x] Software feedback latency target is below 240 seconds.
- [x] Hardware capture and wall-clock floors match repo policy.
- [x] `nyquist_compliant: true` is set.

**Approval:** approved 2026-07-17
