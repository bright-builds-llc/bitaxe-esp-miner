---
generated_by: gsd-phase-researcher
lifecycle_mode: yolo
phase_lifecycle_id: 22-2026-07-04T20-10-36
generated_at: 2026-07-04T20:11:00Z
---

# Phase 22: Claim Ladder And Safety Preconditions - Research

**Researched:** 2026-07-04 [VERIFIED: local date context]
**Domain:** Ultra 205 production-mining claim governance and fail-closed safety preconditions [VERIFIED: .planning/ROADMAP.md]
**Confidence:** HIGH for repo-local architecture and verification surfaces; MEDIUM for exact implementation names because Phase 22 allows Claude discretion. [VERIFIED: .planning/phases/22-claim-ladder-and-safety-preconditions/22-CONTEXT.md]

<user_constraints>
## User Constraints (from CONTEXT.md)

All constraints in this section are copied from Phase 22 context and must be honored by planning. [VERIFIED: .planning/phases/22-claim-ladder-and-safety-preconditions/22-CONTEXT.md]

### Locked Decisions

#### Claim Ladder

- **D-01:** Define an operator-visible claim ladder with at least these tiers: v1.0 controlled no-share evidence, v1.1 prerequisite readiness, v1.1 live socket/runtime evidence, v1.1 live ASIC-derived share outcome, and explicit deferred/non-claim surfaces.
- **D-02:** The ladder must preserve exact evidence semantics. A lower tier can support readiness or implementation but must not imply accepted shares, rejected shares, unbounded production mining, full active safety closure, non-205 board support, Stratum v2, OTA/recovery trust, display/input parity, or BAP behavior.
- **D-03:** Phase 21's approved controlled no-share soak remains useful prior evidence, but Phase 22 must label it as controlled no-share closure, not live accepted/rejected production-share proof.
- **D-04:** Parity and operator docs should use consistent language for "allowed claim", "blocked claim", and "explicit non-claim" so later phases can promote only the exact tier proved by redacted artifacts.

#### Safety Prerequisite Contract

- **D-05:** Production mining cannot enable BM1366 work dispatch unless fresh or explicitly bounded prerequisite observations exist for power, thermal, fan, voltage, and overall safety state.
- **D-06:** "Fresh" should remain a strict runtime observation when existing code already has one, such as `PowerObservation` age checks. "Explicitly bounded" should be a named evidence contract with an observation source, age or validity window, board `205`, evidence id, and reason why it is acceptable for the current mining attempt.
- **D-07:** Prerequisites must be consumed as typed domain inputs before mining-loop work dispatch, not as freeform strings in shell scripts. Existing `crates/bitaxe-safety` evidence, power, thermal, and status modules are the preferred home for pure decision logic.
- **D-08:** Firmware and scripts may remain thin shells that collect observations and render logs, but they must not bypass the pure prerequisite decision model to start production work.

#### Blocker Reasons And Fail-Closed Behavior

- **D-09:** Missing, stale, unavailable, unsafe, ambiguous, or undocumented prerequisite data must produce a specific stable blocker reason instead of a generic "not ready" result.
- **D-10:** Blocker reasons should remain user-visible through logs, runtime state, API/WebSocket projection when available, and evidence summaries. They must be safe to commit and must not reveal pool credentials, device URLs, IPs, MACs, Wi-Fi values, NVS secrets, raw Stratum payloads, raw share payloads, or raw BM1366 frames.
- **D-11:** The default state remains fail-closed: mining disabled, hardware control disabled, and work submission disabled. Any unsafe, ambiguous, stale, or undocumented prerequisite keeps this state and records the blocker.
- **D-12:** The blocker taxonomy should align with existing names where possible, including `power_sample_stale`, `power_sample_unavailable`, `thermal_reading_unavailable`, `thermal_reading_invalid`, `hardware_evidence_ack_missing`, and `safety_preflight_evidence_missing`.

#### Evidence And Checklist Boundaries

- **D-13:** Phase 22 may add claim-ladder documentation, prerequisite contract tests, parity guard tests, and blocked evidence artifacts. It should not claim new live hardware behavior unless a detector-gated, redacted, phase-approved command actually produced that evidence.
- **D-14:** Checklist updates must be conservative. EVD-06, SAFE-10, and SAFE-11 can advance only to the level supported by docs, pure tests, workflow checks, or hardware evidence actually produced in this phase.
- **D-15:** Explicit non-claims for full active voltage control, fan actuation, thermal fault stimulus, self-test hardware closure, fault-stimulus closure, unbounded soak, accepted/rejected shares, non-205 boards, OTA/recovery, runtime display/input, BAP, and Stratum v2 must remain visible.
- **D-16:** Verification must include repo-native checks for changed Rust, scripts, parity docs, lifecycle validation, `just parity`, `just verify-reference`, and relevant targeted tests. Hardware commands require the Ultra 205 detector gate and redaction-safe evidence handling.

### Claude's Discretion

Claude may choose the exact module names, enum names, documentation layout, claim-tier labels, evidence artifact filenames, and test grouping. Those choices must preserve functional core / imperative shell structure, exact-claim governance, stable blocker reason strings, and the repo's secret-redaction rules.

### Deferred Ideas (OUT OF SCOPE)

