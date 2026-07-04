---
phase: 21
slug: live-mining-and-soak-evidence
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-07-04
---

# Phase 21 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

***

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | Bazel `9.1.1` wrapping Rust and shell tests; Cargo/Rust workspace tests |
| **Config file** | `BUILD.bazel`, per-package `BUILD.bazel`, `Cargo.toml`, `MODULE.bazel` |
| **Quick run command** | `bazel test //tools/parity:tests //scripts:phase21_live_mining_package_test //scripts:phase21_live_mining_evidence_test //scripts:phase15_bm1366_diagnostic_package_test` |
| **Full suite command** | `just test` plus Rust pre-commit checks before commits |
| **Estimated runtime** | ~60 seconds for scoped existing tests; hardware and firmware package commands vary by plan |

***

## Sampling Rate

- **After every task commit:** Run the task-scoped Rust/script test command for the changed path, plus Rust pre-commit checks before the commit.
- **After every plan wave:** Run `just test`, `just parity`, and `just verify-reference`.
- **Before `/gsd-verify-work`:** Full suite, redaction review, reference cleanliness, lifecycle validation, and every hardware/network command actually used must be green or explicitly blocked with evidence.
- **Max feedback latency:** 600 seconds for software-only scoped checks; hardware smoke/soak latency must be bounded by the active plan.

***

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Threat Ref | Secure Behavior | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|------------|-----------------|-----------|-------------------|-------------|--------|
| 21-W0-01 | 21-01 | 0 | ASIC-07, STR-06, STR-07 | T-21-01 / credential disclosure, unsafe command shape | New or reused mining wrappers cannot run outside detector, package, allow-manifest, redaction, enablement-summary, and safe-state gates | unit + script | `cargo test -p bitaxe-parity --all-features mining_allow`; `bazel test //tools/parity:tests`; `bazel test //scripts:phase21_live_mining_evidence_test` | ✅ existing parity tests; Phase 21 extensions planned | ⬜ pending |
| 21-W0-02 | 21-01, 21-02 | 0-1 | ASIC-07, STR-06 | T-21-02 / live mining enablement ambiguity | Firmware live-mining readiness is audited and `blocked_by_default` is converted into a controlled package enablement path before any live smoke claim | static + script | `21-01 Task 3 readiness audit command`; `bazel test //scripts:phase21_live_mining_package_test` | ✅ existing code; enablement artifact planned | ⬜ pending |
| 21-W0-03 | 21-07 | 6 | SAFE-09, EVD-05 | T-21-07 / watchdog overclaim, evidence gap | Bounded mining/soak plans include duration, abort conditions, responsiveness observations, and final safe-state markers | script + review | `bazel test //scripts:phase14_self_test_watchdog_load_test //scripts:phase20_failure_paths_test`; `bazel test //scripts:phase21_live_mining_evidence_test` | ✅ existing tests; Phase 21 helper tests planned | ⬜ pending |
| 21-W0-04 | 21-01, 21-08 | 0, 7 | EVD-05 | T-21-08 / secret leakage | Evidence tree has a redaction-review scaffold and deterministic scoped secret-pattern scan before citation | workflow | `21-08 Task 1 deterministic redaction scan command` | ❌ W0 scaffold needed | ⬜ pending |
| 21-PHASE | all | all | ASIC-07, STR-06, STR-07, SAFE-09, EVD-05 | T-21-final / parity overclaim | Checklist promotions are exact-claim only and `just parity` rejects blocker language or missing metadata | workflow | `just test`; `just parity`; `just verify-reference`; `git diff -- reference/esp-miner --exit-code`; lifecycle validation | ✅ existing commands | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

***

## Wave 0 Requirements

- [ ] Add tests for any `tools/parity/src/mining_allow.rs` Phase 21 command-shape extension before new hardware wrapper use.
- [ ] Add a Phase 21 wrapper test target under `scripts/BUILD.bazel` if a new `scripts/phase21-*.sh` wrapper is introduced.
- [ ] Add or record a live-mining readiness audit artifact and controlled live-mining package enablement artifact before planning live smoke, because current firmware startup remains fail-closed by default.
- [ ] Add a redaction-review scaffold for Phase 21 before final checklist citations.

***

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| Detector-gated Ultra 205 hardware smoke | ASIC-07, STR-06, STR-07, SAFE-09, EVD-05 | Requires a connected board `205` and USB/ESP-IDF tooling | Run `just detect-ultra205`; continue only for exactly one candidate and board-info pass; record full detector output in evidence. |
| Explicit-target live API/WebSocket telemetry | STR-07, SAFE-09, EVD-05 | Requires an operator-supplied origin-only `DEVICE_URL` that must not be inferred or committed raw | Run bounded `/api/system/info` and `/api/ws/live` captures only through repo-owned helpers with redaction; block if no explicit target exists. |
| Live-pool smoke or bounded soak | ASIC-07, STR-06, STR-07, SAFE-09, EVD-05 | Requires disposable/non-secret pool configuration, safe-stop recovery, and physical hardware observation | Use allow-manifest-validated commands only; record board, port, source commit, reference commit, package manifest, logs, share/no-share result, watchdog observations, safe-stop, and redaction conclusion. |

***

## Validation Sign-Off

- [ ] All tasks have `<automated>` verify or Wave 0 dependencies
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify
- [ ] Wave 0 covers all MISSING references
- [ ] No watch-mode flags
- [ ] Feedback latency < 600s for software-only scoped checks
- [ ] `nyquist_compliant: true` set in frontmatter

**Approval:** pending
