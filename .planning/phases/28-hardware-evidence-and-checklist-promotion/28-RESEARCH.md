# Phase 28: Hardware Evidence And Checklist Promotion - Research

**Researched:** 2026-07-06
**Domain:** Redacted operator evidence consolidation, parity checklist promotion, hardware-evidence guardrails
**Confidence:** HIGH

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions

- **D-01:** Phase 28 evidence root at `docs/parity/evidence/phase-28-hardware-evidence-and-checklist-promotion/` consolidates Phase 27 via slot cross-links; no raw log duplication.
- **D-02:** Extend Phase 23 slot contract with `source_phase27_root`, `consolidation_status`, inherited `exact_non_claims`.
- **D-03:** Preserve `share_outcome: blocked_safe_prerequisite`, `asic_bridge_status: blocked`, `safe_stop_status: blocked`.
- **D-04:** Promote only rows SAFE-10, SAFE-11, STR-08, STR-09, SAFE-12, SAFE-13, CFG-07, ASIC-09 through ASIC-12 when evidence tokens exactly support target status.
- **D-05:** STR-09 stays below `verified`; accepted/rejected shares remain non-claims.
- **D-06:** CFG-07 stays below `verified`; category labels only, no live credential-use hardware proof.
- **D-07:** SAFE-10/11/12/13 advance only to implemented/workflow tier supported by Phase 27 bridge + Phase 28 consolidation.
- **D-08:** STR-08 and ASIC-09–12 get conservative note/evidence cross-links only; no live socket success or hardware-verified production ASIC beyond Phase 27 categories.
- **D-09:** Do not downgrade rows already `verified` from earlier phases (STR-11, EVD-06, Phase 26 API rows).
- **D-10:** Add `validate_phase28_hardware_promotion_row` in `tools/parity`.
- **D-11:** Regression tests for overbroad promotion (missing summary, redaction, blocker masquerading, STR-09/CFG-07/SAFE verified without tokens).
- **D-12:** `just parity` passes unchanged verified rows; fails missing/overclaim Phase 28 artifacts.
- **D-13:** Phase 28 summary/conclusion preserve deferred non-claims (active safety, OTAWWW/recovery, non-205, Stratum v2, UI/BAP, unbounded stress).
- **D-14:** Add `28-VALIDATION.md`; blocked deterministic workflow proof sufficient when Phase 27 supplies categories.
- **D-15:** Unit/workflow tests for consolidation validation, promotion guardrails, parity regression without fresh hardware per assertion.

### Deferred Ideas (OUT OF SCOPE)

- Fresh detector-gated hardware reruns for accepted/rejected share proof.
- Full active voltage/fan/thermal/fault/self-test, OTAWWW/recovery, non-205, Stratum v2, UI/BAP, unbounded stress mining.
</user_constraints>

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
| --- | --- | --- |
| SAFE-10 | Production mining prerequisite readiness before BM1366 work dispatch. | Phase 22 unit/workflow evidence plus Phase 27 consolidation cross-links; stay `implemented` until detector-gated live prerequisite proof. [VERIFIED: `docs/parity/checklist.md`, Phase 27 `summary.md`] |
| SAFE-11 | Production mining fail-closed blocker reasons. | Phase 22 blocker taxonomy plus Phase 28 consolidation summary; no `verified` without live safety proof. [VERIFIED: `crates/bitaxe-safety/src/mining_preconditions.rs`] |
| SAFE-12 | Production mining safe stop. | Phase 25 safe-stop markers plus Phase 27 `safe_stop_status: blocked`; consolidation cites Phase 25/27 without upgrading to hardware-verified stop. [VERIFIED: `docs/parity/evidence/phase-25-live-stratum-runtime-and-safe-stop/safe-stop.md`] |
| SAFE-13 | Watchdog responsiveness under live runtime load. | Phase 25 `watchdog_responsiveness_status: blocked`; Phase 28 preserves below `verified`. [VERIFIED: Phase 25 summary] |
| CFG-07 | Runtime-only credential labels; no committed secrets. | Phase 23 redaction workflow; row stays below `verified` per safety-critical credential handling policy. [VERIFIED: Phase 23 evidence contract, Phase 27 category labels] |
| ASIC-09 | BM1366 diagnostic and production mode separation. | Phase 24 evidence; Phase 28 adds consolidation cross-link to Phase 27 bridge context without live hardware verified promotion. [VERIFIED: Phase 24 summary] |
| ASIC-12 | BM1366 production fail-closed blockers and redaction. | Phase 24 redaction-safe blockers; Phase 28 consolidation inherits non-claims. [VERIFIED: Phase 24 summary] |
</phase_requirements>

## Summary

