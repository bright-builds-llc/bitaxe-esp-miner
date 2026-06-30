---
generated_by: gsd-discuss-phase
lifecycle_mode: yolo
phase_lifecycle_id: 12-2026-06-30T00-13-19
generated_at: 2026-06-30T00:14:49.245Z
---

# Phase 12: ASIC And Mining Hardware Evidence - Context

**Gathered:** 2026-06-29
**Status:** Ready for planning
**Mode:** Yolo

<domain>
## Phase Boundary

Phase 12 closes the Ultra 205 ASIC and mining hardware evidence gap. It should produce safety-gated board-205 evidence for BM1366 chip-detect/staged initialization, typed work-send/result-receive, and the first controlled mining-loop smoke or soak run before ASIC/mining parity claims are promoted.

This phase does not expand into final release HTTP/OTA/recovery evidence, non-205 boards, additional ASIC families, Stratum v2, BAP, UI replacement, or ad hoc voltage/fan/mining stress. Live ASIC or mining commands may run only after the repo-local Ultra 205 detector gate and the relevant safety, config, power, thermal, and recovery prerequisites are satisfied. If those prerequisites are missing, the phase should record pending evidence without overclaiming.

</domain>

<decisions>
## Implementation Decisions

### Hardware Gate And Recovery Protocol

- **D-01:** Start every live Phase 12 hardware attempt with `just detect-ultra205`. Continue only when it finds exactly one likely ESP USB serial port and `espflash board-info --chip esp32s3 --port <port> --non-interactive` succeeds.
- **D-02:** Treat Phase 11 safety evidence as a prerequisite, not a substitute. BM1366 init, work dispatch, and mining-loop evidence must explicitly name the board, selected port, source commit, reference commit, package manifest or firmware identity, exact commands, logs, observed behavior, conclusion, and redaction review.
- **D-03:** Record a recovery path and exact allowed command set before any live mining, soak, reset sequencing beyond the existing wrapper, or other sustained hardware actuation. Without those prerequisites, stop the live portion and mark the evidence pending or blocked.
- **D-04:** Use repo-owned commands and wrappers first: `just package`, `just flash-monitor board=205 port=<port> evidence-dir=<path>`, `just monitor port=<port>`, and any Phase 12 probe tooling added by the plans. Do not fall back to raw `espflash`, `esptool.py`, pool scripts, or direct serial writes unless the plan documents why the repo-owned path cannot cover the workflow.

### BM1366 Init And Work/Result Evidence

- **D-05:** Preserve the Phase 3 semantic ASIC boundary. Evidence should exercise `crates/bitaxe-asic` command/observation types and firmware adapter behavior, not direct raw BM1366 packet construction in user-facing orchestration.
- **D-06:** Stage evidence from safest to most active: detector gate, package identity, safe boot, chip-detect, staged initialization, diagnostic work-send/result-receive, then production mining-loop smoke only if prior gates pass.
- **D-07:** Chip-detect and staged initialization evidence must fail closed on missing board/config/power/thermal/safety tokens, chip-count mismatch, UART timeout, malformed frames, or setup faults. A failure can be useful evidence when the log clearly proves no mining or unsafe control continued.
- **D-08:** Work-send/result-receive hardware smoke should use typed, bounded diagnostic work before live pool work when possible. It must record the command path, expected BM1366 observation, actual nonce/register/result observation, timeout behavior, and whether the result supports `hardware-smoke`, remains pending, or is blocked.

### Controlled Mining Smoke And Soak

- **D-09:** Run the first mining-loop evidence only after fake-pool and pure Stratum/BM1366 tests are green and the live ASIC gate has passed. Missing pool, chip-detect, power, thermal, watchdog, or recovery evidence must keep mining safe-blocked.
- **D-10:** Prefer controlled or fake-pool conditions for the first smoke when they can prove pool lifecycle, job construction, work dispatch, result parsing, share submission decisions, hashrate inputs, API/telemetry status, and watchdog responsiveness without exposing secrets.
- **D-11:** Public or real-pool smoke is allowed only through a redacted evidence procedure. Evidence may record a sanitized pool host/category and lifecycle state, but must not commit pool credentials, worker secrets, private endpoints, Wi-Fi credentials, or NVS secret values.
- **D-12:** Soak evidence should be bounded by duration, stop conditions, temperature/power/watchdog observations, accepted/rejected shares or a documented controlled no-share condition, reconnect/fallback observations when exercised, and a post-run safe-stop or recovery note.

