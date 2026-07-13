---
generated_by: gsd-discuss-phase
lifecycle_mode: yolo
phase_lifecycle_id: 24-2026-07-05T00-27-27
generated_at: 2026-07-05T00:27:27.205Z
---

# Phase 24: BM1366 Production Work Path - Context

**Gathered:** 2026-07-05
**Status:** Ready for planning
**Mode:** Yolo

<domain>
## Phase Boundary

Phase 24 delivers the trusted Ultra 205 BM1366 production work path. The functional core must separate diagnostic ASIC behavior from production initialization/work/result states, derive BM1366 work only from the active Stratum job context, invalidate stale work on clean-jobs or reconnect, correlate live BM1366 nonce/result observations to active work before any share-submission claim, and fail closed on initialization, UART, reset, timeout, malformed result, or job-correlation failures.

This phase does not own the full live Stratum socket runtime, accepted/rejected pool response proof, watchdog-under-load proof, bounded safe stop, API/WebSocket/statistics promotion, non-205 boards, other ASIC families, active voltage/fan/thermal/fault/self-test closure, OTA/recovery, Stratum v2, display/input, BAP, or unbounded stress mining. Those remain Phase 25, Phase 26, or future work unless a narrow prerequisite gap blocks the trusted BM1366 production work path.
</domain>

<decisions>
## Implementation Decisions

### Production vs Diagnostic ASIC Modes

- **D-01:** Preserve a hard type-level boundary between diagnostic BM1366 chip/work paths and trusted production BM1366 initialization, work dispatch, and result handling. Diagnostic evidence may support readiness, but it must not automatically promote production work claims.
- **D-02:** Production mode requires the Phase 22 prerequisite readiness contract before work dispatch. Missing, stale, unavailable, unsafe, ambiguous, or undocumented safety observations keep mining disabled and work submission disabled with stable redaction-safe blocker reasons.
- **D-03:** Firmware may interpret typed production ASIC actions, but it must not construct raw BM1366 frames in the shell. Raw packet details, CRCs, work payload encoding, valid-job tracking, and result parsing stay in `crates/bitaxe-asic`.

### Pool-Derived Work Dispatch

- **D-04:** BM1366 production work must be derived from the active Stratum v1 pool job, including job id, extranonce context, difficulty or target context, clean-jobs generation, and enough metadata to later validate a nonce/result.
- **D-05:** Work dispatch should use a typed active-work registry or equivalent functional-core model that binds each sent BM1366 job to the current pool session generation. Clean-jobs, reconnect, authorization reset, or pool-session replacement invalidates older active work before another share claim can be recorded.
- **D-06:** Production work dispatch should be testable without hardware by replaying fake-pool or fixture jobs through the Stratum job model into BM1366 work commands and active-work records. Hardware evidence remains required before safety-critical behavior is marked verified.

### Result Correlation and Share Claim Gate

- **D-07:** A live BM1366 nonce or result observation is not a share claim until the functional core maps it to an active, non-stale pool-derived work record and computes the corresponding submit context.
- **D-08:** Uncorrelated, stale-generation, malformed, duplicate, invalid-job, wrong-session, or target-mismatched results must fail closed with stable blocker/status outcomes and no share submission claim.
- **D-09:** The production path may prepare redaction-safe submit intent data for Phase 25, but this phase should not claim accepted or rejected pool response behavior unless the later Stratum runtime actually observes and classifies the response.

### Fail-Closed Errors and Redaction

- **D-10:** Initialization, UART, reset, timeout, malformed frame, result-parse, job-correlation, stale-work, and prerequisite failures must produce typed, redaction-safe reasons suitable for logs, evidence summaries, API/WebSocket projection later, and parity checks.
- **D-11:** Committed evidence and planning artifacts must never include raw BM1366 frames, raw Stratum targets, extranonces, share payloads, pool endpoints, ports, workers, owner addresses, passwords, socket errors, device URLs, IPs, MACs, Wi-Fi values, tokens, or NVS secrets.
- **D-12:** The Phase 23 redacted evidence-root contract remains the committed evidence shape. Slots that depend on Phase 25 or Phase 26 should remain blocked or pending with exact non-claims instead of being inferred from Phase 24 implementation.

### Verification and Evidence Semantics

- **D-13:** Pure unit and fixture tests should cover diagnostic-vs-production mode separation, pool-job-to-BM1366-work derivation, clean-jobs/reconnect invalidation, active-work result correlation, duplicate/stale/malformed rejection, fail-closed reasons, and redaction-safe rendering.
- **D-14:** Hardware-capable verification must follow the Ultra 205 detector gate, use repo-owned commands, record board `205`, selected port, source commit, reference commit, package identity, exact commands, board-info output, captured logs, observed behavior, conclusion, and redaction review, and stop on ambiguous ports or unsafe prerequisites.
- **D-15:** Checklist promotion should be exact. `ASIC-09` through `ASIC-12` may advance only to the level supported by implemented code, deterministic tests, and any detector-gated evidence actually produced during the phase.

### Claude's Discretion

