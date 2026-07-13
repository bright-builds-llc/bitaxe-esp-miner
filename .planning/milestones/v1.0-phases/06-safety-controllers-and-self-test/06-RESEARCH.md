---
generated_by: gsd-phase-researcher
phase: "06"
phase_name: "Safety Controllers And Self-Test"
generated_at: 2026-06-28T02:37:37.594Z
status: complete
---

# Phase 6: Safety Controllers And Self-Test - Research

## RESEARCH COMPLETE

## Executive Summary

Phase 6 should be planned as a safety-core phase with thin firmware adapters. The existing Rust code already has the right upstream-facing seams: Phase 3 BM1366 full init requires power, thermal, and safety preflight evidence; Phase 4 mining work submission requires safety evidence and hardware acknowledgment; Phase 5 API snapshots deliberately zero voltage, fan, thermal, current, and power until Phase 6 owns them.

The primary implementation risk is not missing fields; it is accidentally treating implementation as safe hardware verification. The plan must separate:

- Pure, unit-tested safety decisions and state transitions.
- Firmware adapters for DS4432U, INA260, thermal/fan hardware, input/display, and watchdog-aware task loops.
- Hardware-smoke or hardware-regression evidence before any safety-critical row is `verified`.

## Scope Mapping

| Requirement | Research finding | Planning implication |
| --- | --- | --- |
| SAFE-01 | Ultra 205 has DS4432U and INA260 capabilities in the catalog; upstream voltage control and telemetry are split across DS4432U, INA260, VCORE, and Power paths. | Add typed power/voltage/current decision core plus firmware adapters. |
| SAFE-02 | Upstream thermal abstraction returns sentinel values such as `-1` for unavailable chip temperature and routes fan duty through EMC/TMP abstractions. | Parse raw readings into typed fresh/stale/fault/unavailable observations before control. |
| SAFE-03 | Upstream PID has deterministic constants, 100 ms sample time, output clamps, and initialization behavior. | Unit-test PID and fan-control decisions independently from firmware I/O. |
| SAFE-04 | Upstream power management stops mining, cuts voltage, holds reset low, sets overheat mode, cools, then restarts at reduced settings. | Model safe-stop/cool/recover as a typed state machine and keep restart hardware-gated. |
| SAFE-05 | Upstream self-test starts from factory flag or boot-button path, drives fan/VCORE/ASIC behavior, records nonce/domain measurements, and clears flags only under specific outcomes. | Model self-test lifecycle as pure states/effects; hardware subtests require evidence gates. |
| SAFE-06 | Current Rust display is startup-only SSD1306; upstream display/input includes LVGL screens, button short/long press, self-test and overheat screens. | Plan bounded safety display/input only if hardware evidence is available; otherwise document gaps. |
| SAFE-07 | Phase 5 wire DTOs already expose power, voltage, current, fan duty, RPM, temp, temp2, and VR temp. | Replace zeroed `SafeTelemetrySnapshot` values with typed telemetry where trustworthy. |
| SAFE-08 | ADR-0012 and checklist rules require hardware evidence for safety-critical verification. | Plan checklist/evidence updates carefully: implemented != verified. |
| SAFE-09 | Upstream tasks run at 100 ms loops with delays; self-test and safe-mode loops can block if copied directly. | Use watchdog-friendly step supervision and responsiveness checks. |

## Existing Rust Seams

### Reusable Boundaries

