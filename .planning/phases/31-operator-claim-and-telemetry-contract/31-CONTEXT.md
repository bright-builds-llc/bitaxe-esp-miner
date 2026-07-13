---
generated_by: gsd-discuss-phase
lifecycle_mode: yolo
phase_lifecycle_id: 31-2026-07-13T19-47-51
generated_at: 2026-07-13T19:47:51.504Z
---

# Phase 31: Operator Claim and Telemetry Contract - Context

**Gathered:** 2026-07-13
**Status:** Ready for planning
**Mode:** Yolo

<domain>
## Phase Boundary

Phase 31 defines the pure, typed v1.2 contract that lets operators and downstream code distinguish observation truth, compatibility values, settings authority, and parity eligibility before any new firmware effect can occur. It covers the four observation states, producer-owned acquisition metadata, the hostname-only v1.2 PATCH allowlist, and closed admission/non-promotion types.

This phase does not acquire sensors, change I2C ownership, persist or reboot-test settings, compose the Phase 34 coherent operator snapshot, access hardware or credentials, promote checklist rows, or perform active control, self-test, mining, OTA, direct-UART/pin, other-board, or archived Phase 28.1.1 work.

</domain>

<decisions>
## Implementation Decisions

### Observation truth states

- **D-01:** Model each power and thermal fact with a state-carrying enum whose variants make `fresh`, `stale`, `unavailable`, and `fault` mutually exclusive and prevent invalid state/value combinations.
- **D-02:** `Fresh` carries a valid last successful sample. `Stale` carries the same last-good sample and a typed stale reason. `Unavailable` means no usable sample exists. `Fault` carries a typed failure reason and may carry the prior last-good sample, but never the invalid or failed attempt.
- **D-03:** Use serde-stable, state-specific reason enums at the contract boundary. Detailed adapter or vendor errors stay in logs rather than becoming free-form public status strings.
- **D-04:** Project AxeOS-compatible numeric fields separately from observation truth. Preserve a required numeric fallback such as zero only for compatibility; numeric zero never implies fresh, available, healthy, or successfully acquired data.
- **D-05:** Keep facts independently representable. A failed temperature acquisition must not make tachometer or power facts unavailable when those producers still have valid state.

### Producer-owned freshness

- **D-06:** Bind every successful sample atomically to a boot/session identity, source-local sequence, and monotonic acquisition time. Sequence ordering is meaningful only inside that boot/session scope.
- **D-07:** Only the acquisition producer may advance a sample sequence or acquisition time, and only after a successful validated read. A failure may change the public state to stale or fault while preserving the last successful stamp unchanged.
- **D-08:** API reads, WebSocket projections, retained-log projections, evidence projections, serializers, and other consumers only copy stored observation state. They must not read sensors, advance sequences, rewrite acquisition time, or turn a stale value fresh.
- **D-09:** Producer-owned cadence or timeout processing owns stale transitions. Query time must not mint a new observation state independently for each endpoint.
- **D-10:** Keep source-local observation sequence separate from the global operator-snapshot revision planned for Phase 34. Phase 31 defines the composable stamp contract without pulling the coherent-snapshot reducer into this phase.

### Settings and claim admission

- **D-11:** Preserve the existing broad AxeOS compatibility parser as an imperative shell, but translate it into a closed v1.2 settings authority type whose only constructible write is a validated `hostname` change.
- **D-12:** Broader known, unknown, or compatibility fields cannot produce a v1.2 persistence-authority token. Compatibility parsing or response handling must not be confused with eligibility to write, claim, or promote that field.
- **D-13:** Represent promotion admission as a closed decision between a narrow v1.2-eligible claim type and a typed ineligible reason. Do not use open strings, generic maps, or schema enumeration as the authority to widen v1.2 outcomes.
- **D-14:** Active fan/voltage/reset/power/ASIC control, self-test effects, watchdog intervention or load tests, mining and archived Phase 28.1.1 work, credentials, direct UART or pin work, OTA/recovery, non-205 boards, telemetry history, UI/display/input/BAP, and broad production-ready or verified claims must be unrepresentable as eligible v1.2 Phase 31 outcomes.
- **D-15:** Keep admission exact and row/capability scoped. A passing contract test, compatibility response, or phase lifecycle may support only the specifically modeled Phase 31 claim; it cannot authenticate another capability or future phase.

### the agent's Discretion

- Exact type, variant, module, and serde tag names, provided illegal state/value combinations and excluded eligible claims remain unconstructible.
- Exact stale, unavailable, and fault reason taxonomy, provided reasons are typed, stable, redaction-safe, and do not leak adapter internals.
- Whether the shared observation type lives in `bitaxe-core`, `bitaxe-safety`, or another existing pure crate, provided API and firmware adapters depend inward on one authoritative contract.
- Migration order for existing aggregate telemetry and settings models, provided legacy compatibility DTOs never remain the source of observation truth or v1.2 write authority.

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### v1.2 scope and requirements

