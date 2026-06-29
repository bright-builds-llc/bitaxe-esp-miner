---
generated_by: gsd-discuss-phase
lifecycle_mode: yolo
phase_lifecycle_id: 11-2026-06-29T20-23-34
generated_at: 2026-06-29T20:24:23.587Z
---

# Phase 11: Safety Controller Hardware Regression Evidence - Context

**Gathered:** 2026-06-29
**Status:** Ready for planning
**Mode:** Yolo

<domain>
## Phase Boundary

Phase 11 closes the Ultra 205 safety-controller hardware evidence gap. It should produce board-205-only evidence for voltage, power, current, thermal, fan, self-test, display/input, watchdog, and load-responsiveness surfaces before any safety-critical row is promoted beyond implemented or explicit safe-unavailable status.

This phase does not expand into ASIC/mining smoke or soak evidence, final release HTTP/OTA/recovery evidence, non-205 boards, all-board factory images, Stratum v2, BAP, full LVGL display/input parity, or ad hoc destructive/fault-injection testing. Any hardware actuation, stress, destructive, or fault-injection step must have a documented recovery path before it runs. If the hardware gate or recovery prerequisites are missing, the phase should record pending evidence without overclaiming.

</domain>

<decisions>
## Implementation Decisions

### Hardware Run And Recovery Protocol

- **D-01:** Use a strict phase-gated runbook as the baseline for all live Ultra 205 hardware work. The runbook must start with `just detect-ultra205` and may continue only when it finds exactly one likely ESP USB serial port and `espflash board-info --chip esp32s3 --port <port> --non-interactive` succeeds.
- **D-02:** Record a recovery path before any hardware actuation, destructive, stress, or fault-injection verification. Without a documented recovery path and exact allowed command set, the task must stop and record evidence as pending or blocked.
- **D-03:** Use observe-only evidence as the safe fallback when detection passes but bench recovery, stimulus, or actuation prerequisites are incomplete. Observe-only evidence can prove safe-unavailable or read-only observations, but it cannot verify voltage writes, fan actuation, overheat/fault injection, destructive recovery, or true load stress.
- **D-04:** Prefer scripted bounded regression probes for repeatable safety evidence only when the probe has explicit limits, redaction, recovery instructions, and a fail-closed outcome. Manual bench evidence is acceptable for physical observations such as visible fan behavior, display/input, or recovery observations that cannot yet be safely automated.

### Sensor And Actuator Evidence Coverage

- **D-05:** Build the Phase 11 evidence around a tiered per-surface matrix. Each matrix row should name the checklist row, component, claim type, allowed command or probe, required metadata, pass criteria, failure criteria, evidence artifact, and whether the claim supports `hardware-smoke`, `hardware-regression`, or remains below verified.
- **D-06:** Separate telemetry reads from actuator writes. INA260 current/voltage/power freshness, EMC2101 or equivalent thermal readings, and fan RPM observations may be evidenced independently from DS4432U voltage writes, fan duty effects, ASIC reset/power sequencing, or overheat/fault actuation.
- **D-07:** Use component-scoped evidence packs under one Phase 11 ledger so DS4432U, INA260, thermal/fan/PID, self-test/watchdog, and display/input conclusions can be promoted or held independently without treating one happy-path log as proof for the full safety surface.
- **D-08:** Do not promote broad rows from black-box flash/monitor smoke alone. A boot log can support board identity, safe startup, and safe-unavailable status, but active voltage, fan, thermal, power, self-test hardware, and failure-path parity need targeted evidence.

### Self-Test, Display/Input, Watchdog, And Load Responsiveness

- **D-09:** Attempt narrow live evidence only where an existing firmware, API, log, WebSocket, or serial route can safely expose the state. Self-test, watchdog status, safety supervisor yield behavior, API/log/WebSocket responsiveness, and safe-unavailable telemetry are good candidates when they require no unsafe stimulus.
- **D-10:** Keep runtime display/input below verified unless a real runtime display/input route exists and can be exercised safely. Existing startup-only SSD1306 evidence is supporting evidence only; it must not be used as verified runtime display/input parity.
- **D-11:** Watchdog/load evidence should prove observable liveness or responsiveness under a bounded, documented workload. Do not infer watchdog parity only from the Phase 6 pure model or from a boot log. Fault-injection or stress-style watchdog checks require explicit stimulus, stop conditions, and recovery.
- **D-12:** If self-test, display/input, watchdog, or load responsiveness cannot be safely run, update the evidence ledger and checklist notes with owner/follow-up instead of silently leaving stale Phase 6 wording.

### Checklist Promotion And Parity Guard Rules