- `crates/bitaxe-config/src/catalog.rs` already identifies Ultra 205 capabilities: DS4432U, INA260, ASIC enable, and power target.
- `crates/bitaxe-config/src/validation.rs` already has typed `CoreVoltageMv`, `FanDutyPercent`, `MinFanDutyPercent`, and `TemperatureCelsius`.
- `crates/bitaxe-asic/src/bm1366/init_plan.rs` already defines `PowerPreflightEvidence`, `ThermalPreflightEvidence`, and `SafetyPreflightEvidence`; these are placeholders to replace or enrich with Phase 6 safety evidence.
- `crates/bitaxe-stratum/src/v1/mining_loop.rs` already blocks work submission without `safety_evidence` and `hardware_evidence_ack`.
- `crates/bitaxe-api/src/snapshot.rs` contains `SafeTelemetrySnapshot::unavailable_until_phase_6()`, making Phase 6 the owner of live hardware telemetry.
- `firmware/bitaxe/src/runtime_snapshot.rs` is the correct firmware snapshot collection seam for API-visible safety telemetry.
- `firmware/bitaxe/src/display_adapter.rs` is startup-only SSD1306 evidence, not full display parity.

### Recommended Ownership

Use one focused safety domain surface rather than spreading safety policy through adapters. The planner may place it in `crates/bitaxe-core` or a new safety crate, but it should expose data/effect plans that can feed:

- API snapshots.
- BM1366 init preflight evidence.
- Stratum mining-loop safety gates.
- Firmware adapter actions.
- Parity evidence records.

## Reference Findings

### Power, Voltage, And Current

Relevant upstream files:

- `reference/esp-miner/main/power/DS4432U.c`
- `reference/esp-miner/main/power/INA260.c`
- `reference/esp-miner/main/power/vcore.c`
- `reference/esp-miner/main/tasks/power_management_task.c`

Key findings:

- DS4432U uses I2C address `0x48`, output registers `0xF8`/`0xF9`, and a board-specific transfer function. Rust should avoid copying GPL expression into MIT-first files unless intentionally isolated; independently derive or isolate with GPL-compatible labeling.
- Upstream DS4432U rejects voltages outside board min/max before writes. Rust should additionally restrict user-facing Ultra 205 setpoints to modeled BM1366 options unless planning explicitly separates regulator raw limits from accepted config limits.
- INA260 upstream returns cached last values on I2C read failure. Rust should represent read status explicitly instead of silently treating a cache as fresh telemetry.
- VCORE toggles ASIC enable around voltage effects and routes DS4432U/TPS546 behavior by board capability. Ultra 205 path should use DS4432U/INA260 while TPS546 remains deferred for non-205 boards.
- Power management stops mining by winding frequency down, cutting voltage to zero, holding reset low, marking ASIC uninitialized, delaying for UART completion, flushing UART, and publishing logs.

Planning recommendations:

- Model `PowerObservation` with `Fresh`, `Stale`, `Fault`, and `Unavailable` status.
- Model `VoltageEffectPlan` separately from telemetry observations.
- Provide fail-closed actions: no voltage write, hold reset low, disable ASIC enable where available, block mining work, publish visible fault.
- Treat live voltage/current/power evidence as board-named hardware evidence only.

### Thermal, Fan, PID, And Overheat

Relevant upstream files:

- `reference/esp-miner/main/thermal/thermal.c`
- `reference/esp-miner/main/thermal/PID.c`
- `reference/esp-miner/main/tasks/fan_controller_task.c`
- `reference/esp-miner/main/tasks/power_management_task.c`

Key findings:

- `Thermal_get_chip_temp` and `Thermal_get_chip_temp2` return `-1` before ASIC init or when no relevant sensor exists.
- Fan duty is written through `Thermal_set_fan_percent` and is clamped to `0..=100`.
- Fan controller modes include overheat `100%`, paused/no-pool `30%`, startup `70%`, manual fan, and auto PID.
- Upstream PID constants are `P=5.0`, `I=0.1`, `D=2.0`, sample time `100 ms`, output limit updated by min fan speed, and an EMA filter with alpha `0.2` is used before PID input.
- Fan set failure flags a hardware fault and writes a visible fault message.
- Overheat thresholds include ASIC throttle at `75 C`, safe temperature `45 C`, TPS546 throttle `105 C`, and TPS546 max `145 C`. Ultra 205 planning should verify which thresholds apply to DS4432U/INA260/EMC2101 path before live use.

Planning recommendations:

