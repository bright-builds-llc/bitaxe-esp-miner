---
phase: 14
slug: safety-hardware-evidence-completion
status: draft
nyquist_compliant: true
wave_0_complete: false
created: 2026-06-30
---

# Phase 14 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

## Test Infrastructure

| Property | Value |
| --- | --- |
| **Framework** | Bazel `9.1.1` with `rules_rust 0.70.0`, plus Cargo `1.88.0-nightly` for Rust pre-commit checks |
| **Config file** | `MODULE.bazel`, per-crate `BUILD.bazel`, `scripts/BUILD.bazel`, root `Cargo.toml` |
| **Quick run command** | `cargo test -p bitaxe-safety --all-features && cargo test -p bitaxe-parity --all-features` |
| **Full suite command** | `just test` plus `just parity` |
| **Estimated runtime** | ~180 seconds for quick Rust checks; full suite depends on Bazel cache |

## Sampling Rate

- **After every task commit:** Run the focused command for the touched crate/script plus `git diff --check` on touched paths.
- **After every plan wave:** Run `just parity` and affected Bazel/Cargo test targets.
- **Before `/gsd-verify-work`:** Run redaction review, `just parity`, `just test`, `just verify-reference`, relevant Rust pre-commit checks, and lifecycle verification.
- **Max feedback latency:** 1 task commit for pure/tooling changes; 1 component evidence pack for hardware-gated evidence.

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Threat Ref | Secure Behavior | Test Type | Automated Command | File Exists | Status |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| 14-01-01 | 01 | 1 | SAFE-08, EVD-05 | T-14-01 | Allow manifests reject non-205 boards, stale package identity, missing recovery, missing redaction, and active claims without `hardware-regression`. | unit/workflow | `cargo test -p bitaxe-parity --all-features safety_allow` | no W0 | pending |
| 14-01-02 | 01 | 1 | SAFE-01, SAFE-04 | T-14-01 | Active voltage and fault probes cannot run without board-info, bounded inputs, abort conditions, recovery, and safe-state markers. | unit/workflow | `cargo test -p bitaxe-parity --all-features safety_allow` | no W0 | pending |
| 14-02-01 | 02 | 1 | SAFE-07, EVD-05 | T-14-02 | Component evidence packs preserve read-only, bounded actuation, projected telemetry, safe-unavailable, and pending conclusions separately. | workflow | `just parity` | no W0 | pending |
| 14-02-02 | 02 | 1 | SAFE-02, SAFE-07 | T-14-02 | Thermal/fan evidence separates sensor/RPM observations from fan duty actuation. | workflow/hardware-gated | `bazel test //scripts:phase14_thermal_fan_test` if wrapper is added | no W0 | pending |
| 14-03-01 | 03 | 2 | SAFE-05, SAFE-09 | T-14-03 | Self-test hardware submodes and watchdog/load checks stay blocked unless a bounded route and recovery path exist. | unit/workflow/hardware-gated | `cargo test -p bitaxe-safety --all-features self_test && cargo test -p bitaxe-safety --all-features watchdog` | partial | pending |
| 14-03-02 | 03 | 2 | SAFE-06 | T-14-04 | Runtime display/input remains pending unless a real runtime route is exercised and observed; startup-only display evidence cannot verify runtime parity. | workflow/manual | `just parity` | yes | pending |
| 14-04-01 | 04 | 2 | SAFE-01, SAFE-02, SAFE-07 | T-14-02 | Live API/WebSocket telemetry uses explicit `DEVICE_URL`; missing client dependency or target writes blocked evidence. | workflow/hardware-gated | `bazel test //scripts:phase14_live_telemetry_test` if wrapper is added | no W0 | pending |
| 14-05-01 | 05 | 3 | SAFE-08, EVD-05 | T-14-05 | Checklist rows promote only exact claims supported by matching evidence class and redaction review. | workflow | `just parity` | yes | pending |

## Wave 0 Requirements

- [ ] `tools/parity/src/safety_allow.rs` — typed allow-manifest parser and validator for Phase 14 active hardware gates.
- [ ] `tools/parity` tests for non-205 board, port mismatch, stale package, missing recovery, unsupported surface, missing safe-state marker, missing redaction reviewer, and active claim without `hardware-regression`.
- [ ] `docs/parity/evidence/phase-14-safety-hardware-evidence-completion/README.md` — component-pack contract.
- [ ] `docs/parity/evidence/phase-14-safety-hardware-evidence-completion/redaction-review.md` — artifact-specific redaction checklist.
- [ ] `scripts/phase14-*-test.sh` targets for any new per-surface wrapper added by the plans.
- [ ] Explicit dependency decision for live WebSocket frame evidence because `websocat` and Python WebSocket modules are absent locally.

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
| --- | --- | --- | --- |
| Connected Ultra 205 detection | SAFE-01, SAFE-02, SAFE-07, EVD-05 | Requires physical board and USB serial access | Run `just detect-ultra205`; proceed only when exactly one port and board-info succeed. |
| Active voltage/fan/fault/self-test/load stimulus | SAFE-01, SAFE-02, SAFE-04, SAFE-05, SAFE-09 | Hardware actuation can damage the board if bounds or recovery are wrong | Run only through a Phase 14 allow manifest and surface wrapper that documents allowed inputs, stop conditions, recovery, and post-action safe state. |
| Runtime display/input observation | SAFE-06 | Requires physical display/input or a real runtime status route | Record physical/log/API/WebSocket observation; otherwise update evidence as pending with owner/blocker. |
| Redaction review | EVD-05 | Requires artifact-specific inspection for secrets and private endpoints | Inspect serial logs, JSON, API responses, WebSocket frames, terminal output, and manual observations before citation. |

## Validation Sign-Off

- [x] All tasks have an automated verify path or Wave 0 dependency.
- [x] Sampling continuity: no 3 consecutive tasks without automated verify.
- [x] Wave 0 covers all missing references identified in research.
- [x] No watch-mode flags.
- [x] Feedback latency is bounded by task commit or component evidence pack.
- [x] `nyquist_compliant: true` set in frontmatter.

**Approval:** approved 2026-06-30