### Checklist Promotion And Evidence Semantics

- **D-13:** Keep ASIC-07, STR-06, and STR-07 below `verified` until the captured evidence matches the exact claim. Pure tests, fake-pool tests, package evidence, or serial boot logs can support implementation, but live ASIC and mining claims require `hardware-smoke` or soak evidence.
- **D-14:** Split or narrow broad checklist claims instead of letting one happy-path log verify chip-detect, full init, work/result handling, mining loop, watchdog, and telemetry together. Each promoted row should cite the precise evidence artifact and scope.
- **D-15:** Extend `tools/parity` only when Phase 12 introduces stricter machine-checkable evidence semantics. Do not create a second parity validator or release gate.
- **D-16:** Every Phase 12 evidence artifact must include an explicit conclusion such as `passed for chip-detect smoke`, `passed for bounded mining smoke`, `controlled no-share condition`, `blocked by detector gate`, or `pending recovery prerequisite`.

### the agent's Discretion

The agent may choose the exact plan count, evidence file names, probe command shape, JSON schema details, helper module names, and whether the first smoke uses a diagnostic BM1366 path, fake-pool path, or controlled real-pool path. Those choices must preserve functional core plus imperative shell, keep `reference/esp-miner` read-only, use repo-owned ESP/esp-rs tooling before custom hardware paths, avoid unsafe or unbounded mining stress, and keep all evidence free of secrets.

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Phase Scope And Prior Decisions

- `.planning/ROADMAP.md` - Phase 12 goal, requirements, success criteria, verification expectations, and hardware/pool-soak research flags.
- `.planning/REQUIREMENTS.md` - ASIC-07, STR-06, STR-07, and EVD-05 traceability for ASIC/mining hardware evidence.
- `.planning/PROJECT.md` - Ultra 205 BM1366 first target, ESP-IDF Rust stack, reference policy, parity evidence policy, hardware priority, architecture, licensing, and safety constraints.
- `.planning/STATE.md` - Current project position after Phase 11 and active Phase 12 focus.
- `.planning/phases/03-bm1366-asic-protocol-and-safe-initialization/03-CONTEXT.md` - BM1366 semantic command/observation boundary, staged init gate, and hardware evidence rules.
- `.planning/phases/04-stratum-v1-and-first-mining-loop/04-CONTEXT.md` - Stratum v1, fake-pool, mining-loop, runtime state, and first-loop evidence boundaries.
- `.planning/phases/08-parity-evidence-and-ultra-205-release-gate/08-CONTEXT.md` - Evidence governance, verified-claim policy, release-gate integration, and secret/redaction boundaries.
- `.planning/phases/11-safety-controller-hardware-regression-evidence/11-CONTEXT.md` - Detector gate, recovery protocol, evidence metadata, and safety-critical promotion rules.
- `.planning/phases/11-safety-controller-hardware-regression-evidence/11-VERIFICATION.md` - Latest Phase 11 verification result and residual hardware-evidence limits.

### Current Evidence And Tooling

- `AGENTS.md` - Repo-local autonomous Ultra 205 hardware-verification permission, detector gate, stop conditions, and evidence requirements.
- `scripts/detect-ultra205.sh` - Required read-only detector gate before autonomous hardware use.
- `Justfile` - Human command surface for package, flash-monitor, monitor, detect-ultra205, verify-reference, parity, and tests.
- `tools/flash/src/main.rs` - Wrapper-owned flash/monitor command construction, package manifest lookup, evidence JSON/log capture, and trusted-output behavior.
- `docs/parity/checklist.md` - ASIC, STR, STAT/API-adjacent, and evidence rows whose statuses must not overclaim.
- `docs/parity/evidence/phase-03-ultra-205-bm1366-chip-detect.md` - Earlier BM1366 chip-detect evidence status and limits.
- `docs/parity/evidence/phase-04-stratum-v1-mining-loop.md` - Earlier Stratum/mining-loop evidence status and limits.
- `docs/parity/evidence/phase-09-flash-monitor-evidence-wrapper-hardening.md` - Current wrapper-owned serial evidence pattern.
- `docs/parity/evidence/phase-11-safety-controller-hardware-regression-evidence.md` - Latest safety hardware evidence ledger and residual limits.
- `docs/parity/evidence/phase-11-safety-controller-hardware-regression-evidence/flash-command-evidence.json` - Latest wrapper-generated machine evidence shape.
- `docs/parity/evidence/phase-11-safety-controller-hardware-regression-evidence/flash-monitor.log` - Latest serial capture pattern and firmware identity markers.
- `docs/parity/evidence/phase-11-safety-controller-hardware-regression-evidence/redaction-review.md` - Current redaction review pattern for generated evidence.

