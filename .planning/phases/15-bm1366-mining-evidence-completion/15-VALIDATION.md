---
phase: 15
slug: bm1366-mining-evidence-completion
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-07-01
lifecycle_mode: yolo
phase_lifecycle_id: 15-2026-07-01T02-07-59
---

# Phase 15 - Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | Cargo/Rust tests plus Bazel script/tool tests |
| **Config file** | `Cargo.toml`, package `BUILD.bazel` files, `Justfile` |
| **Quick run command** | `cargo test -p bitaxe-asic --all-features bm1366 && cargo test -p bitaxe-stratum --all-features mining && cargo test -p bitaxe-parity --all-features` |
| **Full suite command** | `just test && just parity && just verify-reference` |
| **Estimated runtime** | ~300 seconds for full local suite, excluding hardware capture |

## Sampling Rate

- **After every task commit:** Run the narrow command for the touched surface: crate tests for Rust logic, `bash -n` plus Bazel script tests for shell wrappers, or `cargo test -p bitaxe-parity --all-features` for checklist/evidence guards.
- **After every plan wave:** Run `just test`, `just parity`, and `just verify-reference`; run `just detect-ultra205` and hardware commands only for waves that own hardware evidence.
- **Before `/gsd-verify-work`:** Full Rust pre-commit sequence, `just test`, `just parity`, `just verify-reference`, reference diff cleanliness, redaction review, lifecycle validation, and every detector/hardware command actually used must be green or explicitly blocked with evidence.
- **Max feedback latency:** 300 seconds for non-hardware checks; hardware-gated checks must record detector/board-info output and stop conditions before they run.

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Threat Ref | Secure Behavior | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|------------|-----------------|-----------|-------------------|-------------|--------|
| 15-W0-01 | TBD | 0 | ASIC-07 | T-15-01 | Package-backed diagnostic preserves trusted wrapper markers and does not weaken trust classification. | unit/workflow/hardware-gated | `cargo test -p bitaxe-asic --all-features adapter_gate chip_detect init_plan && bazel test //tools/flash:tests && just detect-ultra205` | ❌ W0 | ⬜ pending |
| 15-W0-02 | TBD | 0 | ASIC-07 | T-15-02 | Typed work/result diagnostic emits bounded result-or-timeout and fail-closed markers without raw serial writes. | unit/integration/hardware-gated | `cargo test -p bitaxe-asic --all-features work result transcript && cargo test -p bitaxe-stratum --all-features mining_loop` | ❌ W0 | ⬜ pending |
| 15-W0-03 | TBD | 0 | STR-06 | T-15-03 | Mining loop remains safety-gated and reaches active or controlled no-share only under exact prerequisites. | unit/workflow/hardware-gated | `cargo test -p bitaxe-stratum --all-features mining_loop fake_pool queue && cargo test -p bitaxe-api --all-features mining` | ✅ | ⬜ pending |
| 15-W0-04 | TBD | 0 | STR-07 | T-15-04 | Smoke/soak evidence records share or controlled no-share, duration, telemetry/watchdog, safe-stop, redaction, and conclusion. | workflow/hardware-gated | `bash -n scripts/phase15-*.sh && bazel test //scripts:phase15_*_test && just parity` after scripts exist | ❌ W0 | ⬜ pending |
| 15-W0-05 | TBD | 0 | SAFE-09 | T-15-05 | Bounded runs preserve watchdog/API/WebSocket/serial responsiveness or record missing `DEVICE_URL` as a blocker. | workflow/hardware-gated | `curl "$DEVICE_URL/api/system/info"` plus maintained WebSocket helper when `DEVICE_URL` exists; otherwise record pending evidence | ❌ W0 | ⬜ pending |
| 15-W0-06 | TBD | 0 | EVD-05 | T-15-06 | Checklist and parity guards reject overclaims and generated artifacts pass redaction review. | unit/workflow | `cargo test -p bitaxe-parity --all-features && just parity && rg -n "pool|password|token|DEVICE_URL|ssid|secret" docs/parity/evidence/phase-15-bm1366-mining-evidence-completion` | ❌ W0 | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

## Wave 0 Requirements

- [ ] Package-backed BM1366 chip-detect/staged-init diagnostic target or wrapper flow that preserves trusted package/SPIFFS markers.
- [ ] Bounded firmware diagnostic mode for typed BM1366 work-send/result-receive.
- [ ] Mining-specific allow manifest or parity allow extension for `bm1366-chip-detect`, `bm1366-work-result`, `mining-smoke`, `bounded-soak`, and `parity-redaction` surfaces.
- [ ] Phase 15 evidence scaffold and artifact-specific redaction review.
- [ ] Controlled mining smoke/soak wrapper with conditional `DEVICE_URL`, pool, WebSocket, watchdog, and safe-stop behavior.

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| Detector-gated Ultra 205 hardware run | ASIC-07, STR-06, STR-07, SAFE-09 | Requires a physically connected board and board-info output. | Run `just detect-ultra205`; continue only when exactly one ESP32-S3 port is selected and board-info succeeds. Record output in Phase 15 evidence. |
| Trusted BM1366 chip-detect/staged init capture | ASIC-07 | Requires flashing and monitoring a real board. | Run the plan-approved package-backed diagnostic wrapper. Evidence must show trusted wrapper output, package/SPIFFS markers, safe-state markers, chip-detect/staged-init markers, and no mining/work overclaim. |
| Typed work-send/result-receive diagnostic | ASIC-07, STR-06 | Requires a live BM1366 UART path and bounded timing. | Run the plan-approved typed diagnostic. Evidence must record command, board, port, commits, package manifest, result-or-timeout markers, fail-closed state, and conclusion. |
| Controlled mining smoke or bounded soak | STR-06, STR-07, SAFE-09 | Requires explicit pool or controlled no-share setup, stop conditions, and telemetry access when available. | Run only after earlier tiers pass. Record pool category without secrets, share/no-share outcome, hashrate inputs, watchdog/API/WebSocket or blocker status, safe-stop, and redaction review. |
| Redaction review | EVD-05 | Requires human-readable review of generated artifacts before commit. | Inspect logs, JSON, Markdown, API/WebSocket output, and terminal transcripts for pool credentials, Wi-Fi credentials, API tokens, private endpoints, NVS secret values, and local secrets. Document result in `redaction-review.md`. |

## Validation Sign-Off

- [ ] All tasks have `<automated>` verify commands or Wave 0 dependencies.
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify.
- [ ] Wave 0 covers all missing diagnostic, evidence, and redaction surfaces.
- [ ] No watch-mode flags.
- [ ] Feedback latency < 300 seconds for non-hardware checks.
- [ ] Hardware commands record detector/board-info gates and stop conditions.
- [ ] `nyquist_compliant: true` set in frontmatter after plans map every requirement and manual-only verification.

**Approval:** pending
