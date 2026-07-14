---
phase: 33
slug: confirmed-settings-durability
status: hardware_pending
nyquist_compliant: true
wave_0_complete: true
created: 2026-07-14
lifecycle_mode: yolo
phase_lifecycle_id: 33-2026-07-14T01-50-49
---

# Phase 33 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

## Test Infrastructure

| Property | Value |
| --- | --- |
| **Framework** | Rust built-in test harness through Cargo and Bazel, plus repo-owned Bash integration tests |
| **Config file** | Workspace `Cargo.toml`, `Cargo.lock`, `MODULE.bazel`, `scripts/BUILD.bazel`, and repo `justfile` |
| **Quick run command** | `cargo test -p bitaxe-api settings` or the narrow affected package/script target |
| **Full suite command** | `cargo test --all-features` plus affected Bazel targets |
| **Estimated runtime** | ~20 seconds host-only; firmware/package and the ≥360-second hardware capture are separate bounded gates |

## Sampling Rate

- **After every task commit:** Run the narrow affected Cargo or shell/Bazel test plus any task-specific source guard.
- **After every plan wave:** Run `cargo test --all-features`, affected Bazel targets, and `git diff --check`.
- **Before phase verification:** Run the ordered Rust gate, canonical firmware build/package/reference checks, all Phase 33 simulation/source guards, then the approved detector-gated durability run.
- **Max host feedback latency:** 60 seconds.

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Threat Ref | Secure Behavior | Test Type | Automated Command | File Exists | Status |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| 33-01-01 | 01 | 1 | CFG-09, CFG-13 | T-33-01, T-33-04 | Full compatibility validation precedes exact hostname authority; excluded inputs cannot construct effects. | unit | `cargo test -p bitaxe-api v12_settings` | ✅ | ✅ green |
| 33-01-02 | 01 | 1 | CFG-10 | T-33-02, T-33-03, T-33-05 | Ordered confirmation handles every failure and serializes concurrent writers through publication. | unit | `cargo test -p bitaxe-api settings` | ✅ | ✅ green |
| 33-02-01 | 02 | 2 | CFG-10, CFG-11 | T-33-02, T-33-03 | Firmware reload returns a fallible candidate and publishes only an exact typed match. | integration/source/build | affected firmware tests, Phase 33 source guard, and `just build` | ✅ | ✅ green |
| 33-02-02 | 02 | 2 | CFG-09, CFG-11, CFG-13 | PATCH routing preserves generic/no-op responses and immediate system-info reads confirmed storage truth without overlays. | integration | `cargo test -p bitaxe-api settings`, firmware host tests, and source guard | ✅ | ✅ green |
| 33-03-01 | 03 | 3 | CFG-12 | T-33-06, T-33-07, T-33-08 | Simulated evidence rejects extra reset, identity/origin ambiguity, leaks, and unredacted output. | shell/Bazel | `bash scripts/phase33-confirmed-settings-durability-test.sh` and `bazel test //scripts:phase33_confirmed_settings_durability_test` | ✅ | ✅ green |
| 33-03-02 | 03 | 3 | CFG-09, CFG-10, CFG-11, CFG-12, CFG-13 | T-33-01 through T-33-08 | Full host/firmware/package/reference gates pass before one bounded same-board application-reboot proof. | build/hardware/policy | ordered Rust gate, affected Bazel tests, `just build`, `just package`, `just verify-reference`, then Phase 33 hardware wrapper | ✅ | ⬜ pending — fresh same-session origin `fresh_origin_not_unique` |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

## Wave 0 Requirements

