---
generated_by: gsd-discuss-phase
lifecycle_mode: yolo
phase_lifecycle_id: 21-2026-07-04T01-35-47
generated_at: 2026-07-04T01:37:38.709Z
---

# Phase 21: Live Mining And Soak Evidence - Context

**Gathered:** 2026-07-04
**Status:** Ready for planning
**Mode:** Yolo

<domain>
## Phase Boundary

Phase 21 closes the `live-production-mining-soak` evidence gap for Ultra 205 board `205`: live production mining or explicitly bounded no-share mining behavior, BM1366 work/result behavior under the mining path, accepted/rejected share handling, pool lifecycle, live API/WebSocket telemetry correlation, watchdog responsiveness, and bounded soak evidence. The phase does not add non-205 board verification, Stratum v2 scope, BAP scope, release/OTA recovery claims, active voltage/fan/fault stimulus claims, or secret-bearing pool configuration storage.

</domain>

<decisions>
## Implementation Decisions

### Controlled Pool And Target Gates

- **D-01:** Begin every hardware-capable path with `just detect-ultra205`; continue only when exactly one likely ESP32-S3 port is detected and board-info passes for board `205`.
- **D-02:** Live-pool smoke is allowed only with disposable or non-secret pool configuration, no committed pool credentials, an explicit user/operator-supplied `DEVICE_URL`, safe-stop/recovery instructions, redaction review, and an allow manifest that binds board, port, package identity, source commit, reference commit, exact command, abort conditions, and post-action safe-state markers.
- **D-03:** Do not infer `DEVICE_URL` from serial logs, network scans, router state, mDNS, ARP, or redacted evidence. If no explicit target is available, record a blocked or controlled no-share artifact instead of running live HTTP/WebSocket correlation.
- **D-04:** Pool credentials, worker secrets, private endpoints, Wi-Fi credentials, API tokens, NVS secret values, and private `DEVICE_URL` values must not be read into chat output, committed, or summarized in evidence. Redacted logs may keep non-secret category labels, board `205`, port, source/reference commits, package paths, and conclusions.

### Mining Evidence Ladder

- **D-05:** Use the existing Phase 15 ladder as the starting structure: detector/package/safe boot, package-backed chip-detect or staged init, typed work/result evidence, live-pool mining smoke, bounded soak, then exact checklist promotion.
- **D-06:** A later tier must not run when an earlier required tier is missing, failed, stale, redaction-blocked, or lacks trusted wrapper/package markers. Write useful pending evidence with the exact blocker instead of bypassing the ladder.
- **D-07:** Keep `tools/parity` mining allow-manifest validation in the path. Phase 21 may extend the validator beyond Phase 15 command shapes when needed, but the extension must preserve board `205`, detector, board-info, package identity, prohibited command, required abort condition, safe-state marker, live-pool, and bounded-soak checks.
- **D-08:** Production mining evidence must not rely on raw BM1366 writes, raw pool commands, ad hoc voltage/fan controls, erase commands, rollback commands, interrupted-update commands, unbounded stress, or hidden local scripts.

### Live Mining Smoke

- **D-09:** Prefer a short live-pool micro-smoke after detector, safety, chip-detect/work-result, explicit-target, safe-stop, and redaction gates pass. The smoke should record pool connection lifecycle, subscribe/authorize behavior, notify/job flow, BM1366 work dispatch, result handling, accepted/rejected share behavior when observed, hashrate inputs, API/WebSocket status, watchdog breadcrumbs, and final safe-stop.
- **D-10:** If a live-pool run produces no shares within the bounded window, it may support only an explicit bounded no-share conclusion when the artifact proves the pool lifecycle, job/work path, duration, abort conditions, telemetry/watchdog checks, safe-stop, and redaction status. It must not be presented as accepted-share proof.
- **D-11:** Accepted share and rejected share evidence should be treated as exact observed outcomes. Do not synthesize rejected-share claims unless the controlled setup safely and explicitly produces a redaction-reviewed rejection path.
- **D-12:** If a controlled fake-pool or local harness is used, label it as controlled evidence. It can support flow, work, telemetry, watchdog, and no-share boundaries, but it cannot by itself prove live production pool behavior.

### Bounded Soak

