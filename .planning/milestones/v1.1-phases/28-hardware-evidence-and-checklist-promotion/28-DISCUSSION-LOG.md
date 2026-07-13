# Phase 28: Hardware Evidence And Checklist Promotion - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-07-06
**Phase:** 28-hardware-evidence-and-checklist-promotion
**Mode:** Yolo
**Areas discussed:** Evidence consolidation scope, Checklist promotion boundaries, Parity guard enforcement, Non-claims preservation

---

## Evidence Consolidation Scope

| Option | Description | Selected |
| --- | --- | --- |
| New standalone root with copied logs | Duplicate Phase 27 logs into Phase 28 root | |
| Consolidated root with cross-links | Slot files reference Phase 27 committed artifacts and add consolidation metadata | ✓ |
| Skip consolidation; promote checklist only | Update checklist without Phase 28 evidence root | |

**User's choice:** Consolidated root with cross-links (recommended default)
**Notes:** Preserves redaction review and avoids duplicating raw or large local artifacts.

---

## Checklist Promotion Boundaries

| Option | Description | Selected |
| --- | --- | --- |
| Promote STR-09 to verified from bridge code | Treat implementation as hardware proof | |
| Conservative promotion from Phase 27 categories only | Keep STR-09/CFG-07/live safety rows below verified; update notes and evidence links only | ✓ |
| No checklist changes | Leave checklist untouched | |

**User's choice:** Conservative promotion from Phase 27 categories only (recommended default)
**Notes:** Phase 27 `share_outcome: blocked_safe_prerequisite` blocks verified STR-09 promotion.

---

## Parity Guard Enforcement

| Option | Description | Selected |
| --- | --- | --- |
| Manual checklist discipline only | Rely on human review | |
| Phase 28 validator + regression tests | Mirror Phase 26 `validate_phase26_telemetry_verified_row` pattern | ✓ |
| Disable parity guard for v1.1 closure | Faster but unsafe | |

**User's choice:** Phase 28 validator + regression tests (recommended default)

---

## Non-Claims Preservation

| Option | Description | Selected |
| --- | --- | --- |
| Minimal non-claims in summary only | Shorter docs | |
| Explicit non-claims in every Phase 28 slot and summary | Matches Phase 23/27 evidence contract | ✓ |
| Drop deferred non-claims after consolidation | Would overclaim v1.1 closure | |

**User's choice:** Explicit non-claims in every Phase 28 slot and summary (recommended default)

---

## Claude's Discretion

Exact validator names, evidence filenames, plan count, and summary table layout.

## Deferred Ideas

- Fresh hardware rerun for accepted/rejected share proof — future phase.
- Active safety hardware closure — future phase.
