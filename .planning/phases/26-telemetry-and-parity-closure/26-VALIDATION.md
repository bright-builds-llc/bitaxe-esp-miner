---
phase: 26
slug: telemetry-and-parity-closure
status: complete
nyquist_compliant: true
wave_0_complete: true
created: 2026-07-05
---

# Phase 26 - Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

## Test Infrastructure

| Property | Value |
| --- | --- |
| **Framework** | Bazel `rust_test` / `sh_test` targets over Rust crates, tools, and scripts. |
| **Config file** | `BUILD.bazel` files per crate/tool/script. |
| **Quick run command** | `bazel test //crates/bitaxe-stratum:tests //crates/bitaxe-api:tests` |
| **Full suite command** | `just test`, plus `just parity`, `just verify-reference`, and lifecycle validation before closure. |
| **Estimated runtime** | Targeted crate tests under 120 seconds; full phase gate depends on firmware/package surfaces touched. |

## Sampling Rate

- **After every task commit:** Run the narrow Bazel target for the touched crate, tool, script, or firmware surface.
- **After every plan wave:** Run `bazel test //crates/bitaxe-stratum:tests //crates/bitaxe-api:tests //tools/parity:tests` and any touched script tests.
- **Before `/gsd-verify-work`:** Run relevant targeted tests, firmware build if firmware files changed, `just parity`, `just verify-reference`, and GSD lifecycle validation.
- **Max feedback latency:** Prefer under 120 seconds for narrow crate/tool tasks; document any longer firmware build gate in the plan summary.

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Threat Ref | Secure Behavior | Test Type | Automated Command | File Exists | Status |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| 26-01-01 | 01 | 1 | API-11, API-13 | T-26-01, T-26-04 | Projection contracts fold lifecycle, hashrate, blocked, safe-stop, and bounded sample marker events with redaction-safe labels. | unit | `bazel test //crates/bitaxe-stratum:tests` | yes | passed |
| 26-01-02 | 01 | 1 | API-13 | T-26-02, T-26-03 | Runtime share counters do not advance without current-generation submit intent plus parsed pool response, and submit classifications do not fabricate statistics samples. | unit | `bazel test //crates/bitaxe-stratum:tests` | yes | passed |
| 26-02-01 | 02 | 2 | API-11, API-12, API-13 | T-26-05, T-26-06, T-26-07 | API projection derives system info, statistics, scoreboard, and telemetry payloads from the shared projection and explicit sample markers. | unit | `bazel test //crates/bitaxe-api:tests` | yes | passed |
| 26-02-02 | 02 | 2 | API-11, API-12, API-13 | T-26-05, T-26-06, T-26-08 | Upstream-compatible API/WebSocket serialization preserves public fields, rejects request-time statistics fabrication, and avoids stale active-mining frames. | unit | `bazel test //crates/bitaxe-api:tests` | yes | passed |
| 26-03-01 | 03 | 3 | API-11, API-13 | T-26-09, T-26-10, T-26-11 | Phase 25 lifecycle, hashrate, submit classification, blocked prerequisite, and safe-stop producers fold into the projection before serialization. | unit + firmware compile | `bazel test //crates/bitaxe-stratum:tests //crates/bitaxe-api:tests && bazel build //firmware/bitaxe:firmware` | yes | passed |
| 26-03-02 | 03 | 3 | API-11, API-12, API-13 | T-26-10, T-26-11, T-26-12 | HTTP and WebSocket firmware consumers use projection-backed helpers and drain sample markers exactly once per runtime boundary. | unit + firmware compile | `bazel test //crates/bitaxe-api:tests && bazel build //firmware/bitaxe:firmware` | yes | passed |
| 26-04-01 | 04 | 4 | API-11, API-12, API-13, EVD-08 | T-26-13 | Redacted evidence artifacts map exact Phase 26 projection behavior and preserve blocked live-share non-claims. | artifact check | `test -f docs/parity/evidence/phase-26-telemetry-and-parity-closure/summary.md && rg -n "API-11|API-12|API-13|EVD-08|redaction_status: passed" docs/parity/evidence/phase-26-telemetry-and-parity-closure` | yes | passed |
| 26-04-02 | 04 | 4 | EVD-08 | T-26-14, T-26-15 | Parity tooling rejects overbroad verified telemetry/statistics/scoreboard claims and checklist rows cite exact non-claims. | tool/workflow | `bazel test //tools/parity:tests && just parity` | yes | passed |
| 26-04-03 | 04 | 4 | API-11, API-12, API-13, EVD-08 | T-26-16 | Final gate records real command results or exact blockers without secrets, and validation metadata reflects actual coverage. | phase gate | `bazel test //crates/bitaxe-stratum:tests //crates/bitaxe-api:tests //tools/parity:tests && bazel build //firmware/bitaxe:firmware && just parity && just verify-reference && node "$HOME/.cursor/get-shit-done/bin/gsd-tools.cjs" verify lifecycle 26 --expect-id 26-2026-07-05T03-48-38 --expect-mode yolo --require-plans` | yes | passed |

## Wave 0 Requirements

- [x] `crates/bitaxe-stratum/src/v1/telemetry_projection.rs` projection tests for event folding, bounded sample markers, stale sequence/generation rejection, safe-stop reset, and counter gating.
- [x] `crates/bitaxe-api` tests proving projection-backed `SystemInfoWire`, `StatisticsWire`, `ScoreboardEntryWire`, request-time sample non-fabrication, and `/api/ws/live` connect/cadence frames after safe stop.
- [x] Firmware producer/consumer compile-backed checks proving Phase 25 producer events fold into `RuntimeTelemetryProjection` and route helpers consume projected views.
- [x] `tools/parity` tests rejecting Phase 26 telemetry/statistics/scoreboard `verified` rows with missing artifacts, blocker language, pending evidence, or absent redaction review.
- [x] Phase 26 evidence directory with redaction-reviewed API/WebSocket/statistics/scoreboard artifacts and explicit blocked non-claims.

## Final Gate Results

- `bazel test //crates/bitaxe-stratum:tests //crates/bitaxe-api:tests //tools/parity:tests` passed.
- `bazel build //firmware/bitaxe:firmware` passed.
- `just parity` passed.
- `just verify-reference` passed.
- `node "$HOME/.cursor/get-shit-done/bin/gsd-tools.cjs" verify lifecycle 26 --expect-id 26-2026-07-05T03-48-38 --expect-mode yolo --require-plans` passed.
- Hardware evidence was not run during this static closure; detector-gated accepted/rejected live-share proof remains a non-claim.

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
| --- | --- | --- | --- |
| Detector-gated live Ultra 205 API/WebSocket observation | API-11, API-12, EVD-08 | Hardware evidence is allowed only if `just detect-ultra205` succeeds and safe prerequisites are available. | Run only through the repo-owned detector gate and evidence wrappers. If blocked, record exact non-claims instead of substituting stale or inferred targets. |

## Validation Sign-Off

- [x] All tasks have `<automated>` verify or Wave 0 dependencies.
- [x] Sampling continuity: no 3 consecutive tasks without automated verify.
- [x] Wave 0 covers all missing references.
- [x] No watch-mode flags.
- [x] Feedback latency is documented for any firmware build step over 120 seconds.
- [x] `nyquist_compliant: true` set in frontmatter after Wave 0 is complete.

**Approval:** passed by repo-native final gate.
