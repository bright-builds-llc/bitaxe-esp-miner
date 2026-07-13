---
generated_by: gsd-discuss-phase
lifecycle_mode: yolo
phase_lifecycle_id: 26-2026-07-05T03-48-38
generated_at: 2026-07-05T03:48:38.522Z
---

# Phase 26: Telemetry And Parity Closure - Context

**Gathered:** 2026-07-05
**Status:** Ready for planning
**Mode:** Yolo

<domain>
## Phase Boundary

Phase 26 closes the v1.1 telemetry and parity projection surface. API, WebSocket, statistics, scoreboard, and checklist updates must derive from the same v1.1 runtime events and redacted Ultra 205 artifacts, so operators can see mining state, counters, hashrate inputs, share outcomes, and post-stop state without stale active-mining values or unsupported claim promotion.

This phase owns `/api/system/info`, `/api/system/statistics`, `/api/system/scoreboard`, `/api/ws`, `/api/ws/live`, statistics and scoreboard invariants, redacted session-correlated mining telemetry, and final v1.1 parity checklist closure. It does not own new BM1366 production work behavior, new Stratum socket behavior, accepted/rejected share proof when the underlying evidence is blocked, full active voltage/fan/thermal/fault/self-test closure, OTA/recovery, non-205 boards, other ASIC families, Stratum v2, runtime display/input, BAP, or unbounded stress mining.

</domain>

<decisions>
## Implementation Decisions

### Runtime Event Source And Counter Invariants

- **D-01:** Use a shared typed v1.1 runtime-event projection as the source of truth for API, WebSocket, statistics, scoreboard, and parity-visible mining state. Endpoint handlers should consume derived views rather than each inventing local counter or activity rules.
- **D-02:** Accepted and rejected share counters may advance only from a live ASIC-derived `SubmitIntent` plus matching parsed pool response for the current pool-session generation. Fake-pool-only responses, placeholder data, stale generations, missing submit intent, blocked prerequisites, stopped sockets, or implementation-only tests must not advance production-visible share counters.
- **D-03:** Hashrate inputs, mining activity, pool lifecycle, fallback status, blocked reasons, and post-stop state should be folded from the same typed runtime events or explicitly safe snapshots. Existing direct mutation helpers in `MiningRuntimeState` must be wrapped, constrained, or tested so Phase 26 call paths cannot bypass the event/projection gate.
- **D-04:** The projection should carry enough generation, session, or sequence metadata to prove stale events are ignored and safe-stop events reset active mining, work-submission, active-work, and lifecycle state before API or WebSocket serialization.

### API Projection Shape

- **D-05:** Keep public responses AxeOS-compatible. Preserve established wire fields and upstream-compatible shapes in `crates/bitaxe-api`, while deriving their values from the shared Phase 26 projection.
- **D-06:** `/api/system/info` should continue to use `ApiSnapshot` and `SystemInfoWire`, but its mining fields must reflect the Phase 26 projection rather than ad hoc controlled-evidence state replacement.
- **D-07:** `/api/system/statistics` should stop being only an empty placeholder when runtime-event evidence can support a bounded sample. Any emitted sample must name or encode that it came from the runtime projection; no historical series may be fabricated.
- **D-08:** `/api/system/scoreboard` should remain an upstream-shaped array. Populate it only from runtime events tied to parsed pool responses or active share outcomes. If no such events exist, return the compatible empty array and preserve the non-claim.
- **D-09:** Richer evidence semantics that do not fit upstream API fields should stay in internal projection types, tests, or evidence artifacts rather than adding non-upstream public API keys without a deliberate compatibility decision.

### WebSocket Session Correlation And Redaction

- **D-10:** `/api/ws/live` must serialize the same redacted projection used by the HTTP API, preserving the existing full-on-connect and 500 ms diff cadence semantics.
- **D-11:** `/api/ws` should remain compatible with the raw retained-log stream route. If Phase 26 needs mining lifecycle markers there, emit redaction-safe log markers from the shared projection instead of introducing raw pool, share, target, extranonce, device, Wi-Fi, NVS, or ASIC-frame values.
- **D-12:** Redaction must happen before telemetry fan-out. The projection and logs must never expose pool URLs, ports, workers, owner addresses, passwords, targets, extranonces, share payloads, socket errors, device URLs, IP addresses, MAC addresses, Wi-Fi values, tokens, NVS secrets, or raw BM1366 frames in committed evidence.
- **D-13:** WebSocket tests and evidence should prove no stale active-mining frame is emitted after safe stop, including connect-time full updates and cadence diffs after the safe-stop event has updated the projection.

### Parity Checklist And Evidence Promotion

