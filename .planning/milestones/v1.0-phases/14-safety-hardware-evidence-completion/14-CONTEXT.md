---
generated_by: gsd-discuss-phase
lifecycle_mode: yolo
phase_lifecycle_id: 14-2026-06-30T23-56-34
generated_at: 2026-06-30T23:56:34.602Z
---

# Phase 14: Safety Hardware Evidence Completion - Context

**Gathered:** 2026-06-30
**Status:** Ready for planning
**Mode:** Yolo

<domain>
## Phase Boundary

Phase 14 closes the current Ultra 205 safety hardware evidence gap. It must produce board-205-only evidence for active safety-control and runtime telemetry surfaces, or keep those claims explicitly below `verified` with bounded recovery instructions, owner/follow-up, and no overclaim.

The phase owns recovery gates, active voltage/power, current telemetry, fan/thermal/PID observations, self-test hardware submode boundaries, watchdog/load responsiveness, runtime display/input status, live API/WebSocket safety telemetry, checklist promotion, redaction review, and final parity validation.

This phase does not own BM1366 trusted chip-detect or mining smoke/soak evidence, same-commit release HTTP/OTA/recovery evidence, non-205 boards, Stratum v2, BAP, all-board factory images, full LVGL display parity, or ad hoc destructive/fault-injection work. Any voltage, fan, thermal, self-test, load/stress, display/input, raw write, erase, rollback, or fault-injection run must be blocked unless the active plan documents exact allow gates, recovery path, stop conditions, and post-action safe-state checks.

</domain>

<decisions>
## Implementation Decisions

### Recovery And Allow Gates

- **D-01:** Use a machine-enforced preflight allow manifest as the minimum gate before any active hardware verification can run. The manifest should bind `board=205`, the selected detector `port=<path>`, `espflash board-info --chip esp32s3 --port <port> --non-interactive`, package manifest identity, source commit, reference commit, allowed surface, allowed command or probe, input bounds, abort conditions, recovery steps, post-action safe-state checks, evidence destination, and redaction reviewer.
- **D-02:** Continue to start all live Ultra 205 work with `just detect-ultra205`. The run may continue only when exactly one likely ESP USB serial port is found and board-info succeeds for that port. Zero ports, multiple ports, board-info failure, a target other than board `205`, stale package identity, missing recovery instructions, or redaction uncertainty must stop the run and produce pending evidence instead of a workaround.
- **D-03:** Add surface-scoped bounded probe wrappers only where Phase 14 needs active `hardware-regression` claims. Wrappers should be per-surface rather than one broad safety script, so voltage, fan, thermal, self-test, watchdog/load, display/input, and live telemetry each define their own limits, stop conditions, recovery, logs, and conclusions.
- **D-04:** Treat watchdog panic, reset, bootloader hold, detector failure, board-info failure, missing safe-state marker, or unavailable recovery package as stop/recovery signals, not as normal passing output. A probe that cannot prove post-action safe state must not promote any active safety-control claim.

### Active Safety Telemetry And Control Evidence

- **D-05:** Keep Phase 14 evidence component-scoped with claim tiers. Use separate packs for safe baseline, power/current telemetry, voltage control, thermal/fan, self-test/watchdog/load, display/input, live API/WebSocket telemetry, and parity/redaction. Each pack should state whether it proves read-only observation, bounded actuation, API/WebSocket projection, safe-unavailable status, or unsupported/pending claims.
- **D-06:** Do not conflate read-only sensor observations with actuator behavior. INA260 current/voltage/power freshness, thermal readings, fan RPM observations, and live API/WebSocket projected values can support narrow board-205 claims only for the exact observed data. DS4432U voltage writes, fan duty effects, ASIC reset/power sequencing, overheat/fault behavior, and self-test hardware submodes require bounded `hardware-regression`.
- **D-07:** Procedure-scoped manifests are required for any probe that actuates hardware or proves live cadence under load. The manifest should capture exact command, allowed inputs, stimulus, expected markers, failure markers, timeout, recovery command, safe-state markers, raw artifact paths, redaction status, and the exact checklist row or subclaim it can support.
- **D-08:** If bounded actuation, sensor feedback, live API/WebSocket access, recovery, or redaction prerequisites are missing, record conservative observe-only or pending evidence. Observe-only evidence may prove safe-unavailable or read-only observations, but it must not verify active voltage, fan duty, overheat/fault, self-test hardware, runtime input/display, watchdog/load stress, or recovery parity.

### Self-Test, Watchdog/Load, And Runtime Display/Input