- Redacted end-to-end operator evidence root belongs to Phase 23.
- Trusted BM1366 production initialization, pool-derived work dispatch, live result mapping, and fail-closed ASIC production errors belong to Phase 24.
- Real Stratum v1 socket lifecycle, accepted/rejected share outcome, deterministic fake-pool production tests, safe stop, and watchdog under load belong to Phase 25.
- API/WebSocket/statistics/scoreboard projection and final parity promotion belong to Phase 26.
- Full active voltage/fan/thermal fault-stimulus, self-test hardware closure, OTA/recovery, runtime display/input, BAP, Stratum v2, non-205 boards, and unbounded stress mining remain future work.
</user_constraints>

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
| --- | --- | --- |
| EVD-06 | An Ultra 205 operator can distinguish v1.0 controlled no-share evidence from v1.1 live production mining claims through a documented claim ladder. [VERIFIED: .planning/REQUIREMENTS.md] | Use a documented claim ladder plus parity guard tests that forbid Phase 21 controlled no-share evidence from implying accepted/rejected production shares. [VERIFIED: .planning/phases/21-live-mining-and-soak-evidence/21-VERIFICATION.md] [VERIFIED: docs/parity/evidence/phase-21-live-mining-and-soak-evidence/summary.md] |
| SAFE-10 | Ultra 205 production mining requires fresh or explicitly bounded power, thermal, fan, voltage, and safety observations before BM1366 work dispatch is enabled. [VERIFIED: .planning/REQUIREMENTS.md] | Extend the existing typed safety/mining gate model instead of shell strings; current `MiningLoopGate` already blocks missing power, thermal, safety, hardware ack, and ASIC init before dispatch. [VERIFIED: crates/bitaxe-stratum/src/v1/mining_loop.rs] |
| SAFE-11 | Ultra 205 production mining fails closed with user-visible blocker reasons when safety prerequisites are stale, unavailable, unsafe, ambiguous, or undocumented. [VERIFIED: .planning/REQUIREMENTS.md] | Build a stable blocker taxonomy over existing reason strings from power, thermal, status, mining loop, API mapping, and firmware status logging. [VERIFIED: crates/bitaxe-safety/src/power.rs] [VERIFIED: crates/bitaxe-safety/src/thermal.rs] [VERIFIED: firmware/bitaxe/src/asic_adapter/status.rs] |
</phase_requirements>

## Summary

Phase 22 should plan a small typed prerequisite and claim-governance layer, not a new production mining runtime. The codebase already has fail-closed power, thermal, safety, hardware-ack, and ASIC-init gates; the missing Phase 22 work is to make the v1.1 claim ladder explicit for operators and to make prerequisite readiness/blocker decisions precise enough for later production phases to consume safely. [VERIFIED: crates/bitaxe-stratum/src/v1/mining_loop.rs] [VERIFIED: .planning/phases/22-claim-ladder-and-safety-preconditions/22-CONTEXT.md]

The strongest implementation path is to put pure prerequisite decision logic in `crates/bitaxe-safety`, let `crates/bitaxe-stratum` consume that result before BM1366 work dispatch, and let firmware/scripts/docs only collect observations, render stable blocker strings, and write redaction-safe evidence. [VERIFIED: standards/core/architecture.md] [VERIFIED: standards/languages/rust.md] [VERIFIED: .planning/phases/22-claim-ladder-and-safety-preconditions/22-CONTEXT.md]

**Primary recommendation:** Add a typed `production mining preconditions` model that accepts fresh or explicitly bounded observations for power, thermal, fan, voltage, and safety, returns either `Ready` or a stable fail-closed blocker reason, and is covered by unit tests and parity/evidence guard tests before any live mining promotion. [VERIFIED: .planning/phases/22-claim-ladder-and-safety-preconditions/22-CONTEXT.md] [VERIFIED: crates/bitaxe-safety/src/power.rs] [VERIFIED: crates/bitaxe-stratum/src/v1/mining_loop.rs]

## Project Constraints (from .cursor/rules/)

No `.cursor/rules/` files were found in the repository during research. [VERIFIED: Glob .cursor/rules/**/*]

The materially applicable repo constraints instead come from `AGENTS.md`, `AGENTS.bright-builds.md`, `standards/*`, and Phase 22 context: keep `reference/esp-miner` read-only, follow functional core / imperative shell, use typed domain values over raw strings, unit test pure logic, preserve Ultra 205 detector/evidence/redaction gates, and avoid standalone body `---` separators in parsed Markdown. [VERIFIED: AGENTS.md] [VERIFIED: AGENTS.bright-builds.md] [VERIFIED: standards/core/architecture.md] [VERIFIED: standards/core/testing.md] [VERIFIED: .planning/phases/22-claim-ladder-and-safety-preconditions/22-CONTEXT.md]

No `.cursor/skills/` or `.agents/skills/` project skills were found during research. [VERIFIED: Glob .cursor/skills/**/SKILL.md] [VERIFIED: Glob .agents/skills/**/SKILL.md]

## Standard Stack

### Core

| Library / Surface | Version | Purpose | Why Standard |
| --- | --- | --- | --- |
| `crates/bitaxe-safety` | Workspace crate, Rust 2021 [VERIFIED: crates/bitaxe-safety/BUILD.bazel] | Own pure safety evidence, power, thermal, status, and effect decisions. [VERIFIED: crates/bitaxe-safety/src/evidence.rs] [VERIFIED: crates/bitaxe-safety/src/power.rs] [VERIFIED: crates/bitaxe-safety/src/thermal.rs] | Phase 22 context names this as the preferred home for pure prerequisite decision logic. [VERIFIED: .planning/phases/22-claim-ladder-and-safety-preconditions/22-CONTEXT.md] |
| `crates/bitaxe-stratum` | Workspace crate, Rust 2021 [VERIFIED: crates/bitaxe-stratum/BUILD.bazel] | Consume safety readiness before mining-loop dispatch and keep work submission blocked unless all gates pass. [VERIFIED: crates/bitaxe-stratum/src/v1/mining_loop.rs] | Existing `MiningLoopGate` already models fail-closed work-dispatch readiness. [VERIFIED: crates/bitaxe-stratum/src/v1/mining_loop.rs] |
| `crates/bitaxe-api` | Workspace crate, Rust 2021 [VERIFIED: crates/bitaxe-api/BUILD.bazel] | Project mining state and blocker reason into API-compatible wire data. [VERIFIED: crates/bitaxe-api/src/mining.rs] | Phase 22 requires user-visible blocker reasons through API/WebSocket when available. [VERIFIED: .planning/phases/22-claim-ladder-and-safety-preconditions/22-CONTEXT.md] |
| `tools/parity` | Workspace binary/test target, Rust 2021 [VERIFIED: tools/parity/BUILD.bazel] | Enforce exact-claim evidence guardrails, claim tiers, safety evidence classes, and checklist validation. [VERIFIED: tools/parity/src/main.rs] [VERIFIED: tools/parity/src/mining_allow.rs] [VERIFIED: tools/parity/src/safety_allow.rs] | Existing parity tooling already rejects overbroad verified safety and mining claims. [VERIFIED: tools/parity/src/main.rs] |

