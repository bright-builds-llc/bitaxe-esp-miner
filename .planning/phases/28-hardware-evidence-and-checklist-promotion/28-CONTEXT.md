---
generated_by: gsd-discuss-phase
lifecycle_mode: yolo
phase_lifecycle_id: 28-2026-07-06T17-21-15
generated_at: 2026-07-06T17:21:15.000Z
---

# Phase 28: Hardware Evidence And Checklist Promotion - Context

**Gathered:** 2026-07-06
**Status:** Ready for planning
**Mode:** Yolo

<domain>
## Phase Boundary

Phase 28 consolidates Phase 27 detector-gated hardware artifacts into the operator evidence root contract, promotes parity checklist rows only to statuses exactly supported by committed redacted evidence, and extends parity guardrails so `just parity` rejects overbroad verified promotion without matching artifacts.

This phase does not rerun live hardware capture unless a plan explicitly needs a deterministic blocked-mode workflow proof. It does not claim accepted/rejected shares, active voltage/fan/thermal/fault/self-test closure, OTAWWW/recovery destructive evidence, non-205 boards, Stratum v2, runtime display/input, BAP, or unbounded stress mining.
</domain>

<decisions>
## Implementation Decisions

### Evidence Consolidation

- **D-01:** Create a committed Phase 28 evidence root at `docs/parity/evidence/phase-28-hardware-evidence-and-checklist-promotion/` that consolidates Phase 27 hardware artifacts through slot files, cross-links, and a deterministic redaction review. Do not duplicate raw local artifacts; reference Phase 27 committed paths and category labels only.
- **D-02:** Extend the Phase 23 evidence contract shape for Phase 28 consolidation: required slots remain package, detector, board-info, command, log, api, websocket, share-outcome, safe-stop, redaction-review, and conclusion. Phase 28 slot files must record `source_phase27_root`, `consolidation_status`, and exact non-claims inherited from Phase 27.
- **D-03:** Phase 28 evidence must preserve Phase 27 outcomes: `share_outcome: blocked_safe_prerequisite`, `asic_bridge_status: blocked`, and `safe_stop_status: blocked` until detector-gated live proof exists. Consolidation may not rewrite those categories into success language.

### Checklist Promotion Boundaries

- **D-04:** Promote checklist rows only when committed Phase 27/28 evidence tokens exactly support the target status. Rows in scope: SAFE-10, SAFE-11, STR-08, STR-09, SAFE-12, SAFE-13, CFG-07, ASIC-09, ASIC-10, ASIC-11, and ASIC-12.
- **D-05:** Because Phase 27 committed `share_outcome: blocked_safe_prerequisite`, STR-09 must remain below `verified`. Accepted/rejected share language stays an explicit non-claim.
- **D-06:** CFG-07 remains below `verified` because runtime credential handling is safety-critical and Phase 27/28 evidence records category labels only, not hardware proof of live credential use during mining.
- **D-07:** SAFE-10, SAFE-11, SAFE-12, and SAFE-13 may advance only to the exact implemented/workflow tier supported by Phase 27 bridge evidence plus Phase 28 consolidation summaries. Do not promote to `verified` without detector-gated live prerequisite, blocker, safe-stop, or watchdog proof matching the row claim.
- **D-08:** STR-08, ASIC-09 through ASIC-12 may receive conservative checklist note updates and evidence cross-links from Phase 27 workflow artifacts, but must not claim live socket success, accepted/rejected shares, or hardware-verified production ASIC behavior beyond what Phase 27 summary and share-outcome files support.
- **D-09:** Any row that already reached `verified` from earlier phases with deterministic unit/workflow evidence (for example STR-11, EVD-06, API rows closed in Phase 26) must not be downgraded or rewritten by Phase 28.

### Parity Guard Enforcement

- **D-10:** Extend `tools/parity` with Phase 28 validators that reject checklist rows promoted above the evidence tier recorded in Phase 28 summary and mining-allow manifests.
- **D-11:** Add regression tests for overbroad promotion attempts: missing Phase 28 summary, missing redaction review, blocker language masquerading as verified proof, absent exact non-claims, and verified claims for STR-09/CFG-07/SAFE rows without matching hardware evidence tokens.
- **D-12:** `just parity` must continue to pass for unchanged verified rows while failing when Phase 28 promotion artifacts are missing or overclaim.

### Non-Claims And Closure

- **D-13:** Phase 28 summary and conclusion must preserve explicit non-claims for deferred active safety, OTAWWW/recovery, non-205 boards, Stratum v2, UI/BAP, and unbounded stress mining.
- **D-14:** Add Phase 28 validation metadata (`28-VALIDATION.md`) and verification artifacts following the Phase 26/27 closure pattern. Hardware mode is optional; blocked deterministic workflow proof is sufficient when Phase 27 artifacts already supply the hardware categories.
- **D-15:** Unit/workflow tests must prove consolidation slot validation, checklist promotion guardrails, and parity regression coverage without requiring fresh hardware for every assertion.