- **D-13:** Bounded soak may run only after live smoke passes or the plan explicitly justifies an approved controlled no-share soak. The allowed duration should stay bounded and operator-readable, with `tools/parity` currently enforcing `duration_seconds` from 60 to 600 for a `bounded-soak` claim tier.
- **D-14:** Soak evidence must record duration, abort conditions, thermal/power/watchdog observations, pool lifecycle, share outcomes or bounded no-share status, periodic API/WebSocket snapshots when a target is explicit, final safe-stop/restore markers, and conclusion.
- **D-15:** Watchdog responsiveness must be proven through bounded observations during mining or soak, not merely by startup supervisor/yield breadcrumbs. Missing bounded load or watchdog recovery proof remains below verified.
- **D-16:** Any unexpected reboot, watchdog panic, unsafe temperature/power marker, detector mismatch, missing trusted wrapper marker, redaction uncertainty, lost pool control, or missing safe-state marker is a stop condition and should produce blocked evidence plus recovery notes.

### Telemetry, API, And Statistics Correlation

- **D-17:** Live API and WebSocket telemetry correlation requires an explicit `DEVICE_URL`, bounded `/api/system/info` and `/api/ws/live` captures, redaction review, and correlation to serial or runtime observations from the same run.
- **D-18:** Statistics, scoreboard, accepted/rejected counters, pool difficulty, hashrate inputs, mining activity, and work-submission state should be checked across the runtime state, API response, WebSocket frame, and evidence summary when those surfaces are available.
- **D-19:** Route presence, a no-upgrade WebSocket response, stale cached API bodies, startup-only logs, or Phase 15/20 blocked-target artifacts are supporting breadcrumbs only. They do not prove Phase 21 live telemetry freshness, cadence, or mining statistics behavior.

### Checklist, Redaction, And Verification

- **D-20:** Use exact-claim checklist promotion. `ASIC-002` through `ASIC-005`, `STR-006`, `STR-007`, `STR-008`, statistics/API rows, and `EVD-05` may move only to the evidence level supported by the final artifact.
- **D-21:** `ASIC-007` remains below verified unless Phase 21 intentionally includes a bounded frequency-transition hardware-regression artifact. Live mining or soak evidence alone must not accidentally verify frequency transition behavior.
- **D-22:** `STR-008` verified status requires mining-smoke or soak details with board, port, firmware/source commit, reference commit, redaction, conclusion, and either accepted/rejected share outcome or an approved bounded controlled no-share soak without blocker language.
- **D-23:** Final phase verification must include targeted checks for changed scripts/tools/Rust code, `just test`, `just parity`, `just verify-reference`, reference cleanliness, redaction review, lifecycle validation, and every detector/hardware/network command actually used. No wrapper-level commit/push should happen unless `21-VERIFICATION.md` reports `status: passed` and lifecycle validation succeeds for `21-2026-07-04T01-35-47`.

### the agent's Discretion

The agent may choose the exact plan count, evidence pack names, helper filenames, whether to extend Phase 15 scripts or introduce Phase 21-named wrappers, JSON field names, WebSocket capture helper reuse, and exact checklist note wording. Those choices must preserve ESP-IDF/esp-rs tooling preference, functional-core/imperative-shell structure, read-only reference policy, secret redaction, detector gates, package identity, safe-stop markers, and conservative evidence semantics.

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Phase Scope And Governance

- `.planning/ROADMAP.md` - Phase 21 goal, dependency on Phase 20, requirements, success criteria, verification expectations, and controlled pool/recovery research flag.
- `.planning/REQUIREMENTS.md` - ASIC-07, STR-06, STR-07, SAFE-09, EVD-05, and evidence-sensitive traceability rows involving Phase 21.
- `.planning/PROJECT.md` - Ultra 205 BM1366 first target, ESP-IDF Rust stack, read-only reference, parity evidence policy, architecture, licensing, and safety constraints.
- `.planning/STATE.md` - Current project state after Phase 20 and accumulated decisions affecting mining, safety, telemetry, and evidence.
- `AGENTS.md` - Repo-local Ultra 205 hardware permission, detector gate, evidence metadata, redaction rules, destructive/fault-injection limits, and frontmatter separator rule.
- `AGENTS.bright-builds.md` - Bright Builds workflow, verification, task artifact, and standards routing requirements.
- `standards/core/architecture.md` - Functional core / imperative shell and boundary parsing expectations.
- `standards/core/code-shape.md` - Early-return, script rerun-safety, and module-size expectations.
- `standards/core/verification.md` - Sync and repo-native verification before commit.
- `standards/core/testing.md` - Unit-test expectations for changed pure logic.
- `standards/languages/rust.md` - Rust module, naming, invariant, test, and verification guidance.