- `.planning/ROADMAP.md` — Phase 31 goal, dependency, success criteria, and separation from Phases 32-35.
- `.planning/PROJECT.md` — v1.2 operator-ready runtime goal, observation-first decision, and excluded active effects.
- `.planning/REQUIREMENTS.md` — OBS-01, CFG-08, later-phase ownership, deferred requirements, and explicit v1.2 exclusions.
- `.planning/STATE.md` — current milestone routing and active planning state.

### Claim and promotion semantics

- `.planning/milestones/v1.1-phases/30-live-share-outcome-and-verified-promotion/30-CONTEXT.md` — closed typed admission, exact-claim promotion, and conservative no-promotion precedent.
- `.planning/milestones/v1.1-phases/30-live-share-outcome-and-verified-promotion/30-VERIFICATION.md` — passed policy verification that remains distinct from requirement or capability promotion.
- `docs/adr/0006-parity-checklist-as-audit-evidence.md` — checklist audit and promotion policy.
- `docs/adr/0012-parity-verification-evidence.md` — evidence-class and verified-status semantics.

### Repository constraints

- `AGENTS.md` — ESP-IDF boundary, archived-lineage prohibition, hardware authorization, evidence redaction, and frontmatter rules.
- `AGENTS.bright-builds.md` — required workflow and high-signal architecture, testing, and verification rules.
- `standards/core/architecture.md` — functional core, parse-at-boundaries, and illegal-states-unrepresentable rules.
- `standards/core/code-shape.md` — early-return, optional naming, and local readability rules.
- `standards/core/testing.md` — behavior-focused Arrange/Act/Assert test expectations.
- `standards/core/verification.md` — sync and clean verification requirements.
- `standards/languages/rust.md` — Rust domain modeling, error handling, testing, and pre-commit gate order.

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets

- `crates/bitaxe-safety/src/power.rs` and `crates/bitaxe-safety/src/thermal.rs` already provide pure observation and safety decisions that can be migrated behind one shared truth contract.
- `crates/bitaxe-api/src/telemetry.rs` already owns full/diff telemetry projection behavior and is the natural adapter for explicit wire-state fields plus legacy numeric compatibility values.
- `crates/bitaxe-api/src/settings.rs` and `crates/bitaxe-config/src/persistence.rs` already separate request planning from persistence effects, providing a boundary where the closed hostname-only authority can be introduced.
- `tools/parity/src/claim_ladder.rs` and the Phase 30 admission guard provide exact-claim and explicit-non-claim patterns for a closed v1.2 admission model.

### Established Patterns

- Pure Rust crates own typed decisions; firmware and HTTP layers remain imperative adapters.
- Current unavailable safety reports fill compatibility numeric fields with zeros, so Phase 31 must add explicit per-fact truth without reinterpreting those zeros.
- Existing runtime projections use producer sequences, but `firmware/bitaxe/src/runtime_snapshot.rs` currently collects safety state while serving snapshots. Phase 31 must make the observation contract read-only from the consumer side before Phase 32 adds the sole acquisition owner.
- Existing parity governance separates lifecycle success, implementation status, evidence class, and exact verified claims.

### Integration Points

- Add the authoritative observation/stamp/reason and v1.2 admission types to pure crates with exhaustive unit tests.
- Adapt `crates/bitaxe-api` wire projections so explicit truth accompanies compatibility numeric fields.
- Narrow settings authority between API parsing and `crates/bitaxe-config` persistence planning without implementing Phase 33 durability.
- Add contract-level claim/admission validation in `tools/parity` without promoting checklist rows or creating hardware evidence.

</code_context>

<specifics>
## Specific Ideas

- Prefer a shape equivalent to `Fresh { sample }`, `Stale { last_good, reason }`, `Unavailable { reason }`, and `Fault { reason, maybe_last_good }`; exact names remain implementation discretion.
- A sample stamp should compose boot/session identity, source-local sequence, and monotonic acquisition time. It must not reuse Phase 34's future global snapshot revision.
- A compatibility request may be understood and answered without receiving a v1.2 write or promotion capability.
- Recommended admission shape: an eligible narrow claim type versus an ineligible typed reason, with no conversion from excluded reasons into eligible tokens.

</specifics>

<deferred>
## Deferred Ideas

- Phase 32 owns shared I2C0 lifecycle and actual INA260/EMC2101 producer acquisition.
- Phase 33 owns committed, reloaded, reconciled, and reboot-durable hostname persistence.
- Phase 34 owns global boot-session/operator-snapshot revision, coherent projections, provenance, and passive runtime health.
- Phase 35 owns detector-gated correlated evidence and exact checklist promotion.
- Active hardware control, self-test execution, watchdog intervention/load experiments, mining, OTA/recovery, other boards, telemetry history, and UI expansion remain outside v1.2 Phase 31.

</deferred>

***

*Phase: 31-operator-claim-and-telemetry-contract*
*Context gathered: 2026-07-13*
