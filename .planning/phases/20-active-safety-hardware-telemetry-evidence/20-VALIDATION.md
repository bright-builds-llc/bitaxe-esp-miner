---
phase: 20
slug: active-safety-hardware-telemetry-evidence
status: draft
nyquist_compliant: true
wave_0_complete: false
created: 2026-07-03T21:06:06.180Z
lifecycle_mode: yolo
phase_lifecycle_id: 20-2026-07-03T20-48-00
---

# Phase 20 - Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | Bazel shell/Rust tests, Cargo Rust tests, repo-owned `just` aggregate commands |
| **Config file** | `scripts/BUILD.bazel`, `Cargo.toml`, `Justfile` |
| **Quick run command** | `bazel test //scripts:phase14_power_voltage_test //scripts:phase14_thermal_fan_test //scripts:phase14_self_test_watchdog_load_test //scripts:phase14_display_input_test //scripts:phase14_live_telemetry_test` |
| **Full suite command** | `just test && just parity && just verify-reference` |
| **Estimated runtime** | ~1200 seconds for full suite; narrow checks vary by changed files |

## Sampling Rate

- **After every task commit:** Run the narrow changed-path command named in the plan task, and run `just parity` after checklist or evidence-class changes.
- **After every plan wave:** Run all changed wrapper tests, changed Rust crate tests, `just parity`, and `just verify-reference`.
- **Before `/gsd-verify-work`:** `just test`, `just parity`, `just verify-reference`, redaction review, lifecycle validation, and every hardware/network command actually used must be complete.
- **Max feedback latency:** 1200 seconds for the full local suite unless ESP-IDF firmware rebuilds extend the run; hardware blockers must be recorded as evidence instead of silently skipped.

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Threat Ref | Secure Behavior | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|------------|-----------------|-----------|-------------------|-------------|--------|
| 20-W0-01 | 20-01 | 0 | SAFE-08, EVD-05 | T-20-01-01 | Safety allow and parity guards reject unsupported active safety claims and include first-class `failure-paths` surface coverage. | unit/workflow | `cargo test -p bitaxe-parity --all-features safety_allow && bazel test //tools/parity:tests --test_filter=safety_allow` | yes | complete |
| 20-W0-02 | TBD | 0 | SAFE-01, SAFE-02, SAFE-04 | T20-02 | Active hardware probes require board `205`, detector gate, bounded inputs, recovery steps, and final safe-state markers. | wrapper/parity | `bazel test //scripts:phase14_power_voltage_test //scripts:phase14_thermal_fan_test` plus any Phase 20 wrapper test targets | partial | pending |
| 20-W0-03 | TBD | 0 | SAFE-05, SAFE-09 | T20-03 | Self-test and load evidence cannot run without documented stimulus, abort conditions, recovery, and safe-state checks. | unit/wrapper | `cargo test -p bitaxe-safety --all-features self_test && cargo test -p bitaxe-safety --all-features watchdog && bazel test //scripts:phase14_self_test_watchdog_load_test` | yes | pending |
| 20-W0-04 | TBD | 0 | SAFE-06 | T20-04 | Runtime display/input claims stay below verified unless a real runtime route and observation exist. | wrapper/evidence | `bazel test //scripts:phase14_display_input_test` plus any Phase 20 display/input test targets | partial | pending |
| 20-W0-05 | TBD | 0 | SAFE-07 | T20-05 | Live API/WebSocket telemetry uses explicit target input, redaction, bounded capture, and correlation with hardware observations. | wrapper/live evidence | `bazel test //scripts:phase14_live_telemetry_test` plus `node scripts/phase17-websocket-capture.mjs --help` or a Phase 20 capture test target | partial | pending |
| 20-W0-06 | TBD | 0 | EVD-05 | T20-06 | Evidence committed to the repo has redaction review and cites only reviewed artifacts. | docs/check | redaction scan command defined by the plan, followed by `just parity` | no | pending |

## Wave 0 Requirements

- [x] Create Phase 20 evidence directory and redaction review scaffold before any hardware or network probe is cited.
- [ ] Reuse existing Phase 14 wrapper tests unless a Phase 20-specific helper changes behavior.
- [ ] Add Phase 20 wrapper tests for every new helper script created by the planner.
- [x] Extend `tools/parity/src/safety_allow.rs` and its tests if the planner chooses a standalone `failure-paths` safety surface.
- [ ] Record blocked evidence for missing detector, board-info, package identity, recovery path, explicit `DEVICE_URL`, WebSocket client, active route, or redaction prerequisite.

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| Physical fan response, runtime display/input, and any bench-visible behavior | SAFE-02, SAFE-06 | These require physical observation unless a safe firmware/API/log route exists. | Record observer, board `205`, selected port, package identity, exact stimulus, observed behavior, conclusion, and redaction status in the Phase 20 evidence pack. |
| Active voltage/fault/self-test/load probes | SAFE-01, SAFE-04, SAFE-05, SAFE-09 | These can affect hardware state and require recovery/abort judgment before execution. | Run only from a plan-approved allow manifest with bounded inputs, recovery steps, abort conditions, final safe-state markers, and redaction review. |
| Live `DEVICE_URL` target selection | SAFE-07, EVD-05 | Network target must be explicit and must not be inferred from scans or private network state. | Use explicit `DEVICE_URL` or a trusted target-lock artifact; record blocked evidence if unavailable. |

## Validation Sign-Off

- [x] All planned tasks must include automated verify commands or explicit manual-only evidence gates.
- [x] Sampling continuity: no 3 consecutive tasks without an automated verify command or blocked-evidence artifact.
- [x] Wave 0 covers missing wrapper tests, parity guard choices, evidence scaffold, and redaction prerequisites.
- [x] No watch-mode flags.
- [x] Feedback latency target documented.
- [x] `nyquist_compliant: true` set in frontmatter.

**Approval:** draft pending plan checker