- **D-09:** Run self-test, watchdog/load, and display/input checks only where a safe firmware route, API/log/WebSocket marker, serial marker, or physical stimulus exists and the plan documents the stimulus and recovery path. Do not add temporary diagnostic firmware routes unless they are compile-gated or otherwise impossible to expose accidentally in production firmware.
- **D-10:** Keep self-test hardware submodes below `verified` unless the run proves the exact submode safely, including any voltage, fan, ASIC work, fake work, pass/fail/cancel, and production-mining gate behavior it touches. Pure self-test unit tests and boot logs are not hardware proof.
- **D-11:** Watchdog/load evidence must prove observable liveness or responsiveness under a bounded documented workload. A supervisor startup/yield log is useful evidence for the supervisor shell, but it does not verify load stress, blocked task behavior, or watchdog recovery without a safe stimulus and pass/fail criteria.
- **D-12:** Runtime display/input claims remain below `verified` unless a real runtime route is exercised and physically or log/API/WebSocket-observed. Startup-only SSD1306 evidence remains a breadcrumb and may support startup display only; it cannot verify runtime display pages, screen flow, LVGL parity, or input hardware behavior.

### Checklist Promotion, Redaction, And Final Verification

- **D-13:** Use exact-claim promotion. Promote only rows whose evidence class matches the exact claim. Active safety-control rows need `hardware-regression`; narrow read-only or safe-unavailable observations can use `hardware-smoke` only when board `205`, port, source commit, reference commit, command/log, conclusion, and redaction review are present.
- **D-14:** Preserve existing checklist row IDs unless a broad row prevents truthful documentation of a narrow verified subclaim. Prefer precise checklist notes and evidence links over large row-model churn. Split rows only when the plan proves that exact subclaims cannot otherwise be represented safely.
- **D-15:** Rows with missing stimulus, missing recovery, failed detector gate, unavailable `DEVICE_URL`, unavailable hardware route, stale package identity, or redaction uncertainty must stay `implemented`, `in-progress`, `deferred`, or pending with owner/follow-up. Do not reuse Phase 11 or Phase 13 evidence to verify fresh Phase 14 active claims unless the row clearly names the older evidence as a narrow historical subclaim.
- **D-16:** Final commit/push must be gated by redaction review, `just parity` with no validation errors, relevant Rust checks for changed code, `just test`, `just verify-reference`, diff review, clean GSD verification status `passed`, and lifecycle validation for this phase attempt.

### the agent's Discretion

The agent may choose the exact plan count, evidence directory layout, allow-manifest schema, probe command names, JSON field names, and whether the manifest validation lives in `tools/parity`, a repo-owned script, or a small Rust host tool. Those choices must preserve repo-owned ESP/esp-rs tooling, keep `reference/esp-miner` read-only, keep active hardware use phase-gated, avoid secrets in evidence, keep functional core plus imperative shell, and avoid standalone body `---` separators in GSD artifacts.

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Phase Scope And Governance

- `.planning/ROADMAP.md` - Phase 14 goal, gap closure, requirements, success criteria, verification expectations, and hardware recovery research flag.
- `.planning/REQUIREMENTS.md` - SAFE-01 through SAFE-09 and EVD-05 traceability for safety hardware evidence.
- `.planning/PROJECT.md` - Ultra 205 first target, ESP-IDF Rust stack, read-only reference, parity, hardware, architecture, licensing, and safety constraints.
- `.planning/STATE.md` - Current project state after Phase 13 and accumulated safety/evidence decisions.
- `AGENTS.md` - Repo-local autonomous Ultra 205 detector gate, stop conditions, evidence metadata, destructive/fault-injection limits, and frontmatter separator rule.
- `standards/core/verification.md` - Repo-native verification and pre-commit expectations.
- `standards/core/testing.md` - Unit-test expectations for changed pure logic.
- `standards/languages/rust.md` - Rust module, naming, invariant, test, and verification guidance.

### Prior Phase Decisions And Evidence

- `.planning/phases/06-safety-controllers-and-self-test/06-CONTEXT.md` - Safety-controller architecture, self-test/watchdog decisions, startup-only display boundary, and evidence policy.
- `.planning/phases/11-safety-controller-hardware-regression-evidence/11-CONTEXT.md` - Phase 11 recovery protocol, component evidence packs, and checklist promotion rules.
- `.planning/phases/13-final-ultra-205-release-evidence/13-CONTEXT.md` - Package identity, detector gate, destructive/recovery stop conditions, `DEVICE_URL` blocker policy, and final verification expectations.
- `docs/parity/evidence/phase-11-safety-controller-hardware-regression-evidence.md` - Current safe boot evidence, residual active safety-control gaps, evidence matrix, redaction policy, and final verification list.
- `docs/parity/evidence/phase-13-final-ultra-205-release-evidence.md` - Latest package/serial evidence, missing `DEVICE_URL`, and release evidence blockers.
- `docs/parity/evidence/phase-06-safety-controllers-and-self-test.md` - Phase 6 safety evidence ledger and hardware-pending boundaries.
- `docs/parity/evidence/phase-06-display-input-runtime-gap.md` - Startup-only display evidence and runtime display/input gap.
- `docs/parity/evidence/phase-09-flash-monitor-evidence-wrapper-hardening.md` - Trusted wrapper-owned serial evidence pattern.

