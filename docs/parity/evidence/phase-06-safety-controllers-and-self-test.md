# Phase 06 Safety Controllers And Self-Test Evidence

**Date:** 2026-06-28
**Firmware baseline before this evidence commit:** `00e21c6`
**Reference commit:** `c1915b0a63bfabebdb95a515cedfee05146c1d50`

## Scope

Phase 06 adds a pure safety-controller core, explicit safety evidence labels, power/voltage/current safety decisions, thermal/fan/PID/fault decisions, self-test and watchdog lifecycle decisions, ASIC/mining safety gates, API safety telemetry projection, and observe-only firmware safety adapters.

This evidence covers host-testable controller behavior, API/gate integration, firmware buildability, and parity governance. It does not record live Ultra 205 voltage, fan, thermal, power, self-test, or runtime display/input hardware proof.

## Decision Boundary

D-19 requires parity rows `PWR-001` through `PWR-006`, `THR-001` through `THR-003`, `SELF-001`, and relevant `IO`/`UI` rows to carry implementation pointers and evidence without overclaiming verification. SAFE-08 keeps safety-critical hardware-control parity below `verified` unless hardware-smoke or hardware-regression evidence exists.

## Commands Run

| Command | Result | Evidence Boundary |
| --- | --- | --- |
| `cargo test -p bitaxe-safety --all-features` | passed during Phase 06 plans | Pure safety status, evidence, power, thermal, fault, self-test, watchdog, and fixture coverage. |
| `cargo test -p bitaxe-asic --all-features init_plan` | passed during Phase 06 plan verification | ASIC initialization gates require power, thermal, and safety evidence. |
| `cargo test -p bitaxe-stratum --all-features mining_loop` | passed during Phase 06 plan verification | Mining loop remains blocked without safety evidence and hardware acknowledgment. |
| `cargo test -p bitaxe-api --all-features safety_telemetry` | passed during Phase 06 plan verification | API system info, statistics, and live telemetry projections use explicit safety telemetry status. |
| `cargo build -p bitaxe-firmware --target xtensa-esp32s3-espidf` | passed during Phase 06 plans | Firmware observe-only safety adapters and supervisor shell compile for ESP32-S3. |
| `bazel build //firmware/bitaxe:firmware` | passed during Phase 06 plans | Bazel firmware wrapper builds the safety adapter firmware. |
| `bazel test //tools/parity:tests --test_filter=safety_critical` | passed during Phase 06 Plan 10 | Parity tooling rejects unsupported verified safety-critical claims. |
| `cargo test -p bitaxe-parity --all-features safety_critical` | passed during Phase 06 Plan 10 | Host parity tests cover SELF-001, UI-003, and implemented-only THR-003 safety cases. |
| `just parity` | passed during Phase 06 Plan 10 | Checklist parse, reference guard, and invalid verified-claim validation pass with `validation_errors: none`. |
| `just test` | passed during Phase 06 Plan 10 | Full Bazel test wrapper remains green after Phase 6 evidence and parity guard updates. |

## Pure Safety Evidence

- `crates/bitaxe-safety` defines `SafetyStatus`, `SafetyCriticalEvidence`, and fail-closed `SafetyEffect` contracts.
- Power telemetry decisions classify missing, stale, unsafe, and faulted observations before they can feed hardware effects.
- DS4432U voltage effect planning suppresses writes unless valid setpoints, fresh power, hardware evidence, and armed mode all agree.
- Thermal/fan/PID logic clamps fan decisions, rejects invalid sentinels, and treats overheat/fault states as fail-closed.
- Self-test lifecycle decisions model factory/manual start, pass, fail, restart, cancel, result recording, and missing-evidence behavior.
- Watchdog decisions bound safety/self-test work and require yields or watchdog reset/feed behavior when budgets are exceeded.

## API And Gate Integration Evidence

- `crates/bitaxe-asic` full initialization now requires power, thermal, and safety evidence before initialized/no-mining status can be reached.
- `crates/bitaxe-stratum` mining loop stays blocked unless ASIC, safety, and hardware-evidence gates are all ready.
- `crates/bitaxe-api` carries `SafetyTelemetryReport`, `SafetyTelemetryStatus`, and `SafetyCriticalEvidence` through `SafeTelemetrySnapshot`.
- System info and statistics projections keep unavailable, stale, faulted, or unverified telemetry zero-compatible rather than treating zero as safe hardware data.
- SAFE-08 is enforced by `tools/parity`: safety-critical verified rows must use `hardware-smoke` or `hardware-regression` evidence.

## Firmware Observe-Only Evidence

- `firmware/bitaxe/src/safety_adapter.rs` collects explicit unavailable safety telemetry and interprets typed safety effects without enabling hardware writes.
- `firmware/bitaxe/src/safety_adapter/power.rs` records Ultra 205 DS4432U and INA260 constants while returning `hardware_evidence_pending` by default.
- `firmware/bitaxe/src/safety_adapter/thermal.rs` records the EMC2101 address, parses raw thermal observations, and keeps fan writes suppressed by default.
- `firmware/bitaxe/src/safety_adapter/watchdog.rs` starts a named supervisor shell that yields at the 100 ms watchdog cadence.
- `firmware/bitaxe/src/display_adapter.rs` logs the runtime display/input gap instead of treating startup OLED evidence as complete runtime parity.

## Hardware Evidence Status

Phase 06 did not run the Ultra 205 safety hardware-smoke protocol. The hardware-smoke template is recorded in `docs/parity/evidence/phase-06-ultra-205-safety-hardware-smoke.md` with `Conclusion: not run - hardware verification pending`.

Rows affected by this boundary include `PWR-001`, `PWR-002`, `PWR-003`, `PWR-005`, `PWR-006`, `THR-001`, `THR-002`, `SELF-001`, and `UI-003`. These rows must not be marked `verified` until board-named Ultra 205 hardware evidence exists.

## Checklist Impact

- `PWR-001` through `PWR-006` now cite safety core, API, ASIC gate, and firmware adapter targets where applicable.
- `THR-001` through `THR-003` now cite thermal/fan/PID logic and firmware thermal adapter targets.
- `SELF-001` now cites the pure self-test lifecycle and firmware watchdog shell.
- `STAT-002`, API telemetry rows, ASIC initialization, and mining-loop gate rows cite Phase 6 safety telemetry/evidence integration.
- Relevant `IO`/`UI` rows record startup-only evidence, observe-only safety adapter constants, and the runtime display/input gap.

## Residual Hardware Risk

- DS4432U voltage writes are not live-verified.
- INA260 current, voltage, and power reads are not live-verified.
- Fan PWM/duty, RPM feedback, and fan-fault behavior are not live-verified.
- Thermal overheat stop/cool/restart behavior is not live-verified.
- Self-test hardware submodes are not live-verified.
- Runtime display/input behavior is not live-verified.

## Conclusion

Pure/controller/API/gate/firmware observe-only behavior is implemented and covered by unit, workflow, build, and parity-tool evidence. Safety-critical hardware-control parity remains below `verified` until board-named Ultra 205 hardware-smoke or hardware-regression evidence exists.