### Supporting

| Library / Surface | Version | Purpose | When to Use |
| --- | --- | --- | --- |
| `firmware/bitaxe` | Bazel genrule around ESP-IDF Rust firmware [VERIFIED: firmware/bitaxe/BUILD.bazel] | Collect runtime observations and render logs/status without owning prerequisite business rules. [VERIFIED: firmware/bitaxe/src/safety_adapter.rs] [VERIFIED: firmware/bitaxe/src/controlled_mining_runtime.rs] | Use only as the imperative shell around pure safety/mining decisions. [VERIFIED: standards/core/architecture.md] |
| `docs/parity/evidence/phase-22-claim-ladder-and-safety-preconditions/` | New evidence directory [VERIFIED: .planning/phases/22-claim-ladder-and-safety-preconditions/22-CONTEXT.md] | Store claim ladder, safety preconditions, blocker reasons, summary, and redaction review artifacts. [VERIFIED: .planning/phases/22-claim-ladder-and-safety-preconditions/22-CONTEXT.md] | Use for operator-facing evidence and exact non-claims produced by this phase. [VERIFIED: .planning/phases/22-claim-ladder-and-safety-preconditions/22-CONTEXT.md] |
| `Justfile` commands | `just 1.48.0` available locally [VERIFIED: local command probe] | Run repo-native build/test/parity/reference workflows. [VERIFIED: Justfile] | Use for phase verification: `just parity`, `just verify-reference`, and full `just test`. [VERIFIED: Justfile] |

### Alternatives Considered

| Instead of | Could Use | Tradeoff |
| --- | --- | --- |
| `crates/bitaxe-safety` prerequisite model | Shell-script preflight checks | Rejected because Phase 22 explicitly requires typed domain inputs before work dispatch, not freeform shell strings. [VERIFIED: .planning/phases/22-claim-ladder-and-safety-preconditions/22-CONTEXT.md] |
| Shared stable blocker enum/string constants | Ad hoc log strings in firmware/scripts | Rejected because Phase 22 requires stable user-visible blocker reasons and redaction-safe evidence summaries. [VERIFIED: .planning/phases/22-claim-ladder-and-safety-preconditions/22-CONTEXT.md] |
| Conservative docs/checklist promotion | Promoting Phase 21 controlled no-share evidence as production share proof | Rejected because Phase 21 explicitly did not observe accepted or rejected shares. [VERIFIED: .planning/phases/21-live-mining-and-soak-evidence/21-VERIFICATION.md] [VERIFIED: docs/parity/evidence/phase-21-live-mining-and-soak-evidence/summary.md] |

**Installation:** No new package installation is needed for the recommended stack; this phase should use existing workspace crates, Bazel targets, `just`, and docs surfaces. [VERIFIED: crates/bitaxe-safety/BUILD.bazel] [VERIFIED: crates/bitaxe-stratum/BUILD.bazel] [VERIFIED: tools/parity/BUILD.bazel] [VERIFIED: Justfile]

## Architecture Patterns

### Recommended Project Structure

```text
crates/bitaxe-safety/src/
  mining_preconditions.rs   # New pure prerequisite contract for production mining readiness. [VERIFIED: .planning/phases/22-claim-ladder-and-safety-preconditions/22-CONTEXT.md]
  evidence.rs               # Existing evidence classes and hardware verification labels. [VERIFIED: crates/bitaxe-safety/src/evidence.rs]
  power.rs                  # Existing fresh/stale/unavailable/fault power observations. [VERIFIED: crates/bitaxe-safety/src/power.rs]
  thermal.rs                # Existing thermal/fan decisions and evidence tokens. [VERIFIED: crates/bitaxe-safety/src/thermal.rs]

crates/bitaxe-stratum/src/v1/
  mining_loop.rs            # Existing work-dispatch gate consumes precondition result. [VERIFIED: crates/bitaxe-stratum/src/v1/mining_loop.rs]
  state.rs                  # Existing blocked/ready runtime state projection. [VERIFIED: crates/bitaxe-stratum/src/v1/state.rs]

tools/parity/src/
  claim_ladder.rs           # Optional new exact-claim doc/checklist guard helpers. [VERIFIED: tools/parity/src/mining_allow.rs]
  mining_allow.rs           # Existing mining allow-manifest validator to extend if needed. [VERIFIED: tools/parity/src/mining_allow.rs]
  safety_allow.rs           # Existing safety allow-manifest validator to reuse for non-claims. [VERIFIED: tools/parity/src/safety_allow.rs]

docs/parity/evidence/phase-22-claim-ladder-and-safety-preconditions/
  claim-ladder.md
  safety-preconditions.md
  blocker-reasons.md
  redaction-review.md
  summary.md
```

### Pattern 1: Typed Prerequisite Contract

**What:** Parse raw observations and evidence facts into a pure Rust input struct such as `ProductionMiningPreconditions`, then return `Ready` only when each prerequisite is fresh or explicitly bounded and board-scoped to `205`. [VERIFIED: .planning/phases/22-claim-ladder-and-safety-preconditions/22-CONTEXT.md]

**When to use:** Use this before any BM1366 production work dispatch or later full ASIC production initialization path. [VERIFIED: .planning/ROADMAP.md] [VERIFIED: crates/bitaxe-stratum/src/v1/mining_loop.rs]

**Recommended shape:**

```rust
// Source: repo-local pattern from crates/bitaxe-safety/src/power.rs and
// crates/bitaxe-stratum/src/v1/mining_loop.rs.
pub enum ProductionMiningPreconditionDecision {
    Ready,
    Blocked { reason: &'static str },
}

pub struct BoundedObservationEvidence {
    pub source: &'static str,
    pub board: &'static str,
    pub evidence_id: &'static str,
    pub validity_window_ms: u32,
    pub reason: &'static str,
}
```

### Pattern 2: Stable Blocker Reason Taxonomy

