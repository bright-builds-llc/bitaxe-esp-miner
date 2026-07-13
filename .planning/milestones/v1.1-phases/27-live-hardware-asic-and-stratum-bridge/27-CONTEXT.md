---
generated_by: gsd-discuss-phase
lifecycle_mode: yolo
phase_lifecycle_id: 27-2026-07-05T14-51-50
generated_at: 2026-07-05T14:51:50.000Z
---

# Phase 27: Live Hardware ASIC And Stratum Bridge - Context

**Gathered:** 2026-07-05
**Status:** Ready for planning
**Mode:** Yolo

<domain>
## Phase Boundary

Phase 27 closes the live-hardware integration gap between Phase 24 BM1366 production dispatch/correlation and Phase 25 live Stratum socket runtime. Live production firmware must dispatch `Bm1366ProductionCommand` from pool-derived work inside the Phase 25 socket loop, feed `ProductionNonceObservation` back into runtime correlation and submit classification, and record detector-gated redacted share-outcome evidence as accepted, rejected, or an explicit safe-prerequisite blocker.

This phase does not own Phase 28 checklist promotion beyond producing the Phase 27 hardware artifacts it depends on, Phase 26 telemetry projection changes, non-205 boards, other ASIC families, Stratum v2, full active voltage/fan/thermal/fault/self-test closure, OTA/recovery fault injection, runtime display/input, BAP, or unbounded stress mining.
</domain>

<decisions>
## Implementation Decisions

### Live Bridge Boundary

- **D-01:** Extend `firmware/bitaxe/src/live_stratum_runtime.rs` as the sole live production bridge. Do not create a parallel production runtime module or route live socket mining back through `controlled_mining_runtime.rs`.
- **D-02:** Keep the functional-core / imperative-shell split: guarded dispatch plans, `ProductionWorkRegistry`, nonce correlation, submit intent, and submit classification stay in `crates/bitaxe-stratum`; ESP-IDF UART/GPIO effects, ASIC adapter execution, socket I/O, watchdog yields, and evidence markers stay in firmware.
- **D-03:** Preserve Phase 25 safe-stop, prerequisite, watchdog, and post-stop snapshot behavior. Phase 27 adds ASIC dispatch and nonce feedback inside the existing live runtime loop rather than replacing the socket shell.

### Production Command Dispatch In Live Runtime

- **D-04:** When the pure live runtime emits work-dispatch actions from pool-derived notify/clean-jobs state, the firmware bridge must translate them into `GuardedBm1366DispatchPlan` output and execute `maybe_production_command` through the existing ASIC adapter path used by the controlled runtime.
- **D-05:** Live dispatch must use `Bm1366ProductionCommand::SendProductionWork` and `Bm1366ProductionCommand::ReadProductionResult` only. Diagnostic `Bm1366Command::SendDiagnosticWork` must remain unreachable from the live socket path.
- **D-06:** Production ASIC status logs in the live path must reuse the Phase 24 redaction-safe keys: `asic_production_status=initialized`, `asic_production_status=work_dispatched`, `asic_production_status=result_correlated`, and `asic_production_status=fail_closed` with `reason={label} mining=disabled work_submission=disabled` for fail-closed cases.

### Nonce Observation Feedback And Submit Classification

- **D-07:** Hardware nonce/result reads must be wrapped as `ProductionNonceObservation { observed_generation, result }` at the firmware boundary, using the generation associated with the dispatch/read attempt rather than inferring session identity from parsed ASIC bytes alone.
- **D-08:** The live runtime must feed observations back into `ProductionWorkRegistry::correlate_nonce_result`, then only emit `mining.submit` when a current-generation `SubmitIntent` exists. Uncorrelated, stale-generation, malformed, duplicate, or blocked outcomes must fail closed without share claims.
- **D-09:** Accepted or rejected share classification remains tied to live pool response plus matching submit intent. Phase 27 may produce the first detector-gated live share-outcome artifact, but fake-pool-only or implementation-only paths must not promote STR-09 above the evidence actually captured.