- **D-14:** Promote checklist rows only to the exact level proven by Phase 26 artifacts. Code, unit tests, fake-pool fixtures, workflow evidence, and detector-gated hardware evidence each support different claim levels and must not be collapsed into a single "verified" claim.
- **D-15:** Add or update the Phase 26 evidence artifacts to map API-11, API-12, API-13, and EVD-08 to exact source artifacts. A checklist-first exact delta is enough for a narrow closure; add a machine-readable promotion manifest only if the planner finds reusable v1.1 closure validation is needed.
- **D-16:** Preserve explicit non-claims for accepted/rejected live share proof when Phase 25 evidence remains blocked, plus full active safety closure, OTA/recovery, non-205 boards, other ASIC families, Stratum v2, display/input, BAP, and unbounded stress.
- **D-17:** `tools/parity` should reject overbroad verified telemetry/statistics/scoreboard claims with pending/blocker language, missing Phase 26 artifacts, or absent redaction review.

### Verification Gate

- **D-18:** Unit tests must prove one concern at a time: event folding, counter gating, stale-generation rejection, safe-stop reset, API projection, statistics projection, scoreboard projection, WebSocket full/diff projection, and redaction-safe rendering.
- **D-19:** Repo-native verification should include targeted Rust tests for changed crates, firmware compile or relevant Bazel targets when firmware files change, script/helper tests when touched, `just parity`, `just verify-reference`, and lifecycle validation.
- **D-20:** Hardware verification is allowed only through the Ultra 205 detector gate and repo-owned commands. If detector, package identity, safe prerequisites, local credentials, live socket, share outcome, API, or WebSocket evidence is blocked, record the blocker and keep the affected claim below verified.

### Claude's Discretion

Claude may choose exact module names, event enum names, projection struct names, fixture shapes, route helper names, evidence filenames, and whether the projection lives primarily in `crates/bitaxe-stratum`, `crates/bitaxe-api`, or a narrow bridge between them. Those choices must preserve functional core / imperative shell structure, AxeOS public compatibility, runtime-only secrets, redaction-safe artifacts, Ultra 205 detector gating, and exact parity semantics.

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Phase Scope And Requirements

- `.planning/ROADMAP.md` - Phase 26 goal, dependency on Phase 25, requirements, and success criteria.
- `.planning/REQUIREMENTS.md` - API-11, API-12, API-13, EVD-08, v1.1 active requirements, out-of-scope boundaries, and traceability.
- `.planning/PROJECT.md` - v1.1 Ultra 205 trusted production mining scope, ESP-IDF Rust stack, parity evidence policy, architecture, licensing, and safety constraints.
- `.planning/STATE.md` - Carried v1.1 decisions, Phase 25 closure status, live-share blockers, and current next-step state.
- `AGENTS.md` - Ultra 205 detector gate, local credential handling, `DEVICE_URL` derivation limits, redaction rules, hardware evidence requirements, phase-gated unsafe action limits, and frontmatter separator rule.
- `AGENTS.bright-builds.md` - Bright Builds workflow, standards routing, verification, and code-shape expectations.
- `standards/core/architecture.md` - Functional core / imperative shell and parse-boundary guidance.
- `standards/core/code-shape.md` - Early returns, optional naming, script rerun-safety, and module-size guidance.
- `standards/core/testing.md` - Unit-test structure and pure logic coverage expectations.
- `standards/core/verification.md` - Sync and repo-native verification before commit.
- `standards/languages/rust.md` - Rust module layout, `maybe_` naming, invariants, and testing expectations.

### Prior v1.1 Decisions And Evidence

- `.planning/phases/22-claim-ladder-and-safety-preconditions/22-CONTEXT.md` - Claim ladder, typed prerequisites, blocker reasons, evidence boundaries, and exact non-claim handling.
- `.planning/phases/23-redacted-operator-evidence-workflow/23-CONTEXT.md` - Redacted evidence-root slots, runtime-only credential handling, redaction contract, and exact non-claim governance.
- `.planning/phases/24-bm1366-production-work-path/24-CONTEXT.md` - Production BM1366 mode split, pool-derived work dispatch, result correlation, submit-intent boundary, and Phase 26 deferral.
- `.planning/phases/25-live-stratum-runtime-and-safe-stop/25-CONTEXT.md` - Live Stratum runtime boundary, submit response classification, safe-stop postconditions, watchdog proof, and Phase 26 deferral.
- `.planning/phases/05-axeos-api-logs-and-telemetry/05-CONTEXT.md` - AxeOS API, statistics, scoreboard, log, and WebSocket compatibility decisions.
- `.planning/phases/17-live-http-api-and-static-evidence/17-CONTEXT.md` - Live route, WebSocket capture, explicit target, redaction, and checklist promotion decisions.
- `docs/parity/evidence/phase-23-redacted-operator-evidence-workflow/evidence-contract.md` - Required evidence-root slots and promotion contract.
- `docs/parity/evidence/phase-25-live-stratum-runtime-and-safe-stop/summary.md` - Phase 25 exact closure status and live evidence non-claims.
- `docs/parity/evidence/phase-25-live-stratum-runtime-and-safe-stop/share-outcome.md` - Current share-outcome blocker/non-claim evidence.
- `docs/parity/evidence/phase-25-live-stratum-runtime-and-safe-stop/safe-stop.md` - Safe-stop postcondition evidence to project through API and WebSocket surfaces.
- `docs/parity/checklist.md` - Current API, WebSocket, statistics, scoreboard, Stratum, safety, and v1.1 evidence rows plus verified-row guardrails.

