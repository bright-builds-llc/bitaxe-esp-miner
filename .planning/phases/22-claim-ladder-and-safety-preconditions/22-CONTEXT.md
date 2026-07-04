---
generated_by: gsd-discuss-phase
lifecycle_mode: yolo
phase_lifecycle_id: 22-2026-07-04T20-10-36
generated_at: 2026-07-04T20:10:36.848Z
---

# Phase 22: Claim Ladder And Safety Preconditions - Context

**Gathered:** 2026-07-04
**Status:** Ready for planning
**Mode:** Yolo

<domain>
## Phase Boundary

Phase 22 defines the v1.1 production-mining claim ladder and the prerequisite safety preconditions that must fail closed before BM1366 production work dispatch. Operators must be able to tell the difference between v1.0 controlled no-share evidence, v1.1 prerequisite readiness, and later live production mining outcomes. Firmware and tooling must expose specific blocker reasons when required power, thermal, fan, voltage, or safety observations are stale, unavailable, unsafe, ambiguous, or undocumented.

This phase does not run a new live Stratum production session, add the redacted evidence root, implement the trusted BM1366 production work path, prove accepted/rejected share outcomes, close active voltage/fan/thermal/self-test/fault-stimulus parity, or promote v1.1 telemetry rows. Those belong to later v1.1 phases unless Phase 22 discovers a prerequisite contract gap that blocks planning.

</domain>

<decisions>
## Implementation Decisions

### Claim Ladder

- **D-01:** Define an operator-visible claim ladder with at least these tiers: v1.0 controlled no-share evidence, v1.1 prerequisite readiness, v1.1 live socket/runtime evidence, v1.1 live ASIC-derived share outcome, and explicit deferred/non-claim surfaces.
- **D-02:** The ladder must preserve exact evidence semantics. A lower tier can support readiness or implementation but must not imply accepted shares, rejected shares, unbounded production mining, full active safety closure, non-205 board support, Stratum v2, OTA/recovery trust, display/input parity, or BAP behavior.
- **D-03:** Phase 21's approved controlled no-share soak remains useful prior evidence, but Phase 22 must label it as controlled no-share closure, not live accepted/rejected production-share proof.
- **D-04:** Parity and operator docs should use consistent language for "allowed claim", "blocked claim", and "explicit non-claim" so later phases can promote only the exact tier proved by redacted artifacts.

### Safety Prerequisite Contract

- **D-05:** Production mining cannot enable BM1366 work dispatch unless fresh or explicitly bounded prerequisite observations exist for power, thermal, fan, voltage, and overall safety state.
- **D-06:** "Fresh" should remain a strict runtime observation when existing code already has one, such as `PowerObservation` age checks. "Explicitly bounded" should be a named evidence contract with an observation source, age or validity window, board `205`, evidence id, and reason why it is acceptable for the current mining attempt.
- **D-07:** Prerequisites must be consumed as typed domain inputs before mining-loop work dispatch, not as freeform strings in shell scripts. Existing `crates/bitaxe-safety` evidence, power, thermal, and status modules are the preferred home for pure decision logic.
- **D-08:** Firmware and scripts may remain thin shells that collect observations and render logs, but they must not bypass the pure prerequisite decision model to start production work.

### Blocker Reasons And Fail-Closed Behavior

- **D-09:** Missing, stale, unavailable, unsafe, ambiguous, or undocumented prerequisite data must produce a specific stable blocker reason instead of a generic "not ready" result.
- **D-10:** Blocker reasons should remain user-visible through logs, runtime state, API/WebSocket projection when available, and evidence summaries. They must be safe to commit and must not reveal pool credentials, device URLs, IPs, MACs, Wi-Fi values, NVS secrets, raw Stratum payloads, raw share payloads, or raw BM1366 frames.
- **D-11:** The default state remains fail-closed: mining disabled, hardware control disabled, and work submission disabled. Any unsafe, ambiguous, stale, or undocumented prerequisite keeps this state and records the blocker.
- **D-12:** The blocker taxonomy should align with existing names where possible, including `power_sample_stale`, `power_sample_unavailable`, `thermal_reading_unavailable`, `thermal_reading_invalid`, `hardware_evidence_ack_missing`, and `safety_preflight_evidence_missing`.

### Evidence And Checklist Boundaries

- **D-13:** Phase 22 may add claim-ladder documentation, prerequisite contract tests, parity guard tests, and blocked evidence artifacts. It should not claim new live hardware behavior unless a detector-gated, redacted, phase-approved command actually produced that evidence.
- **D-14:** Checklist updates must be conservative. EVD-06, SAFE-10, and SAFE-11 can advance only to the level supported by docs, pure tests, workflow checks, or hardware evidence actually produced in this phase.
- **D-15:** Explicit non-claims for full active voltage control, fan actuation, thermal fault stimulus, self-test hardware closure, fault-stimulus closure, unbounded soak, accepted/rejected shares, non-205 boards, OTA/recovery, runtime display/input, BAP, and Stratum v2 must remain visible.
- **D-16:** Verification must include repo-native checks for changed Rust, scripts, parity docs, lifecycle validation, `just parity`, `just verify-reference`, and relevant targeted tests. Hardware commands require the Ultra 205 detector gate and redaction-safe evidence handling.