### Distinct Phase 27 Evidence Mode

- **D-10:** Add a distinct compile-time opt-in mode for Phase 27 live hardware bridge evidence, analogous to Phase 25's mode/ack pair. Missing or mismatched mode/ack values must keep the bridge fail-closed and must not silently fall back to controlled or Phase 25-only behavior.
- **D-11:** Phase 27 mode must require Phase 22 prerequisite readiness before any ASIC dispatch, pool settings access, or socket connect attempt, preserving the existing fail-closed ordering from Phase 25.

### Hardware Evidence And Redaction

- **D-12:** Add a repo-owned Phase 27 evidence wrapper with blocked and hardware modes, detector-first hardware path, board-info gate, allow-manifest validation, redaction review, and exact non-claims when safe prerequisites or live share proof cannot proceed.
- **D-13:** Committed evidence must record share outcome as `accepted`, `rejected`, or `blocked_safe_prerequisite` using category labels only. Raw pool endpoints, ports, workers, owner addresses, passwords, targets, extranonces, share payloads, socket errors, device URLs, IPs, MACs, Wi-Fi values, tokens, NVS secrets, and raw BM1366 frames must not appear in committed artifacts.
- **D-14:** Update Phase 23/25 evidence-root slots and `tools/parity` only after exact Phase 27 artifacts exist. Preserve explicit non-claims when hardware prerequisites block live share proof.

### Verification And Checklist Semantics

- **D-15:** Unit and firmware tests must prove live-bridge dispatch, nonce observation stamping, correlation gating, fail-closed blockers, mode gating, and evidence-wrapper blocked paths without requiring hardware for every assertion.
- **D-16:** Hardware verification must follow `just detect-ultra205`, board `205`, repo-owned commands, runtime-only local credentials, redaction review, and exact evidence recording. If detection, safe prerequisites, ASIC dispatch, socket behavior, or share outcome proof is blocked, record the blocker instead of inferring success.
- **D-17:** STR-08, STR-09, ASIC-10, and ASIC-11 may advance only to the exact level supported by Phase 27 source, tests, workflow evidence, detector-gated hardware evidence, and redaction review actually produced.

### Claude's Discretion

Claude may choose exact module names, adapter helper names, evidence filenames, timeout budgets, mode/ack constant names, fixture shapes, and plan count. Those choices must preserve functional core / imperative shell structure, typed fail-closed behavior, redaction rules, Ultra 205 detector gating, repo-owned verification, and conservative parity semantics.
</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Phase Scope And Requirements

- `.planning/ROADMAP.md` - Phase 27 goal, dependency on Phase 26, requirements, and success criteria.
- `.planning/REQUIREMENTS.md` - STR-08, STR-09, ASIC-10, ASIC-11, and v1.1 traceability.
- `.planning/PROJECT.md` - v1.1 Ultra 205 trusted production mining scope, ESP-IDF Rust stack, parity evidence policy, architecture, licensing, and safety constraints.
- `.planning/STATE.md` - Carried v1.1 decisions, Phase 26 closure status, and live-share blockers.
- `AGENTS.md` - Ultra 205 detector gate, local credential handling, `DEVICE_URL` derivation limits, redaction rules, hardware evidence requirements, and phase-gated unsafe action limits.
- `AGENTS.bright-builds.md` - Bright Builds workflow, standards routing, verification, and code-shape expectations.
- `standards/core/architecture.md` - Functional core / imperative shell and parse-boundary guidance.
- `standards/core/code-shape.md` - Early returns, optional naming, script rerun-safety, and module-size guidance.
- `standards/core/testing.md` - Unit-test structure and pure logic coverage expectations.
- `standards/core/verification.md` - Sync and repo-native verification before commit.
- `standards/languages/rust.md` - Rust module layout, `maybe_` naming, invariants, and testing expectations.

### Prior v1.1 Decisions And Evidence

