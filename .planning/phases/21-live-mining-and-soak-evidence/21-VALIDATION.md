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
| **Quick run command** | `bazel test //tools/parity:tests //scripts:phase15_controlled_mining_test //scripts:phase15_bm1366_diagnostic_package_test` |
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
| 21-W0-01 | TBD | 0 | ASIC-07, STR-06, STR-07 | T-21-W0 / credential disclosure, unsafe command shape | New or reused mining wrappers cannot run outside detector, package, allow-manifest, redaction, and safe-state gates | unit + script | `cargo test -p bitaxe-parity --all-features mining_allow`; `bazel test //tools/parity:tests` | ✅ existing parity tests; Phase 21 extensions TBD | ⬜ pending |
| 21-W0-02 | TBD | 0 | ASIC-07, STR-06 | T-21-W0 / live mining enablement ambiguity | Firmware live-mining readiness is audited before any live smoke claim; missing enablement produces blocked evidence instead of a false run | static + script | `rg -n "mining_loop_status|work_submission|hardware_evidence_ack|BITAXE_ASIC_DIAGNOSTIC" firmware/bitaxe crates/bitaxe-stratum scripts tools` | ✅ existing code; audit artifact TBD | ⬜ pending |
| 21-W0-03 | TBD | 0 | SAFE-09, EVD-05 | T-21-W0 / watchdog overclaim, evidence gap | Bounded mining/soak plans include duration, abort conditions, responsiveness observations, and final safe-state markers | script + review | `bazel test //scripts:phase14_self_test_watchdog_load_test //scripts:phase20_failure_paths_test` plus Phase 21 helper tests if added | ✅ existing tests; Phase 21 helper tests TBD | ⬜ pending |
| 21-W0-04 | TBD | 0 | EVD-05 | T-21-W0 / secret leakage | Evidence tree has a redaction-review scaffold and scoped secret-pattern scan before citation | workflow | `rg -n -i "ssid|wifi|password|pool|worker|token|device_url|nvs|stratum|https?://|([0-9]{1,3}\\.){3}[0-9]{1,3}|([[:xdigit:]]{2}:){5}[[:xdigit:]]{2}|secret|credential" docs/parity/evidence/phase-21-live-mining-and-soak-evidence` | ❌ W0 scaffold needed | ⬜ pending |
| 21-PHASE | all | all | ASIC-07, STR-06, STR-07, SAFE-09, EVD-05 | T-21-final / parity overclaim | Checklist promotions are exact-claim only and `just parity` rejects blocker language or missing metadata | workflow | `just test`; `just parity`; `just verify-reference`; `git diff -- reference/esp-miner --exit-code`; lifecycle validation | ✅ existing commands | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

***

## Wave 0 Requirements

- [ ] Add tests for any `tools/parity/src/mining_allow.rs` Phase 21 command-shape extension before new hardware wrapper use.
- [ ] Add a Phase 21 wrapper test target under `scripts/BUILD.bazel` if a new `scripts/phase21-*.sh` wrapper is introduced.
- [ ] Add or record a live-mining readiness audit artifact before planning live smoke, because current firmware startup remains fail-closed by default.
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
