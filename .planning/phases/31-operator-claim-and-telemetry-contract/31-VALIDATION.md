---
phase: 31
slug: operator-claim-and-telemetry-contract
status: draft
nyquist_compliant: true
wave_0_complete: true
created: 2026-07-13
generated_by: gsd-plan-phase
lifecycle_mode: yolo
phase_lifecycle_id: 31-2026-07-13T19-47-51
generated_at: 2026-07-13T20:06:00.000Z
---

# Phase 31 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

## Test Infrastructure

| Property | Value |
| --- | --- |
| **Framework** | Rust built-in test harness through Cargo and Bazel `rust_test` |
| **Config file** | `Cargo.toml`, `MODULE.bazel`, crate `BUILD.bazel` files |
| **Quick run command** | `cargo test -p <affected-package> --all-features` |
| **Full suite command** | `cargo test --all-features` |
| **Estimated runtime** | ~25 seconds |

## Sampling Rate

- **After every task:** Run the focused affected-package command named in the plan task.
- **After every plan wave:** Run `cargo test --all-features` plus affected Bazel targets.
- **Before phase verification:** Full repository gates must be green.
- **Max feedback latency:** 30 seconds for focused tests; full gates may take longer.

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Threat Ref | Secure Behavior | Test Type | Automated Command | File Exists | Status |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| 31-01-01 | 01 | 1 | OBS-01 | T-31-01 | Invalid state/value combinations are unconstructible | unit | `cargo test -p bitaxe-safety --all-features observation` | ✅ | ✅ green |
| 31-01-02 | 01 | 1 | OBS-01 | T-31-02 | Failed/stale transitions cannot mint new sample metadata | unit | `cargo test -p bitaxe-safety --all-features observation` | ✅ | ✅ green |
| 31-02-01 | 02 | 2 | OBS-01 | T-31-03 | Compatibility zero never authenticates freshness | unit/golden | `cargo test -p bitaxe-api --all-features safety_telemetry` | ✅ | ✅ green |
| 31-02-02 | 02 | 2 | OBS-01 | T-31-08 | Consumer reads cannot advance producer-owned stamps and firmware still builds | regression/build | `cargo test -p bitaxe-api --all-features projection && just build` | ✅ | ✅ green |
| 31-03-01 | 03 | 3 | CFG-08 | T-31-04 | Only validated hostname can construct an effect-free v1.2 capability; Phase 33 owns persistence integration | unit | `cargo test -p bitaxe-api --all-features settings` | ✅ | ⬜ pending |
| 31-03-02 | 03 | 3 | CFG-08 | T-31-05 | Excluded settings and claims remain ineligible and effect-free | unit/workflow | `cargo test -p bitaxe-parity --all-features phase31_` | ❌ W0 | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

## Wave 0 Requirements

- [x] Existing Cargo and Bazel test infrastructure covers OBS-01 observation, API, and projection tests.
- [x] Existing settings fixtures cover broad known, unknown, invalid, and mixed PATCH input.
- [x] `tools/parity` has established exact-claim fixtures and test infrastructure; the missing Phase 31 cases are created by task 31-03-02 in the same plan before its focused command runs.

## Manual-Only Verifications

All Phase 31 behaviors have automated repository-local verification. Hardware, network, credentials, direct UART/pins, and archived Phase 28.1.1 execution are outside this phase and must not be used as manual substitutes.

## Threat Model Sampling

| Threat | Risk | Required proof |
| --- | --- | --- |
| T-31-01 Contradictory observation state/value | A consumer treats an invalid or fallback value as fresh | Exhaustive variant construction and projection tests |
| T-31-02 Consumer-minted freshness | API traffic changes sequence or acquisition time | Repeated-read equality and failure-transition tests |
| T-31-03 Compatibility fallback confusion | Numeric zero is interpreted as availability or health | Wire/golden tests assert explicit truth beside numeric fallback |
| T-31-04 Authority widening | Full NVS schema fields receive v1.2 persistence authority | Negative settings capability tests for every excluded class |
| T-31-05 Claim widening | String/schema growth admits excluded capability or broad promotion | Closed-enum and row-scoped parity admission tests |

## Validation Sign-Off

- [x] All planned tasks have an automated focused command.
- [x] Sampling continuity: no task lacks automated verification.
- [x] Existing infrastructure covers test execution; task-local red tests are explicitly owned.
- [x] No watch-mode flags are used.
- [x] Focused feedback latency target is under 30 seconds.
- [x] `nyquist_compliant: true` is set in frontmatter.

**Approval:** approved 2026-07-13 for planning; task statuses remain pending until execution.