**What:** Keep stable snake_case reason strings as the public contract and test them as serialized/loggable values. Existing strings include `power_sample_stale`, `power_sample_unavailable`, `thermal_reading_unavailable`, `thermal_reading_invalid`, `hardware_evidence_ack_missing`, `power_preflight_evidence_missing`, `thermal_preflight_evidence_missing`, and `safety_preflight_evidence_missing`. [VERIFIED: crates/bitaxe-safety/src/power.rs] [VERIFIED: crates/bitaxe-safety/src/thermal.rs] [VERIFIED: crates/bitaxe-stratum/src/v1/mining_loop.rs]

**When to use:** Use whenever prerequisites are missing, stale, unavailable, unsafe, ambiguous, or undocumented; do not collapse these states into a generic "not ready" reason. [VERIFIED: .planning/phases/22-claim-ladder-and-safety-preconditions/22-CONTEXT.md]

**Recommended additions:** Add only the missing reason strings that Phase 22 needs for fan, voltage, ambiguous bounded evidence, and undocumented bounded evidence. Recommended candidates are `fan_observation_unavailable`, `fan_observation_stale`, `voltage_observation_unavailable`, `voltage_observation_stale`, `bounded_observation_ambiguous`, and `bounded_observation_undocumented`. These names follow existing snake_case public reason style. [VERIFIED: crates/bitaxe-safety/src/status.rs] [VERIFIED: crates/bitaxe-safety/src/power.rs]

### Pattern 3: Claim Ladder As Evidence Governance

**What:** Treat the claim ladder as an operator-visible evidence contract, not as a mining runtime feature. It should define allowed claims, blocked claims, and explicit non-claims for v1.0 controlled no-share, v1.1 prerequisite readiness, v1.1 live socket/runtime, v1.1 live ASIC-derived share outcome, and deferred surfaces. [VERIFIED: .planning/phases/22-claim-ladder-and-safety-preconditions/22-CONTEXT.md]

**When to use:** Use in parity docs, evidence summaries, checklist notes, and parity tests so later phases can promote only the exact tier proved by artifacts. [VERIFIED: docs/adr/0006-parity-checklist-as-audit-evidence.md] [VERIFIED: tools/parity/src/main.rs]

### Anti-Patterns to Avoid

- **Claim promotion by implication:** Do not let Phase 21 controlled no-share soak imply accepted shares, rejected shares, unbounded production mining, or full active safety closure. [VERIFIED: .planning/phases/21-live-mining-and-soak-evidence/21-VERIFICATION.md]
- **Shell-owned safety decisions:** Do not make scripts decide whether production work dispatch is safe; scripts may collect observations and render evidence only. [VERIFIED: .planning/phases/22-claim-ladder-and-safety-preconditions/22-CONTEXT.md]
- **Firmware-owned prerequisite business logic:** Do not bury prerequisite branching in ESP-IDF firmware loops; firmware should call pure decision logic and render outcomes. [VERIFIED: standards/core/architecture.md] [VERIFIED: firmware/bitaxe/src/safety_adapter.rs]
- **Secret-bearing blocker details:** Do not include pool URLs, workers, owner addresses, passwords, raw Stratum payloads, raw share payloads, device URLs, IPs, MACs, Wi-Fi values, or NVS secrets in blocker reasons or evidence. [VERIFIED: AGENTS.md] [VERIFIED: .planning/phases/22-claim-ladder-and-safety-preconditions/22-CONTEXT.md]

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
| --- | --- | --- | --- |
| Power freshness and unsafe input checks | New voltage/current threshold code in scripts | `PowerObservation::from_ina260_sample` and `PowerSafetyDecision` [VERIFIED: crates/bitaxe-safety/src/power.rs] | Existing code already handles stale, unavailable, invalid, unsafe voltage, and power-limit reasons. [VERIFIED: crates/bitaxe-safety/src/power.rs] |
| Thermal availability and invalid reading checks | New string checks in firmware | `ThermalObservation::from_reading` and `ThermalEvidenceToken` [VERIFIED: crates/bitaxe-safety/src/thermal.rs] | Existing code already models unavailable/invalid readings and fresh-safe evidence tokens. [VERIFIED: crates/bitaxe-safety/src/thermal.rs] |
| Work-dispatch gating | A new "start mining" boolean | `MiningLoopGate` plus the new precondition decision output [VERIFIED: crates/bitaxe-stratum/src/v1/mining_loop.rs] | Existing gate already defaults to blocked and refuses work submission until prerequisite evidence is present. [VERIFIED: crates/bitaxe-stratum/src/v1/mining_loop.rs] |
| Claim-tier validation | Markdown-only claims without tests | `tools/parity` validators and targeted tests [VERIFIED: tools/parity/src/main.rs] [VERIFIED: tools/parity/src/mining_allow.rs] | Parity tooling already validates safety-critical evidence classes, live mining blocker language, and allowed claim tiers. [VERIFIED: tools/parity/src/main.rs] |
| Redaction | Manual "looks safe" review only | Existing redaction review pattern plus forbidden-value tests for new renderers [VERIFIED: firmware/bitaxe/src/controlled_mining_runtime.rs] [VERIFIED: docs/parity/evidence/phase-21-live-mining-and-soak-evidence/summary.md] | Phase 21 tests already prove redacted markers do not expose pool identity or raw frames. [VERIFIED: firmware/bitaxe/src/controlled_mining_runtime.rs] |

**Key insight:** Phase 22 should make the safety and claim contracts more explicit; it should not implement the Phase 24/25 production work path early. [VERIFIED: .planning/phases/22-claim-ladder-and-safety-preconditions/22-CONTEXT.md]

## Common Pitfalls

### Pitfall 1: Treating Controlled No-Share As Production Share Proof

**What goes wrong:** A checklist or summary says Phase 21 proves live accepted/rejected production shares because it observed subscribe/authorize/notify and typed BM1366 work dispatch. [VERIFIED: docs/parity/evidence/phase-21-live-mining-and-soak-evidence/summary.md]

**Why it happens:** Phase 21 has live smoke and bounded soak artifacts, but the closure explicitly remains `approved_controlled_no_share_soak`. [VERIFIED: .planning/phases/21-live-mining-and-soak-evidence/21-VERIFICATION.md]