### Current Tooling And Implementation Surfaces

- `Justfile` - Human command surface for `detect-ultra205`, `package`, `flash-monitor`, `parity`, `build`, and `test`.
- `scripts/detect-ultra205.sh` - Required detector and board-info preflight.
- `tools/flash/src/main.rs` - Flash/monitor/flash-monitor evidence capture, trusted-output classification, package manifest resolution, and JSON/log artifact behavior.
- `tools/parity/src/main.rs` - Safety-critical and active safety-control verified-row guards, including `hardware-regression` enforcement.
- `docs/parity/checklist.md` - Current PWR, THR, IO, UI, SELF, API, STAT, and evidence governance rows.
- `crates/bitaxe-safety/src/power.rs` - Pure voltage, current, power, and hardware-evidence decisions.
- `crates/bitaxe-safety/src/thermal.rs` - Pure thermal, fan, PID, and overheat decisions.
- `crates/bitaxe-safety/src/fault.rs` - Safety fault classification.
- `crates/bitaxe-safety/src/self_test.rs` - Pure self-test lifecycle decisions.
- `crates/bitaxe-safety/src/watchdog.rs` - Watchdog-friendly step supervision.
- `firmware/bitaxe/src/safety_adapter.rs` - Observe-only safety adapter facade and effect suppression.
- `firmware/bitaxe/src/safety_adapter/power.rs` - DS4432U/INA260 constants and suppressed voltage-write behavior.
- `firmware/bitaxe/src/safety_adapter/thermal.rs` - Thermal/fan constants, observation parsing, and suppressed fan-write behavior.
- `firmware/bitaxe/src/safety_adapter/watchdog.rs` - Safety supervisor shell and yield logging.
- `firmware/bitaxe/src/runtime_snapshot.rs` - Firmware safety telemetry to API snapshot integration point.
- `firmware/bitaxe/src/display_adapter.rs` - Startup-only SSD1306 adapter and runtime display/input boundary.
- `crates/bitaxe-api/src/telemetry.rs` - Live telemetry envelope, diff, and cadence behavior.
- `firmware/bitaxe/src/http_api.rs` - ESP-IDF HTTP and WebSocket route shell for live safety telemetry probes where reachable.

### Upstream Reference And Policy

- `reference/esp-miner/config-205.cvs` - Ultra 205 voltage, fan, self-test, and board defaults.
- `reference/esp-miner/main/device_config.h` - Ultra 205 board capabilities and hardware profile.
- `reference/esp-miner/main/power/vcore.c` - Core voltage reference behavior.
- `reference/esp-miner/main/power/DS4432U.c` - Ultra 205 voltage-control reference.
- `reference/esp-miner/main/power/INA260.c` - Ultra 205 current, voltage, and power telemetry reference.
- `reference/esp-miner/main/thermal/thermal.c` - Thermal sensor and temperature behavior.
- `reference/esp-miner/main/thermal/PID.c` - Fan/PID behavior.
- `reference/esp-miner/main/tasks/fan_controller_task.c` - Fan duty/RPM task behavior.
- `reference/esp-miner/main/tasks/power_management_task.c` - Overheat stop/cool/restart behavior.
- `reference/esp-miner/main/self_test/self_test.c` - Self-test lifecycle reference.
- `reference/esp-miner/main/display.c` - Display initialization reference.
- `reference/esp-miner/main/input.c` - Input behavior reference.
- `reference/esp-miner/main/screen.c` - Runtime screen behavior reference.
- `reference/esp-miner/main/http_server/system_api_json.c` - User-visible safety telemetry fields.
- `reference/esp-miner/main/http_server/websocket_api.c` - Live telemetry/WebSocket reference behavior.
- `docs/adr/0001-device-user-parity.md` - Observable device-user parity definition.
- `docs/adr/0005-read-only-reference-implementation.md` - Read-only upstream reference policy.
- `docs/adr/0006-parity-checklist-as-audit-evidence.md` - Checklist evidence policy.
- `docs/adr/0012-parity-verification-evidence.md` - Evidence class and verified status semantics.
- `docs/adr/0013-mit-first-with-gpl-guardrails.md` - GPL guardrails for upstream-derived behavior and fixtures.
- `docs/adr/0014-pivot-to-ultra-205-bm1366-first-parity.md` - Ultra 205/BM1366 first parity target and deferred non-205 scope.
- `PROVENANCE.md` - Reference, GPL, fixture, source-attribution, dependency-license, and release-review policy.

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets

- `just detect-ultra205` already implements the mandatory preflight and prints `port=<path>` only after a single likely Ultra 205 serial port passes board-info.
- `just flash-monitor board=205 port=<path> evidence-dir=<path>` already records wrapper-owned `flash-command-evidence.json` and `flash-monitor.log` with board, port, source commit, reference commit, manifest, trusted-output status, and conclusion.
- `tools/parity` already rejects safety-critical `verified` rows without `hardware-smoke` or `hardware-regression`, and rejects active safety-control `verified` rows without `hardware-regression`.
- Phase 11 evidence already provides the component-pack model, redaction template, and residual risk list for active voltage, fan, fault, self-test, watchdog/load, and runtime display/input evidence.
- Firmware safety adapters are observe-only and suppress voltage/fan effects by default, which is the correct starting state for bounded probe planning.

### Established Patterns

- Pure safety decisions live in `crates/bitaxe-safety`; firmware owns ESP-IDF, I2C, GPIO, PWM, task scheduling, watchdog, display, API/WebSocket, and hardware effects.
- Evidence docs use conservative conclusions, exact evidence classes, and explicit non-claims.
- Checklist rows can cite narrow evidence in notes while the broad row remains below `verified`.
- Live hardware work must name board `205`, selected port, source commit, reference commit, package manifest or firmware identity, exact command, board-info output, logs, observed behavior, conclusion, and redaction review.

### Integration Points

- Add Phase 14 evidence under `docs/parity/evidence/phase-14-safety-hardware-evidence-completion.md` and component-scoped subdirectories if generated artifacts are useful.
- Add an allow-manifest schema and validation path before active probes; this can live in `tools/parity`, a small repo-owned script, or a host tool if it keeps validation testable.
- Add safe, bounded probe wrappers only for surfaces with explicit recovery and safe-state checks.
- Update `docs/parity/checklist.md` only after evidence artifacts exist, using exact-claim wording to avoid broad overclaims.
- Extend `tools/parity` tests only if Phase 14 introduces stricter machine-checkable promotion semantics beyond the existing safety-critical and active safety-control guards.

</code_context>

<specifics>
## Specific Ideas

- Preferred component packs: `safe-baseline`, `power-telemetry`, `voltage-control`, `thermal-fan`, `self-test-watchdog-load`, `display-input`, `live-api-websocket-telemetry`, `parity-redaction`.
- Preferred allow manifest fields: board, port, source commit, reference commit, package manifest path and checksum, recovery image path, surface, claim type, allowed command, allowed inputs, stop conditions, timeout, expected safe-state markers, evidence output paths, redaction reviewer, and checklist rows/subclaims.
- Preferred promotion ladder: `unit` for pure safety logic, `workflow` for wrappers/guards, `hardware-smoke` for exact board-named read-only or safe-unavailable observations, and `hardware-regression` for active control, failure paths, self-test hardware, runtime input/display, or load/stress.
- `DEVICE_URL` should be explicit for live API/WebSocket telemetry. Do not infer it from private network scans unless the plan documents target identification and redaction boundaries.
- Redaction review must inspect serial logs, JSON, API responses, WebSocket frames, pasted output, and manual observations for Wi-Fi credentials, pool credentials, API tokens, private endpoints, NVS secret values, and local secrets.

</specifics>

<deferred>
## Deferred Ideas

- Trusted BM1366 chip-detect, work-send/result-receive, and controlled mining smoke/soak belong to Phase 15.
- Same-commit package, flash, serial boot, live HTTP/static/recovery/OTA, rollback, erase, failed-update, and interrupted-update evidence belongs to Phase 16.
- Non-205 boards, TPS546 active behavior, BM1370/BM1368/BM1397, all-board factory images, Stratum v2, BAP, Angular AxeOS replacement, and production mining performance tuning remain deferred.
- Full LVGL runtime display carousel, display config, timeout, rotation, inversion, and broad button routing remain outside Phase 14 unless a plan proves a safe bounded route and physical evidence path.

</deferred>

***

*Phase: 14-safety-hardware-evidence-completion*
*Context gathered: 2026-06-30*