### Prior Phase Decisions And Evidence

- `.planning/phases/03-bm1366-asic-protocol-and-safe-initialization/03-CONTEXT.md` - BM1366 typed command/observation boundary, diagnostic work boundary, and hardware verification limits.
- `.planning/phases/04-stratum-v1-and-first-mining-loop/04-CONTEXT.md` - Stratum v1, fake-pool, mining work, share, queue, runtime state, and first-loop evidence boundaries.
- `.planning/phases/12-asic-and-mining-hardware-evidence/12-CONTEXT.md` - Prior ASIC/mining evidence ladder and trusted-wrapper failure mode.
- `.planning/phases/15-bm1366-mining-evidence-completion/15-CONTEXT.md` - Phase 15 decisions for chip-detect, work/result diagnostics, controlled mining smoke, bounded soak, parity guards, and redaction.
- `.planning/phases/15-bm1366-mining-evidence-completion/15-VERIFICATION.md` - Phase 15 lifecycle and verification outcome.
- `.planning/phases/17-live-http-api-and-static-evidence/17-CONTEXT.md` - Explicit `DEVICE_URL` gate and live HTTP/WebSocket capture rules.
- `.planning/phases/20-active-safety-hardware-telemetry-evidence/20-CONTEXT.md` - Current safety gates, live telemetry blockers, redaction policy, and final verification expectations.
- `.planning/phases/20-active-safety-hardware-telemetry-evidence/20-VERIFICATION.md` - Phase 20 lifecycle and verification outcome.
- `docs/parity/evidence/phase-15-bm1366-mining-evidence-completion.md` - Final Phase 15 ledger, exact claim matrix, controlled no-share boundary, missing live prerequisites, and remaining below-verified mining claims.
- `docs/parity/evidence/phase-15-bm1366-mining-evidence-completion/redaction-review.md` - Phase 15 redaction review pattern for mining artifacts.
- `docs/parity/evidence/phase-20-active-safety-hardware-telemetry-evidence/summary.md` - Final Phase 20 safe-baseline, active safety boundaries, live telemetry blocked-boundary, and citation rules.
- `docs/parity/evidence/phase-20-active-safety-hardware-telemetry-evidence/redaction-review.md` - Phase 20 redaction review pattern for safety and telemetry artifacts.

### Current Tooling And Implementation Surfaces