Claude may choose the exact module names, type names, plan count, fixture format, active-work registry shape, error enum names, helper script boundaries, and checklist wording. Those choices must preserve functional core / imperative shell structure, read-only reference policy, ESP-IDF tooling preference, conservative evidence semantics, typed fail-closed behavior, and redaction rules.
</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Phase Scope and Standards

- `.planning/ROADMAP.md` - Phase 24 goal, dependency on Phase 23, requirements, and success criteria.
- `.planning/REQUIREMENTS.md` - ASIC-09, ASIC-10, ASIC-11, ASIC-12, and v1.1 traceability.
- `.planning/PROJECT.md` - v1.1 Ultra 205 trusted production mining scope, ESP-IDF Rust stack, parity evidence policy, architecture, licensing, and safety constraints.
- `.planning/STATE.md` - Current project state, carried v1.1 decisions, Phase 24 focus, and blockers.
- `AGENTS.md` - Ultra 205 detector gate, local credential handling, `DEVICE_URL` derivation limits, redaction rules, hardware evidence requirements, and frontmatter separator rule.
- `AGENTS.bright-builds.md` - Bright Builds workflow, standards routing, and verification expectations.
- `standards/core/architecture.md` - Functional core / imperative shell and parse-boundary guidance.
- `standards/core/code-shape.md` - Early returns, optional naming, script rerun-safety, and module-size guidance.
- `standards/core/testing.md` - Unit-test structure and pure logic coverage expectations.
- `standards/core/verification.md` - Sync and repo-native verification before commit.
- `standards/languages/rust.md` - Rust module layout, `maybe_` naming, invariants, and testing expectations.

### Prior Decisions and Evidence

- `.planning/phases/03-bm1366-asic-protocol-and-safe-initialization/03-CONTEXT.md` - BM1366 typed command/observation boundary, diagnostic work boundary, staged init gate, and hardware verification limits.
- `.planning/phases/12-asic-and-mining-hardware-evidence/12-CONTEXT.md` - ASIC/mining evidence ladder, detector gate, controlled no-share semantics, and exact checklist promotion rules.
- `.planning/phases/15-bm1366-mining-evidence-completion/15-CONTEXT.md` - Chip-detect, work/result diagnostics, controlled mining smoke, bounded soak, parity guards, and redaction decisions.
- `.planning/phases/21-live-mining-and-soak-evidence/21-CONTEXT.md` - Controlled pool gates, mining evidence ladder, live-pool/non-share boundaries, telemetry correlation, safe-stop, and final verification expectations.
- `.planning/phases/22-claim-ladder-and-safety-preconditions/22-CONTEXT.md` - Claim ladder, typed prerequisite contract, blocker reasons, evidence boundaries, and exact non-claim handling.
- `.planning/phases/23-redacted-operator-evidence-workflow/23-CONTEXT.md` - Redacted evidence-root slots, runtime-only credential handling, redaction contract, and exact non-claim governance.
- `.planning/phases/23-redacted-operator-evidence-workflow/23-VERIFICATION.md` - Latest lifecycle and verification result for the operator evidence workflow.
- `docs/parity/evidence/phase-22-claim-ladder-and-safety-preconditions/claim-ladder.md` - v1.0/v1.1 claim tiers and production-mining non-claim boundaries.
- `docs/parity/evidence/phase-22-claim-ladder-and-safety-preconditions/blocker-reasons.md` - Stable redaction-safe blocker strings.
- `docs/parity/evidence/phase-23-redacted-operator-evidence-workflow/` - Current redacted evidence-root contract and review artifacts.
- `docs/parity/checklist.md` - Current ASIC, Stratum, API, statistics, safety, and evidence rows plus safety-critical verification rules.

### Current Implementation Surfaces

- `crates/bitaxe-asic/src/bm1366.rs` - BM1366 module entrypoint and typed ASIC facade.
- `crates/bitaxe-asic/src/bm1366/adapter_gate.rs` - Hardware evidence acknowledgement and diagnostic gate model.
- `crates/bitaxe-asic/src/bm1366/chip_detect.rs` - Pure chip-detect validation and follow-up actions.
- `crates/bitaxe-asic/src/bm1366/command.rs` - Typed BM1366 command boundary.
- `crates/bitaxe-asic/src/bm1366/init_plan.rs` - Staged initialization plan and no-mining states.
- `crates/bitaxe-asic/src/bm1366/observation.rs` - Typed BM1366 observations.
- `crates/bitaxe-asic/src/bm1366/result.rs` - Result parsing and valid-job tracking.
- `crates/bitaxe-asic/src/bm1366/work.rs` - Diagnostic work payload and BM1366 work modeling.
- `crates/bitaxe-stratum/src/v1/fake_pool.rs` - Deterministic fake-pool harness.
- `crates/bitaxe-stratum/src/v1/messages.rs` - Stratum v1 subscribe, authorize, notify, set-difficulty, submit, and response parsing/serialization.
- `crates/bitaxe-stratum/src/v1/mining.rs` - Mining job construction and share submission mapping.
- `crates/bitaxe-stratum/src/v1/mining_loop.rs` - Guarded mining-loop state machine and safety evidence gates.
- `crates/bitaxe-stratum/src/v1/queue.rs` - Work queue and active work tracking.
- `crates/bitaxe-stratum/src/v1/state.rs` - Pool lifecycle, share counters, pool difficulty, mining activity, and runtime state.
- `crates/bitaxe-safety/src/evidence.rs` - Safety evidence classes and hardware verification labels.
- `crates/bitaxe-safety/src/status.rs` - Stable public safety reason strings.
- `firmware/bitaxe/src/asic_adapter.rs` - Firmware interpreter for typed BM1366 actions.
- `firmware/bitaxe/src/asic_adapter/status.rs` - ASIC and mining status logging.
- `firmware/bitaxe/src/controlled_mining_runtime.rs` - Redacted controlled runtime markers and current controlled mining shell.
- `firmware/bitaxe/src/runtime_snapshot.rs` - Runtime status and telemetry integration point.
- `tools/parity/src/mining_allow.rs` - Mining allow-manifest validation and claim-tier guardrails.
- `tools/parity/src/main.rs` - Checklist validation and verified-row guardrails.

