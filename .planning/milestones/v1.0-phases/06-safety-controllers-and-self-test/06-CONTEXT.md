---
generated_by: gsd-discuss-phase
lifecycle_mode: yolo
phase_lifecycle_id: 6-2026-06-28T02-37-37
generated_at: 2026-06-28T02:37:37.594Z
---

# Phase 6: Safety Controllers And Self-Test - Context

**Gathered:** 2026-06-28
**Status:** Ready for planning
**Mode:** Yolo

<domain>
## Phase Boundary

Phase 6 delivers Ultra 205 safety-controller behavior for voltage, current, power, thermal, fan, fault, self-test, watchdog, and required display/input/status surfaces. It replaces Phase 5's safe zeroed hardware telemetry with typed safety snapshots where evidence exists, feeds Phase 3 ASIC preflight tokens, feeds Phase 4 mining safety gates, and preserves upstream-visible AxeOS status without claiming unsafe hardware parity from implementation alone.

This phase does not deliver OTA, filesystem, release packaging, recovery flows, non-205 board verification, full Angular UI replacement, full LVGL display parity, Stratum v2, BAP, or all-board factory behavior. Safety-critical voltage, fan, thermal, power, ASIC, and self-test hardware effects can advance to `implemented` with pure tests and fixtures, but they cannot be marked `verified` without board-named Ultra 205 hardware-smoke or hardware-regression evidence.

</domain>

<decisions>
## Implementation Decisions

### Power, Voltage, And Current Safety

- **D-01:** Build a typed pure power-safety decision core before enabling firmware effects. It should treat Ultra 205 BM1366 voltage options as bounded config data, parse live observations into typed values, classify stale or missing samples, and emit explicit effect plans rather than performing I2C writes directly.
- **D-02:** Separate DS4432U voltage-control decisions from INA260 current, voltage, and power observations. Firmware adapters may own I2C transactions, but pure code owns setpoint bounds, unsafe-reading classification, communication-failure handling, and fail-closed status.
- **D-03:** Invalid config, I2C/read failures, stale samples, unsafe readings, missing evidence, or impossible board capabilities must fail closed with visible actions such as holding reset low, disabling ASIC enable where available, suppressing voltage writes, blocking work submission, and publishing a power-fault or safe-blocked status.
- **D-04:** If live Ultra 205 hardware evidence is unavailable during execution, implement observe-only telemetry and explicit unavailable/fault statuses instead of enabling voltage or power actuation. Observe-only data must not be recorded as verified hardware-control parity.

### Thermal, Fan, PID, And Fault Behavior

- **D-05:** Prefer a typed pure safety core with upstream wire projection over a firmware-local direct task port. Pure code should own PID decisions, thermal-control state, fan duty targets, RPM fault classification, overheat stop/cool/restart decisions, and user-visible status mapping.
- **D-06:** Firmware adapters own EMC2101 or equivalent sensor reads, fan PWM/duty effects, I2C errors, and task scheduling. Raw sentinel values, missing readings, diode-fault readings, zero RPM, fan set failures, and implausible temperatures must be parsed into typed faults before they influence control decisions.
- **D-07:** Safety bias is allowed where exact transient upstream behavior would continue unsafe operation. The public API/log/status layer should preserve upstream-visible fields and units, while internal policy may escalate to safe blocked for sustained or high-confidence faults.
- **D-08:** Controller tasks must be planned as watchdog-friendly stepwise work. Long loops should yield or reset watchdog supervision and record watchdog/load responsiveness evidence before the behavior is marked verified.

### Self-Test And Watchdog Lifecycle

- **D-09:** Model self-test as a supervised pure lifecycle with explicit states and effects for factory flag handling, manual start, running, pass, fail, restart, cancel, and result reporting.
- **D-10:** Keep production mining gates closed during self-test unless a hardware-gated diagnostic submode explicitly has all required safety, power, thermal, ASIC, and hardware-evidence acknowledgments. Unit tests may use fake pool, mock Stratum work, and nonce injection; those tests are not hardware proof.
- **D-11:** Preserve upstream-visible factory semantics: self-test may start from the configured self-test flag or boot-button path, and the stored self-test flag should be cleared only on the modeled successful or canceled factory lifecycle where upstream behavior requires it.
- **D-12:** Avoid a blocking reference-style task port. Firmware should supervise self-test in bounded steps, keep API/log/WebSocket responsiveness observable, avoid watchdog starvation, and expose safe failure reasons rather than hiding them behind generic pass/fail booleans.

### Display, Input, And User-Visible Safety Status

