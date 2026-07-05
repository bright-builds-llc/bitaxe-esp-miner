---
generated_by: gsd-discuss-phase
lifecycle_mode: yolo
phase_lifecycle_id: 25-2026-07-05T01-55-45
generated_at: 2026-07-05T01:55:45.817Z
---

# Phase 25: Live Stratum Runtime And Safe Stop - Context

**Gathered:** 2026-07-05
**Status:** Ready for planning
**Mode:** Yolo

<domain>
## Phase Boundary

Phase 25 owns the transition from controlled no-share and production-work implementation evidence to a real Ultra 205 Stratum v1 production runtime. The firmware shell must drive real TCP socket I/O for connect, subscribe, authorize, difficulty/extranonce, notify, submit, response classification, reconnect, and safe stop while keeping protocol, BM1366 work/result, prerequisite safety, watchdog, and evidence decisions in pure tested crates where practical.

This phase may prove a real pool response to a live ASIC-derived `mining.submit` as accepted or rejected, or it may record an explicit safe-prerequisite blocker when safe mining cannot proceed. It does not own Phase 26 API/WebSocket/statistics/scoreboard promotion beyond the post-stop state required for SAFE-12, and it does not claim non-205 boards, non-BM1366 ASICs, Stratum v2, full active voltage/fan/thermal/fault/self-test closure, OTA/recovery fault injection, runtime display/input, BAP, or unbounded stress mining.

</domain>

<decisions>
## Implementation Decisions

### Real Stratum Runtime Boundary

- **D-01:** Implement the live Stratum v1 path as a firmware socket adapter around the existing pure Stratum protocol/runtime core. ESP-IDF networking, TCP reads/writes, task yielding, timing, and shutdown stay in firmware; message parsing, lifecycle state, fake-pool behavior, submit mapping, and response classification stay in `crates/bitaxe-stratum`.
- **D-02:** Preserve the Phase 24 production BM1366 boundary: firmware interprets typed production actions and observations, but raw BM1366 work construction, result parsing, active-work correlation, submit intent construction, and redaction-safe result rendering stay in pure ASIC/Stratum modules.
- **D-03:** Live runtime startup must remain fail-closed behind Phase 22 prerequisite readiness. Missing, stale, unavailable, unsafe, ambiguous, or undocumented safety observations keep socket mining disabled or stopped with stable redaction-safe blocker reasons.

### Submit Response Classification

- **D-04:** Classify accepted, rejected, blocked, timeout, reconnect, malformed, or no-observed share outcomes only when a pool response is tied to a live ASIC-derived submit intent from the active production work registry.
- **D-05:** A nonce/result observation plus submit intent is still not an accepted/rejected share claim until the real socket runtime observes and classifies the pool's `mining.submit` response. Implementation-only or fake-pool evidence remains below live accepted/rejected proof.
- **D-06:** Raw pool endpoints, ports, workers, owner addresses, passwords, targets, extranonces, share payloads, socket errors, device URLs, IPs, MACs, Wi-Fi values, tokens, NVS secrets, and raw BM1366 frames must not appear in committed logs, evidence, API captures, WebSocket captures, discussion artifacts, or parity updates.

### Deterministic Fake-Pool Coverage

- **D-07:** Extend the existing deterministic fake-pool/fixture harness instead of creating a second protocol simulator. Coverage must prove subscribe, authorize, notify, set-difficulty, clean-jobs, submit response, reconnect, fallback, timeout, malformed response, and error classification behavior.
- **D-08:** Fake-pool tests should exercise both successful and fail-closed paths: clean-jobs invalidation, reconnect generation changes, stale work rejection, blocked prerequisite outcomes, accepted response classification, rejected response classification, and no-response timeout handling.
- **D-09:** Fake-pool or fixture tests can prove deterministic STR-11 behavior, but cannot by themselves promote STR-09 live pool response evidence.

### Bounded Safe Stop

- **D-10:** Define safe stop as an explicit runtime postcondition: socket activity stopped, reads/writes no longer advancing mining state, work queues drained or invalidated, active production work invalidated, mining disabled, hardware control disabled, work submission blocked, and post-stop runtime/API-visible state refreshed.
- **D-11:** Safe-stop behavior must be callable from normal stop, reconnect/fallback exhaustion, prerequisite failure, operator cancellation, and verification cleanup paths without leaking secrets or leaving stale active-mining state.
- **D-12:** Committed evidence may record safe-stop categories and status labels, but any raw local logs used for diagnosis must remain ignored/local or be redacted before promotion.

