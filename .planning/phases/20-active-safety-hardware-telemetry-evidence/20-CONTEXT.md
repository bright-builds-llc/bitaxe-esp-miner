---
generated_by: gsd-discuss-phase
lifecycle_mode: yolo
phase_lifecycle_id: 20-2026-07-03T20-48-00
generated_at: 2026-07-03T20:50:01.088Z
---

# Phase 20: Active Safety Hardware Telemetry Evidence - Context

**Gathered:** 2026-07-03
**Status:** Ready for planning
**Mode:** Yolo

<domain>
## Phase Boundary

Phase 20 closes the remaining Ultra 205 active safety hardware and live telemetry evidence gap. It must produce board-205-only hardware-regression or live-route evidence for active voltage/power, fan/thermal, self-test, watchdog/load, runtime display/input, failure-path, and live API/WebSocket telemetry surfaces where safe prerequisites exist. Where prerequisites are missing, it must record exact blockers, owners, recovery expectations, and conservative checklist notes without promoting unsupported claims.

This phase does not own live production mining or soak evidence, firmware OTA/OTAWWW/recovery regressions, non-205 boards, Stratum v2, BAP, all-board release images, full LVGL display parity, or performance tuning. Any voltage, fan, thermal, self-test, load, stress, fault-injection, runtime input/display, or raw hardware action must be blocked unless the active plan documents exact allow gates, recovery path, stop conditions, post-action safe-state checks, and redaction rules.

</domain>

<decisions>
## Implementation Decisions

### Gated Evidence Runbook

- **D-01:** Reuse and tighten the Phase 14 safety allow-manifest pattern rather than inventing a new broad hardware script. Every active Phase 20 probe must bind board `205`, selected detector port, passed board-info, package manifest identity, source commit, reference commit, allowed surface, claim tier, exact command, bounded inputs, abort conditions, recovery steps, post-action safe-state markers, evidence directory, redaction reviewer, and checklist rows or subclaims.
- **D-02:** Start live hardware work with `just detect-ultra205`. Continue only when it finds exactly one likely ESP32-S3 USB serial port and `espflash board-info --chip esp32s3 --port <port> --non-interactive` succeeds. Zero ports, multiple ports, board-info failure, stale package identity, missing recovery instructions, or target other than board `205` must create blocked evidence instead of a workaround.
- **D-03:** Use surface-scoped probes and evidence packs. Keep active voltage/power, fan/thermal, self-test/watchdog/load, display/input, failure paths, and live API/WebSocket telemetry independently promotable or independently blocked.
- **D-04:** Treat resets, watchdog panics, unexpected reboot, detector failure, missing safe-state markers, missing restore package, or unavailable route evidence as stop/recovery signals. A probe that cannot prove post-action safe state must not support an active safety verified claim.

### Active Safety Surface Coverage

- **D-05:** Separate read-only observations from actuator or failure-path proof. INA260, thermal, fan RPM, `/api/system/info`, statistics, and WebSocket readings can support only exact observed telemetry subclaims. DS4432U writes, fan duty effects, ASIC reset/power sequencing, overheat/fault behavior, self-test hardware submodes, bounded load stress, runtime input, and runtime display need hardware-regression evidence.
- **D-06:** Prefer observe-only or safe-unavailable evidence when a production-safe stimulus route does not exist. Observe-only evidence may prove blockers, safe state, stale/unavailable projection, or transport shape, but it must not verify active control behavior.
- **D-07:** If Phase 20 introduces any new active probe route or diagnostic trigger, it must be compile-gated, bounded, impossible to expose accidentally in production flows, and covered by tests plus redaction review. Repo-owned ESP/esp-rs tooling remains the preferred path.
- **D-08:** Live API/WebSocket telemetry must be correlated with hardware observations and safe-state markers. Route presence, a no-upgrade response, or a stale cached API body is not enough to prove live safety telemetry freshness or cadence.

### Runtime Display, Input, Self-Test, Watchdog, And Load

- **D-09:** Runtime display/input remains below verified unless a real runtime route is exercised and physically or log/API/WebSocket-observed. Startup SSD1306 evidence and `display_input_status=runtime_gap` remain supporting breadcrumbs only.
- **D-10:** Self-test hardware submodes remain below verified unless the run safely proves the exact submode, pass/fail/cancel behavior, production-mining gate behavior, recovery path, and post-action safe state.
- **D-11:** Watchdog/load evidence must use a bounded workload or safe stimulus with pass/fail criteria. Supervisor startup/yield logs prove only the supervisor shell, not load stress, blocked task behavior, or watchdog recovery.
- **D-12:** Failure-path evidence must name the stimulus, expected fault, abort condition, restore path, observed status, API/log/WebSocket projection, and final safe-state marker before any fault-path checklist row can be promoted.