- `Justfile` - Human command surface for `detect-ultra205`, `package`, `flash-monitor`, `monitor`, `parity`, `verify-reference`, `build`, and `test`.
- `scripts/detect-ultra205.sh` - Required detector and board-info preflight before autonomous hardware use.
- `scripts/phase15-bm1366-diagnostic-package.sh` - Existing package-backed diagnostic firmware builder for chip-detect and work-result evidence.
- `scripts/phase15-controlled-mining.sh` - Existing controlled mining smoke and bounded-soak wrapper, redaction filter, allowed-command generation, `DEVICE_URL` handling, and prerequisite checks.
- `scripts/phase15-websocket-capture.mjs` - Existing bounded WebSocket helper for mining evidence when an explicit target is available.
- `scripts/BUILD.bazel` - Existing Bazel test registration for phase helper scripts.
- `tools/parity/src/mining_allow.rs` - Mining allow-manifest validator, live-pool and bounded-soak gate rules, allowed surfaces, claim tiers, prohibited command tokens, and required safe-state markers.
- `tools/parity/src/main.rs` - Checklist validation, live ASIC/mining verified-row guards, blocker-language checks, and parity report command.
- `tools/flash/src/main.rs` - Flash/monitor evidence capture, package manifest lookup, trusted output classification, redaction-sensitive field handling, and JSON/log artifact behavior.
- `crates/bitaxe-asic/src/bm1366.rs` - BM1366 module entrypoint.
- `crates/bitaxe-asic/src/bm1366/adapter_gate.rs` - Hardware evidence acknowledgement and diagnostic gate model.
- `crates/bitaxe-asic/src/bm1366/chip_detect.rs` - Pure chip-detect validation and follow-up actions.
- `crates/bitaxe-asic/src/bm1366/command.rs` - Typed BM1366 command boundary.
- `crates/bitaxe-asic/src/bm1366/init_plan.rs` - Staged initialization plan and no-mining states.
- `crates/bitaxe-asic/src/bm1366/observation.rs` - Typed BM1366 observations.
- `crates/bitaxe-asic/src/bm1366/result.rs` - Result parsing and valid-job tracking.
- `crates/bitaxe-asic/src/bm1366/work.rs` - Work payload and diagnostic job modeling.
- `crates/bitaxe-stratum/src/v1/fake_pool.rs` - Deterministic fake-pool harness.
- `crates/bitaxe-stratum/src/v1/messages.rs` - Stratum v1 subscribe, authorize, notify, set-difficulty, submit, and response parsing/serialization.
- `crates/bitaxe-stratum/src/v1/mining.rs` - Mining job construction and share submission mapping.
- `crates/bitaxe-stratum/src/v1/mining_loop.rs` - Guarded first mining-loop state machine and safety evidence gates.
- `crates/bitaxe-stratum/src/v1/queue.rs` - Work queue and active work tracking.
- `crates/bitaxe-stratum/src/v1/state.rs` - Pool lifecycle, share counters, pool difficulty, mining activity, and runtime state.
- `crates/bitaxe-api/src/mining.rs` - API-visible mining status model.
- `crates/bitaxe-api/src/telemetry.rs` - Live telemetry envelope, diff, and cadence behavior.
- `firmware/bitaxe/src/asic_adapter.rs` - Firmware interpreter for typed BM1366 actions.
- `firmware/bitaxe/src/asic_adapter/status.rs` - ASIC and mining status logging.
- `firmware/bitaxe/src/http_api.rs` - ESP-IDF HTTP and WebSocket route shell for live API/WebSocket probes when an explicit target is available.
- `firmware/bitaxe/src/runtime_snapshot.rs` - Runtime status and telemetry integration point.

### Upstream Reference And Policy

- `reference/esp-miner/config-205.cvs` - Ultra 205 BM1366 defaults and pool-related configuration.
- `reference/esp-miner/main/device_config.h` - Ultra 205 board profile, capabilities, and expected ASIC count.
- `reference/esp-miner/components/asic/asic.c` - Shared ASIC dispatch behavior.
- `reference/esp-miner/components/asic/bm1366.c` - BM1366 initialization, packet, work, result, nonce, and frequency behavior.
- `reference/esp-miner/components/asic/asic_common.c` - Chip counting, receive-work validation, difficulty mask, and shared result behavior.
- `reference/esp-miner/main/power/asic_init.c` - Reset, UART, chip-detect, and max-baud initialization shell.
- `reference/esp-miner/main/power/asic_reset.c` - ASIC reset GPIO timing behavior.
- `reference/esp-miner/components/stratum/stratum_api.c` - Stratum v1 message handling.
- `reference/esp-miner/components/stratum/stratum_socket.c` - Pool socket lifecycle, reconnect, and fallback behavior.
- `reference/esp-miner/components/stratum/mining.c` - Coinbase hashing, merkle root, BM job construction, extranonce, difficulty, and submit behavior.
- `reference/esp-miner/main/work_queue.c` - Upstream work queue behavior.
- `reference/esp-miner/main/tasks/protocol_coordinator.c` - Protocol lifecycle coordination and watchdog-sensitive task behavior.
- `reference/esp-miner/main/system.c` - Accepted/rejected share counters and clean-jobs handling.
- `reference/esp-miner/main/global_state.h` - Pool, share, queue, mining, and runtime status fields.
- `reference/esp-miner/main/http_server/system_api_json.c` - User-visible mining/statistics/safety telemetry fields.
- `reference/esp-miner/main/http_server/websocket_api.c` - Live telemetry/WebSocket reference behavior.
- `docs/parity/checklist.md` - Current parity rows and evidence semantics for ASIC, Stratum, API, statistics, safety, and EVD claims.
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