- **D-13:** Use tiered promotion by claim type. `hardware-smoke` can verify a narrow board-named observation such as boot identity, safe-unavailable status, read-only telemetry, or visible bounded behavior. Active voltage/fan/thermal/power/self-test/failure-path parity should require `hardware-regression`.
- **D-14:** Mixed checklist rows should either stay below `verified` or be split into exact subclaims when one row combines pure logic, safe-unavailable telemetry, active control, and failure handling. Do not let one narrow observation verify an entire mixed safety surface.
- **D-15:** Preserve the existing parity guard behavior that rejects safety-critical `verified` rows without `hardware-smoke` or `hardware-regression`; extend tests only when Phase 11 needs more precise row or evidence semantics.
- **D-16:** Every Phase 11 evidence artifact must record board `205`, selected port, source commit, reference commit, exact command or probe, package manifest or firmware identity when applicable, logs, observed behavior, conclusion, and a secret-redaction review.

### the agent's Discretion

The agent may choose exact plan count, evidence file names, JSON schema details, Rust helper names, and whether to implement a minimal hardware-regression CLI or keep evidence as structured docs. Those choices must keep `reference/esp-miner` read-only, preserve functional core plus imperative shell, use repo-owned ESP/esp-rs tooling before custom hardware paths, keep unsafe operations phase-gated, and avoid committing secrets, pool credentials, Wi-Fi credentials, private endpoints, or NVS secret values in evidence.

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Phase Scope And Prior Decisions

- `.planning/ROADMAP.md` - Phase 11 goal, requirements, success criteria, verification expectations, and hardware-planning research flags.
- `.planning/REQUIREMENTS.md` - SAFE-01 through SAFE-09 and EVD-05 traceability for safety hardware evidence.
- `.planning/PROJECT.md` - Ultra 205 BM1366 first target, ESP-IDF Rust stack, reference, parity, hardware, architecture, licensing, and safety constraints.
- `.planning/STATE.md` - Current project position, recent Phase 10 completion state, and accumulated safety/evidence decisions.
- `.planning/v1.0-MILESTONE-AUDIT.md` - Audit gap identifying safety-critical hardware evidence as intentionally pending release-readiness debt.
- `.planning/phases/06-safety-controllers-and-self-test/06-CONTEXT.md` - Phase 6 safety-controller decisions and evidence boundaries.
- `.planning/phases/06-safety-controllers-and-self-test/06-VERIFICATION.md` - Passed Phase 6 host/workflow/API/gate scope and explicit hardware-pending result.
- `docs/parity/evidence/phase-06-safety-controllers-and-self-test.md` - Phase 6 evidence ledger and residual hardware risks.
- `docs/parity/evidence/phase-06-ultra-205-safety-hardware-smoke.md` - Existing hardware-smoke template with pending conclusion and required observations.
- `docs/parity/evidence/phase-06-display-input-runtime-gap.md` - Startup-only display evidence and runtime display/input gap.

### Current Tooling And Evidence Paths

- `AGENTS.md` - Repo-local autonomous Ultra 205 hardware-verification permission, detector gate, stop conditions, and evidence requirements.
- `scripts/detect-ultra205.sh` - Read-only detector gate and board-info command used before autonomous hardware use.
- `tools/flash/src/main.rs` - Wrapper-owned `flash-monitor` evidence JSON/log behavior and trusted-output logic.
- `docs/parity/evidence/phase-09-flash-monitor-evidence-wrapper-hardening.md` - Current trusted wrapper-owned serial evidence pattern.
- `docs/release/ultra-205.md` - Flash/monitor evidence capture and recovery guidance.
- `tools/parity/src/main.rs` - Safety-critical verified-row guard and evidence-token validation.
- `docs/parity/checklist.md` - Current PWR, THR, IO, UI, SELF, API, STAT, and evidence type rows.

### Safety Firmware And Pure Logic Seams

- `crates/bitaxe-safety/src/power.rs` - Pure voltage, power, current, and evidence decisions.
- `crates/bitaxe-safety/src/thermal.rs` - Pure thermal, fan, PID, and evidence decisions.
- `crates/bitaxe-safety/src/fault.rs` - Fault classification and fail-closed behavior.
- `crates/bitaxe-safety/src/self_test.rs` - Pure self-test lifecycle decisions.
- `crates/bitaxe-safety/src/watchdog.rs` - Watchdog-friendly step supervision.
- `firmware/bitaxe/src/safety_adapter.rs` - Observe-only firmware safety adapter facade.
- `firmware/bitaxe/src/safety_adapter/power.rs` - DS4432U/INA260 constants and suppressed voltage-write behavior.
- `firmware/bitaxe/src/safety_adapter/thermal.rs` - EMC2101 address, thermal observation parsing, and suppressed fan-write behavior.
- `firmware/bitaxe/src/safety_adapter/watchdog.rs` - Safety supervisor shell and yield logging.
- `firmware/bitaxe/src/runtime_snapshot.rs` - Firmware snapshot collection and API telemetry integration point.
- `firmware/bitaxe/src/display_adapter.rs` - Startup-only SSD1306 adapter and display/input evidence boundary.

### Upstream Reference And Policy