### Rust And Firmware Integration Points

- `crates/bitaxe-asic/src/bm1366.rs` - BM1366 module entrypoint.
- `crates/bitaxe-asic/src/bm1366/adapter_gate.rs` - Evidence gate model for adapter actions.
- `crates/bitaxe-asic/src/bm1366/chip_detect.rs` - Chip-detect decision logic.
- `crates/bitaxe-asic/src/bm1366/command.rs` - Typed BM1366 command boundary.
- `crates/bitaxe-asic/src/bm1366/init_plan.rs` - Staged initialization plan and fail-closed states.
- `crates/bitaxe-asic/src/bm1366/observation.rs` - Typed BM1366 observations.
- `crates/bitaxe-asic/src/bm1366/result.rs` - Result parsing and valid-job tracking.
- `crates/bitaxe-asic/src/bm1366/transcript.rs` - Fake UART transcript seam.
- `crates/bitaxe-asic/src/bm1366/work.rs` - Work payload and diagnostic job modeling.
- `crates/bitaxe-stratum/src/v1/fake_pool.rs` - Deterministic fake-pool harness.
- `crates/bitaxe-stratum/src/v1/mining.rs` - Mining job construction and share decision logic.
- `crates/bitaxe-stratum/src/v1/mining_loop.rs` - First mining-loop state machine.
- `crates/bitaxe-stratum/src/v1/queue.rs` - Work queue behavior.
- `crates/bitaxe-stratum/src/v1/state.rs` - Pool lifecycle, counters, and runtime state.
- `crates/bitaxe-api/src/mining.rs` - API-visible mining status model.
- `crates/bitaxe-api/src/asic.rs` - API-visible ASIC status/settings model.
- `crates/bitaxe-safety/src/evidence.rs` - Safety evidence token model.
- `firmware/bitaxe/src/asic_adapter.rs` - Firmware interpreter for typed ASIC actions.
- `firmware/bitaxe/src/asic_adapter/status.rs` - ASIC and mining status logging.
- `firmware/bitaxe/src/asic_adapter/uart.rs` - ESP-IDF UART adapter.
- `firmware/bitaxe/src/safety_adapter.rs` - Firmware safety facade and preflight inputs.
- `firmware/bitaxe/src/runtime_snapshot.rs` - Runtime status and telemetry integration point.
- `tools/parity/src/main.rs` - Checklist validation and evidence-token guard.

### Upstream Reference And Policy

- `reference/esp-miner/config-205.cvs` - Ultra 205 BM1366 defaults and pool-related config.
- `reference/esp-miner/main/device_config.h` - Ultra 205 board profile, capabilities, and expected ASIC count.
- `reference/esp-miner/components/asic/asic.c` - Shared ASIC dispatch behavior.
- `reference/esp-miner/components/asic/bm1366.c` - BM1366 initialization, packet, work, result, and nonce behavior.
- `reference/esp-miner/components/asic/asic_common.c` - Chip counting, receive-work validation, difficulty mask, and shared result behavior.
- `reference/esp-miner/main/power/asic_init.c` - Reset, UART, chip-detect, and max-baud initialization shell.
- `reference/esp-miner/main/power/asic_reset.c` - ASIC reset GPIO timing behavior.
- `reference/esp-miner/components/stratum/stratum_api.c` - Stratum v1 message handling.
- `reference/esp-miner/components/stratum/stratum_socket.c` - Pool socket lifecycle, reconnect, and fallback behavior.
- `reference/esp-miner/components/stratum/mining.c` - Coinbase hashing, merkle root, BM job construction, extranonce, and difficulty behavior.
- `reference/esp-miner/main/work_queue.c` - Upstream work queue behavior.
- `reference/esp-miner/main/tasks/protocol_coordinator.c` - Protocol lifecycle coordination.
- `reference/esp-miner/main/system.c` - Accepted/rejected share counters and clean-jobs handling.
- `reference/esp-miner/main/global_state.h` - Pool, share, queue, and mining status fields.
- `docs/adr/0001-device-user-parity.md` - Observable device-user parity definition.
- `docs/adr/0003-esp-idf-rust-production-stack.md` - ESP-IDF Rust stack and adapter boundary.
- `docs/adr/0005-read-only-reference-implementation.md` - Read-only upstream reference policy.
- `docs/adr/0006-parity-checklist-as-audit-evidence.md` - Checklist evidence policy.
- `docs/adr/0012-parity-verification-evidence.md` - Evidence requirements and hardware-control verification gate.
- `docs/adr/0013-mit-first-with-gpl-guardrails.md` - GPL guardrails for upstream-derived behavior and fixtures.
- `docs/adr/0014-pivot-to-ultra-205-bm1366-first-parity.md` - Ultra 205/BM1366 first parity target and deferred non-205 scope.
- `PROVENANCE.md` - Reference, GPL, fixture, and release artifact provenance policy.

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets

- `just detect-ultra205` already implements the required board-info preflight gate and prints the selected `port=<path>` only after success.
- `just flash-monitor board=205 port=<port> evidence-dir=<path>` already writes wrapper-owned evidence JSON and serial logs with firmware/source identity markers.
- `crates/bitaxe-asic` already owns typed BM1366 protocol, init, work, result, transcript, and adapter-gate logic from Phase 3.
- `crates/bitaxe-stratum` already owns Stratum v1 fake-pool, mining job, work queue, mining loop, and runtime state logic from Phase 4.
- `crates/bitaxe-safety` and `firmware/bitaxe/src/safety_adapter.rs` provide the safety evidence/preflight concepts Phase 12 must not bypass.
- `tools/parity` already rejects safety-critical verified claims without appropriate evidence tokens.

### Established Patterns

- Pure ASIC, Stratum, safety, and parity decisions live in host-testable Rust crates; ESP-IDF UART, GPIO, timing, serial monitoring, network, and task effects stay in firmware or tool adapters.
- Hardware evidence records use conservative, scoped conclusions and name board, port, commit, reference commit, package identity, command, logs, observed behavior, and redaction result.
- Checklist status changes follow evidence, not implementation enthusiasm. Broad mixed claims stay below `verified` unless evidence covers the exact hardware behavior.
- GSD artifacts and evidence Markdown avoid standalone body `---` separators after YAML frontmatter.

### Integration Points

- Add Phase 12 evidence under `docs/parity/evidence/phase-12-asic-and-mining-hardware-evidence.md` plus component-scoped generated artifacts if probe tooling is introduced.
- Add or extend a repo-owned Phase 12 probe path only when existing package/flash-monitor/monitor commands cannot capture the needed BM1366 or mining observation.
- Update `docs/parity/checklist.md` only for rows whose exact claim is supported by the new evidence, and keep residual mining or soak gaps explicit.
- Extend `tools/parity` tests only for new machine-checkable evidence semantics such as soak artifact requirements, redaction review presence, or ASIC/mining row promotion rules.

</code_context>

<specifics>
## Specific Ideas

- Use a tiered evidence ladder: detector/package/safe boot, chip-detect, staged init, diagnostic work/result, controlled mining smoke, bounded soak.
- Prefer diagnostic typed BM1366 work before real-pool work when it can prove work-send/result-receive without exposing credentials or running unbounded mining.
- Treat controlled no-share evidence as acceptable only when the plan explains why accepted/rejected shares are not expected and still proves pool lifecycle, job dispatch, telemetry, watchdog responsiveness, and safe-stop behavior.
- Keep mining soak bounded, redacted, and reversible. Stop conditions should include temperature/power/watchdog concerns, missing telemetry, reconnect loops, serial silence, or any safety gate fault.
- If a live board, pool condition, or recovery prerequisite is unavailable, produce a useful pending evidence artifact and checklist note rather than widening the scope or overclaiming.

</specifics>

<deferred>
## Deferred Ideas

- Final Ultra 205 package-to-hardware release evidence, live HTTP/static/recovery/OTA/rollback/erase/interrupted-update proof, and `DEVICE_URL` release evidence belong to Phase 13.
- Non-205 boards, BM1370/BM1368/BM1397, TPS546 hardware behavior, all-board factory image matrices, Stratum v2, BAP, and Angular UI replacement remain deferred.
- Long-term mining performance tuning, production pool optimization, and unbounded stress testing are outside Phase 12 unless a later roadmap phase defines recovery, safety, and evidence requirements.

</deferred>

*Phase: 12-asic-and-mining-hardware-evidence*
*Context gathered: 2026-06-29*