- Unit-test PID and fan-mode decisions with exact constants where applicable.
- Parse sentinel values and implausible readings before fan or overheat decisions.
- Model overheat as a state machine: normal -> safe stop -> cool-down -> reduced-settings restart candidate -> resumed or blocked.
- Keep exact thresholds evidence-backed and source-breadcrumbed.

### Self-Test And Watchdog

Relevant upstream file:

- `reference/esp-miner/main/self_test/self_test.c`

Key findings:

- Factory self-test starts when best diff is below one and `selftest` is enabled; boot-button path can also start self-test.
- Self-test sets ASIC difficulty, drives fan percent, validates fan RPM, temperature, voltage/power/current, nonce/hashrate, and domain results.
- It uses thresholds such as minimum fan RPM `1000`, fan range `10%..=100%`, PID sample `100 ms`, domain hashrate tolerance `0.33`, rejected warn ratio `0.25`, core voltage tolerance `0.10`, power margin `3 W`, input voltage margin `10%`, and difficulty `16`.
- It rejects invalid temperatures including non-finite, `-1`, and `127`.
- The lifecycle has active/finished states, pass/fail messaging, cancel/restart paths, and stored factory-flag behavior.

Planning recommendations:

- Implement self-test first as pure lifecycle and effect planning.
- Use fake pool/mock work and nonce injection for deterministic tests.
- Keep live self-test submodes opt-in and hardware-gated.
- Represent watchdog/load responsiveness as a plan requirement: long-running self-test work must be stepwise and yield-friendly, not a direct blocking port.

### Display And Input

Relevant upstream files:

- `reference/esp-miner/main/display.c`
- `reference/esp-miner/main/input.c`
- `reference/esp-miner/main/screen.c`

Key findings:

- Upstream display uses LVGL and display config from NVS. It handles display type, inversion, rotation, SH1107 offset, LVGL theme/style, and panel setup.
- Input uses boot button GPIO as an LVGL keypad with short-click and long-press callbacks. Long press duration is `2000 ms`.
- Screens include self-test, overheat, ASIC status, connection, mining, stats, Wi-Fi, logos, URLs, and firmware update states.
- Current Rust has only startup debug text on SSD1306 address `0x3c` using GPIO47/GPIO48 at 400 kHz.

Planning recommendations:

- Keep Phase 6 bounded: implement safety/self-test/status display/input only if Ultra 205 OLED/button hardware smoke can be recorded.
- Otherwise document display/input as explicit gaps while preserving API/log/WebSocket administration.
- Do not introduce full LVGL parity as hidden work inside safety controllers.

## Validation Architecture

Phase 6 needs layered validation because pure behavior and hardware behavior prove different claims.

### Unit Tests

Required pure tests:

- Voltage setpoint parsing and effect planning.
- INA260-style telemetry freshness/staleness/fault classification.
- DS4432U voltage plan bounds without firmware I2C writes.
- PID compute behavior, sample timing, output limits, and min fan floor.
- Fan mode decisions: overheat, paused, pools unavailable, startup, auto, manual.
- Thermal sentinels and diode-fault style values parsed into typed faults.
- Overheat state transitions and reduced-settings planning.
- Self-test lifecycle: factory start, manual start, pass, fail, cancel, restart, flag clearing, result reporting.
- Watchdog-friendly step progression.
- Display/input scope decisions when hardware evidence is absent.

### Fixture Tests

Recommended fixtures:

- `safety/power-telemetry-cases.json`
- `safety/voltage-effect-cases.json`
- `safety/fan-pid-cases.json`
- `safety/thermal-fault-cases.json`
- `safety/overheat-state-cases.json`
- `safety/self-test-lifecycle-cases.json`
- `safety/watchdog-step-cases.json`

Each fixture should include source breadcrumbs and provenance metadata, following Phase 2/3 fixture patterns.

### Firmware Smoke

Only run when a connected Ultra 205 and explicit hardware evidence path are available.