- [x] Pure compatibility/authority matrix covering malformed, invalid known, exact hostname, empty, unknown, unsupported, valid mixed, and invalid-known mixed bodies.
- [x] Deterministic fake-adapter transaction tests covering write/commit/reload/reconcile/publish order, same-value requests, every failure point, post-commit uncertainty, and concurrent writers.
- [x] Firmware source/integration guard proving fallible candidate reload, atomic confirmed publication, immediate system-info projection, and absence of requested-write overlays.
- [x] Phase 33 evidence-wrapper simulation fixtures for detector preflight, stable physical identity, passive ownership, fresh origin, exactly one application restart, redaction, timeout, and cleanup outcomes.

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
| --- | --- | --- | --- |
| Storage-confirmed hostname survives one approved application reboot on the same Ultra 205. | CFG-12 | Host simulation and firmware compilation cannot prove physical ESP-IDF NVS durability across reboot. | After all software/build/package gates pass, run the repo-owned Phase 33 wrapper with ≥360-second passive capture and ≥420-second wall clock. The wrapper owns the sole detector preflight. Require immediate and post-reboot hostname-digest equality, stable physical identity, exactly one application restart, complete cleanup, and a redacted non-promotional conclusion. The fresh 2026-07-14 execution on source `7f213d9` passed the exact ordered Rust gates, direct and Bazel serial/detector/passive/Phase 33/parity guards, canonical build/package/reference checks, exact manifest checks, sole detector and board-info preflight, stable pre-proof physical identity, required exact-package flash, the 360-second setup capture, original-hostname system-info readback, confirmed PATCH, immediate readback, passive-monitor arming, and the sole application-restart response-before-effect gate. After the full 360-second passive capture, the wrapper stopped fail-closed at `fresh_origin_not_unique`, so fresh same-session origin, post-reboot identity, and durable readback were not proved. The wrapper performed no retry, completed process/holder cleanup and original-hostname restoration, and emitted no shareable hardware evidence. |

## Threat Model

| Ref | Threat | Severity | Mitigation |
| --- | --- | --- | --- |
| T-33-01 | Broad compatibility or a mixed request gains write authority. | high | Full known-field validation plus closed exact hostname field-set authority and exhaustive matrix tests. |
| T-33-02 | Requested values become API truth without confirmed storage. | high | Fallible independent reload, typed reconciliation, atomic candidate publication, and source guard removing the overlay path. |
| T-33-03 | Concurrent requests interleave writes, reloads, or publication. | high | One transaction lock through publication and deterministic concurrency tests. |
| T-33-04 | Request bodies, credentials, origins, or device identifiers leak to logs/evidence. | high | Category-only retained logs, denylist tests, protected local traces, and redacted summaries. |
| T-33-05 | A post-commit failure is falsely reported as rollback or unchanged storage. | high | Typed post-commit uncertainty, no compensating write, and explicit failure-order tests. |
| T-33-06 | HTTP/tty continuity is mistaken for same-board identity. | high | Detector preflight and stable physical-USB identity digest excluding enumeration fields. |
| T-33-07 | A detector or monitor adds an extra reset inside the proof. | high | Detector outside proof interval, full passive no-reset flags, exactly-one-restart classifier, and fail-closed simulation. |
| T-33-08 | Hardware proof invokes archived, destructive, credential, UART/pin, mining, or promotion paths. | high | Closed source/command guard, repo-local authorization checks, protected inputs, and explicit Phase 35-only promotion boundary. |

## Validation Sign-Off

- [x] Every anticipated task group has automated verification or a Wave 0 dependency.
- [x] Sampling continuity: no three consecutive tasks without automated verification.
- [x] Wave 0 names every missing behavior fixture and source guard.
- [x] No watch-mode flags.
- [x] Host feedback latency target is below 60 seconds.
- [x] `nyquist_compliant: true` is set in frontmatter.

**Approval:** Wave 0, firmware integration, ordered Rust verification, direct and Bazel serial/detector/passive/Phase 33/parity regressions, canonical build/package, exact source/reference manifest checks, and reference cleanliness are green. The fresh 2026-07-14 execution on source `7f213d9` passed all pre-restart gates and proved that the sole application-restart response completed before its effect. It then stopped fail-closed after the full passive capture because exactly one fresh same-session origin could not be derived (`fresh_origin_not_unique`). Cleanup and original-hostname restoration passed, no shareable hardware evidence was emitted, and no retry occurred. CFG-12, Plan 03, and Phase 33 completion remain blocked.