### Current Implementation Surfaces

- `crates/bitaxe-stratum/src/v1/state.rs` - `MiningRuntimeState`, `ShareCounters`, `HashrateInputs`, lifecycle, work-submission, mining-activity, and blocked-reason state.
- `crates/bitaxe-stratum/src/v1/production_work.rs` - `PoolSessionGeneration`, production active-work registry, nonce correlation, and redacted submit intent boundary.
- `crates/bitaxe-stratum/src/v1/submit_response.rs` - Submit response classification that must gate accepted/rejected share outcomes.
- `crates/bitaxe-stratum/src/v1/live_runtime.rs` - Pure live runtime lifecycle and events introduced in Phase 25.
- `crates/bitaxe-api/src/wire.rs` - `SystemInfoWire` mapping from `ApiSnapshot`.
- `crates/bitaxe-api/src/snapshot.rs` - API snapshot model used by firmware projection.
- `crates/bitaxe-api/src/statistics.rs` - Upstream-compatible statistics response shape.
- `crates/bitaxe-api/src/scoreboard.rs` - Upstream-compatible scoreboard entry mapping and empty-array fixture.
- `crates/bitaxe-api/src/telemetry.rs` - `/api/ws/live` update envelope, diff, and 500 ms cadence contract.
- `crates/bitaxe-api/src/websocket_state.rs` - Pure WebSocket session state for logs and live telemetry.
- `firmware/bitaxe/src/live_stratum_runtime.rs` - Firmware live socket shell, submit classification usage, safe-stop publication, and runtime snapshot update.
- `firmware/bitaxe/src/runtime_snapshot.rs` - Firmware API snapshot collection and current command-visible mining state boundary.
- `firmware/bitaxe/src/http_api.rs` - HTTP route handlers; current statistics and scoreboard handlers still use placeholder empty responses.
- `firmware/bitaxe/src/websocket_api.rs` - Firmware WebSocket state bridge.
- `scripts/phase25-live-stratum-evidence.sh` - Latest live Stratum evidence wrapper and blocked-outcome behavior.
- `tools/parity/src/main.rs` - Checklist validation and verified-row guardrails.
- `tools/parity/src/mining_allow.rs` - Mining evidence allow-manifest and redaction/safe-state guardrails.

### Upstream Reference And Policy

- `reference/esp-miner/main/http_server/system_api_json.c` - Full-state API fields and system info behavior.
- `reference/esp-miner/main/http_server/websocket_api.c` - `/api/ws/live` update and cadence semantics.
- `reference/esp-miner/main/http_server/websocket_log.c` - `/api/ws` raw log stream semantics.
- `reference/esp-miner/main/tasks/statistics_task.h` - Statistics source labels and row shape.
- `reference/esp-miner/main/tasks/hashrate_monitor_task.c` - Hashrate monitor values surfaced in API responses.
- `reference/esp-miner/main/tasks/scoreboard.h` - Scoreboard task surface.
- `reference/esp-miner/main/system.c` - Accepted/rejected share counters and clean-jobs handling.
- `reference/esp-miner/main/global_state.h` - Pool, share, queue, mining, and runtime status fields.
- `docs/adr/0001-device-user-parity.md` - Observable device-user parity scope.
- `docs/adr/0003-esp-idf-rust-production-stack.md` - ESP-IDF Rust stack and adapter boundary.
- `docs/adr/0005-read-only-reference-implementation.md` - Read-only upstream reference policy.
- `docs/adr/0006-parity-checklist-as-audit-evidence.md` - Checklist evidence policy.
- `docs/adr/0010-axeos-api-and-asset-compatibility.md` - API and static asset compatibility before UI rewrite.
- `docs/adr/0012-parity-verification-evidence.md` - Evidence class and verified status semantics.
- `docs/adr/0013-mit-first-with-gpl-guardrails.md` - GPL provenance guardrails.
- `docs/adr/0014-pivot-to-ultra-205-bm1366-first-parity.md` - Ultra 205 BM1366 first parity target and deferred non-205 scope.
- `PROVENANCE.md` - Reference, GPL, fixture/source-attribution, dependency-license, and release-review policy.

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets

