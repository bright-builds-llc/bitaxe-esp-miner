---
phase: 25
slug: live-stratum-runtime-and-safe-stop
status: complete
nyquist_compliant: true
wave_0_complete: true
created: 2026-07-05T02:02:00.729Z
---

# Phase 25 - Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

## Test Infrastructure

| Property | Value |
| --- | --- |
| **Framework** | Bazel `rust_test`, Bazel `sh_test`, repo-owned `just` commands |
| **Config file** | `MODULE.bazel`, per-package `BUILD.bazel`, `Justfile` |
| **Quick run command** | `bazel test //crates/bitaxe-stratum:tests //crates/bitaxe-safety:tests` |
| **Full suite command** | `just test` or `bazel test //...` when practical before phase closure |
| **Estimated runtime** | Depends on firmware/toolchain state; use affected-target checks after each task and full relevant checks at phase gate |

## Sampling Rate

- **After every task commit:** Run the affected pure target, usually `bazel test //crates/bitaxe-stratum:tests` or `bazel test //crates/bitaxe-safety:tests`.
- **After every firmware adapter wave:** Run pure tests plus `bazel build //firmware/bitaxe:firmware`.
- **After every evidence tooling wave:** Run the relevant script test target, parity validator target, and `just parity`.
- **Before `/gsd-verify-work`:** Full relevant repo-native verification must be green, and detector-gated hardware evidence or an explicit blocked non-claim must be recorded.
- **Max feedback latency:** Prefer one task or one wave between automated feedback runs; do not let three implementation tasks pass without an automated check.

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Threat Ref | Secure Behavior | Test Type | Automated Command | File Exists | Status |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| 25-01-01 | 01 | 1 | STR-08 | T-25-02 / T-25-05 | Live runtime state is typed and redaction-safe before firmware socket I/O uses it. | unit | `bazel test //crates/bitaxe-stratum:tests` | created | pass |
| 25-01-02 | 01 | 1 | STR-09, STR-11 | T-25-01 / T-25-02 / T-25-03 | Submit response classification cannot overclaim fake-pool, stale, or unrelated responses. | unit | `bazel test //crates/bitaxe-stratum:tests` | created | pass |
| 25-01-03 | 01 | 1 | STR-11 | T-25-02 / T-25-03 | Fake-pool fixtures prove accepted, rejected, blocked, timeout, reconnect, clean-jobs, and malformed paths. | unit | `bazel test //crates/bitaxe-stratum:tests` | extended | pass |
| 25-02-01 | 02 | 2 | SAFE-12 | T-25-03 / T-25-05 | Safe stop invalidates socket/runtime/work state and disables mining/hardware-control surfaces. | unit + build | `bazel test //crates/bitaxe-stratum:tests //crates/bitaxe-safety:tests && bazel build //firmware/bitaxe:firmware` | created | pass |
| 25-02-02 | 02 | 2 | SAFE-13 | T-25-04 | Runtime loop uses bounded socket timeouts and watchdog checkpoints/yields. | unit + build | `bazel test //crates/bitaxe-safety:tests && bazel build //firmware/bitaxe:firmware` | extended | pass |
| 25-03-01 | 03 | 3 | STR-08, STR-09, SAFE-12, SAFE-13 | T-25-01 / T-25-04 / T-25-05 | Evidence wrapper preserves detector gating, runtime-only secrets, redaction, safe-stop, and exact non-claims. | script + parity | `bazel test //scripts:phase25_live_stratum_evidence_test && just parity` | created | pass |
| 25-03-02 | 03 | 3 | STR-08, STR-09, SAFE-12, SAFE-13 | T-25-02 / T-25-03 / T-25-05 | Mining allow rules admit only the repo-owned Phase 25 live-or-blocked command surface and reject unsafe or overclaiming manifests. | parity unit | `bazel test //tools/parity:tests` | extended | pass |
| 25-03-03 | 03 | 3 | STR-08, STR-09, STR-11, SAFE-12, SAFE-13 | T-25-01 / T-25-02 / T-25-03 / T-25-04 / T-25-05 | Evidence docs, checklist rows, redaction review, validation metadata, reference cleanliness, and lifecycle state close with exact supported claims. | parity + lifecycle | `bazel test //scripts:phase25_live_stratum_evidence_test //tools/parity:tests && just parity && just verify-reference && node "$HOME/.cursor/get-shit-done/bin/gsd-tools.cjs" verify lifecycle 25 --expect-id 25-2026-07-05T01-55-45 --expect-mode yolo --require-plans` | created | pass |

## Wave 0 Requirements

- [x] `crates/bitaxe-stratum/src/v1/live_runtime.rs` - Pure live lifecycle, outbound sequencing, stop, reconnect, and fallback transitions for STR-08 and SAFE-12.
- [x] `crates/bitaxe-stratum/src/v1/submit_response.rs` - Pure submit response classifier for STR-09 and STR-11.
- [x] `crates/bitaxe-stratum/src/v1/fake_pool.rs` - Expanded clean-jobs, reconnect generation, stale work, timeout, malformed, blocked, accepted, and rejected tests.
- [x] `firmware/bitaxe/src/live_stratum_runtime.rs` - ESP-IDF socket shell with bounded timeouts, watchdog checkpoints, and redacted lifecycle markers.
- [x] `scripts/phase25-live-stratum-evidence.sh` and `scripts/phase25-live-stratum-evidence-test.sh` - Detector-gated evidence wrapper and redaction tests.
- [x] `tools/parity/src/mining_allow.rs` - Phase 25 live Stratum surface, claim tier, and allowed command validation.

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
| --- | --- | --- | --- |
| Detector-gated live pool submit response classified as accepted or rejected, or explicit safe-prerequisite blocker recorded | STR-09 | Requires connected Ultra 205, safe prerequisites, runtime-only pool credentials, Wi-Fi/network, and live pool behavior | Run `just detect-ultra205`; if exactly one Ultra 205 is detected and runtime-only credentials are available, use the repo-owned Phase 25 evidence wrapper. Commit only redacted artifacts. If blocked, record exact blocker and non-claim. |
| Watchdog responsiveness under live hardware socket/ASIC/API/WebSocket/evidence load | SAFE-13 | Hardware timing and ESP-IDF scheduling cannot be fully proven by pure tests | Use detector-gated Phase 25 evidence wrapper when available; otherwise keep hardware-level SAFE-13 below verified and cite pure tests plus explicit non-claim. |

## Validation Sign-Off

- [x] All tasks have automated verify commands or Wave 0 dependencies.
- [x] Sampling continuity: no three consecutive tasks without automated verification.
- [x] Wave 0 covers all missing references.
- [x] No watch-mode flags are used in verification commands.
- [x] Phase gate includes `just parity`, `just verify-reference`, lifecycle validation, and relevant affected-target checks.
- [x] `nyquist_compliant: true` is set only after the planned task map and verification commands are complete.

**Approval:** passed by automated phase gate on 2026-07-05.
