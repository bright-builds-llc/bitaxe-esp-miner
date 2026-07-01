---
phase: 14
slug: safety-hardware-evidence-completion
status: passed
nyquist_compliant: true
wave_0_complete: true
created: 2026-06-30
verified: 2026-07-01T02:00:20Z
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
| 14-01-01 | 01 | 1 | SAFE-01, SAFE-04, SAFE-08, EVD-05 | T-14-01 | RED tests define allow-manifest invariants before implementation: board `205`, detector port match, passed board-info, package identity, recovery, redaction, and active `hardware-regression`. | unit/workflow | `cargo test -p bitaxe-parity --all-features safety_allow` | no W0 | passed |
| 14-01-02 | 01 | 1 | SAFE-01, SAFE-04, SAFE-08, EVD-05 | T-14-01 | `safety-allow` validates board, detector, package identity, surface, claim tier, evidence class, allowed command, abort conditions, recovery, safe-state markers, evidence dir, redaction reviewer, and checklist rows before wrappers run. | unit/workflow | `cargo test -p bitaxe-parity --all-features safety_allow && bazel test //tools/parity:tests --test_filter=safety_allow && just parity` | no W0 | passed |
| 14-02-01 | 02 | 1 | SAFE-01, SAFE-02, SAFE-05, SAFE-06, SAFE-07, SAFE-08, SAFE-09, EVD-05 | T-14-02 | Component evidence contract defines all eight Phase 14 packs, required metadata, allow-manifest command shape, prohibited actions, and conservative `hardware evidence pending` outcomes. | workflow | `rg -n "safe-baseline|power-telemetry|voltage-control|thermal-fan|self-test-watchdog-load|display-input|live-api-websocket-telemetry|parity-redaction|safety-allow --manifest|hardware evidence pending" docs/parity/evidence/phase-14-safety-hardware-evidence-completion/README.md` | no W0 | passed |
| 14-02-02 | 02 | 1 | EVD-05, SAFE-08 | T-14-05 | Redaction review template starts pending and enumerates every artifact class plus serial, JSON, API, WebSocket, terminal, manual, and secret surfaces before citation. | workflow | `rg -n "Current status: pending|Wi-Fi|pool URLs|private DEVICE_URL|NVS secret values|WebSocket frames|manual observations|safe-baseline|final-ledger|pending" docs/parity/evidence/phase-14-safety-hardware-evidence-completion/redaction-review.md` | no W0 | passed |
| 14-03-01 | 03 | 2 | SAFE-01, SAFE-02, SAFE-03, SAFE-04, SAFE-07, SAFE-08, EVD-05 | T-14-03 | Power/voltage and thermal/fan wrappers validate allow manifests, never run prohibited active commands, and fail closed to pending evidence when prerequisites are absent. | workflow/hardware-gated | `bash -n scripts/phase14-power-voltage.sh && bash -n scripts/phase14-power-voltage-test.sh && bash -n scripts/phase14-thermal-fan.sh && bash -n scripts/phase14-thermal-fan-test.sh && bazel test //scripts:phase14_power_voltage_test //scripts:phase14_thermal_fan_test` | no W0 | passed |
| 14-03-02 | 03 | 2 | SAFE-01, SAFE-02, SAFE-03, SAFE-04, SAFE-07, SAFE-08, EVD-05 | T-14-02 | Power/current/voltage and thermal/fan evidence either captures exact read-only observations or records pending blockers and non-claims for active voltage, fan duty, and fault behavior. | workflow/hardware-gated | `test -f docs/parity/evidence/phase-14-safety-hardware-evidence-completion/power-telemetry.md && test -f docs/parity/evidence/phase-14-safety-hardware-evidence-completion/voltage-control.md && test -f docs/parity/evidence/phase-14-safety-hardware-evidence-completion/thermal-fan.md && test -f docs/parity/evidence/phase-14-safety-hardware-evidence-completion/power-telemetry/power-voltage.log && test -f docs/parity/evidence/phase-14-safety-hardware-evidence-completion/thermal-fan/thermal-fan.log` | no W0 | passed with active/read-only blockers recorded |
| 14-04-01 | 04 | 3 | SAFE-05, SAFE-06, SAFE-08, SAFE-09, EVD-05 | T-14-04 | Self-test/watchdog/load and display/input wrappers parse exact safe markers but keep self-test hardware, load stress, and runtime display/input pending without bounded route and observation. | workflow/hardware-gated | `bash -n scripts/phase14-self-test-watchdog-load.sh && bash -n scripts/phase14-self-test-watchdog-load-test.sh && bash -n scripts/phase14-display-input.sh && bash -n scripts/phase14-display-input-test.sh && bazel test //scripts:phase14_self_test_watchdog_load_test //scripts:phase14_display_input_test` | no W0 | passed |
| 14-04-02 | 04 | 3 | SAFE-05, SAFE-06, SAFE-08, SAFE-09, EVD-05 | T-14-04 | Self-test/watchdog/load and display/input evidence records exact observed startup/supervisor markers or pending blockers and direct log artifacts. | workflow/hardware-gated/manual | `test -f docs/parity/evidence/phase-14-safety-hardware-evidence-completion/self-test-watchdog-load.md && test -f docs/parity/evidence/phase-14-safety-hardware-evidence-completion/display-input.md && test -f docs/parity/evidence/phase-14-safety-hardware-evidence-completion/self-test-watchdog-load/self-test-watchdog-load.log && test -f docs/parity/evidence/phase-14-safety-hardware-evidence-completion/display-input/display-input.log` | no W0 | passed with self-test/load/runtime-input blockers recorded |
| 14-05-01 | 05 | 4 | SAFE-01, SAFE-02, SAFE-07, SAFE-08, SAFE-09, EVD-05 | T-14-05 | Live API/WebSocket helper requires explicit `DEVICE_URL`, sanitizes API output, refuses network scans, and leaves frame proof pending without a maintained client. | workflow/hardware-gated | `bash -n scripts/phase14-live-telemetry.sh && bash -n scripts/phase14-live-telemetry-test.sh && bazel test //scripts:phase14_live_telemetry_test` | no W0 | passed |
| 14-05-02 | 05 | 4 | SAFE-01, SAFE-02, SAFE-07, SAFE-08, SAFE-09, EVD-05 | T-14-05 | Live telemetry evidence records explicit `DEVICE_URL` blockers, API/WebSocket route status, frame-client blockers, redaction status, and non-claims. | workflow/hardware-gated | `test -f docs/parity/evidence/phase-14-safety-hardware-evidence-completion/live-api-websocket-telemetry.md && test -f docs/parity/evidence/phase-14-safety-hardware-evidence-completion/live-api-websocket-telemetry/live-telemetry.log` | no W0 | blocked - missing explicit reachable `DEVICE_URL` and maintained WebSocket client |
| 14-06-01 | 06 | 5 | SAFE-01, SAFE-02, SAFE-03, SAFE-04, SAFE-05, SAFE-06, SAFE-07, SAFE-08, SAFE-09, EVD-05 | T-14-05 | Final ledger and redaction review state every exact safety row, supported claim, below-verified blocker, residual risk, and artifact review status. | workflow | `test -f docs/parity/evidence/phase-14-safety-hardware-evidence-completion.md && rg -n "PWR-003|PWR-006|THR-002|SELF-001|UI-003|API-006|STAT-002|SAFE-09|EVD-05|below verified|redaction review|hardware-regression|hardware-smoke" docs/parity/evidence/phase-14-safety-hardware-evidence-completion.md docs/parity/evidence/phase-14-safety-hardware-evidence-completion/redaction-review.md` | no W0 | passed |
| 14-06-02 | 06 | 5 | SAFE-01, SAFE-02, SAFE-03, SAFE-04, SAFE-05, SAFE-06, SAFE-07, SAFE-08, SAFE-09, EVD-05 | T-14-05 | Checklist and validation status promote only exact claims supported by matching evidence class and keep active safety-control rows guarded by `hardware-regression`. | workflow | `just parity && rg -n "Phase 14|hardware-regression|hardware-smoke|below verified|pending|blocked|phase-14-safety-hardware-evidence-completion" docs/parity/checklist.md .planning/phases/14-safety-hardware-evidence-completion/14-VALIDATION.md` | yes | passed |
| 14-06-03 | 06 | 5 | SAFE-08, EVD-05 | T-14-05 | Final verification runs all shell, Bazel, Cargo, parity, reference, diff, and lifecycle gates before phase completion. | workflow | `just parity && just verify-reference && node /Users/peterryszkiewicz/.codex/get-shit-done/bin/gsd-tools.cjs verify lifecycle 14 --expect-id 14-2026-06-30T23-56-34 --expect-mode yolo --require-plans --raw` | yes | passed |

