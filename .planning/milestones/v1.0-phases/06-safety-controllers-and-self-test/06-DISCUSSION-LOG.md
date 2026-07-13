# Phase 6: Safety Controllers And Self-Test - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md; this log preserves the alternatives considered.

**Date:** 2026-06-28T02:37:37.594Z
**Phase:** 6 - Safety Controllers And Self-Test
**Mode:** Yolo
**Areas discussed:** Power, voltage, and current safety; Thermal, fan, PID, and fault behavior; Self-test and watchdog lifecycle; Display/input/status evidence

---

## Power, Voltage, And Current Safety

| Option | Description | Selected |
| --- | --- | --- |
| Typed pure safety controller + effect plan | Unit-testable fail-closed decisions, bounded Ultra 205 voltage options, DS4432U/INA260 separation, evidence can remain implemented until hardware smoke. | yes |
| Observe-only telemetry gate before actuation | Capture availability and safe telemetry without changing voltage when hardware evidence is unavailable. | fallback |
| Firmware-local upstream-style power task | Closer task shape but harder to test and easier to overclaim. | no |
| GPL-isolated upstream formula/driver port | Possible for exact formula parity, but adds licensing and provenance overhead. | no |

**Selected answer:** Typed pure safety controller + effect plan, with observe-only telemetry as the fallback when hardware evidence is unavailable.
**Notes:** Voltage/power/current observations should be classified as fresh, stale, faulted, or unavailable before influencing control. Invalid config, failed reads, unsafe readings, or missing evidence fail closed and remain below verified until Ultra 205 hardware evidence exists.

---

## Thermal, Fan, PID, And Fault Behavior

| Option | Description | Selected |
| --- | --- | --- |
| Upstream-faithful firmware tasks | Fast observable parity but mixes I/O and policy and carries sentinel quirks. | no |
| Typed pure safety core with upstream wire projection | Unit-testable PID/thermal decisions, typed faults, compatible API projection, and functional-core structure. | yes |
| Safety-biased fault escalation | Stronger hardware protection on failures and sustained unsafe readings. | policy flavor |
| Chip-native fan-control offload | Possible later but diverges from upstream firmware PID before baseline evidence. | no |

**Selected answer:** Typed pure safety core with upstream wire projection, using safety-biased fault escalation where exact transient behavior would continue unsafe operation.
**Notes:** Raw sentinel values, EMC diode faults, read failures, zero RPM, and implausible readings must be parsed into typed faults. Controller tasks should be watchdog-friendly and evidence-backed before verified.

---

## Self-Test And Watchdog Lifecycle

| Option | Description | Selected |
| --- | --- | --- |
| Supervised pure lifecycle with safe-diagnostic mode | Models factory/manual lifecycle and reporting without unsafe live mining; testable with fake work and nonce injection. | yes |
| Hardware-gated self-test submode with mock Stratum work | Closest safe path to upstream hardware behavior when controlled Ultra 205 evidence is available. | gated option |
| Reference-style blocking task port | Closer sequence but higher safety, watchdog, testability, and provenance risk. | no |
| Status-only self-test facade | Safe interim surface but not real parity and must remain unverified. | no |

**Selected answer:** Supervised pure lifecycle with safe-diagnostic mode, plus hardware-gated subtests only when safety, ASIC, power, thermal, and evidence gates are present.
**Notes:** Self-test should cover factory flag, manual start, pass, fail, restart, cancel, and result reporting. Long-running work must be stepwise and watchdog-supervised instead of copied as an opaque blocking loop.

---

## Display/Input/Status Evidence

| Option | Description | Selected |
| --- | --- | --- |
| Status-first preservation with explicit UI gaps | Keeps safety work focused and preserves API/log/WebSocket administration when hardware/UI evidence is limited. | fallback |
| Bounded safety display/input slice | Adds on-device safety/self-test/blocked status and button behavior if OLED/button smoke is available. | yes when evidence is available |
| Full upstream LVGL display/input parity now | Broadest parity but large scope expansion competing with safety work. | no |

**Selected answer:** Implement a bounded safety display/input slice only if Ultra 205 OLED/button smoke can be recorded; otherwise preserve API/log/WebSocket administration and document display/input gaps explicitly.
**Notes:** Existing startup SSD1306 evidence is not full display parity. Full carousel, display config, timeout, rotation, inversion, and broad input parity remain deferred unless planning proves they fit safely.

---

## the agent's Discretion

- Exact Rust module and type names.
- Fixture file formats and evidence record shape.
- Whether shared safety logic belongs in `crates/bitaxe-core` or a new focused crate.
- Adapter trait boundaries for I2C, PWM, GPIO, watchdog, display, and input.

## Deferred Ideas

- Full LVGL display/input parity may be planned separately or documented as a V1 gap.
- TPS546 behavior remains for deferred non-205 board evidence.
- OTA, filesystem, release packaging, and recovery remain Phase 7.