**How to avoid:** The claim ladder must name Phase 21 as v1.0 controlled no-share closure and list accepted shares, rejected shares, full production mining, and unbounded soak as non-claims. [VERIFIED: .planning/phases/22-claim-ladder-and-safety-preconditions/22-CONTEXT.md]

**Warning signs:** Checklist notes contain "accepted", "rejected", "production share", or "verified production mining" without an artifact that actually observed a parsed pool response to ASIC-derived work. [VERIFIED: .planning/REQUIREMENTS.md]

### Pitfall 2: Freshness Without A Clock Or Validity Window

**What goes wrong:** The implementation accepts a prerequisite because a value exists, even though no timestamp, age threshold, or bounded evidence window proves it is current enough for the mining attempt. [VERIFIED: .planning/phases/22-claim-ladder-and-safety-preconditions/22-CONTEXT.md]

**Why it happens:** `PowerObservation` already has an age check, but thermal/fan/voltage may need fresh runtime metadata or an explicit bounded evidence contract. [VERIFIED: crates/bitaxe-safety/src/power.rs] [VERIFIED: crates/bitaxe-safety/src/thermal.rs]

**How to avoid:** Require either fresh runtime observation metadata or `BoundedObservationEvidence` with source, age or validity window, board `205`, evidence id, and reason. [VERIFIED: .planning/phases/22-claim-ladder-and-safety-preconditions/22-CONTEXT.md]

**Warning signs:** Tests only cover `Some(token)` versus `None` and do not cover stale, ambiguous, undocumented, or board-mismatched bounded evidence. [VERIFIED: standards/core/testing.md]

### Pitfall 3: API Blocker Reason Drift

**What goes wrong:** Logs show one blocker reason, runtime state stores another, and API/WebSocket projection falls back to `hardware_evidence_ack_missing` for every blocked state. [VERIFIED: crates/bitaxe-api/src/mining.rs]

**Why it happens:** `MiningRuntimeState` currently stores activity and work gate status, but not a specific blocked reason; the API mapper currently returns `hardware_evidence_ack_missing` for safe-blocked work submission. [VERIFIED: crates/bitaxe-stratum/src/v1/state.rs] [VERIFIED: crates/bitaxe-api/src/mining.rs]

**How to avoid:** Plan a single typed blocked-reason source that can be applied to mining runtime state and rendered by logs/API/evidence summaries. [VERIFIED: .planning/phases/22-claim-ladder-and-safety-preconditions/22-CONTEXT.md]

**Warning signs:** Unit tests assert only `WorkSubmissionGate::Blocked` and not the exact reason string. [VERIFIED: crates/bitaxe-stratum/src/v1/state.rs]

### Pitfall 4: Accidentally Verifying Full Active Safety

**What goes wrong:** Phase 22 prerequisite readiness language is read as verified active DS4432U voltage actuation, fan duty response, overheat/fault stimulus, or self-test hardware closure. [VERIFIED: docs/parity/evidence/phase-20-active-safety-hardware-telemetry-evidence/summary.md]

**Why it happens:** The same power, thermal, fan, voltage, and safety words appear in prerequisite readiness and full active safety closure, but Phase 22 owns only prerequisite gates. [VERIFIED: .planning/ROADMAP.md]

**How to avoid:** Evidence summaries and checklist notes must preserve explicit non-claims for full active voltage, fan, thermal, self-test, and fault-stimulus closure. [VERIFIED: .planning/phases/22-claim-ladder-and-safety-preconditions/22-CONTEXT.md]

**Warning signs:** `PWR-003`, `PWR-005`, `THR-001`, `THR-002`, or `SELF-001` moves to `verified` without `hardware-regression` evidence. [VERIFIED: tools/parity/src/main.rs] [VERIFIED: tools/parity/src/safety_allow.rs]

## Code Examples

Verified patterns from repo-local sources:

### Existing Power Freshness Pattern

```rust
// Source: crates/bitaxe-safety/src/power.rs
if age.0 > POWER_SAMPLE_STALE_AFTER_MS {
    return Self::with_status(
        sample,
        PowerObservationStatus::Stale {
            reason: "power_sample_stale",
        },
    );
}
```

### Existing Mining Gate Pattern

```rust
// Source: crates/bitaxe-stratum/src/v1/mining_loop.rs
if self.maybe_power_evidence.is_none() {
    return MiningLoopDecision::Blocked {
        reason: POWER_PREFLIGHT_EVIDENCE_MISSING,
    };
}
```

### Existing Fail-Closed Effect Pattern

```rust
// Source: crates/bitaxe-safety/src/effects.rs
Self {
    status,
    effects: vec![
        SafetyEffect::HoldResetLow,
        SafetyEffect::DisableAsicEnable,
        SafetyEffect::SuppressVoltageWrite,
        SafetyEffect::BlockWorkSubmission { reason },
        SafetyEffect::PublishStatus(status),
    ],
    evidence: SafetyCriticalEvidence::Missing,
}
```

## State of the Art

| Old / Current Approach | Phase 22 Current Approach | When Changed | Impact |
| --- | --- | --- | --- |
| v1.0 controlled no-share evidence can support mining foundation confidence but not production share outcomes. [VERIFIED: docs/parity/evidence/phase-21-live-mining-and-soak-evidence/summary.md] | v1.1 claim ladder names exact tier boundaries and explicit non-claims. [VERIFIED: .planning/phases/22-claim-ladder-and-safety-preconditions/22-CONTEXT.md] | Phase 22 planning boundary. [VERIFIED: .planning/ROADMAP.md] | Operators can tell readiness evidence from live share evidence. [VERIFIED: .planning/REQUIREMENTS.md] |
| Existing `MiningLoopGate` checks missing evidence tokens and hardware ack. [VERIFIED: crates/bitaxe-stratum/src/v1/mining_loop.rs] | Add an upstream typed prerequisite contract for fresh or bounded power, thermal, fan, voltage, and safety observations. [VERIFIED: .planning/phases/22-claim-ladder-and-safety-preconditions/22-CONTEXT.md] | Phase 22. [VERIFIED: .planning/ROADMAP.md] | Later BM1366 production dispatch can consume one stable preflight result. [VERIFIED: .planning/research/ARCHITECTURE.md] |
| Active safety evidence remains below verified for voltage actuation, fan response, fault stimulus, and self-test hardware closure. [VERIFIED: docs/parity/evidence/phase-20-active-safety-hardware-telemetry-evidence/summary.md] | Phase 22 may prove prerequisite readiness and blockers without claiming full active safety closure. [VERIFIED: .planning/phases/22-claim-ladder-and-safety-preconditions/22-CONTEXT.md] | Phase 22. [VERIFIED: .planning/ROADMAP.md] | Checklist updates stay conservative and exact. [VERIFIED: .planning/REQUIREMENTS.md] |