## Wave 0 Requirements

- [x] `tools/parity/src/safety_allow.rs` — typed allow-manifest parser and validator for Phase 14 active hardware gates.
- [x] `tools/parity` tests for non-205 board, port mismatch, stale package, missing recovery, unsupported surface, missing safe-state marker, missing redaction reviewer, and active claim without `hardware-regression`.
- [x] `docs/parity/evidence/phase-14-safety-hardware-evidence-completion/README.md` — component-pack contract.
- [x] `docs/parity/evidence/phase-14-safety-hardware-evidence-completion/redaction-review.md` — artifact-specific redaction checklist.
- [x] `scripts/phase14-*-test.sh` targets for any new per-surface wrapper added by the plans.
- [x] Explicit dependency decision for live WebSocket frame evidence because `websocat` and Python WebSocket modules are absent locally.

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
- [x] Final Phase 14 verification passed on 2026-07-01T02:00:20Z:
  - `bash -n scripts/phase14-*.sh`
  - `bazel test //scripts:phase14_power_voltage_test //scripts:phase14_thermal_fan_test //scripts:phase14_self_test_watchdog_load_test //scripts:phase14_display_input_test //scripts:phase14_live_telemetry_test`
  - `cargo test -p bitaxe-safety --all-features power`
  - `cargo test -p bitaxe-safety --all-features thermal`
  - `cargo test -p bitaxe-safety --all-features self_test`
  - `cargo test -p bitaxe-safety --all-features watchdog`
  - `cargo test -p bitaxe-parity --all-features safety_allow`
  - `just parity`
  - `just test`
  - `just verify-reference`
  - `git diff -- reference/esp-miner --exit-code`
  - lifecycle validation returned `valid`.

**Approval:** approved 2026-06-30