- `reference/esp-miner/config-205.cvs` - Ultra 205 voltage, fan, self-test, and board defaults.
- `reference/esp-miner/main/device_config.h` - Ultra 205 board capabilities and hardware profile.
- `reference/esp-miner/main/power/vcore.c` - Core voltage reference behavior.
- `reference/esp-miner/main/power/DS4432U.c` - Ultra 205 voltage-control reference.
- `reference/esp-miner/main/power/INA260.c` - Ultra 205 current/voltage/power telemetry reference.
- `reference/esp-miner/main/thermal/thermal.c` - Thermal sensor and temperature behavior.
- `reference/esp-miner/main/thermal/PID.c` - Fan/PID behavior.
- `reference/esp-miner/main/tasks/fan_controller_task.c` - Fan duty/RPM task behavior.
- `reference/esp-miner/main/tasks/power_management_task.c` - Overheat stop/cool/restart behavior.
- `reference/esp-miner/main/self_test/self_test.c` - Self-test lifecycle reference.
- `reference/esp-miner/main/display.c` - Display initialization reference.
- `reference/esp-miner/main/input.c` - Input behavior reference.
- `reference/esp-miner/main/screen.c` - Runtime screen behavior reference.
- `docs/adr/0012-parity-verification-evidence.md` - Hardware evidence requirement for safety-critical verification.
- `docs/adr/0014-pivot-to-ultra-205-bm1366-first-parity.md` - Ultra 205/BM1366 first parity target.
- `PROVENANCE.md` - Reference, GPL, and fixture provenance policy.

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets

- `just detect-ultra205` already gives the required preflight gate for autonomous hardware use and prints the selected `port=<path>` only after board-info succeeds.
- `just flash-monitor board=205 port=<port> evidence-dir=<path>` already writes wrapper-owned `flash-command-evidence.json` and `flash-monitor.log` with source/reference commits and trusted-output status.
- `tools/parity` already rejects safety-critical `verified` rows without `hardware-smoke` or `hardware-regression`.
- Phase 6 evidence files already identify the pending PWR/THR/SELF/UI hardware boundaries and provide a starting hardware-smoke template.
- Firmware safety adapters are currently observe-only and suppress voltage/fan effects by default, which is the correct baseline for evidence capture without unsafe actuation.

### Established Patterns

- Pure safety decisions live in `crates/bitaxe-safety`; firmware adapters own ESP-IDF, I2C, GPIO, PWM, task scheduling, watchdog, display, and hardware effects.
- Evidence files use conservative conclusions and explicit scope boundaries rather than treating implementation as verification.
- Checklist rows can stay `implemented`, `in-progress`, `deferred`, or `not-started` while still citing implementation and pending evidence.
- Hardware evidence names board, port, source commit, reference commit, exact commands, logs, observed behavior, conclusion, and redaction review.

### Integration Points

- Add Phase 11 evidence under `docs/parity/evidence/phase-11-safety-controller-hardware-regression-evidence.md` plus component-scoped generated artifacts if scripted probes are implemented.
- Update `docs/parity/checklist.md` only for rows whose claim type is actually supported by the new evidence.
- Extend `tools/parity` tests if Phase 11 introduces stricter distinction between `hardware-smoke` and `hardware-regression` for active-control claims.
- If live hardware is available, use repo-owned commands only and record blocked/pending conclusions when the detector, firmware route, or recovery prerequisite is missing.

</code_context>

<specifics>
## Specific Ideas

- Evidence matrix should distinguish read-only telemetry, safe-unavailable status, bounded actuation, failure-path behavior, and load/responsiveness.
- Component evidence packs can group DS4432U voltage, INA260 power telemetry, EMC2101 thermal/fan/PID, self-test/watchdog, and display/input without forcing all rows to share one conclusion.
- Runtime display/input remains a V1 gap unless a safe runtime route is implemented and physically observed. Startup-only OLED evidence is a breadcrumb, not runtime parity proof.
- Watchdog/load proof should show observable responsiveness or liveness under bounded conditions, not only a boot log or pure unit test.
- If a row is too broad for the evidence collected, split or leave it below verified instead of making a broad verified claim.

</specifics>

<deferred>
## Deferred Ideas

- Full LVGL runtime display carousel, display config, timeout, rotation, inversion, and broad button-routing parity remain outside Phase 11 unless a later roadmap phase explicitly owns them.
- ASIC initialization, work-send/result-receive, mining-loop smoke, and mining soak evidence belong to Phase 12.
- Final package-to-hardware release evidence, live HTTP/static/recovery/OTA/rollback/erase/interrupted-update proof, and `DEVICE_URL` release evidence belong to Phase 13.
- Non-205 boards, BM1370/BM1368/BM1397, TPS546 hardware behavior, all-board factory images, Stratum v2, BAP, and Angular UI replacement remain deferred.

</deferred>

*Phase: 11-safety-controller-hardware-regression-evidence*
*Context gathered: 2026-06-29*
