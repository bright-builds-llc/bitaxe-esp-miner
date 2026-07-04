---
phase: 21
slug: live-mining-and-soak-evidence
status: green
nyquist_compliant: true
wave_0_complete: true
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
| **Quick run command** | `cargo test -p bitaxe-stratum --all-features controlled_runtime && cargo test -p bitaxe-stratum --all-features mining_loop && bazel test //tools/parity:tests //crates/bitaxe-stratum:tests --test_filter=controlled_runtime //scripts:phase21_live_mining_package_test //scripts:phase21_live_mining_evidence_test //scripts:phase15_bm1366_diagnostic_package_test` |
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
| 21-W0-01 | 21-01 | 0 | ASIC-07, STR-06, STR-07 | T-21-01 / credential disclosure, unsafe command shape | Phase 21 mining wrappers cannot run outside detector, package, allow-manifest, redaction, enablement-summary, and safe-state gates | unit + script | `cargo test -p bitaxe-parity --all-features mining_allow`; `bazel test //tools/parity:tests --test_filter=mining_allow`; `bazel test //scripts:phase21_live_mining_evidence_test` | ✅ `tools/parity/src/mining_allow.rs`; `scripts/phase21-live-mining-evidence.sh`; `scripts/phase21-live-mining-evidence-test.sh` | ✅ green |
| 21-W0-02 | 21-01 | 0 | ASIC-07, STR-06 | T-21-02 / readiness-only mining path, missing runtime harness | Firmware live-mining readiness is audited as `blocked_by_default`; live commands require a later bounded controlled runtime/harness pack before any live smoke claim | static + review | `rg -n "mining_loop_status\|work_submission\|hardware_evidence_ack\|BITAXE_ASIC_DIAGNOSTIC" firmware/bitaxe crates/bitaxe-stratum scripts tools` | ✅ `docs/parity/evidence/phase-21-live-mining-and-soak-evidence/readiness-audit.md`; `docs/parity/evidence/phase-21-live-mining-and-soak-evidence/preflight.md`; `docs/parity/evidence/phase-21-live-mining-and-soak-evidence/live-mining-enablement.md`; `docs/parity/evidence/phase-21-live-mining-and-soak-evidence/bm1366-init-work-result.md` | ✅ green |
| 21-W0-03 | 21-01, 21-06 | 0 | SAFE-09, EVD-05 | T-21-07 / watchdog overclaim, evidence gap | Live smoke and bounded soak plans require controlled runtime/harness markers, duration, abort conditions, runtime snapshot/API/WebSocket updates, responsiveness observations, and final safe-state markers. Plan 21-06 sampled the live-smoke/API-WebSocket boundary as blocked evidence only; missing live prerequisites did not become watchdog, telemetry, share, or soak proof. | script + review | `bazel test //scripts:phase21_live_mining_evidence_test`; `rg -n "Phase 21 evidence ladder\|D-16\|60..600" docs/parity/evidence/phase-21-live-mining-and-soak-evidence/evidence-contract.md`; `rg -n "live_mining_smoke_status: blocked\|telemetry_correlation_status: blocked\|network_scan: disabled\|websocket_frame_status: blocked" docs/parity/evidence/phase-21-live-mining-and-soak-evidence/live-mining-smoke.md docs/parity/evidence/phase-21-live-mining-and-soak-evidence/live-api-websocket-telemetry.md` | ✅ `docs/parity/evidence/phase-21-live-mining-and-soak-evidence/evidence-contract.md`; `docs/parity/evidence/phase-21-live-mining-and-soak-evidence/live-mining-smoke.md`; `docs/parity/evidence/phase-21-live-mining-and-soak-evidence/live-api-websocket-telemetry.md` | ✅ green - blocked boundary sampled |
| 21-W0-04 | 21-01 | 0 | EVD-05 | T-21-08 / secret leakage | Evidence tree has a redaction-review scaffold and deterministic scoped secret-pattern scan before citation | workflow | `rg -n -i "ssid\|wifi\|password\|pool\|worker\|token\|device_url\|nvs\|stratum\|https?://\|([0-9]{1,3}\\.){3}[0-9]{1,3}\|([[:xdigit:]]{2}:){5}[[:xdigit:]]{2}\|secret\|credential" docs/parity/evidence/phase-21-live-mining-and-soak-evidence` | ✅ `docs/parity/evidence/phase-21-live-mining-and-soak-evidence/redaction-review.md` | ✅ green |
| 21-PHASE | all | all | ASIC-07, STR-06, STR-07, SAFE-09, EVD-05 | T-21-final / parity overclaim | Checklist promotions are exact-claim only and `just parity` rejects blocker language or missing metadata | workflow | `just test`; `just parity`; `just verify-reference`; `git diff -- reference/esp-miner --exit-code`; lifecycle validation | ✅ existing commands | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

***

## Wave 0 Requirements

- [x] Add tests for any `tools/parity/src/mining_allow.rs` Phase 21 command-shape extension before new hardware wrapper use.
- [x] Add a Phase 21 wrapper test target under `scripts/BUILD.bazel` if a new `scripts/phase21-*.sh` wrapper is introduced.
- [x] Add or record a live-mining readiness audit artifact and the required bounded controlled runtime/harness plus live-mining package enablement markers that later plans must satisfy before live smoke, because current firmware startup remains fail-closed by default and readiness-only evidence cannot satisfy STR-06.
- [x] Add a redaction-review scaffold for Phase 21 before final checklist citations.

***

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| Detector-gated Ultra 205 hardware smoke | ASIC-07, STR-06, STR-07, SAFE-09, EVD-05 | Requires a connected board `205` and USB/ESP-IDF tooling | Run `just detect-ultra205`; continue only for exactly one candidate and board-info pass; record full detector output in evidence. |
| Explicit-target live API/WebSocket telemetry | STR-07, SAFE-09, EVD-05 | Requires an operator-supplied origin-only `DEVICE_URL` that must not be inferred or committed raw | Run bounded `/api/system/info` and `/api/ws/live` captures only through repo-owned helpers with redaction; block if no explicit target exists. |
| Live-pool smoke or bounded soak | ASIC-07, STR-06, STR-07, SAFE-09, EVD-05 | Requires disposable/non-secret pool configuration, safe-stop recovery, and physical hardware observation | Use allow-manifest-validated commands only; record board, port, source commit, reference commit, package manifest, logs, share/no-share result, watchdog observations, safe-stop, and redaction conclusion. |

***

## Validation Sign-Off

Plan 21-01 artifacts provide the Wave 0 software validation scaffold and blocked-by-default readiness audit. They do not claim that the controlled runtime/harness or live-mining package enablement packs are complete.

- [x] All tasks have `<automated>` verify or Wave 0 dependencies
- [x] Sampling continuity: no 3 consecutive tasks without automated verify
- [x] Wave 0 covers all MISSING references
- [x] No watch-mode flags
- [x] Feedback latency < 600s for software-only scoped checks
- [x] `nyquist_compliant: true` set in frontmatter

**Approval:** Plan 21-01 Wave 0 validation scaffolding complete.