Suggested smoke sequence:

1. Build/package current firmware.
2. Flash/monitor with board and port recorded.
3. Capture boot identity and reference commit.
4. Observe safety telemetry without actuation.
5. Exercise fan/read-only or bounded fan behavior only after safe preconditions.
6. Exercise voltage actuation only with conservative, bounded test plan.
7. Record logs, observed API/live telemetry values, and conclusion.

If hardware smoke is not run, create evidence records concluding `not run - hardware verification pending` and keep rows below `verified`.

### Hardware Regression / Soak

Plan repeatable hardware regression for:

- Sensor read failure and recovery.
- Fan RPM zero or set failure.
- Thermal over-limit safe stop and cool-down.
- Self-test pass/fail/cancel.
- Watchdog/load responsiveness while API/log/WebSocket telemetry remains available.

### Checklist Gate

`docs/parity/checklist.md` must continue enforcing:

- Safety-critical rows require `hardware-smoke` or `hardware-regression` before `verified`.
- Unit/golden evidence can verify pure logic only where the row is not hardware-control behavior.
- Non-205 boards do not inherit Ultra 205 evidence.

## Plan Shape Recommendation

A robust Phase 6 likely needs six or seven plans:

1. Safety domain model and fixture scaffolding.
2. Power/voltage/current safety decisions and telemetry snapshot integration.
3. Thermal/fan/PID and overheat decisions.
4. Self-test lifecycle and watchdog-friendly supervisor decisions.
5. Firmware adapters for Ultra 205 safety telemetry/effects with fail-closed defaults.
6. Display/input bounded safety status or explicit gap evidence.
7. Parity evidence, hardware-smoke templates, checklist updates, and final verification.

Keep hardware-effect plans split from pure decision plans so verification and rollback stay precise.

## Risks And Mitigations

| Risk | Mitigation |
| --- | --- |
| Overclaiming safety parity from unit tests | Checklist/evidence tasks must keep safety-critical rows below `verified` without hardware evidence. |
| Unsafe voltage or fan behavior from adapter failures | Pure decisions classify missing/faulted observations and emit fail-closed effect plans. |
| GPL expression copied into MIT files | Use breadcrumbs and independently authored Rust; isolate deliberate ports with GPL-compatible marking. |
| Watchdog starvation from direct upstream-style loops | Model self-test/power/fan loops as bounded steps and add responsiveness checks. |
| Display/input scope expansion delays safety work | Bound UI to safety/self-test status or document a gap. |
| Cached INA260 values treated as live readings | Track telemetry freshness explicitly. |

## Sources Read

- `.planning/phases/06-safety-controllers-and-self-test/06-CONTEXT.md`
- `.planning/ROADMAP.md`
- `.planning/REQUIREMENTS.md`
- `.planning/STATE.md`
- `AGENTS.md`
- `crates/bitaxe-config/src/catalog.rs`
- `crates/bitaxe-config/src/validation.rs`
- `crates/bitaxe-asic/src/bm1366/init_plan.rs`
- `crates/bitaxe-stratum/src/v1/mining_loop.rs`
- `crates/bitaxe-api/src/snapshot.rs`
- `crates/bitaxe-api/src/wire.rs`
- `firmware/bitaxe/src/runtime_snapshot.rs`
- `firmware/bitaxe/src/display_adapter.rs`
- `reference/esp-miner/main/power/DS4432U.c`
- `reference/esp-miner/main/power/INA260.c`
- `reference/esp-miner/main/power/vcore.c`
- `reference/esp-miner/main/thermal/thermal.c`
- `reference/esp-miner/main/thermal/PID.c`
- `reference/esp-miner/main/tasks/fan_controller_task.c`
- `reference/esp-miner/main/tasks/power_management_task.c`
- `reference/esp-miner/main/self_test/self_test.c`
- `reference/esp-miner/main/display.c`
- `reference/esp-miner/main/input.c`
- `reference/esp-miner/main/screen.c`
