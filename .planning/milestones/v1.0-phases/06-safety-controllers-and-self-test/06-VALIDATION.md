---
phase: 06
slug: safety-controllers-and-self-test
status: draft
nyquist_compliant: true
wave_0_complete: true
created: 2026-06-28
---

# Phase 06 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | Rust unit tests plus Bazel/Just wrappers |
| **Config file** | `Cargo.toml`, `MODULE.bazel`, `Justfile` |
| **Quick run command** | Affected plan command from the map below, usually `cargo test -p bitaxe-safety --all-features <filter>` or the affected crate-scoped equivalent |
| **Full suite command** | `just test && just parity` |
| **Estimated runtime** | ~180 seconds for host checks; hardware smoke is manual and variable |

---

## Sampling Rate

- **After every task commit:** Run the affected crate-scoped Rust test command named in the plan.
- **After every plan wave:** Run `just test && just parity`.
- **Before `/gsd-verify-work`:** Full host suite must be green; hardware evidence must be recorded or explicitly marked not run.
- **Max feedback latency:** 300 seconds for host checks.

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Threat Ref | Secure Behavior | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|------------|-----------------|-----------|-------------------|-------------|--------|
| 06-01-01 | 06-01 | 1 | SAFE-01, SAFE-08 | T-06-01-01, T-06-01-03 | Safety crate facade builds as a host-only pure contract crate. | unit | `cargo test -p bitaxe-safety --all-features safety_contract && bazel test //crates/bitaxe-safety:tests` | planned | ⬜ pending |
| 06-01-02 | 06-01 | 1 | SAFE-01, SAFE-08 | T-06-01-01, T-06-01-02 | Shared contracts distinguish fail-closed effects, status, and hardware evidence. | unit | `cargo test -p bitaxe-safety --all-features safety_contract && cargo test -p bitaxe-safety --all-features evidence && bazel test //crates/bitaxe-safety:tests` | planned | ⬜ pending |
| 06-02-01 | 06-02 | 2 | SAFE-01, SAFE-02, SAFE-05, SAFE-09 | T-06-02-01 | Safety module graph exists without firmware or hardware effects. | unit | `cargo test -p bitaxe-safety --all-features safety_module_graph && bazel test //crates/bitaxe-safety:tests --test_filter=safety_module_graph` | planned | ⬜ pending |
| 06-03-01 | 06-03 | 3 | SAFE-01, SAFE-07, SAFE-08 | T-06-03-01, T-06-03-02 | Invalid voltage/current/power observations fail closed before effects. | unit/fixture | `cargo test -p bitaxe-safety --all-features safety_power && bazel test //crates/bitaxe-safety:tests --test_filter=safety_power` | planned | ⬜ pending |
| 06-03-02 | 06-03 | 3 | SAFE-01, SAFE-08 | T-06-03-01, T-06-03-04 | DS4432U voltage planning stays observe-only without hardware evidence. | unit/fixture | `cargo test -p bitaxe-safety --all-features voltage_effect && bazel test //crates/bitaxe-safety:tests --test_filter=voltage_effect` | planned | ⬜ pending |
| 06-04-01 | 06-04 | 3 | SAFE-02, SAFE-03, SAFE-07 | T-06-04-01, T-06-04-02 | Thermal/fan/PID faults become typed safe decisions before firmware effects. | unit/fixture | `cargo test -p bitaxe-safety --all-features safety_thermal && bazel test //crates/bitaxe-safety:tests --test_filter=safety_thermal` | planned | ⬜ pending |
| 06-04-02 | 06-04 | 3 | SAFE-04, SAFE-08 | T-06-04-03 | Overheat and sustained faults block ASIC/mining and publish visible status. | unit/fixture | `cargo test -p bitaxe-safety --all-features safety_fault && bazel test //crates/bitaxe-safety:tests --test_filter=safety_fault` | planned | ⬜ pending |
| 06-05-01 | 06-05 | 3 | SAFE-05 | T-06-05-01, T-06-05-02 | Self-test lifecycle is stepwise, cancellable, result-reporting, and safe-gated. | unit/fixture | `cargo test -p bitaxe-safety --all-features self_test && bazel test //crates/bitaxe-safety:tests --test_filter=self_test` | planned | ⬜ pending |
| 06-05-02 | 06-05 | 3 | SAFE-09 | T-06-05-03 | Safety/self-test work is bounded and watchdog-friendly. | unit/fixture | `cargo test -p bitaxe-safety --all-features watchdog && cargo test -p bitaxe-safety --all-features self_test && bazel test //crates/bitaxe-safety:tests --test_filter='watchdog|self_test'` | planned | ⬜ pending |
| 06-06-01 | 06-06 | 4 | SAFE-01, SAFE-02, SAFE-04, SAFE-08, SAFE-09 | T-06-06-01, T-06-06-02 | ASIC and mining gates require power, thermal, safety, and hardware evidence. | unit | `cargo test -p bitaxe-asic --all-features init_plan && cargo test -p bitaxe-stratum --all-features mining_loop && bazel test //crates/bitaxe-asic:tests //crates/bitaxe-stratum:tests --test_filter='init_plan|mining_loop'` | planned | ⬜ pending |
| 06-07-01 | 06-07 | 5 | SAFE-01, SAFE-02, SAFE-07, SAFE-08 | T-06-07-01, T-06-07-02 | API telemetry model distinguishes fresh, stale, faulted, and unavailable values. | unit | `cargo test -p bitaxe-api --all-features safety_telemetry_model && bazel test //crates/bitaxe-api:tests --test_filter=safety_telemetry_model` | planned | ⬜ pending |
| 06-07-02 | 06-07 | 5 | SAFE-01, SAFE-02, SAFE-07, SAFE-08 | T-06-07-01, T-06-07-03 | API system info, statistics, and live telemetry project explicit safety telemetry. | unit/fixture | `cargo test -p bitaxe-api --all-features safety_telemetry_projection && bazel test //crates/bitaxe-api:tests --test_filter=safety_telemetry_projection` | planned | ⬜ pending |
| 06-08-01 | 06-08 | 6 | SAFE-01, SAFE-02, SAFE-04, SAFE-07, SAFE-08 | T-06-08-01, T-06-08-02 | Firmware adapters publish safety telemetry while remaining observe-only/fail-closed. | build/unit | `cargo build -p bitaxe-firmware --target xtensa-esp32s3-espidf && bazel build //firmware/bitaxe:firmware && cargo test -p bitaxe-api --all-features safety_telemetry` | planned | ⬜ pending |
| 06-09-01 | 06-09 | 7 | SAFE-06, SAFE-09 | T-06-09-01, T-06-09-02 | Firmware safety supervisor is bounded and runtime display/input gaps are explicit. | build/workflow | `cargo build -p bitaxe-firmware --target xtensa-esp32s3-espidf && bazel build //firmware/bitaxe:firmware && just test` | planned | ⬜ pending |
| 06-10-01 | 06-10 | 8 | SAFE-08 | T-06-10-01, T-06-10-02 | Safety-critical rows cannot become verified without hardware evidence. | workflow/unit | `bazel test //tools/parity:tests --test_filter=safety_critical && cargo test -p bitaxe-parity --all-features safety_critical` | planned | ⬜ pending |
| 06-10-02 | 06-10 | 8 | SAFE-01, SAFE-02, SAFE-03, SAFE-04, SAFE-05, SAFE-06, SAFE-07, SAFE-08, SAFE-09 | T-06-10-03, T-06-10-04 | Phase 6 checklist and evidence docs avoid unsupported verified claims. | workflow | `just parity && bazel test //tools/parity:tests --test_filter=safety_critical` | planned | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Plan-Created Test Scaffolds