- **D-13:** Implement a bounded safety display/input slice only if Ultra 205 OLED/button smoke can be recorded in this phase. The bounded slice should show self-test, overheat, ASIC/safety, connection, and safe-blocked/mining-safe status needed for normal administration.
- **D-14:** If display/input hardware evidence is unavailable, preserve administration through AxeOS API, logs, and WebSocket status first, and document full display/input parity as a V1 gap rather than silently treating startup-only OLED evidence as complete.
- **D-15:** Do not expand Phase 6 into full upstream LVGL display/input parity unless planning proves it can fit without delaying safety-controller work. Full carousel, display config, timeout, rotation, inversion, and broad button-routing parity may remain deferred or explicitly gapped.
- **D-16:** Existing startup SSD1306 rendering is a reusable adapter and evidence breadcrumb, not full display parity. Any runtime display/input claims need separate hardware evidence and checklist entries.

### Evidence, Parity, And Status Integration

- **D-17:** Replace Phase 5's `SafeTelemetrySnapshot::unavailable_until_phase_6()` zeroed values with typed safety telemetry only where the new safety core or firmware adapters can provide trustworthy observations. Unavailable, stale, or faulted values should remain explicit in status and evidence.
- **D-18:** Feed the resulting power, thermal, and safety evidence tokens into the existing BM1366 full-init preflight and mining-loop safety gates. Missing tokens must keep ASIC initialization and work submission blocked.
- **D-19:** Update parity rows `PWR-001` through `PWR-006`, `THR-001` through `THR-003`, `SELF-001`, and relevant `IO`/`UI` rows with implementation pointers and evidence. Use `implemented` for pure or workflow proof and reserve `verified` for hardware-smoke or hardware-regression evidence.
- **D-20:** Reference breadcrumbs belong at module and behavior boundaries. Use independently authored Rust logic for formulas and state machines unless a deliberate GPL-compatible port is isolated and documented.

### the agent's Discretion

The agent may choose exact crate/module names, type names, fixture formats, telemetry snapshot shapes, adapter trait boundaries, plan count, and whether shared safety logic lives in `crates/bitaxe-core` or a new focused crate. Those choices must preserve functional core plus imperative shell, keep upstream reference files read-only, use `maybe_` names for optional internal Rust values, keep tests focused with Arrange/Act/Assert sections, and avoid marking safety-critical behavior verified without evidence.

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Phase Scope And Requirements

- `.planning/ROADMAP.md` - Phase 6 goal, dependencies, success criteria, verification expectations, and research flags.
- `.planning/REQUIREMENTS.md` - SAFE-01 through SAFE-09 plus evidence, API, ASIC, Stratum, and release boundaries.
- `.planning/PROJECT.md` - Ultra 205 BM1366 first target, architecture constraints, safety constraints, and current state.
- `.planning/STATE.md` - Completed Phase 5 status, safety evidence blockers, and recent display smoke context.
- `.planning/phases/01-foundation-and-gamma-601-boot-log/01-CONTEXT.md` - Safe boot/log boundary and hardware-control evidence policy.
- `.planning/phases/02-ultra-205-config-and-nvs-model/02-CONTEXT.md` - Ultra 205 defaults, board capabilities, config validation, and self-test defaults.
- `.planning/phases/03-bm1366-asic-protocol-and-safe-initialization/03-CONTEXT.md` - BM1366 preflight tokens, fail-closed init gate, and hardware evidence limits.
- `.planning/phases/04-stratum-v1-and-first-mining-loop/04-CONTEXT.md` - Mining-loop safety gate, runtime state, and hardware-evidence acknowledgment boundary.
- `.planning/phases/05-axeos-api-logs-and-telemetry/05-CONTEXT.md` - API telemetry/status placeholders and command/log/WebSocket integration points.

### Existing Rust Integration Points