**Deprecated/outdated for this phase:**

- Generic "not ready" blockers are inadequate because Phase 22 requires specific stable blocker reasons for stale, unavailable, unsafe, ambiguous, and undocumented prerequisites. [VERIFIED: .planning/phases/22-claim-ladder-and-safety-preconditions/22-CONTEXT.md]
- Production readiness based only on docs or shell preflight is inadequate because Phase 22 requires typed domain inputs before work dispatch. [VERIFIED: .planning/phases/22-claim-ladder-and-safety-preconditions/22-CONTEXT.md]

## Assumptions Log

All claims in this research were verified against repo-local files, local command probes, or the Phase 22 prompt; no `[ASSUMED]` claims are intentionally used. [VERIFIED: research session tool outputs]

| # | Claim | Section | Risk if Wrong |
| --- | --- | --- | --- |
| None | No unverified assumptions recorded. [VERIFIED: research session tool outputs] | All sections | None. |

## Open Questions (RESOLVED)

1. **RESOLVED: Phase 22 adds runtime state storage for the exact blocked reason and API projection of that same reason.** [VERIFIED: crates/bitaxe-api/src/mining.rs] [VERIFIED: .planning/phases/22-claim-ladder-and-safety-preconditions/22-02-PLAN.md]
   - What we know: API mapping currently exposes `blocked_reason`, but the mapper returns `hardware_evidence_ack_missing` for any safe-blocked work submission. [VERIFIED: crates/bitaxe-api/src/mining.rs]
   - Resolution: Plan 22-02 Task 2 updates `MiningRuntimeState` with `maybe_blocked_reason`, records exact fail-closed reasons through `block_work_submission(reason)`, and projects that value through `crates/bitaxe-api/src/mining.rs`. [VERIFIED: .planning/phases/22-claim-ladder-and-safety-preconditions/22-02-PLAN.md]
   - Boundary: Broader API/WebSocket/statistics/scoreboard promotion remains deferred to Phase 26; Phase 22 only stores and projects the exact blocked reason needed for SAFE-11. [VERIFIED: .planning/phases/22-claim-ladder-and-safety-preconditions/22-CONTEXT.md]

2. **RESOLVED: Bounded observations are accepted through the typed evidence contract for every production-mining prerequisite, while fresh runtime observations remain strict where they already exist.** [VERIFIED: .planning/phases/22-claim-ladder-and-safety-preconditions/22-CONTEXT.md] [VERIFIED: .planning/phases/22-claim-ladder-and-safety-preconditions/22-02-PLAN.md]
   - What we know: D-06 says fresh remains strict when existing code already has it, and explicitly bounded evidence is a named contract. [VERIFIED: .planning/phases/22-claim-ladder-and-safety-preconditions/22-CONTEXT.md]
   - Resolution: Plan 22-02 Task 1 defines `BoundedObservationEvidence` and requires board `205`, nonempty source, nonempty evidence id, nonzero validity window, and nonempty reason before bounded evidence can satisfy power, thermal, fan, voltage, or safety readiness. [VERIFIED: .planning/phases/22-claim-ladder-and-safety-preconditions/22-02-PLAN.md]
   - Boundary: Phase 22 requires the typed validity-window field but does not hard-code production timing policy beyond existing runtime freshness checks such as `POWER_SAMPLE_STALE_AFTER_MS = 1000`. [VERIFIED: crates/bitaxe-safety/src/power.rs] [VERIFIED: .planning/phases/22-claim-ladder-and-safety-preconditions/22-02-PLAN.md]

## Environment Availability

| Dependency | Required By | Available | Version | Fallback |
| --- | --- | --- | --- | --- |
| Node.js | GSD lifecycle and script tooling [VERIFIED: .planning/config.json] | yes [VERIFIED: local command probe] | `v24.13.0` [VERIFIED: local command probe] | None needed. |
| `just` | Repo command surface [VERIFIED: Justfile] | yes [VERIFIED: local command probe] | `just 1.48.0` [VERIFIED: local command probe] | Use underlying Bazel/script commands only for diagnostics. [VERIFIED: Justfile] |
| Bazel | Canonical automation graph [VERIFIED: .planning/PROJECT.md] | yes [VERIFIED: local command probe] | `bazel 9.1.1` [VERIFIED: local command probe] | Blocking if missing for repo-native verification. [VERIFIED: Justfile] |
| Cargo | Rust targeted tests and optional diagnostics [VERIFIED: standards/languages/rust.md] | yes [VERIFIED: local command probe] | `cargo 1.88.0-nightly` [VERIFIED: local command probe] | Prefer Bazel test targets when possible. [VERIFIED: Justfile] |
| Rustc | Rust compilation [VERIFIED: crates/bitaxe-safety/BUILD.bazel] | yes [VERIFIED: local command probe] | `rustc 1.88.0-nightly` [VERIFIED: local command probe] | Blocking if missing for Rust changes. |
| `rg` | Repo searches and redaction scans [VERIFIED: tools/parity/src/mining_allow.rs] | yes [VERIFIED: local command probe] | `ripgrep 15.1.0` [VERIFIED: local command probe] | Use repo scripts only if they avoid new dependencies. |
| Git | Reference cleanliness, source/reference commits [VERIFIED: tools/parity/src/main.rs] | yes [VERIFIED: local command probe] | `git 2.53.0` [VERIFIED: local command probe] | Blocking for parity/reference checks. |
| `espflash` | Detector-gated hardware board-info and flashing [VERIFIED: AGENTS.md] | yes [VERIFIED: local command probe] | `espflash 4.0.1` [VERIFIED: local command probe] | Pure/static Phase 22 evidence can proceed without hardware; hardware claims remain blocked unless `just detect-ultra205` passes. [VERIFIED: AGENTS.md] |
| Ultra 205 USB hardware | Any new hardware evidence [VERIFIED: AGENTS.md] | not probed during research [VERIFIED: research session did not run `just detect-ultra205`] | unknown | Record hardware evidence as pending/blocked unless an active plan runs the detector gate. [VERIFIED: AGENTS.md] |