### Checklist, Redaction, And Verification

- **D-13:** Use exact-claim promotion. Active safety-control and failure-path rows require `hardware-regression`; narrow read-only, safe-unavailable, startup, or transport observations may use `hardware-smoke` only when the artifact actually proves the subclaim.
- **D-14:** Preserve checklist row IDs where precise notes can communicate the boundary. Split or add subclaim wording only if a broad row would otherwise force a false verified claim.
- **D-15:** Redaction review is mandatory before any evidence is committed. It must cover serial logs, JSON manifests, API bodies, WebSocket frames, detector output, board-info output, package logs, pasted command output, manual observations, `DEVICE_URL`, IP addresses, MAC addresses, SSIDs, Wi-Fi credentials, pool credentials, worker secrets, API tokens, NVS secret values, and local terminal secrets.
- **D-16:** Final verification must include relevant wrapper/script tests, changed Rust checks, `just test`, `just parity`, `just verify-reference`, reference diff cleanliness, redaction review, lifecycle validation, and every hardware/network command actually used. The wrapper-level commit/push gate may run only when `20-VERIFICATION.md` reports `status: passed`.

### the agent's Discretion

The agent may choose the exact plan count, evidence directory layout, JSON field names, helper names, whether to extend Phase 14 helpers or add Phase 20 wrappers, and whether new checks live in `tools/parity`, a repo-owned script, or a small host tool. Those choices must keep `reference/esp-miner` read-only, preserve functional core plus imperative shell, use ESP-IDF/esp-rs tooling before custom hardware paths, avoid secrets in evidence, avoid standalone body `---` separators in parsed Markdown, and avoid broad verified claims.

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Phase Scope And Governance

- `.planning/ROADMAP.md` - Phase 20 goal, gap closure, requirements, success criteria, verification expectations, and hardware recovery research flag.
- `.planning/REQUIREMENTS.md` - SAFE-01 through SAFE-09 and EVD-05 traceability for active safety hardware and live telemetry evidence.
- `.planning/PROJECT.md` - Ultra 205 BM1366 first target, ESP-IDF Rust stack, read-only reference, parity evidence policy, hardware priority, architecture, licensing, and safety constraints.
- `.planning/STATE.md` - Current project position after Phase 19 and accumulated safety, release, ASIC, mining, and evidence decisions.
- `AGENTS.md` - Repo-local autonomous Ultra 205 detector gate, evidence metadata, redaction rules, destructive/fault-injection limits, and frontmatter separator rule.
- `standards/core/verification.md` - Repo-native verification and pre-commit expectations.
- `standards/core/testing.md` - Unit-test expectations for changed pure logic.
- `standards/languages/rust.md` - Rust module, naming, invariant, test, and verification guidance.

### Prior Phase Decisions And Evidence

- `.planning/phases/06-safety-controllers-and-self-test/06-CONTEXT.md` - Safety-controller architecture, self-test/watchdog decisions, startup-only display boundary, and evidence policy.
- `.planning/phases/11-safety-controller-hardware-regression-evidence/11-CONTEXT.md` - Recovery protocol, component evidence packs, tiered promotion, and hardware evidence metadata rules.
- `.planning/phases/14-safety-hardware-evidence-completion/14-CONTEXT.md` - Safety allow-manifest pattern, component-scoped probes, exact-claim promotion, and final verification expectations.
- `.planning/phases/14-safety-hardware-evidence-completion/14-VERIFICATION.md` - Passed Phase 14 evidence-governance result and residual active-control/live-telemetry blockers.
- `.planning/phases/15-bm1366-mining-evidence-completion/15-CONTEXT.md` - Mining boundary and active safety surfaces that remain outside mining evidence unless prerequisite-only.
- `.planning/phases/17-live-http-api-and-static-evidence/17-CONTEXT.md` - Explicit `DEVICE_URL` gate, route/WebSocket proof depth, redaction, and target-lock policy.
- `.planning/phases/19-recovery-regression-and-otawww-evidence/19-CONTEXT.md` - Recovery/fault-injection gating, package identity, and no ad hoc raw-command policy.
- `docs/parity/evidence/phase-14-safety-hardware-evidence-completion.md` - Current safety ledger, residual blockers, exact claims supported, redaction review, and final verification commands.
- `docs/parity/evidence/phase-14-safety-hardware-evidence-completion/power-telemetry.md` - Power telemetry blocker and read-only evidence boundary.
- `docs/parity/evidence/phase-14-safety-hardware-evidence-completion/voltage-control.md` - Active voltage-control blocker and no-overclaim conclusion.
- `docs/parity/evidence/phase-14-safety-hardware-evidence-completion/thermal-fan.md` - Thermal/fan blocker and fan duty non-claim.
- `docs/parity/evidence/phase-14-safety-hardware-evidence-completion/self-test-watchdog-load.md` - Watchdog marker evidence and self-test/load blockers.
- `docs/parity/evidence/phase-14-safety-hardware-evidence-completion/display-input.md` - Startup display and runtime input/display gap evidence.
- `docs/parity/evidence/phase-14-safety-hardware-evidence-completion/live-api-websocket-telemetry.md` - Missing `DEVICE_URL` live telemetry blocker.
- `docs/parity/evidence/phase-17-live-http-api-and-static-evidence/summary.md` - Final live HTTP/API/WebSocket route evidence and redaction pattern for explicit target runs.
- `docs/parity/evidence/phase-19-recovery-regression-and-otawww-evidence/summary.md` - Latest release/recovery/OTAWWW evidence boundary and redaction pattern.