- `crates/bitaxe-config/src/catalog.rs` - Ultra 205 board capabilities including DS4432U, INA260, ASIC-enable, power target, and verification scope.
- `crates/bitaxe-config/src/defaults.rs` - Ultra 205 default voltage, fan, thermal, and self-test values from `config-205.cvs`.
- `crates/bitaxe-config/src/validation.rs` - Existing typed frequency, voltage, fan, and temperature validation.
- `crates/bitaxe-config/src/nvs.rs` - NVS key names and defaults for fan, thermal, overheat, power, and self-test settings.
- `crates/bitaxe-asic/src/bm1366/init_plan.rs` - Full-init requires power, thermal, and safety preflight evidence.
- `crates/bitaxe-stratum/src/v1/mining_loop.rs` - Mining work submission remains blocked without safety evidence and hardware acknowledgment.
- `crates/bitaxe-api/src/snapshot.rs` - `SafeTelemetrySnapshot` is deliberately zeroed until Phase 6 owns live hardware telemetry.
- `crates/bitaxe-api/src/wire.rs` - AxeOS system info fields for voltage, current, power, temperature, fan duty, RPM, and status.
- `crates/bitaxe-api/src/statistics.rs` - Statistics columns for thermal, fan, voltage, power, current, and related telemetry.
- `crates/bitaxe-api/src/telemetry.rs` - Live telemetry envelope/diff/cadence behavior that should carry Phase 6 status.
- `firmware/bitaxe/src/runtime_snapshot.rs` - Firmware snapshot collection boundary that should incorporate Phase 6 safety telemetry.
- `firmware/bitaxe/src/display_adapter.rs` - Existing startup-only SSD1306 adapter and evidence boundary.
- `firmware/bitaxe/src/asic_adapter/reset.rs` - Existing ASIC reset fail-closed action boundary.
- `firmware/bitaxe/src/asic_adapter/status.rs` - Existing visible ASIC and mining-loop blocked status logs.
- `docs/parity/checklist.md` - PWR, THR, SELF, IO, UI, and safety evidence rows to update.

### Upstream Safety, Power, Thermal, Self-Test, And UI References

- `reference/esp-miner/main/device_config.h` - Ultra 205 board capabilities, ASIC profile, voltage options, power target, and device catalog.
- `reference/esp-miner/config-205.cvs` - Ultra 205 defaults for voltage, fan, self-test, and related settings.
- `reference/esp-miner/main/nvs_config.c` - Upstream defaults, NVS keys, validation, and persistence side effects for safety settings.
- `reference/esp-miner/main/power/asic_init.c` - ASIC power initialization sequencing and gate behavior.
- `reference/esp-miner/main/power/asic_reset.c` - ASIC reset GPIO behavior.
- `reference/esp-miner/main/power/vcore.c` - Core voltage behavior and regulator control boundary.
- `reference/esp-miner/main/power/DS4432U.c` - Ultra 205 voltage-control reference.
- `reference/esp-miner/main/power/INA260.c` - Ultra 205 current, voltage, and power telemetry reference.
- `reference/esp-miner/main/power/TPS546.c` - Deferred TPS546 behavior for non-205 boards.
- `reference/esp-miner/main/thermal/thermal.c` - Thermal sensor abstraction and temperature behavior.
- `reference/esp-miner/main/thermal/PID.c` - Upstream PID behavior for fan/thermal control.
- `reference/esp-miner/main/tasks/fan_controller_task.c` - Fan controller task, duty/RPM behavior, and related user-visible values.
- `reference/esp-miner/main/tasks/power_management_task.c` - Overheat stop/cool/restart behavior and power-management loop.
- `reference/esp-miner/main/self_test/self_test.c` - Self-test factory/manual lifecycle, mock work, pass/fail/cancel behavior, and stored flag semantics.
- `reference/esp-miner/main/display.c` - Upstream display initialization and config behavior.
- `reference/esp-miner/main/input.c` - Upstream button/input handling.
- `reference/esp-miner/main/screen.c` - Upstream runtime screen rendering and safety status pages.
- `reference/esp-miner/main/http_server/system_api_json.c` - User-visible API fields for voltage, fan, thermal, power, self-test-adjacent status, and statistics projection.

### Architecture, Evidence, And Policy

- `docs/adr/0001-device-user-parity.md` - Observable device-user parity definition.
- `docs/adr/0003-esp-idf-rust-production-stack.md` - ESP-IDF Rust stack and adapter boundary.
- `docs/adr/0005-read-only-reference-implementation.md` - Read-only upstream reference policy.
- `docs/adr/0006-parity-checklist-as-audit-evidence.md` - Checklist evidence policy.
- `docs/adr/0008-reference-breadcrumb-comments.md` - Breadcrumb placement policy.
- `docs/adr/0009-monorepo-package-layout.md` - Crate and firmware path ownership.
- `docs/adr/0010-axeos-api-and-asset-compatibility.md` - API/static compatibility boundary.
- `docs/adr/0012-parity-verification-evidence.md` - Verification evidence requirements and safety-critical hardware gate.
- `docs/adr/0013-mit-first-with-gpl-guardrails.md` - GPL guardrails for upstream-derived behavior and fixtures.
- `docs/adr/0014-pivot-to-ultra-205-bm1366-first-parity.md` - Ultra 205/BM1366 first parity target and deferred Gamma 601 scope.
- `PROVENANCE.md` - Provenance, SPDX, upstream reference, fixture/source attribution, and release review policy.
- `docs/parity/evidence/ultra-205-startup-display-debug-2026-06-27.md` - Startup-only OLED smoke; not full display/input parity.
- `docs/parity/evidence/phase-03-ultra-205-bm1366-chip-detect.md` - Phase 3 chip-detect evidence status and hardware-verification limits.
- `docs/parity/evidence/phase-04-stratum-v1-mining-loop.md` - Mining-loop gate evidence and remaining hardware limits.
- `docs/parity/evidence/phase-05-axeos-api-logs-and-telemetry.md` - API/log/telemetry evidence and Phase 6 telemetry deferrals.

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets

- `crates/bitaxe-config` already owns Ultra 205 catalog data, DS4432U/INA260 capability flags, default voltage/fan/self-test values, and typed validation for voltage, fan duty, and temperature.
- `crates/bitaxe-asic/src/bm1366/init_plan.rs` already has placeholder power, thermal, and safety evidence token types and keeps full init fail-closed when those tokens are missing.
- `crates/bitaxe-stratum/src/v1/mining_loop.rs` already requires safety evidence and hardware-evidence acknowledgment before work submission can become ready.
- `crates/bitaxe-api/src/snapshot.rs`, `wire.rs`, `statistics.rs`, and `telemetry.rs` already expose the public API/telemetry fields that Phase 6 must populate or explicitly mark unavailable/faulted.
- `firmware/bitaxe/src/runtime_snapshot.rs` already overlays firmware facts onto `ApiSnapshot` and is the natural adapter seam for Phase 6 telemetry.
- `firmware/bitaxe/src/display_adapter.rs` already proves startup-only SSD1306 rendering on Ultra 205 and can be extended only if Phase 6 intentionally owns a bounded safety display/input slice.

### Established Patterns

- Pure decisions live in crates and firmware owns ESP-IDF, I2C, PWM, GPIO, reset, task scheduling, watchdog, and hardware side effects.
- Existing safety gates fail closed by default: ASIC full init and mining work submission both require explicit evidence tokens.
- API wire DTOs are handwritten and separate from internal runtime/domain structs.
- Tests use Arrange, Act, Assert and should prove one concern per unit test.
- Parity checklist rows distinguish unit/golden/API/workflow evidence from hardware-smoke or hardware-regression evidence.

### Integration Points

- Add or extend a safety core that can feed `ApiSnapshot.safe_telemetry`, BM1366 `PowerPreflightEvidence`, `ThermalPreflightEvidence`, `SafetyPreflightEvidence`, and Stratum `MiningLoopGate`.
- Add thin firmware adapters for Ultra 205 DS4432U, INA260, EMC2101 or equivalent thermal/fan paths, button/input where scoped, and watchdog-friendly task supervision.
- Update `docs/parity/checklist.md` and `docs/parity/evidence/` as implementation and hardware evidence land, without changing safety-critical rows to `verified` prematurely.
- Preserve existing AxeOS command/log/WebSocket surfaces from Phase 5 so safety status is visible even if OLED/button evidence is unavailable.

</code_context>

<specifics>
## Specific Ideas

- Treat live telemetry as fresh, stale, faulted, or unavailable instead of assuming a numeric reading is trustworthy.
- Use safe action planning as the common language between power, thermal, ASIC init, mining gate, API status, and firmware adapters.
- Prefer hardware-smoke records that name the board, port, firmware commit, reference commit, command, observed readings/status transitions, and conclusion.
- Include fixtures for missing sensor reads, stale samples, unsafe high temperature, zero RPM, invalid voltage request, I2C failure, self-test cancel, self-test pass, self-test fail, and watchdog-friendly step progression.
- Keep display/input scope bounded to safety status and self-test administration unless planning explicitly accepts the cost of broader display parity.

</specifics>

<deferred>
## Deferred Ideas

- Full LVGL display carousel, display config, timeout behavior, rotation/inversion parity, and broad button-routing parity may remain a documented V1 gap unless Phase 6 planning proves they fit.
- TPS546 live behavior remains deferred to Gamma 601 or another non-205 board path.
- OTA, SPIFFS, recovery, release packaging, static-asset update behavior, and operator release docs remain Phase 7.
- Non-205 boards, BM1370, BM1368, BM1397, all-board factory images, Stratum v2, BAP, and Angular UI replacement remain outside Phase 6.

</deferred>

---

*Phase: 06-safety-controllers-and-self-test*
*Context gathered: 2026-06-28*