**Missing dependencies with no fallback:**

- None for pure docs, Rust unit tests, parity tests, and lifecycle research/planning. [VERIFIED: local command probe] [VERIFIED: Justfile]

**Missing dependencies with fallback:**

- Ultra 205 hardware availability was not probed during research; Phase 22 can still produce claim-ladder docs, pure prerequisite tests, parity guard tests, and blocked/non-claim evidence without making new hardware claims. [VERIFIED: .planning/phases/22-claim-ladder-and-safety-preconditions/22-CONTEXT.md]

## Validation Architecture

`workflow.nyquist_validation` is enabled, so the planner should include explicit test mapping. [VERIFIED: .planning/config.json]

### Test Framework

| Property | Value |
| --- | --- |
| Framework | Bazel `rust_test` targets for Rust crates/tools, plus repo `just` aggregate commands. [VERIFIED: crates/bitaxe-safety/BUILD.bazel] [VERIFIED: tools/parity/BUILD.bazel] [VERIFIED: Justfile] |
| Config file | `BUILD.bazel` files per crate/tool and root `Justfile`. [VERIFIED: crates/bitaxe-safety/BUILD.bazel] [VERIFIED: crates/bitaxe-stratum/BUILD.bazel] [VERIFIED: tools/parity/BUILD.bazel] [VERIFIED: Justfile] |
| Quick run command | `bazel test //crates/bitaxe-safety:tests //crates/bitaxe-stratum:tests //crates/bitaxe-api:tests //tools/parity:tests` [VERIFIED: BUILD.bazel files] |
| Full suite command | `just test` [VERIFIED: Justfile] |
| Evidence gates | `just parity` and `just verify-reference` [VERIFIED: Justfile] |

### Phase Requirements To Test Map

| Req ID | Behavior | Test Type | Automated Command | File Exists? |
| --- | --- | --- | --- | --- |
| EVD-06 | Claim ladder distinguishes v1.0 controlled no-share, v1.1 prerequisite readiness, v1.1 live socket/runtime, v1.1 ASIC-derived share outcome, and explicit non-claims. [VERIFIED: .planning/REQUIREMENTS.md] | parity/doc guard | `bazel test //tools/parity:tests` and `just parity` [VERIFIED: tools/parity/BUILD.bazel] [VERIFIED: Justfile] | Existing target yes; likely new `tools/parity` tests and Phase 22 docs needed. [VERIFIED: tools/parity/BUILD.bazel] |
| SAFE-10 | Production mining readiness requires fresh or explicitly bounded power, thermal, fan, voltage, and safety observations before BM1366 work dispatch. [VERIFIED: .planning/REQUIREMENTS.md] | unit | `bazel test //crates/bitaxe-safety:tests //crates/bitaxe-stratum:tests` [VERIFIED: crates/bitaxe-safety/BUILD.bazel] [VERIFIED: crates/bitaxe-stratum/BUILD.bazel] | Existing targets yes; new safety module/tests likely needed. [VERIFIED: crates/bitaxe-safety/BUILD.bazel] |
| SAFE-11 | Missing, stale, unavailable, unsafe, ambiguous, or undocumented prerequisites fail closed with user-visible specific blocker reasons. [VERIFIED: .planning/REQUIREMENTS.md] | unit + API/projection + evidence guard | `bazel test //crates/bitaxe-safety:tests //crates/bitaxe-stratum:tests //crates/bitaxe-api:tests //tools/parity:tests` [VERIFIED: BUILD.bazel files] | Existing targets yes; exact blocked-reason propagation tests likely needed. [VERIFIED: crates/bitaxe-api/BUILD.bazel] |

### Sampling Rate

- **Per task commit:** Run the narrow Bazel target for the changed crate/tool, plus `just parity` when docs/checklist/evidence surfaces change. [VERIFIED: standards/core/verification.md] [VERIFIED: Justfile]
- **Per wave merge:** Run `bazel test //crates/bitaxe-safety:tests //crates/bitaxe-stratum:tests //crates/bitaxe-api:tests //tools/parity:tests`, `just parity`, and `just verify-reference`. [VERIFIED: BUILD.bazel files] [VERIFIED: Justfile]
- **Phase gate:** Run `just test`, `just parity`, `just verify-reference`, lifecycle validation, and any detector-gated hardware command that the phase actually used. [VERIFIED: .planning/phases/22-claim-ladder-and-safety-preconditions/22-CONTEXT.md] [VERIFIED: Justfile]

### Wave 0 Gaps

- [ ] `crates/bitaxe-safety/src/mining_preconditions.rs` or equivalent new module for typed production-mining prerequisite readiness. [VERIFIED: .planning/phases/22-claim-ladder-and-safety-preconditions/22-CONTEXT.md]
- [ ] Tests covering every required blocker class: missing, stale, unavailable, unsafe, ambiguous, and undocumented. [VERIFIED: .planning/REQUIREMENTS.md]
- [ ] `tools/parity` tests for claim ladder tier/non-claim language and Phase 21 controlled no-share non-promotion. [VERIFIED: tools/parity/src/main.rs] [VERIFIED: docs/parity/evidence/phase-21-live-mining-and-soak-evidence/summary.md]
- [ ] Phase 22 evidence docs: `claim-ladder.md`, `safety-preconditions.md`, `blocker-reasons.md`, `summary.md`, and `redaction-review.md`. [VERIFIED: .planning/phases/22-claim-ladder-and-safety-preconditions/22-CONTEXT.md]

## Security Domain