### Claude's Discretion

Claude may choose exact evidence filenames, validator function names, plan count, and summary table layout. Those choices must preserve redaction rules, exact non-claims, conservative promotion semantics, Ultra 205 scope, and repo-owned verification patterns.
</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Phase Scope And Requirements

- `.planning/ROADMAP.md` — Phase 28 goal, dependencies, requirements, and success criteria.
- `.planning/REQUIREMENTS.md` — SAFE-10, SAFE-11, SAFE-12, SAFE-13, CFG-07, ASIC-09, ASIC-12 traceability.
- `.planning/PROJECT.md` — v1.1 scope, parity evidence policy, architecture, and safety constraints.
- `.planning/STATE.md` — Phase 27 closure decisions and live-share blockers.
- `AGENTS.md` — Detector gate, redaction rules, evidence workflow, and hardware verification limits.

### Prior Phase Artifacts

- `.planning/phases/27-live-hardware-asic-and-stratum-bridge/27-CONTEXT.md` — Phase 27 boundary and deferral of Phase 28 promotion.
- `.planning/phases/27-live-hardware-asic-and-stratum-bridge/27-VERIFICATION.md` — Phase 27 passed verification with blocked share-outcome non-claims.
- `docs/parity/evidence/phase-23-redacted-operator-evidence-workflow/evidence-contract.md` — Operator evidence root slot contract.
- `docs/parity/evidence/phase-27-live-hardware-asic-and-stratum-bridge/summary.md` — Phase 27 evidence categories and non-claims.
- `docs/parity/evidence/phase-27-live-hardware-asic-and-stratum-bridge/share-outcome.md` — Current share-outcome blocker slot.
- `docs/parity/evidence/phase-27-live-hardware-asic-and-stratum-bridge/redaction-review.md` — Phase 27 redaction review baseline.
- `docs/parity/evidence/phase-26-telemetry-and-parity-closure/summary.md` — Phase 26 parity closure and validator pattern to mirror.
- `docs/parity/checklist.md` — Current checklist rows and evidence notes.

### Implementation Surfaces

- `tools/parity/src/main.rs` — Checklist validation and verified-row guardrails.
- `tools/parity/src/mining_allow.rs` — Phase 27 mining-allow tiers and scope validators.
- `tools/parity/src/safety_allow.rs` — Safety allow-manifest validation.
- `scripts/phase27-live-hardware-bridge-evidence.sh` — Phase 27 evidence wrapper pattern.
- `docs/adr/0012-parity-verification-evidence.md` — Evidence class and verified status semantics.
- `docs/adr/0006-parity-checklist-as-audit-evidence.md` — Checklist evidence policy.
</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets

- Phase 27 committed evidence root already contains detector, board-info, command, share-outcome, redaction-review, mining-allow, and summary artifacts.
- Phase 26 added `validate_phase26_telemetry_verified_row` and regression tests — Phase 28 should follow the same validator-plus-test pattern.
- Phase 27 mining-allow tiers in `tools/parity/src/mining_allow.rs` already encode conservative promotion for bridge evidence.

### Established Patterns

- Evidence roots use slot files with `slot_status`, category labels, `raw_artifacts_committed: no`, and `exact_non_claims`.
- Checklist updates cite exact evidence paths and preserve non-claims inline.
- Closure phases add summary, redaction-review, conclusion, validation metadata, verification report, and parity regression tests.

### Integration Points

- Add Phase 28 planning artifacts under `.planning/phases/28-hardware-evidence-and-checklist-promotion/`.
- Add committed evidence under `docs/parity/evidence/phase-28-hardware-evidence-and-checklist-promotion/`.
- Extend `docs/parity/checklist.md` and `tools/parity` only to the exact promotion tier Phase 28 proves.
</code_context>

<specifics>
## Specific Ideas

- Preferred consolidation shape: Phase 28 evidence root summarizes and cross-links Phase 27 slot files rather than copying large logs.
- Preferred promotion shape: checklist note refinements plus evidence path updates; no new `verified` rows unless Phase 27 categories explicitly support them (they do not for STR-09/CFG-07/live safety rows).
- Preferred guard shape: `validate_phase28_hardware_promotion_row` mirroring Phase 26 telemetry validator semantics.
- Preferred closure shape: blocked deterministic parity tests plus committed summary mapping each in-scope requirement ID to exact artifacts and non-claims.
</specifics>

<deferred>
## Deferred Ideas

- Fresh detector-gated hardware reruns for accepted/rejected share proof belong to a future hardware evidence phase, not Phase 28 consolidation.
- Full active voltage, fan, thermal, fault-stimulus, self-test, OTAWWW/recovery, non-205 boards, Stratum v2, UI/BAP, and unbounded stress mining remain deferred.
</deferred>

*Phase: 28-hardware-evidence-and-checklist-promotion*
*Context gathered: 2026-07-06*