- `.planning/phases/24-bm1366-production-work-path/24-CONTEXT.md` - Production BM1366 mode split, pool-derived work dispatch, result correlation, and submit-intent boundary.
- `.planning/phases/24-bm1366-production-work-path/24-VERIFICATION.md` - Phase 24 verification result and controlled-runtime wiring evidence.
- `.planning/phases/25-live-stratum-runtime-and-safe-stop/25-CONTEXT.md` - Live Stratum runtime boundary, submit response classification, safe-stop postconditions, and Phase 27 deferral.
- `.planning/phases/25-live-stratum-runtime-and-safe-stop/25-VERIFICATION.md` - Phase 25 verification result and blocked share-outcome non-claims.
- `.planning/phases/26-telemetry-and-parity-closure/26-CONTEXT.md` - Runtime projection source-of-truth and counter invariants.
- `docs/parity/evidence/phase-23-redacted-operator-evidence-workflow/evidence-contract.md` - Required evidence-root slots and promotion contract.
- `docs/parity/evidence/phase-25-live-stratum-runtime-and-safe-stop/share-outcome.md` - Current share-outcome blocker/non-claim slot to promote only with Phase 27 proof.
- `docs/parity/evidence/phase-24-bm1366-production-work-path/production-work.md` - Production work evidence and live bridge boundary.
- `docs/parity/evidence/phase-24-bm1366-production-work-path/result-correlation.md` - Submit-intent/result-correlation evidence.
- `docs/parity/checklist.md` - Current Stratum, ASIC, safety, and v1.1 evidence rows plus verified-row guardrails.

### Current Implementation Surfaces

- `firmware/bitaxe/src/live_stratum_runtime.rs` - Phase 25 live socket shell to extend with production dispatch and nonce feedback.
- `firmware/bitaxe/src/controlled_mining_runtime.rs` - Existing production dispatch reference implementation to reuse patterns from, not route live socket mining through.
- `firmware/bitaxe/src/asic_adapter.rs` - Firmware interpreter for typed BM1366 actions.
- `firmware/bitaxe/src/asic_adapter/status.rs` - Production ASIC status publishers.
- `firmware/bitaxe/src/mining_evidence_mode.rs` - Compile-time evidence mode gate to extend for Phase 27.
- `firmware/bitaxe/src/runtime_snapshot.rs` - Post-stop and runtime snapshot integration.
- `crates/bitaxe-stratum/src/v1/live_runtime.rs` - Pure live runtime lifecycle and actions.
- `crates/bitaxe-stratum/src/v1/mining_loop.rs` - Guarded production dispatch plan.
- `crates/bitaxe-stratum/src/v1/production_work.rs` - Production work registry, nonce observation, submit intent.
- `crates/bitaxe-stratum/src/v1/submit_response.rs` - Submit response classifier.
- `crates/bitaxe-asic/src/bm1366/production.rs` - `Bm1366ProductionCommand` and production status types.
- `scripts/phase25-live-stratum-evidence.sh` - Phase 25 evidence wrapper pattern to extend or mirror for Phase 27 bridge proof.
- `tools/parity/src/mining_allow.rs` - Mining allow-manifest validation and claim-tier guardrails.

### Upstream Reference And Policy

- `reference/esp-miner/components/stratum/mining.c` - Coinbase hashing, merkle root, BM job construction, extranonce, difficulty, and submit behavior.
- `reference/esp-miner/components/asic/bm1366.c` - BM1366 initialization, work, result, nonce, and frequency behavior.
- `reference/esp-miner/main/tasks/protocol_coordinator.c` - Protocol lifecycle coordination and watchdog-sensitive mining behavior.
- `docs/adr/0001-device-user-parity.md` - Observable device-user parity scope.
- `docs/adr/0003-esp-idf-rust-production-stack.md` - ESP-IDF Rust stack and adapter boundary.
- `docs/adr/0005-read-only-reference-implementation.md` - Read-only upstream reference policy.
- `docs/adr/0006-parity-checklist-as-audit-evidence.md` - Checklist evidence policy.
- `docs/adr/0012-parity-verification-evidence.md` - Evidence class and verified status semantics.
- `docs/adr/0014-pivot-to-ultra-205-bm1366-first-parity.md` - Ultra 205 BM1366 first parity target.
- `PROVENANCE.md` - Reference, GPL, fixture, source-attribution, dependency-license, and release-review policy.
</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets

- Phase 24 already provides `Bm1366ProductionCommand`, `ProductionNonceObservation`, `ProductionWorkRegistry`, guarded mining-loop dispatch, and production ASIC status publishers.
- Phase 25 already provides the live socket shell, submit classifier, safe-stop postconditions, prerequisite gate, watchdog categories, and blocked share-outcome evidence workflow.
- `controlled_mining_runtime.rs` already demonstrates how firmware consumes `maybe_production_command` and `maybe_submit_intent`; Phase 27 should transplant that pattern into the live socket loop.
- Phase 25 evidence wrapper and mining-allow tiers provide the blocked/hardware/redaction pattern for Phase 27 bridge evidence.

### Established Patterns

- Pure protocol, ASIC, safety, evidence, and parity decisions live in Rust crates/tools with focused unit tests.
- ESP-IDF sockets, UART/GPIO, serial capture, local credentials, HTTP/WebSocket capture, NVS, and hardware effects remain thin firmware/tool/script shells.
- Hardware evidence names board `205`, selected port, source commit, reference commit, package or firmware identity, exact commands, board-info output, captured logs, observed behavior, redaction status, safe-state markers, and conclusion.
- Checklist rows cite exact artifacts and remain below `verified` when artifacts prove only fake-pool, blocked, implementation-only, no-share, stale, or startup-only behavior.

### Integration Points

- Add Phase 27 planning artifacts under `.planning/phases/27-live-hardware-asic-and-stratum-bridge/`.
- Extend `live_stratum_runtime.rs` and `mining_evidence_mode.rs`; extend pure live runtime/mining-loop surfaces only where existing types cannot represent notify-to-dispatch and observation-to-submit wiring.
- Reuse `asic_adapter` and production status publishers rather than duplicating ASIC effect code in the live runtime module.
- Add Phase 27 evidence under `docs/parity/evidence/phase-27-live-hardware-asic-and-stratum-bridge/`.
- Update checklist and parity tooling only to the exact evidence tier Phase 27 proves.
</code_context>

<specifics>
## Specific Ideas

- Preferred bridge shape: live socket loop owns notify-to-dispatch and observation-to-submit side effects while pure runtime state decides when dispatch, read-result, correlate, and submit actions are allowed.
- Preferred reuse: copy the controlled runtime's production command execution and status publication pattern into live runtime helpers rather than inventing a third ASIC dispatch route.
- Preferred mode shape: dedicated Phase 27 mode/ack compile-time pair that enables bridge evidence without weakening Phase 21 controlled or Phase 25-only modes.
- Preferred evidence shape: detector-gated hardware run that either records accepted/rejected share-outcome categories or an explicit safe-prerequisite blocker with redaction review.
- Preferred test shape: firmware unit tests with fake socket/ASIC collaborators plus pure crate tests for any new live-runtime dispatch hooks.
</specifics>

<deferred>
## Deferred Ideas

- Phase 28 checklist promotion from Phase 27 artifacts belongs to Phase 28, not Phase 27 closure work beyond producing the artifacts Phase 28 needs.
- Full active voltage, fan, thermal fault-stimulus, self-test hardware closure, OTA/recovery destructive evidence, runtime display/input, BAP, Stratum v2, non-205 boards, other ASIC families, and unbounded stress mining remain future work.
- Broad Phase 26 telemetry or API/WebSocket projection changes remain out of scope unless a minimal post-bridge runtime marker is required for evidence capture.
</deferred>

*Phase: 27-live-hardware-asic-and-stratum-bridge*
*Context gathered: 2026-07-05*