- [ ] Plan 06-03 creates safety fixture files for power telemetry and voltage effect cases.
- [ ] Plan 06-04 creates safety fixture files for fan/PID, thermal fault, and overheat cases.
- [ ] Plan 06-05 creates self-test lifecycle and watchdog step fixture files.
- [ ] Plan 06-07 creates API telemetry fixtures proving Phase 6 values are fresh/stale/faulted/unavailable rather than blindly zeroed.
- [ ] Plan 06-10 creates parity evidence templates for Ultra 205 hardware smoke, skipped hardware evidence, and display/input runtime gaps.

## Plan-Level Rust Gate

Every Rust-changing plan must run these commands before commit:

```bash
cargo fmt --all
cargo clippy --all-targets --all-features -- -D warnings
cargo build --all-targets --all-features
cargo test --all-features
```

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| DS4432U voltage actuation | SAFE-01, SAFE-08 | Safety-critical Ultra 205 hardware effect. | Record board, port, firmware commit, reference commit, requested voltage, observed logs/API values, and safe conclusion. |
| INA260 current/voltage/power telemetry | SAFE-01, SAFE-07, SAFE-08 | Requires live Ultra 205 sensor reads. | Capture monitor/API/WebSocket telemetry and classify fresh/stale/faulted behavior. |
| Fan duty and RPM control | SAFE-02, SAFE-03, SAFE-08 | Requires fan hardware and RPM feedback. | Record duty request, observed RPM/status, failure behavior, and parity row conclusion. |
| Thermal overheat safe stop/recovery | SAFE-04, SAFE-08, SAFE-09 | Requires controlled hardware conditions; unsafe to simulate casually. | Use a bounded bench protocol or mark not run with evidence pending. |
| Self-test hardware submode | SAFE-05, SAFE-08, SAFE-09 | Drives hardware and may heat ASIC. | Run only after safety gates and record pass/fail/cancel/result behavior. |
| OLED/button safety status | SAFE-06, SAFE-08 | Requires physical display/input observation. | Capture startup/runtime display or button logs; otherwise document V1 gap. |

---

## Validation Sign-Off

- [x] All tasks have planned automated verify commands.
- [x] Sampling continuity: no 3 consecutive tasks without automated verify.
- [x] Plan-created scaffolds cover all missing references.
- [x] No watch-mode flags.
- [x] Feedback latency < 300s for host checks.
- [x] `nyquist_compliant: true` set in frontmatter.

**Approval:** approved 2026-06-28
