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
| 32-01-01 | 01 | 1 | OBS-03, OBS-04 | T-32-01 | Signed and sentinel raw values cannot become invalid fresh facts. | unit | `cargo test -p bitaxe-safety sensor_acquisition` | ❌ W0 | ⬜ pending |
| 32-01-02 | 01 | 1 | OBS-03, OBS-04, OBS-05 | T-32-01, T-32-03 | Failures preserve stamps; power is atomic; thermal facts are independent. | unit | `cargo test -p bitaxe-safety sensor_acquisition` | ❌ W0 | ⬜ pending |
| 32-02-01 | 02 | 2 | OBS-02 | T-32-02 | One finite-timeout I2C0 lifecycle excludes producer writes. | source/build | `cargo test -p bitaxe-parity phase32_i2c_source_guard` and `just build` | ❌ W0 | ⬜ pending |
| 32-02-02 | 02 | 2 | OBS-03, OBS-04 | T-32-01, T-32-02 | Closed readers return only typed read outcomes. | unit/source/build | `cargo test -p bitaxe-safety sensor_acquisition`, `cargo test -p bitaxe-parity phase32_sensor_source_guard`, and `just build` | ❌ W0 | ⬜ pending |
| 32-03-01 | 03 | 3 | OBS-02, OBS-03, OBS-04, OBS-05 | T-32-02, T-32-03 | One producer owns sequences and continues after failures. | source/build | `cargo test -p bitaxe-parity phase32_runtime_source_guard` and `just build` | ❌ W0 | ⬜ pending |
| 32-03-02 | 03 | 3 | OBS-05 | T-32-03 | Request reads remain clone-only and preserve metadata; firmware wiring stays acquisition-free. | integration/source/build | `cargo test -p bitaxe-api observation`, `cargo test -p bitaxe-api telemetry`, `cargo test -p bitaxe-parity phase32_consumer_source_guard`, and `just build` | Existing + ❌ W0 guard | ⬜ pending |
| 32-03-03 | 03 | 3 | OBS-02, OBS-03, OBS-04, OBS-05 | T-32-04 | Full host/firmware/package/reference gates pass; hardware remains pending until a compliant wrapper exists. | build/policy | ordered Rust gate, `just build`, `just package`, and `just verify-reference` | Existing | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

## Wave 0 Requirements

- [ ] Pure raw-decoder and sweep-reducer tests for signed INA260 current, EMC2101 sentinel/temperature/tachometer behavior, atomic power admission, independent thermal facts, stamp preservation, and stale timing.
- [ ] `tools/parity` source guards for finite timeout forwarding, one-owner handoff, prohibited identifiers, and normal-producer/store wiring.
- [ ] Producer/store behavior in pure `bitaxe-safety`/`bitaxe-api` tests proving one failed source does not block later facts, API reads, or complete snapshot replacement.

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
| --- | --- | --- | --- |
| Startup SSD1306 and real sensor behavior on one physical I2C0 lifecycle. | OBS-02, OBS-03, OBS-04 | Existing wrappers do not satisfy the mandatory private serial-session identity/ownership trace plus separately sanitized summary contract. | Record pending in Phase 32. A later phase may run only after a repo-owned wrapper satisfies the complete AGENTS.md session-reuse contract. |
| API availability during a naturally observed sensor error. | OBS-05 | Ad hoc fault injection is prohibited and no compliant hardware wrapper exists yet. | Retain automated proof and record hardware evidence pending; do not improvise a failure or serial workflow. |

## Threat Model

| Ref | Threat | Severity | Mitigation |
| --- | --- | --- | --- |
| T-32-01 | Partial/invalid sensor reads become fresh operator facts. | high | Typed decoders, atomic INA260 admission, independent EMC2101 reducers, last-good stamp preservation tests. |
| T-32-02 | Generic bus access or legacy initialization performs an actuator/control write. | high | Closed read-only producer capability, source reachability guard, hardware evidence scans for prohibited-effect markers. |
| T-32-03 | Request traffic acquires sensors or advances provenance. | high | Sole producer ownership, clone-only store API, repeated-read regression tests. |
| T-32-04 | Hardware smoke widens into destructive or sensitive work. | high | `just detect-ultra205`, board-205-only bounded wrappers, no credentials, no direct UART/pins, no raw writes, redacted evidence, Phase 35-only promotion. |

## Validation Sign-Off

- [ ] All planned task groups have automated verification or completed Wave 0 dependencies.
- [ ] Sampling continuity: no three consecutive tasks without automated verification.
- [ ] Wave 0 covers every missing behavior fixture and source guard.
- [x] No watch-mode flags.
- [x] Host feedback latency target is below 60 seconds.
- [x] `nyquist_compliant: true` is set in frontmatter.

**Approval:** pending execution; map aligned to the final three-plan, seven-task structure.