### Watchdog Responsiveness

- **D-13:** The live runtime must preserve watchdog responsiveness under bounded socket, ASIC, API/WebSocket, and evidence-capture load by using explicit checkpoints or yields around blocking-prone operations.
- **D-14:** Watchdog proof should combine pure budget/checkpoint tests with firmware or workflow evidence when hardware is available. A blocked or non-hardware path must keep SAFE-13 below hardware-verified status and record the exact non-claim.

### Evidence And Allow-Manifest Integration

- **D-15:** Update Phase 23 evidence-root and allow/validation tooling deliberately for a Phase 25 live Stratum surface before hardware evidence is promoted. Do not bypass existing redaction, detector, package, safe-state, or prohibited-token validation.
- **D-16:** Hardware use must follow `just detect-ultra205`, board `205` selection, repo-owned commands, runtime-only local credentials, redaction review, and exact evidence recording. If detection, safe prerequisites, credentials, socket behavior, or share outcome proof is blocked, record the blocker instead of inferring success.
- **D-17:** Checklist promotion must be exact: STR-08, STR-09, STR-11, SAFE-12, and SAFE-13 advance only to the level supported by source, deterministic tests, workflow evidence, detector-gated hardware evidence, and redaction review actually produced in this phase.

### Claude's Discretion

Claude may choose exact module names, adapter trait names, fake-pool fixture structure, timeout budgets, retry limits, evidence filenames, redaction labels, and plan count. Those choices must preserve functional core / imperative shell structure, typed fail-closed behavior, redaction rules, Ultra 205 detector gating, repo-owned verification, and conservative parity semantics.

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Phase Scope And Requirements

- `.planning/ROADMAP.md` - Phase 25 goal, dependency on Phase 24, requirements, and success criteria.
- `.planning/REQUIREMENTS.md` - STR-08, STR-09, STR-11, SAFE-12, SAFE-13, and v1.1 traceability.
- `.planning/PROJECT.md` - v1.1 Ultra 205 trusted production mining scope, ESP-IDF Rust stack, parity evidence policy, architecture, licensing, and safety constraints.
- `.planning/STATE.md` - Current progress, carried v1.1 decisions, and live-share/safety blockers.
- `AGENTS.md` - Ultra 205 detector gate, local credential handling, `DEVICE_URL` derivation limits, redaction rules, hardware evidence requirements, and phase-gated unsafe action limits.
- `AGENTS.bright-builds.md` - Bright Builds workflow, standards routing, and verification expectations.
- `standards/core/architecture.md` - Functional core / imperative shell and parse-boundary guidance.
- `standards/core/code-shape.md` - Early returns, optional naming, script rerun-safety, and module-size guidance.
- `standards/core/testing.md` - Unit-test structure and pure logic coverage expectations.
- `standards/core/verification.md` - Sync and repo-native verification before commit.
- `standards/languages/rust.md` - Rust module layout, `maybe_` naming, invariants, and testing expectations.

### Prior v1.1 Decisions And Evidence

- `.planning/phases/22-claim-ladder-and-safety-preconditions/22-CONTEXT.md` - Claim ladder, typed prerequisite contract, blocker reasons, and exact non-claim handling.
- `.planning/phases/23-redacted-operator-evidence-workflow/23-CONTEXT.md` - Redacted evidence-root slots, runtime-only credential handling, redaction contract, and exact non-claim governance.
- `.planning/phases/24-bm1366-production-work-path/24-CONTEXT.md` - Production BM1366 mode split, pool-derived work dispatch, result correlation, and submit-intent boundary.
- `.planning/phases/24-bm1366-production-work-path/24-04-SUMMARY.md` - Phase 24 closure handoff, implemented/test evidence, and remaining Phase 25 ownership.
- `.planning/phases/24-bm1366-production-work-path/24-VERIFICATION.md` - Latest Phase 24 verification result and residual risks.
- `docs/parity/evidence/phase-23-redacted-operator-evidence-workflow/evidence-contract.md` - Required redacted evidence-root slots and promotion contract.
- `docs/parity/evidence/phase-23-redacted-operator-evidence-workflow/safe-stop.md` - Existing safe-stop slot semantics and redaction-safe status language.
- `docs/parity/evidence/phase-23-redacted-operator-evidence-workflow/share-outcome.md` - Existing share-outcome blocked/non-claim slot to promote only with Phase 25 proof.
- `docs/parity/evidence/phase-24-bm1366-production-work-path/production-work.md` - Phase 24 production work evidence and remaining live socket non-claims.
- `docs/parity/evidence/phase-24-bm1366-production-work-path/result-correlation.md` - Submit-intent/result-correlation evidence and accepted/rejected response boundary.
- `docs/parity/checklist.md` - Current Stratum, ASIC, safety, API, statistics, and evidence rows plus verified-row guardrails.