### Current Tooling And Implementation Surfaces

- `Justfile` - Human command surface for `detect-ultra205`, `package`, `flash-monitor`, `monitor`, `parity`, `verify-reference`, `build`, and `test`.
- `scripts/detect-ultra205.sh` - Required detector and board-info preflight before autonomous hardware use.
- `scripts/phase14-power-voltage.sh` - Existing power/voltage evidence wrapper and active voltage blocker behavior.
- `scripts/phase14-thermal-fan.sh` - Existing thermal/fan evidence wrapper and fan duty blocker behavior.
- `scripts/phase14-self-test-watchdog-load.sh` - Existing self-test/watchdog/load evidence wrapper and supervisor marker parsing.
- `scripts/phase14-display-input.sh` - Existing startup display/runtime input gap evidence wrapper.
- `scripts/phase14-live-telemetry.sh` - Existing explicit `DEVICE_URL` live API/WebSocket telemetry wrapper.
- `scripts/phase14-*-test.sh` - Existing wrapper tests to reuse or extend when Phase 20 changes helper behavior.
- `scripts/phase17-live-http-api-smoke.sh` - Explicit target live HTTP/API evidence helper pattern.
- `scripts/phase17-websocket-capture.mjs` - Bounded WebSocket capture helper pattern.
- `scripts/BUILD.bazel` - Bazel shell targets and tests for phase helper integration.
- `tools/parity/src/safety_allow.rs` - Safety allow-manifest validator, active claim tiers, required abort conditions, and safe-state marker enforcement.
- `tools/parity/src/main.rs` - Checklist validation, evidence-token guards, blocker-language checks, and parity report command.
- `crates/bitaxe-safety/src/evidence.rs` - Safety evidence labels and hardware verification classes.
- `crates/bitaxe-safety/src/power.rs` - Pure voltage, current, power, and evidence decisions.
- `crates/bitaxe-safety/src/thermal.rs` - Pure thermal, fan, PID, and overheat decisions.
- `crates/bitaxe-safety/src/fault.rs` - Safety fault classification.
- `crates/bitaxe-safety/src/self_test.rs` - Pure self-test lifecycle decisions.
- `crates/bitaxe-safety/src/watchdog.rs` - Watchdog-friendly step supervision.
- `firmware/bitaxe/src/safety_adapter.rs` - Firmware safety facade and observe-only effect suppression.
- `firmware/bitaxe/src/safety_adapter/power.rs` - DS4432U/INA260 constants and suppressed voltage-write behavior.
- `firmware/bitaxe/src/safety_adapter/thermal.rs` - Thermal/fan constants, observation parsing, and suppressed fan-write behavior.
- `firmware/bitaxe/src/safety_adapter/watchdog.rs` - Safety supervisor shell and yield logging.
- `firmware/bitaxe/src/runtime_snapshot.rs` - Firmware safety telemetry to API snapshot integration point.
- `firmware/bitaxe/src/display_adapter.rs` - Startup-only SSD1306 adapter and runtime display/input boundary.
- `crates/bitaxe-api/src/telemetry.rs` - Live telemetry envelope, diff, and cadence behavior.
- `firmware/bitaxe/src/http_api.rs` - ESP-IDF HTTP/WebSocket route shell for live telemetry probes when `DEVICE_URL` is available.
- `docs/parity/checklist.md` - Current PWR, THR, SELF, IO, UI, API, STAT, SAFE, and EVD rows to update conservatively.

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
- `docs/adr/0003-esp-idf-rust-production-stack.md` - ESP-IDF Rust stack and adapter boundary.
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

