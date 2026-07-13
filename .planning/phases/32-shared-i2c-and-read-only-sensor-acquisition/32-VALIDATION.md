---
phase: 32
slug: shared-i2c-and-read-only-sensor-acquisition
status: draft
nyquist_compliant: true
wave_0_complete: false
created: 2026-07-13
lifecycle_mode: yolo
phase_lifecycle_id: 32-2026-07-13T23-12-34
---

# Phase 32 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

## Test Infrastructure

| Property | Value |
| --- | --- |
| **Framework** | Rust built-in test harness through Cargo and Bazel |
| **Config file** | Workspace `Cargo.toml`, `Cargo.lock`, `MODULE.bazel`, and repo `justfile` |
| **Quick run command** | `cargo test -p bitaxe-safety` or the narrow affected package |
| **Full suite command** | `cargo test --all-features` |
| **Estimated runtime** | ~20 seconds host-only; firmware/package and hardware are separate bounded gates |

## Sampling Rate

- **After every task commit:** Run the narrow affected Cargo test target plus any task-specific source guard.
- **After every plan wave:** Run `cargo test --all-features` and the plan's relevant Bazel target.
- **Before phase verification:** Run the ordered Rust gate, canonical firmware build/package, reference-clean check, and any approved detector-gated read-only smoke.
- **Max host feedback latency:** 60 seconds.

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Threat Ref | Secure Behavior | Test Type | Automated Command | File Exists | Status |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| 32-01-01 | 01 | 1 | OBS-03, OBS-04, OBS-05 | T-32-01 | Invalid/failed reads cannot mint freshness or erase independent facts. | unit | `cargo test -p bitaxe-safety` | ❌ W0 | ⬜ pending |
| 32-02-01 | 02 | 1 | OBS-02 | T-32-02 | One finite-timeout I2C0 lifecycle preserves display handoff and excludes producer writes. | unit/source | `cargo test --all-features` | ❌ W0 | ⬜ pending |
| 32-03-01 | 03 | 2 | OBS-02, OBS-03, OBS-04, OBS-05 | T-32-03 | Producer-only stamps, failure-isolated sweep, clone-only requests. | integration | `cargo test --all-features` | ❌ W0 | ⬜ pending |
| 32-04-01 | 04 | 3 | OBS-02, OBS-03, OBS-04, OBS-05 | T-32-04 | Firmware/package succeeds; hardware path remains detector-gated and read-only. | build/smoke | `just build && just package` plus phase-owned bounded hardware command if approved | Existing wrappers; phase evidence script may be W0 | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

## Wave 0 Requirements

- [ ] Pure raw-decoder and sweep-reducer tests for signed INA260 current, EMC2101 sentinel/temperature/tachometer behavior, atomic power admission, independent thermal facts, stamp preservation, and stale timing.
- [ ] Bounded-bus/display fake verifying finite timeout forwarding and success/failure handoff.
- [ ] Source/reachability guard for prohibited EMC2101 init/fan, DS4432U/voltage, reset/power/ASIC/mining/fault/self-test, credential, UART/pin, OTA, and archived-lineage calls from the normal producer.
- [ ] Producer/store integration test proving one failed source does not block later sources, API reads, or complete snapshot replacement.

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
| --- | --- | --- | --- |
| Startup SSD1306 remains observable while the same physical I2C0 lifecycle later produces fresh telemetry. | OBS-02 | Physical display and shared bus ownership require the Ultra 205. | Run the phase-owned detector-gated ≥360-second read-only smoke, observe startup display, and correlate redacted producer markers without any control-write marker. |
| Real INA260/EMC2101 values and independent tachometer availability are attributable to producer stamps. | OBS-03, OBS-04 | Host fixtures cannot prove real sensor wiring or values. | Use the same bounded read-only smoke and inspect redacted API/monitor evidence; do not promote claims until Phase 35. |
| API and unaffected facts remain available across a naturally observed sensor error. | OBS-05 | Ad hoc fault injection is prohibited in Phase 32. | Admit only a naturally occurring failure during the approved smoke; otherwise retain automated proof and mark this hardware case pending. |

## Threat Model

| Ref | Threat | Severity | Mitigation |
| --- | --- | --- | --- |
| T-32-01 | Partial/invalid sensor reads become fresh operator facts. | high | Typed decoders, atomic INA260 admission, independent EMC2101 reducers, last-good stamp preservation tests. |
| T-32-02 | Generic bus access or legacy initialization performs an actuator/control write. | high | Closed read-only producer capability, source reachability guard, hardware evidence scans for prohibited-effect markers. |
| T-32-03 | Request traffic acquires sensors or advances provenance. | high | Sole producer ownership, clone-only store API, repeated-read regression tests. |
| T-32-04 | Hardware smoke widens into destructive or sensitive work. | high | `just detect-ultra205`, board-205-only bounded wrappers, no credentials, no direct UART/pins, no raw writes, redacted evidence, Phase 35-only promotion. |

## Validation Sign-Off

- [x] All planned task groups have automated verification or Wave 0 dependencies.
- [x] Sampling continuity: no three consecutive tasks without automated verification.
- [x] Wave 0 covers every missing behavior fixture and source guard.
- [x] No watch-mode flags.
- [x] Host feedback latency target is below 60 seconds.
- [x] `nyquist_compliant: true` is set in frontmatter.

**Approval:** approved 2026-07-13 for planning; task IDs may be refined by the plan checker.