### Current Implementation Surfaces

- `crates/bitaxe-stratum/src/v1/messages.rs` - Stratum v1 subscribe, authorize, notify, set-difficulty, submit, and response parsing/serialization.
- `crates/bitaxe-stratum/src/v1/fake_pool.rs` - Deterministic fake-pool harness to extend for Phase 25.
- `crates/bitaxe-stratum/src/v1/production_work.rs` - Production work registry, submit intent, and redaction-safe correlation types.
- `crates/bitaxe-stratum/src/v1/mining.rs` - Mining job construction and share submission mapping.
- `crates/bitaxe-stratum/src/v1/mining_loop.rs` - Guarded mining-loop state machine, prerequisite gates, and blocked work submission.
- `crates/bitaxe-stratum/src/v1/queue.rs` - Work queue and active work tracking.
- `crates/bitaxe-stratum/src/v1/state.rs` - Pool lifecycle, share counters, pool difficulty, mining activity, and runtime state.
- `crates/bitaxe-asic/src/bm1366/production.rs` - BM1366 production action/result bridge and redaction-safe status surface.
- `crates/bitaxe-safety/src/watchdog.rs` - Watchdog budget/checkpoint model for bounded runtime proof.
- `crates/bitaxe-safety/src/effects.rs` - Safety effect/hardware-control disable modeling.
- `firmware/bitaxe/src/network_stack.rs` - ESP-IDF network setup boundary.
- `firmware/bitaxe/src/controlled_mining_runtime.rs` - Existing controlled runtime shell and redacted status markers to evolve or replace carefully.
- `firmware/bitaxe/src/asic_adapter.rs` - Firmware interpreter for typed BM1366 actions.
- `firmware/bitaxe/src/asic_adapter/status.rs` - ASIC and mining status logging.
- `firmware/bitaxe/src/runtime_snapshot.rs` - Runtime status and telemetry integration point for post-stop state.
- `firmware/bitaxe/src/http_api.rs` - API projection touchpoint for post-stop state only.
- `firmware/bitaxe/src/websocket_api.rs` - WebSocket projection touchpoint for post-stop state only.
- `tools/parity/src/mining_allow.rs` - Mining allow-manifest validation and claim-tier guardrails to update for Phase 25 live Stratum evidence.
- `tools/parity/src/main.rs` - Checklist validation and verified-row guardrails.

### Upstream Reference And Policy

- `reference/esp-miner/components/stratum/stratum_socket.c` - Upstream pool socket lifecycle, reconnect, and fallback behavior.
- `reference/esp-miner/components/stratum/stratum_api.c` - Upstream Stratum message handling.
- `reference/esp-miner/components/stratum/mining.c` - Coinbase hashing, merkle root, BM job construction, extranonce, difficulty, and submit behavior.
- `reference/esp-miner/main/tasks/protocol_coordinator.c` - Protocol lifecycle coordination and watchdog-sensitive mining behavior.
- `reference/esp-miner/main/system.c` - Accepted/rejected share counters and clean-jobs handling.
- `reference/esp-miner/main/global_state.h` - Pool, share, queue, mining, and runtime status fields.
- `reference/esp-miner/main/work_queue.c` - Upstream work queue behavior.
- `docs/adr/0001-device-user-parity.md` - Observable device-user parity scope.
- `docs/adr/0003-esp-idf-rust-production-stack.md` - ESP-IDF Rust stack and adapter boundary.
- `docs/adr/0005-read-only-reference-implementation.md` - Read-only upstream reference policy.
- `docs/adr/0006-parity-checklist-as-audit-evidence.md` - Checklist evidence policy.
- `docs/adr/0012-parity-verification-evidence.md` - Evidence class and verified status semantics.
- `docs/adr/0013-mit-first-with-gpl-guardrails.md` - GPL provenance guardrails.
- `docs/adr/0014-pivot-to-ultra-205-bm1366-first-parity.md` - Ultra 205 BM1366 first parity target and deferred non-205 scope.
- `PROVENANCE.md` - Reference, GPL, fixture, source-attribution, dependency-license, and release-review policy.

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets

- `crates/bitaxe-stratum` already owns Stratum v1 messages, deterministic fake-pool fixtures, mining job construction, runtime state, guarded mining-loop gates, work queueing, and production submit-intent concepts.
- `crates/bitaxe-asic` already owns BM1366 production action/result modeling and diagnostic-vs-production boundaries from Phase 24.
- `crates/bitaxe-safety` already owns prerequisite evidence classes, stable safety reason strings, safety effects, and watchdog budget/checkpoint primitives.
- Phase 23 evidence-root artifacts already define committed slots for package, detector, board-info, command, log, API, WebSocket, share-outcome, safe-stop, redaction-review, and conclusion.
- `tools/parity` already enforces claim-tier, redaction, safe-state, and verified-row guardrails, but Phase 25 must deliberately add any new live Stratum evidence category instead of bypassing the guard.

### Established Patterns

- Pure protocol, ASIC, safety, evidence, and parity decisions live in Rust crates/tools with focused unit tests.
- ESP-IDF sockets, Wi-Fi/network availability, UART/GPIO/I2C, serial capture, local credentials, HTTP/WebSocket capture, NVS, and hardware effects remain thin firmware/tool/script shells.
- Hardware evidence names board `205`, selected port, source commit, reference commit, package or firmware identity, exact commands, board-info output, captured logs, observed behavior, redaction status, safe-state markers, and conclusion.
- Checklist rows cite exact artifacts and remain below `verified` when artifacts prove only fake-pool, blocked, implementation-only, no-share, stale, or startup-only behavior.
- Frontmatter-parsed Markdown must not use standalone body `---` separators after YAML frontmatter.

### Integration Points

- Add Phase 25 planning artifacts under `.planning/phases/25-live-stratum-runtime-and-safe-stop/`.
- Extend `crates/bitaxe-stratum` for live runtime state transitions, socket-adapter-facing events, submit-response classification, and fake-pool coverage when existing state/message helpers cannot represent the Phase 25 lifecycle.
- Extend firmware narrowly for real ESP-IDF TCP socket I/O, watchdog-friendly runtime loop checkpoints, controlled shutdown, and redaction-safe status markers.
- Extend Phase 23 evidence-root artifacts and `tools/parity` only after exact Phase 25 artifacts exist.
- Keep Phase 26 API/WebSocket/statistics promotion deferred, except for the minimal post-stop state refresh needed to prove SAFE-12.

</code_context>

<specifics>
## Specific Ideas

- Preferred runtime shape: a firmware-owned socket loop that emits typed lifecycle events into pure Stratum state and receives typed outbound messages, with no raw pool values in logs.
- Preferred response gate: a typed classifier that consumes a submit request id, active submit intent, and redacted pool response category to produce accepted, rejected, blocked, timeout, or malformed outcomes.
- Preferred safe-stop proof: unit tests for stop postconditions plus detector-gated evidence when available, with explicit blocked non-claims when live hardware or pool response proof cannot proceed safely.
- Preferred watchdog proof: pure checkpoint/budget tests plus redacted runtime markers showing bounded yields around socket, ASIC, API/WebSocket, and evidence-capture work.
- Preferred fake-pool shape: extend `fake_pool.rs` and related fixtures for clean-jobs, reconnect, fallback, response errors, and accepted/rejected submit classification rather than adding a parallel simulator.
- Preferred evidence shape: promote Phase 23 share-outcome and safe-stop slots only with Phase 25 artifacts, preserving exact non-claims for Phase 26 telemetry and future safety/recovery surfaces.

</specifics>

<deferred>
## Deferred Ideas

- API, WebSocket, statistics, scoreboard, and final parity checklist projection from v1.1 runtime events belong to Phase 26, except for post-stop state required by SAFE-12.
- Full active voltage/fan/thermal fault-stimulus, self-test hardware closure, OTA/recovery destructive or fault-injection evidence, runtime display/input, BAP, Stratum v2, non-205 boards, other ASIC families, and unbounded stress mining remain future work.
- Broad UI or AxeOS surface changes are out of scope for Phase 25.

</deferred>

*Phase: 25-live-stratum-runtime-and-safe-stop*
*Context gathered: 2026-07-05*