- Phase 14 already created safety allow manifests, component evidence packs, and wrappers for power/voltage, thermal/fan, self-test/watchdog/load, display/input, and live API/WebSocket telemetry.
- `tools/parity` already rejects safety-critical `verified` rows without hardware evidence and active safety-control `verified` rows without `hardware-regression`.
- `just detect-ultra205` and `just flash-monitor board=205 port=<path> evidence-dir=<path>` already provide the required detector, board-info, package, serial, source/reference commit, and trusted wrapper evidence chain.
- Phase 17 already provides an explicit-target live HTTP/API/WebSocket capture pattern and redaction-reviewed artifacts for live route and frame evidence.
- Firmware safety adapters currently suppress active voltage and fan effects by default, which is the correct baseline for evidence-first planning.

### Established Patterns

- Pure safety decisions live in `crates/bitaxe-safety`; firmware owns ESP-IDF, I2C, GPIO, PWM, task scheduling, watchdog, display, HTTP/WebSocket, and hardware effects.
- Evidence files use conservative conclusions, exact evidence classes, explicit blockers, and redaction review before citation.
- Checklist rows cite exact artifacts and remain below `verified` when the artifact proves only a narrow or unavailable subclaim.
- Hardware evidence names board `205`, selected port, source commit, reference commit, package manifest or firmware identity, exact command, board-info output, logs, observed behavior, conclusion, and redaction status.

### Integration Points

- Add Phase 20 evidence under `docs/parity/evidence/phase-20-active-safety-hardware-telemetry-evidence/` with summary, component packs, raw/generated logs, and redaction review.
- Reuse or extend Phase 14 wrappers where they already express the right safety boundary; avoid duplicating logic unless Phase 20 needs new proof depth.
- Extend `tools/parity` tests only if Phase 20 promotes rows or adds machine-checkable evidence semantics not covered by current safety guards.
- Update `docs/parity/checklist.md`, `.planning/REQUIREMENTS.md`, release notes, and phase verification only after evidence artifacts exist.
- Keep `DEVICE_URL` explicit for live API/WebSocket telemetry; do not infer targets from serial logs, private network scans, router state, ARP, or mDNS.

</code_context>

<specifics>
## Specific Ideas

- Preferred component packs: `safe-baseline`, `active-power-voltage`, `active-thermal-fan`, `self-test-watchdog-load`, `runtime-display-input`, `failure-paths`, `live-api-websocket-telemetry`, and `parity-redaction`.
- Preferred promotion ladder: `unit` for pure logic, `workflow` for validators/wrappers, `hardware-smoke` for exact board-named read-only or safe-unavailable observations, and `hardware-regression` for active control, fault paths, self-test hardware, runtime input/display, or load/stress.
- Preferred live telemetry proof: detector-gated current package, explicit `DEVICE_URL`, `/api/system/info` safety fields, bounded `/api/ws/live` frame capture, correlated serial or hardware observation, and final safe-state marker.
- Preferred blocked evidence behavior: write useful pending artifacts with exact missing prerequisite, owner/follow-up, non-claims, and checklist rows affected.
- Preferred redaction behavior: commit only redacted or allowlisted micro-artifacts; keep raw local artifacts uncommitted unless a plan defines quarantine and review.

</specifics>

<deferred>
## Deferred Ideas

- Live production mining, accepted/rejected shares, production pool behavior, and bounded soak belong to Phase 21.
- Firmware OTA, whole-`www` OTAWWW, failed-update recovery, large erase, interrupted update, rollback, and boot-validation evidence remain governed by Phases 18 and 19 and are not Phase 20 scope.
- Non-205 boards, BM1370/BM1368/BM1397, TPS546 active behavior, all-board factory images, Stratum v2, BAP, Angular AxeOS replacement, and performance tuning remain deferred.
- Full LVGL runtime display carousel, display config, timeout, rotation, inversion, and broad button-routing parity remain outside Phase 20 unless a plan proves a safe bounded route and physical evidence path.

</deferred>

***

*Phase: 20-active-safety-hardware-telemetry-evidence*
*Context gathered: 2026-07-03*