- `MiningRuntimeState` already carries lifecycle, share counters, pool difficulty, fallback, work-submission gate, hashrate inputs, mining activity, and optional blocker reason.
- `ProductionWorkRegistry` already binds production work to `PoolSessionGeneration`, invalidates on clean-jobs/reconnect, redacts debug output, and creates `SubmitIntent` only after nonce/result correlation.
- Phase 25 live runtime and submit-response modules already distinguish accepted, rejected, blocked, timeout, malformed, and no-response outcomes without making fake-pool-only results into live claims.
- `ApiSnapshot`, `SystemInfoWire`, statistics, scoreboard, telemetry, and WebSocket state already keep AxeOS wire DTOs and pure mapping logic in `crates/bitaxe-api`.
- `runtime_snapshot.rs` already overlays command-visible mining state, platform facts, safety telemetry, Wi-Fi, and settings into the API snapshot used by HTTP and WebSocket routes.
- `http_api.rs` already registers Phase 26-owned routes, but `/api/system/statistics` currently sends `empty_statistics_response(...)` and `/api/system/scoreboard` currently sends `scoreboard_response(&[])`.
- `LiveTelemetryPlanner` already sends a full update on connect and diff-only cadence frames at `LIVE_TELEMETRY_CADENCE_MS`.
- `docs/parity/checklist.md` already preserves Phase 26 non-claims for API/WebSocket/statistics/scoreboard promotion and records Phase 25 share outcome as `blocked_safe_prerequisite`.

### Established Patterns

- Pure protocol, ASIC, safety, API wire, evidence, and parity decisions live in Rust crates or tools with focused unit tests.
- ESP-IDF sockets, HTTP serving, WebSocket session I/O, serial capture, local credentials, NVS, filesystem, and hardware effects remain firmware/tool/script shells.
- Public API fields stay upstream-compatible; exact evidence semantics live in typed internals, parity tools, and evidence artifacts when upstream fields cannot safely carry them.
- Hardware evidence names board `205`, selected port, source commit, reference commit, package or firmware identity, exact commands, board-info output, captured logs, observed behavior, redaction status, safe-state markers, and conclusion.
- Checklist rows cite exact artifacts and remain below `verified` when artifacts prove only fake-pool, blocked, implementation-only, no-share, stale, startup-only, or workflow-only behavior.
- Frontmatter-parsed Markdown must not use standalone body `---` separators after YAML frontmatter.

### Integration Points

- Add Phase 26 planning artifacts under `.planning/phases/26-telemetry-and-parity-closure/`.
- Prefer a small pure projection module or bridge that folds typed runtime events into API/statistics/scoreboard/WebSocket views and proves counter invariants in tests.
- Update firmware handlers narrowly to consume the projection for statistics and scoreboard instead of hard-coded empty responses when evidence-backed runtime data exists.
- Update WebSocket projection through existing `collect_api_snapshot()`, `system_info_from_snapshot()`, and `LiveTelemetryPlanner` paths unless planning proves a narrower route-specific adapter is safer.
- Add Phase 26 evidence under `docs/parity/evidence/phase-26-telemetry-and-parity-closure/`.
- Update `docs/parity/checklist.md` and `tools/parity` only to the exact evidence tier Phase 26 proves.

</code_context>

<specifics>
## Specific Ideas

- Preferred runtime shape: a typed runtime event/projection layer with generation/session/sequence guards that derives `ApiSnapshot` mining fields, statistics samples, scoreboard entries, and WebSocket telemetry from one source.
- Preferred counter rule: no accepted/rejected share counter increments unless the event came from current-generation submit intent plus parsed pool response.
- Preferred statistics rule: emit no fabricated history. Empty compatible response remains valid when there are no evidence-backed samples.
- Preferred scoreboard rule: keep empty array until there is a parsed pool-response-backed share event or other exact upstream-owned scoreboard source.
- Preferred WebSocket rule: sanitize once before fan-out, preserve `/api/ws` log compatibility, and preserve `/api/ws/live` full/diff cadence.
- Preferred evidence rule: Phase 26 can promote API/WebSocket/statistics/scoreboard closure only to the evidence class actually produced; blocked live share proof remains a non-claim.

</specifics>

<deferred>
## Deferred Ideas

- Accepted/rejected live share proof remains below verified when Phase 25 artifacts record `blocked_safe_prerequisite` instead of a detector-gated pool response.
- Full active voltage, fan, thermal fault-stimulus, load, recovery, self-test, non-205 board, and non-BM1366 ASIC hardware closure remain future work.
- OTAWWW, rollback, failed-update, large erase, interrupted-update, and destructive recovery evidence remain future work.
- Runtime display/input, LVGL-like UI flow, BAP, Stratum v2, and unbounded stress mining remain future work.
- Broad AxeOS UI changes remain out of v1.1.

</deferred>

*Phase: 26-telemetry-and-parity-closure*
*Context gathered: 2026-07-05*