### Upstream Reference and Policy

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
- `reference/esp-miner/main/tasks/protocol_coordinator.c` - Protocol lifecycle coordination and watchdog-sensitive mining behavior.
- `reference/esp-miner/main/system.c` - Accepted/rejected share counters and clean-jobs handling.
- `reference/esp-miner/main/global_state.h` - Pool, share, queue, mining, and runtime status fields.
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

- `crates/bitaxe-asic` already owns typed BM1366 protocol, diagnostic gates, staged init, work payload modeling, valid-job result parsing, observations, and fail-closed transcript behavior.
- `crates/bitaxe-stratum` already owns Stratum v1 fake-pool fixtures, mining job construction, queueing, guarded mining-loop state, runtime state, and share-submit mapping concepts.
- `crates/bitaxe-safety` already models safety evidence classes, stable status reasons, power/thermal observation freshness, and fail-closed prerequisite behavior introduced in Phase 22.
- `tools/parity` already validates mining/evidence allow manifests and rejects overbroad verified claims with blocker language.
- Phase 23 evidence-root and redaction tooling already provides the committed artifact shape for later production-mining runs.

### Established Patterns

- Pure ASIC, Stratum, safety, claim, redaction, and parity decisions live in Rust crates/tools with focused unit tests.
- ESP-IDF UART/GPIO/I2C, pool socket I/O, serial capture, HTTP/WebSocket capture, NVS, and hardware effects remain firmware/tool/script shells.
- Hardware evidence names board `205`, selected port, source commit, reference commit, package or firmware identity, exact commands, board-info output, logs, observed behavior, redaction status, safe-state markers, and conclusion.
- Checklist rows cite exact artifacts and remain below `verified` when artifacts prove only diagnostic, blocked, startup-only, stale, no-share, or implementation-only behavior.
- GSD artifacts and frontmatter-parsed Markdown must not use standalone body `---` separators after YAML frontmatter.

### Integration Points

- Add Phase 24 planning artifacts under `.planning/phases/24-bm1366-production-work-path/`.
- Extend `crates/bitaxe-asic` and/or `crates/bitaxe-stratum` with a typed production active-work/correlation model when existing diagnostic work/result and queue types cannot represent the v1.1 production invariants.
- Keep firmware changes narrow: interpret typed production commands, surface redaction-safe status, and fail closed on missing gates; avoid raw protocol construction in firmware.
- Update `docs/parity/checklist.md` and Phase 23 evidence-root slots only after exact Phase 24 artifacts exist.
</code_context>

<specifics>
## Specific Ideas

- Preferred pure model: a typed production work registry that binds BM1366 work to active Stratum session/job/extranonce/difficulty context and exposes explicit invalidation on clean-jobs and reconnect.
- Preferred result gate: a pure correlation function that returns either a redaction-safe submit intent or a typed blocked reason; no raw share payload appears in committed artifacts.
- Preferred mode split: diagnostic commands and statuses remain useful for evidence readiness, while production initialization/work/result states require prerequisite readiness and use distinct names.
- Preferred test shape: table-style unit fixtures for clean-jobs invalidation, reconnect generation changes, duplicate result rejection, malformed frame rejection, stale result rejection, and valid active-work correlation.
- Preferred evidence shape: Phase 23 redacted evidence-root slots updated with Phase 24 implementation/test proof and exact non-claims for Phase 25 live socket/share responses.
</specifics>

<deferred>
## Deferred Ideas

- Real Stratum v1 socket lifecycle, deterministic fake-pool production tests, live accepted/rejected share response classification, watchdog under bounded production load, and bounded safe-stop runtime proof belong to Phase 25.
- API/WebSocket/statistics/scoreboard promotion and final v1.1 parity closure belong to Phase 26.
- Full active voltage/fan/thermal fault-stimulus, self-test hardware closure, OTA/recovery destructive or fault-injection evidence, runtime display/input, BAP, Stratum v2, non-205 boards, other ASIC families, and unbounded stress mining remain future work.
</deferred>

*Phase: 24-bm1366-production-work-path*
*Context gathered: 2026-07-05*