### Claude's Discretion

Claude may choose the exact module names, enum names, documentation layout, claim-tier labels, evidence artifact filenames, and test grouping. Those choices must preserve functional core / imperative shell structure, exact-claim governance, stable blocker reason strings, and the repo's secret-redaction rules.

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Phase Scope And Current Milestone

- `.planning/ROADMAP.md` - Phase 22 goal, dependency on Phase 21, requirements, and success criteria.
- `.planning/REQUIREMENTS.md` - EVD-06, SAFE-10, SAFE-11, and v1.1 traceability.
- `.planning/PROJECT.md` - v1.1 Ultra 205 trusted production mining scope, ESP-IDF Rust stack, parity evidence policy, and safety constraints.
- `.planning/STATE.md` - Current project state, accepted v1.1 decisions, and blockers.
- `AGENTS.md` - Ultra 205 detector gate, hardware evidence requirements, local credential handling, redaction rules, and phase-gated destructive/fault-injection limits.
- `AGENTS.bright-builds.md` - Bright Builds workflow, verification, and standards routing requirements.
- `standards/core/architecture.md` - Functional core / imperative shell and parse-boundary guidance.
- `standards/core/code-shape.md` - Early returns, optional naming, and script rerun-safety guidance.
- `standards/core/testing.md` - Unit-test structure and pure logic coverage expectations.
- `standards/core/verification.md` - Repo-native verification before commit.
- `standards/languages/rust.md` - Rust module layout, `maybe_` naming, invariants, and testing expectations.

### Prior Decisions And Evidence

- `.planning/phases/12-asic-and-mining-hardware-evidence/12-CONTEXT.md` - Earlier ASIC/mining evidence ladder, detector gate, controlled no-share semantics, and exact checklist promotion rules.
- `.planning/phases/15-bm1366-mining-evidence-completion/15-CONTEXT.md` - Chip-detect, work/result diagnostics, controlled mining smoke, bounded soak, parity guards, and redaction decisions.
- `.planning/phases/20-active-safety-hardware-telemetry-evidence/20-CONTEXT.md` - Active safety boundaries, live telemetry blockers, and exact non-claim handling.
- `.planning/phases/21-live-mining-and-soak-evidence/21-CONTEXT.md` - Controlled pool gates, Phase 21 mining evidence ladder, safe-stop, telemetry correlation, and exact verification gate.
- `.planning/phases/21-live-mining-and-soak-evidence/21-VERIFICATION.md` - Phase 21 passed status, approved controlled no-share soak closure, residual accepted/rejected share non-claims, and verification commands.
- `docs/parity/evidence/phase-21-live-mining-and-soak-evidence/readiness-audit.md` - Blocked-by-default mining readiness and controlled enablement requirements.
- `docs/parity/evidence/phase-21-live-mining-and-soak-evidence/live-mining-enablement.md` - Controlled runtime enablement markers, safe-state markers, and redaction requirement.
- `docs/parity/evidence/phase-21-live-mining-and-soak-evidence/summary.md` - Final Phase 21 evidence closure and exact non-claim boundaries.
- `docs/parity/checklist.md` - Current ASIC, Stratum, API, statistics, safety, and evidence rows plus safety-critical verification rules.

### Implementation Surfaces

- `crates/bitaxe-safety/src/evidence.rs` - Safety evidence classes and hardware verification labels.
- `crates/bitaxe-safety/src/power.rs` - Power observation freshness, fault reasons, evidence tokens, and fail-closed voltage behavior.
- `crates/bitaxe-safety/src/thermal.rs` - Thermal/fan observations, thermal evidence token behavior, and fail-closed overheat handling.
- `crates/bitaxe-safety/src/status.rs` - Stable public safety reason strings.
- `crates/bitaxe-stratum/src/v1/mining_loop.rs` - Existing fail-closed mining-loop gate and work submission blocker reasons.
- `crates/bitaxe-stratum/src/v1/state.rs` - Runtime mining activity and work-submission state projection.
- `crates/bitaxe-api/src/mining.rs` - API-visible mining status model.
- `firmware/bitaxe/src/controlled_mining_runtime.rs` - Phase 21 controlled runtime shell and current controlled safe-bench gate.
- `firmware/bitaxe/src/asic_adapter/status.rs` - User-visible mining loop blocked status logging.
- `firmware/bitaxe/src/runtime_snapshot.rs` - Runtime status and telemetry integration point.
- `tools/parity/src/mining_allow.rs` - Existing mining allow-manifest claim tiers and safe-state marker validation.
- `tools/parity/src/main.rs` - Checklist validation and verified-row guardrails.

### Upstream Reference And Policy