Security enforcement is enabled by default because `.planning/config.json` does not set `security_enforcement` to `false`. [VERIFIED: .planning/config.json]

### Applicable ASVS Categories

| ASVS Category | Applies | Standard Control |
| --- | --- | --- |
| V2 Authentication | no | Phase 22 does not add authentication or account flows. [VERIFIED: .planning/ROADMAP.md] |
| V3 Session Management | no | Phase 22 does not add session or token lifecycle behavior. [VERIFIED: .planning/ROADMAP.md] |
| V4 Access Control | yes, for mining enablement gates | Typed precondition decisions must fail closed before BM1366 work dispatch. [VERIFIED: .planning/REQUIREMENTS.md] [VERIFIED: crates/bitaxe-stratum/src/v1/mining_loop.rs] |
| V5 Input Validation | yes | Parse prerequisite and bounded-evidence inputs into domain types before use. [VERIFIED: standards/core/architecture.md] [VERIFIED: standards/languages/rust.md] |
| V6 Cryptography | no new crypto | Do not add custom cryptography for this phase. [VERIFIED: .planning/ROADMAP.md] |
| V8 Data Protection | yes | Blocker logs/evidence must not expose credentials, endpoints, raw Stratum/share payloads, raw BM1366 frames, Wi-Fi values, MACs, IPs, or NVS secrets. [VERIFIED: AGENTS.md] [VERIFIED: .planning/phases/22-claim-ladder-and-safety-preconditions/22-CONTEXT.md] |

### Known Threat Patterns for This Stack

| Pattern | STRIDE | Standard Mitigation |
| --- | --- | --- |
| Safety gate bypass through a temporary force flag | Elevation of privilege / Tampering | No bypass flags; all dispatch consumes typed prerequisite decision output. [VERIFIED: .planning/research/ARCHITECTURE.md] [VERIFIED: .planning/phases/22-claim-ladder-and-safety-preconditions/22-CONTEXT.md] |
| Evidence overclaiming | Tampering / Repudiation | Claim ladder, parity guard tests, exact checklist wording, and `just parity`. [VERIFIED: tools/parity/src/main.rs] [VERIFIED: Justfile] |
| Secret leakage in blocker/log output | Information disclosure | Stable reason strings must be generic and redaction-safe; retain category labels only. [VERIFIED: AGENTS.md] [VERIFIED: firmware/bitaxe/src/controlled_mining_runtime.rs] |
| Stale or ambiguous runtime observations enabling work | Tampering / Safety hazard | Require fresh observation age checks or explicit bounded evidence with source, window, board, evidence id, and reason. [VERIFIED: .planning/phases/22-claim-ladder-and-safety-preconditions/22-CONTEXT.md] |

## Sources

### Primary (HIGH confidence)

- `.planning/phases/22-claim-ladder-and-safety-preconditions/22-CONTEXT.md` - Locked decisions, scope, constraints, implementation surfaces, deferred work. [VERIFIED: ReadFile]
- `.planning/REQUIREMENTS.md` - EVD-06, SAFE-10, SAFE-11 requirements and traceability. [VERIFIED: ReadFile]
- `.planning/ROADMAP.md` - Phase 22 goal, success criteria, dependency, later-phase boundaries. [VERIFIED: ReadFile]
- `.planning/STATE.md` and `.planning/PROJECT.md` - v1.1 milestone state and project constraints. [VERIFIED: ReadFile]
- `AGENTS.md`, `AGENTS.bright-builds.md`, `standards/core/architecture.md`, `standards/core/testing.md`, `standards/core/verification.md`, `standards/languages/rust.md` - Repo and standards guidance. [VERIFIED: ReadFile]
- `crates/bitaxe-safety/src/evidence.rs`, `power.rs`, `thermal.rs`, `status.rs`, `effects.rs` - Existing safety domain types, evidence classes, fail-closed effects, and blocker reasons. [VERIFIED: ReadFile]
- `crates/bitaxe-stratum/src/v1/mining_loop.rs`, `state.rs`, `controlled_runtime.rs` - Existing work dispatch gate, runtime state, controlled runtime blocker behavior. [VERIFIED: ReadFile]
- `crates/bitaxe-api/src/mining.rs` - Existing API blocker projection shape. [VERIFIED: ReadFile]
- `tools/parity/src/main.rs`, `mining_allow.rs`, `safety_allow.rs` - Existing parity, claim-tier, and safety allow validation. [VERIFIED: ReadFile]
- Phase 20 and Phase 21 evidence summaries - Current active safety and controlled no-share evidence boundaries. [VERIFIED: ReadFile]

### Secondary (MEDIUM confidence)

- `.planning/research/ARCHITECTURE.md` - Prior architecture mapping and suggested v1.1 build order. [VERIFIED: ReadFile]

### Tertiary (LOW confidence)

- None used. [VERIFIED: research session tool outputs]

## Metadata

**Confidence breakdown:**

- Standard stack: HIGH - existing workspace crates, Bazel targets, and `just` commands were read directly. [VERIFIED: BUILD.bazel files] [VERIFIED: Justfile]
- Architecture: HIGH - the recommendation follows locked Phase 22 decisions and Bright Builds functional-core guidance. [VERIFIED: .planning/phases/22-claim-ladder-and-safety-preconditions/22-CONTEXT.md] [VERIFIED: standards/core/architecture.md]
- Pitfalls: HIGH - pitfalls are grounded in existing Phase 20/21 evidence and parity code guardrails. [VERIFIED: docs/parity/evidence/phase-20-active-safety-hardware-telemetry-evidence/summary.md] [VERIFIED: docs/parity/evidence/phase-21-live-mining-and-soak-evidence/summary.md] [VERIFIED: tools/parity/src/main.rs]
- Exact file/module names: MEDIUM - Phase 22 gives Claude discretion on exact module and enum names. [VERIFIED: .planning/phases/22-claim-ladder-and-safety-preconditions/22-CONTEXT.md]

**Research date:** 2026-07-04 [VERIFIED: local date context]
**Valid until:** 2026-08-03 for repo-local planning guidance; revisit sooner if Phase 23-26 changes the safety or mining runtime contracts first. [VERIFIED: .planning/ROADMAP.md]