- `just detect-ultra205` already implements the exact autonomous hardware gate required by repo-local guidance.
- `tools/parity/src/mining_allow.rs` already recognizes `mining-smoke`, `bounded-soak`, `live-pool-smoke`, `controlled-no-share`, and `unsupported-pending`, including disposable pool and explicit device target checks for live-pool smoke.
- `scripts/phase15-controlled-mining.sh` already centralizes controlled mining/smoke redaction, allowed command construction, prerequisite summary checks, optional `--device-url`, duration handling, and WebSocket helper invocation.
- `scripts/phase15-bm1366-diagnostic-package.sh` already builds package-backed chip-detect and work-result diagnostic images.
- `tools/parity/src/main.rs` already rejects live ASIC/mining `verified` rows with blocker language and enforces STR-008 details for share outcomes or approved bounded controlled no-share soak.
- `crates/bitaxe-stratum` already contains pure share, queue, runtime-state, fake-pool, and mining-loop logic that should stay under unit tests when changed.
- Firmware HTTP/WebSocket and runtime snapshot code already expose the integration points for live telemetry correlation when an explicit device target exists.

### Established Patterns

- Pure protocol, ASIC, safety, API, and parity decisions stay in Rust crates/tools with focused unit tests; ESP-IDF UART, GPIO, HTTP/WebSocket, pool I/O, timing, serial capture, and hardware effects stay in firmware/tool/script adapters.
- Hardware evidence names board `205`, selected port, source commit, reference commit, package manifest or firmware identity, exact commands, board-info output, logs, observed behavior, conclusion, and redaction status.
- Checklist rows cite exact artifacts and remain below `verified` when artifacts prove only narrow diagnostic, blocked, startup-only, stale, or no-share boundaries.
- Evidence Markdown and GSD artifacts must avoid standalone body `---` separators after YAML frontmatter.

### Integration Points

- Add Phase 21 evidence under `docs/parity/evidence/phase-21-live-mining-and-soak-evidence/` with summary, live-smoke, bounded-soak, telemetry, redaction, and verification artifacts as applicable.
- Extend or wrap Phase 15 mining helpers if Phase 21 needs phase-specific command shapes, but keep allow-manifest validation in `tools/parity`.
- Add tests for any changed Rust logic, shell helper behavior, Node/WebSocket helper behavior, and parity guard semantics.
- Update `docs/parity/checklist.md`, `.planning/REQUIREMENTS.md`, and phase verification only after evidence artifacts exist and redaction passes.

</code_context>

<specifics>
## Specific Ideas

- Preferred phase packs: `preflight`, `live-mining-smoke`, `bounded-soak`, `live-api-websocket-telemetry`, `redaction-review`, and `final-summary`.
- Preferred live smoke: short, explicit-target, disposable/non-secret pool, detector-gated, package-identified, watchdog-observed, telemetry-correlated, and safe-stopped.
- Preferred fallback: if live prerequisites are missing, write blocked or controlled no-share artifacts with exact blocker, owner/follow-up, non-claims, and checklist rows affected.
- Preferred promotion: exact claims only; accepted/rejected share evidence should promote only rows proved by the run, and broad live soak/statistics/API/WebSocket claims remain below verified without matching artifacts.
- Preferred final gate: `just test`, `just parity`, `just verify-reference`, redaction review, reference cleanliness, hardware command evidence, and lifecycle validation must all support `status: passed`.

</specifics>

<deferred>
## Deferred Ideas

- Non-205 board mining or soak evidence remains future board-specific work.
- Stratum v2, BAP accessory behavior, all-board release matrix, and Angular AxeOS rewrite remain out of Phase 21.
- Active voltage/fan/fault/self-test/load hardware regression remains outside Phase 21 unless a plan explicitly adds a prerequisite-only safety artifact without claiming mining proof.
- OTA, OTAWWW, rollback, erase, failed-update recovery, interrupted-update, and release-recovery flows remain outside Phase 21.

</deferred>

*Phase: 21-live-mining-and-soak-evidence*
*Context gathered: 2026-07-04*
