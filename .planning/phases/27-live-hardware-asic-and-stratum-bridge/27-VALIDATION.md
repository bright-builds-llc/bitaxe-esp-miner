---
phase: 27
slug: live-hardware-asic-and-stratum-bridge
status: complete
nyquist_compliant: true
wave_0_complete: true
created: 2026-07-05
---

# Phase 27 - Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

## Test Infrastructure

| Property | Value |
| --- | --- |
| **Framework** | Bazel `rust_test` / `sh_test` over crates, firmware wrapper, tools, and scripts. |
| **Config file** | `BUILD.bazel` files per crate/tool/script. |
| **Quick run command** | `bazel test //crates/bitaxe-stratum:tests //crates/bitaxe-asic:tests` |
| **Full suite command** | Phase gate in Final Gate Results below. |
| **Estimated runtime** | Crate tests under 120 seconds; firmware build adds ~5 seconds locally. |

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Threat Ref | Automated Command | Status |
| --- | --- | --- | --- | --- | --- | --- |
| 27-01-01 | 01 | 1 | STR-08, ASIC-10 | T-27-01, T-27-03, T-27-04 | `bazel test //crates/bitaxe-asic:tests && bazel build //firmware/bitaxe:firmware` | passed |
| 27-01-02 | 01 | 1 | STR-08, ASIC-10 | T-27-02, T-27-05 | `bazel test //crates/bitaxe-stratum:tests //crates/bitaxe-safety:tests && bazel build //firmware/bitaxe:firmware` | passed |
| 27-02-01 | 02 | 2 | STR-08, STR-09, ASIC-11 | T-27-06, T-27-07 | `bazel test //crates/bitaxe-stratum:tests --test_output=errors` | passed |
| 27-02-02 | 02 | 2 | STR-08, STR-09, ASIC-10, ASIC-11 | T-27-08, T-27-09 | `bazel test //crates/bitaxe-stratum:tests //crates/bitaxe-safety:tests && bazel build //firmware/bitaxe:firmware` | passed |
| 27-03-01 | 03 | 3 | STR-09 | T-27-10, T-27-12 | `bazel test //scripts:phase27_live_hardware_bridge_evidence_test --test_output=errors` | passed |
| 27-03-02 | 03 | 3 | STR-09 | T-27-11 | `bazel test //scripts:phase27_live_hardware_bridge_evidence_test && test -f docs/parity/evidence/phase-27-live-hardware-asic-and-stratum-bridge/share-outcome.md` | passed |
| 27-04-01 | 04 | 4 | STR-08, STR-09, ASIC-10, ASIC-11 | T-27-14 | `bazel test //tools/parity:tests --test_output=errors` | passed |
| 27-04-02 | 04 | 4 | STR-08, STR-09, ASIC-10, ASIC-11 | T-27-13, T-27-15 | Final gate commands below | passed |

## Wave 0 Gaps (resolved)

- Phase 27 mode/ack gate and production UART executor.
- Live socket ASIC bridge dispatch and observation feedback loop.
- Repo-owned evidence wrapper with blocked-mode committed artifacts.
- Mining-allow Phase 27 tier and conservative checklist rows.

## Final Gate Results

- `bazel test //crates/bitaxe-stratum:tests //crates/bitaxe-safety:tests //tools/parity:tests //scripts:phase27_live_hardware_bridge_evidence_test --test_output=errors` passed.
- `bazel build //firmware/bitaxe:firmware` passed.
- `just parity` passed.
- `just verify-reference` passed.
- `node "$HOME/.cursor/get-shit-done/bin/gsd-tools.cjs" verify lifecycle 27 --expect-id 27-2026-07-05T14-51-50 --expect-mode yolo --require-plans` passed.
- Hardware mode evidence was not run; committed `share_outcome: blocked_safe_prerequisite` with explicit non-claims.

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Instructions |
| --- | --- | --- | --- |
| Detector-gated accepted/rejected live ASIC-derived share | STR-09 | Requires `just detect-ultra205`, pool credentials, and live pool response. | Run `just phase27-evidence --mode hardware` only after detector gate; never commit raw endpoints or credentials. |

## Validation Sign-Off

- [x] All tasks have automated verify coverage.
- [x] Wave 0 gaps resolved during execution.
- [x] `nyquist_compliant: true`

**Approval:** passed by repo-native final gate.