Phase 28 is a **consolidation and promotion closure** phase, not a new hardware capture phase. Phase 27 already committed detector-gated workflow artifacts with `share_outcome: blocked_safe_prerequisite`, `detector_status: passed`, `board_info_status: passed`, and `redaction_status: passed` under `docs/parity/evidence/phase-27-live-hardware-asic-and-stratum-bridge/`. Phase 28 must create a **second evidence root** that satisfies the Phase 23 operator slot inventory while **referencing** Phase 27 paths instead of copying raw logs. [VERIFIED: Phase 27 summary, share-outcome, redaction-review]

The promotion boundary is narrow: checklist rows may gain **note refinements and Phase 28 evidence citations**, but STR-09 and CFG-07 **must not** reach `verified`, and SAFE rows **must not** reach `verified` without detector-gated live proof matching each row's claim. Phase 27 explicitly deferred Phase 28 promotion except category labels. [VERIFIED: Phase 27 share-outcome `exact_non_claims`]

Parity guardrails should mirror Phase 26's `validate_phase26_telemetry_verified_row` pattern: a dedicated `validate_phase28_hardware_promotion_row` invoked from `validate_rows`, with inline checklist fixture tests rejecting overbroad `verified` claims for in-scope requirement IDs. [VERIFIED: `tools/parity/src/main.rs` lines 1052–1148, 2022–2112]

## Standard Stack

### Core

| Component | Location | Usage |
| --- | --- | --- |
| Operator evidence contract | `docs/parity/evidence/phase-23-redacted-operator-evidence-workflow/evidence-contract.md` | Required slot inventory and forbidden committed values |
| Phase 27 hardware artifacts | `docs/parity/evidence/phase-27-live-hardware-asic-and-stratum-bridge/` | Source cross-links for consolidation slots |
| Checklist | `docs/parity/checklist.md` | Rows SAFE-10 through ASIC-12, STR-08/09 |
| Parity tool | `tools/parity/src/main.rs` | New Phase 28 validator + regression tests |
| Mining allow tiers | `tools/parity/src/mining_allow.rs` | Phase 27 share-outcome tier semantics |
| Operator evidence validator | `tools/parity/src/operator_evidence.rs` | Optional Phase 28 root slot validation extension |

### Closure Pattern (from Phase 26/27)

| Artifact | Purpose |
| --- | --- |
| `summary.md` | Requirement mapping, command results, exact non-claims |
| `redaction-review.md` | Denylist review over consolidated slot inventory |
| `conclusion.md` | Supported claims and inherited blockers |
| Per-slot files | Cross-link consolidation with `source_phase27_root`, `consolidation_status` |
| `28-VALIDATION.md` | Per-task Nyquist map and final gate |
| `validate_phase28_*` | Reject verified promotion without matching evidence tokens |

## Architecture Patterns

### Consolidation Slot Shape (Phase 28 extension)

Each Phase 28 slot file must include Phase 23 required fields **plus**:

```markdown
source_phase27_root: docs/parity/evidence/phase-27-live-hardware-asic-and-stratum-bridge/
consolidation_status: cross_linked|blocked|pending
```

Slots with Phase 27 counterparts (detector, board-info, command, share-outcome, redaction-review, conclusion) set `consolidation_status: cross_linked` and cite exact Phase 27 relative paths. Slots absent from Phase 27 (package, log, api, websocket, safe-stop) cross-link to Phase 25/26/23 roots or record `consolidation_status: blocked` with category labels only—never invent hardware proof.

### Checklist Promotion Tiers

| Row | Current status | Phase 28 ceiling | Evidence token required |
| --- | --- | --- | --- |
| SAFE-10, SAFE-11 | implemented | implemented | Phase 22 + Phase 28 summary; below verified |
| SAFE-12, SAFE-13 | implemented | implemented | Phase 25 + Phase 28 summary; hardware stop/watchdog below verified |
| STR-08 | implemented | implemented | Phase 27 summary; `share_outcome: blocked_safe_prerequisite` |
| STR-09 | implemented | **below verified** | Phase 27 share-outcome; no accepted/rejected language |
| CFG-07 | implemented | **below verified** | Phase 23/28 category labels; no live credential hardware proof |
| ASIC-09, ASIC-12 | implemented | implemented | Phase 24 + Phase 28 cross-link |
| ASIC-10, ASIC-11 | implemented | implemented | Phase 27 summary; `asic_bridge_status: blocked` |

### Parity Validator Pattern

Mirror Phase 26:

