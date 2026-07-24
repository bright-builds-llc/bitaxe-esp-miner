---
phase: "36"
slug: substantive-evidence-admission-and-exact-re-promotion
status: draft
nyquist_compliant: true
wave_0_complete: false
created: "2026-07-23"
---

# Phase 36 — Validation Strategy

> Per-phase validation contract for substantive evidence admission and exact re-promotion.

## Test Infrastructure

| Property | Value |
| --- | --- |
| **Framework** | Rust unit tests, Bazel integration tests, repository shell fixtures |
| **Config file** | `Cargo.toml`, `tools/parity/BUILD.bazel`, `scripts/BUILD.bazel` |
| **Quick run command** | `cargo test -p bitaxe-parity phase36` |
| **Full suite command** | `just test && just parity && just verify-reference && just verify-redaction` |
| **Estimated runtime** | ~180 seconds without hardware |

## Sampling Rate

- **After every task commit:** Run the narrow Phase 36 Rust or Bazel target named by that task.
- **After every plan wave:** Run `just test && just parity && just verify-reference && just verify-redaction`.
- **Before phase verification:** Run the full Rust pre-commit sequence and every Phase 36 integration target.
- **Max feedback latency:** 180 seconds for task-local checks.

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Threat Ref | Secure Behavior | Test Type | Automated Command | File Exists | Status |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| 36-01-01 | 01 | 1 | EVD-11, EVD-12, EVD-14 | T36-01, T36-02 | Fixtures use closed fake values and immutable explicit roots | integration | `bazel test //tools/parity:phase36_evidence_tests` | ✅ | ✅ passed: 1/1 target (fresh 2026-07-24) |
| 36-01-02 | 01 | 1 | EVD-14 | T36-01 | Protected input never appears in terminal or public output | real-process | `bazel test //scripts:phase36_evidence_test` | ✅ | ✅ passed: 1/1 target (fresh 2026-07-24) |
| 36-02-01 | 02 | 2 | EVD-12, EVD-15 | T36-03 | Sensor/health substance and provenance joins fail closed | unit | `cargo test -p bitaxe-parity phase36_substance` | ✅ | ✅ passed: 14/14 tests (fresh 2026-07-24) |
| 36-02-02 | 02 | 2 | SYS-02, EVD-11 | T36-03 | Runtime identity is replayed from observations, not package fields | unit | `cargo test -p bitaxe-parity phase36_runtime_identity` | ✅ | ✅ passed: 11/11 tests (fresh 2026-07-24) |
| 36-02-03 | 02 | 2 | EVD-14 | T36-03, T36-04 | Legacy effect proof is complete or typed insufficient | unit | `cargo test -p bitaxe-parity phase36_effects` | ✅ | ✅ passed: 12/12 tests (fresh 2026-07-24) |
| 36-03-01 | 03 | 3 | EVD-15 | T36-05 | Claim decisions and checklist correction publish atomically | integration | `bazel test //tools/parity:phase36_promotion_tests` | ✅ | ✅ passed: 1/1 target (fresh 2026-07-24) |
| 36-03-02 | 03 | 3 | SYS-02, EVD-11, EVD-12, EVD-14, EVD-15 | T36-06 | Attempt 31 classification cannot trigger hardware | real-process | `bazel test //scripts:phase36_evidence_test` | ✅ | ✅ passed: 1/1 target (fresh 2026-07-24) |
| 36-03-03 | 03 | 3 | SYS-02, EVD-11, EVD-12, EVD-14, EVD-15 | T36-03, T36-06 | Attempt 31 yields exact decisions or aggregate typed insufficiency without hardware | real-process | `bazel test //scripts:phase36_evidence_test //tools/parity:phase36_promotion_tests` | ✅ | ✅ passed: 2/2 targets (fresh 2026-07-24) |
| 36-04-01 | 04 | 4 | SYS-02, EVD-11, EVD-12, EVD-14, EVD-15 | all | Reconciliation occurs only after clean verification | repository | `just test && just parity && just verify-reference && just verify-redaction` | ✅ | ✅ passed: 73/73; no parity errors; reference clean; redaction passed (fresh 2026-07-24) |
| 36-04-02 | 04 | 4 | SYS-02, EVD-11, EVD-12, EVD-14, EVD-15 | all | Independent verifier produces a passed lifecycle-bound artifact before reconciliation | lifecycle | `node "$HOME/.codex/get-shit-done/bin/gsd-tools.cjs" verify lifecycle 36 --require-plans --require-verification --raw` | ❌ W0 | ⬜ pending |
| 36-04-03 | 04 | 4 | SYS-02, EVD-11, EVD-12, EVD-14, EVD-15 | T36-05, T36-06 | Canonical planning truth changes only after independent passed verification | repository | `just parity && just verify-reference && just verify-redaction` | ✅ | ⬜ pending |

## Wave 0 Requirements

- [ ] Phase 36 fixture factory and one-field mutation matrix.
- [ ] Fake protected-root process harness with permission and redaction assertions.
- [ ] Successor publisher rollback/failure-injection fixtures.
- [ ] Named Cargo/Bazel targets referenced in the verification map.

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
| --- | --- | --- | --- |
| Any new hardware evidence attempt | EVD-11, EVD-12, EVD-14 | Not authorized by the base phase; depends on typed immutable-artifact insufficiency and later explicit plan authority | Stop after the insufficiency result. Add a later Phase 36 plan containing the detector, recovery, effects, evidence, timeout, privacy, and progress-gated contract before hardware use. |

## Validation Sign-Off

- [x] All planned tasks have automated verification or Wave 0 dependencies.
- [x] Sampling continuity avoids three consecutive tasks without automated verification.
- [x] Wave 0 names all missing test surfaces.
- [x] No watch-mode flags.
- [x] Feedback latency target is under 180 seconds.
- [x] `nyquist_compliant: true` is set in frontmatter.

**Approval:** pending plan checker and phase execution