- `reference/esp-miner/main/power/INA260.c` - Upstream power telemetry source.
- `reference/esp-miner/main/power/DS4432U.c` - Ultra 205 voltage regulator behavior.
- `reference/esp-miner/main/thermal/thermal.c` - Upstream thermal sensor behavior and sentinel handling.
- `reference/esp-miner/main/tasks/fan_controller_task.c` - Fan controller task behavior.
- `reference/esp-miner/main/tasks/power_management_task.c` - Safety stop, cool, and restart policy.
- `reference/esp-miner/main/tasks/protocol_coordinator.c` - Protocol lifecycle coordination and watchdog-sensitive mining behavior.
- `reference/esp-miner/main/system.c` - Accepted/rejected share counters and clean-jobs handling.
- `reference/esp-miner/main/global_state.h` - Pool, share, queue, mining, and runtime status fields.
- `docs/adr/0001-device-user-parity.md` - Observable device-user parity definition.
- `docs/adr/0003-esp-idf-rust-production-stack.md` - ESP-IDF Rust stack and adapter boundary.
- `docs/adr/0005-read-only-reference-implementation.md` - Read-only upstream reference policy.
- `docs/adr/0006-parity-checklist-as-audit-evidence.md` - Checklist evidence policy.
- `docs/adr/0012-parity-verification-evidence.md` - Evidence class and verified status semantics.
- `docs/adr/0013-mit-first-with-gpl-guardrails.md` - GPL provenance guardrails.
- `docs/adr/0014-pivot-to-ultra-205-bm1366-first-parity.md` - Ultra 205/BM1366 first parity target and deferred non-205 scope.
- `PROVENANCE.md` - Reference, GPL, fixture, source-attribution, dependency-license, and release-review policy.

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets

- `crates/bitaxe-stratum/src/v1/mining_loop.rs` already blocks work submission until power, thermal, safety, hardware acknowledgement, and ASIC initialization gates pass.
- `crates/bitaxe-safety/src/power.rs` already models fresh/stale/unavailable/fault power observations with stable reason strings and a 1000 ms stale threshold.
- `crates/bitaxe-safety/src/thermal.rs` already models fresh/unavailable/fault thermal observations, overheat stop behavior, fan decisions, and thermal evidence tokens.
- `crates/bitaxe-safety/src/evidence.rs` already separates missing, unit-only, hardware-smoke, and hardware-regression evidence.
- `tools/parity/src/mining_allow.rs` already validates detector gates, package identity, claim tiers, abort conditions, recovery steps, safe-state markers, and Phase 21 wrapper command shapes.
- `firmware/bitaxe/src/controlled_mining_runtime.rs` already records controlled runtime markers without logging Stratum credentials or raw BM1366 frames.

### Established Patterns

- Pure claim, prerequisite, safety, and parity decisions should live in Rust crates or tools with focused unit tests.
- ESP-IDF, UART/GPIO/I2C, serial capture, HTTP/WebSocket capture, and hardware evidence orchestration remain firmware/tool/script shells.
- Evidence files name exact scope, board `205`, source commit, reference commit, package/evidence paths, command category, observed behavior, redaction status, and residual non-claims.
- GSD artifacts and frontmatter-parsed Markdown must avoid standalone body `---` separators after YAML frontmatter.

### Integration Points

- Add Phase 22 evidence under `docs/parity/evidence/phase-22-claim-ladder-and-safety-preconditions/`.
- Add or extend pure safety/claim modules only where existing `MiningLoopGate`, `SafetyCriticalEvidence`, `PowerObservation`, `ThermalObservation`, and `SafetyStatus` cannot represent v1.1 prerequisite semantics clearly.
- Update `docs/parity/checklist.md` only after Phase 22 artifacts prove EVD-06, SAFE-10, or SAFE-11 at the exact evidence level claimed.
- Keep any Phase 22 hardware use detector-gated and non-destructive; blocked/static/pure evidence is acceptable if it proves the claim and safety precondition contract.

</code_context>

<specifics>
## Specific Ideas

- Preferred artifact set: `claim-ladder.md`, `safety-preconditions.md`, `blocker-reasons.md`, `summary.md`, and `redaction-review.md` under a Phase 22 evidence directory.
- Preferred pure model: a small claim/prerequisite contract that can render operator-facing claim tiers and fail-closed blocker reasons without requiring live hardware.
- Preferred blocker style: stable snake_case reason strings that can be logged, serialized, checked in tests, and cited in evidence.
- Preferred verification style: targeted unit tests for every claim tier and every stale/unavailable/unsafe/ambiguous prerequisite branch, plus `just parity`, `just verify-reference`, and lifecycle validation.

</specifics>

<deferred>
## Deferred Ideas

- Redacted end-to-end operator evidence root belongs to Phase 23.
- Trusted BM1366 production initialization, pool-derived work dispatch, live result mapping, and fail-closed ASIC production errors belong to Phase 24.
- Real Stratum v1 socket lifecycle, accepted/rejected share outcome, deterministic fake-pool production tests, safe stop, and watchdog under load belong to Phase 25.
- API/WebSocket/statistics/scoreboard projection and final parity promotion belong to Phase 26.
- Full active voltage/fan/thermal fault-stimulus, self-test hardware closure, OTA/recovery, runtime display/input, BAP, Stratum v2, non-205 boards, and unbounded stress mining remain future work.

</deferred>

*Phase: 22-claim-ladder-and-safety-preconditions*
*Context gathered: 2026-07-04*