1. `is_phase28_hardware_promotion_row(row)` — match requirement IDs and surface terms for in-scope rows.
2. For `verified` status on STR-09, CFG-07, SAFE-10–13, STR-08, ASIC-09–12: reject unless Phase 28 summary + redaction review + exact non-claims present **and** row-specific tokens match allowed tier (e.g. reject `verified` STR-09 when haystack contains `blocked_safe_prerequisite` without explicit hardware accepted/rejected category).
3. For `implemented`/`workflow` rows: accept when citing `phase-28-hardware-evidence-and-checklist-promotion/summary.md` with redaction and non-claims.
4. Add fixture tests: missing summary, blocker masquerading, missing redaction, STR-09 verified overclaim, CFG-07 verified overclaim, positive conservative rows.

## Don't Hand-Roll

| Problem | Use instead |
| --- | --- |
| Custom evidence directory layout | Phase 23 slot inventory + Phase 28 consolidation fields |
| Ad hoc checklist promotion | Exact evidence path citations + `tools/parity` guard |
| Copying Phase 27 raw logs | Cross-link paths and category labels only |
| Fresh hardware for every test | Blocked-mode deterministic parity tests + Phase 27 committed artifacts |

## Common Pitfalls

### Pitfall 1: Rewriting blocked outcomes as success

Phase 27 `share_outcome: blocked_safe_prerequisite` must flow unchanged into Phase 28 share-outcome and summary slots. Consolidation must not use "attempted" language that implies partial verified share proof.

### Pitfall 2: Promoting CFG-07 or STR-09 to verified

Both rows are explicitly capped below `verified` in CONTEXT D-05/D-06. Validator must fail `verified` even if notes cite Phase 28 summary.

### Pitfall 3: Downgrading Phase 26 verified telemetry rows

D-09 forbids rewriting STR-11, EVD-06, EVD-08, API-002, etc. Phase 28 checklist edits must touch only in-scope rows.

### Pitfall 4: Missing operator slot inventory

Phase 28 root must include all eleven Phase 23 slots plus summary; `operator_evidence` validator may need Phase 28 path recognition if extended in plan 28-01.

### Pitfall 5: Frontmatter `---` in evidence body

Use headings only for section breaks in evidence Markdown parsed by GSD tooling.

## Code Examples

### Phase 26 validator hook (mirror target)

From `tools/parity/src/main.rs`:

```rust
fn validate_phase26_telemetry_verified_row(row: &ChecklistRow) -> Vec<ValidationError> {
    if !is_phase26_telemetry_row(row) { return Vec::new(); }
    // Require phase-26 summary, redaction, exact_non_claims; reject blocker terms on verified rows
}
```

Phase 28 equivalent should key off `phase-28-hardware-evidence-and-checklist-promotion/summary.md` and row IDs `SAFE-10`, `STR-09`, `CFG-07`, `ASIC-09`, etc.

### Phase 27 evidence tokens (must preserve)

From `docs/parity/evidence/phase-27-live-hardware-asic-and-stratum-bridge/summary.md`:

```markdown
share_outcome: blocked_safe_prerequisite
asic_bridge_status: blocked
safe_stop_status: blocked
redaction_status: passed
raw_artifacts_committed: no
```

## Open Questions

None blocking planning. Executor discretion: exact Phase 28 slot filenames for package/log/api/websocket/safe-stop when cross-linking Phase 25/26 blocked slots vs recording `consolidation_status: blocked` in Phase 28 root only.

## Sources

### Primary (HIGH confidence)

- `.planning/phases/28-hardware-evidence-and-checklist-promotion/28-CONTEXT.md`
- `docs/parity/evidence/phase-27-live-hardware-asic-and-stratum-bridge/summary.md`
- `docs/parity/evidence/phase-23-redacted-operator-evidence-workflow/evidence-contract.md`
- `tools/parity/src/main.rs` — Phase 26 validator and tests
- `docs/parity/checklist.md` — current in-scope row statuses

### Secondary (MEDIUM confidence)

- `tools/parity/src/mining_allow.rs` — Phase 27 tier semantics for share-outcome claims
- `.planning/phases/26-telemetry-and-parity-closure/26-04-PLAN.md` — closure plan template
- `.planning/phases/27-live-hardware-asic-and-stratum-bridge/27-04-PLAN.md` — parity guard pattern

## Metadata

**Confidence breakdown:** Consolidation shape HIGH (Phase 23/27 artifacts exist). Promotion boundaries HIGH (locked in CONTEXT). Validator pattern HIGH (Phase 26 precedent). Operator evidence extension MEDIUM (may reuse existing loader with path filter).

**Planning recommendation:** Three-plan wave structure — 28-01 evidence root (wave 1), 28-02 checklist promotion (wave 2), 28-03 parity guardrails + validation closure (wave 3). No fresh hardware required unless executor chooses optional blocked-mode workflow proof script.
