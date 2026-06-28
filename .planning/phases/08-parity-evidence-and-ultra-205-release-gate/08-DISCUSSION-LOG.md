# Phase 8: Parity Evidence And Ultra 205 Release Gate - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md. This log preserves the alternatives considered.

**Date:** 2026-06-28T21:54:06.223Z
**Phase:** 8-Parity Evidence And Ultra 205 Release Gate
**Mode:** Yolo
**Areas discussed:** Evidence governance and claim policy, Ultra 205 release evidence workflow, release gate and documentation closure, deferred scope and gap handling

## Evidence Governance And Claim Policy

| Option | Description | Selected |
| --- | --- | --- |
| Evidence-led release audit | Use checklist rows, evidence files, and automated parity validation as the source of truth for release claims. | yes |
| Prose-only release review | Rely on human release notes to describe readiness without tightening validation. | no |
| Implementation-status release review | Treat completed code paths as enough to claim parity. | no |

**Yolo choice:** Evidence-led release audit.
**Notes:** This follows ADR-0012 and the existing checklist semantics. `verified` must mean evidence-backed parity.

## Ultra 205 Release Evidence Workflow

| Option | Description | Selected |
| --- | --- | --- |
| Phase-gated hardware evidence | Start with `just detect-ultra205`, record board/port/package/source/reference facts, and run destructive checks only with documented recovery procedures. | yes |
| Host-only closure | Close Phase 8 from package and compile evidence only. | no |
| Ad hoc manual probing | Run live HTTP, erase, rollback, or interrupted-update commands without a written recovery path. | no |

**Yolo choice:** Phase-gated hardware evidence.
**Notes:** Repo-local guidance grants standing permission for connected Ultra 205 verification after detection succeeds, but destructive or fault-injection checks still need phase-gated recovery instructions.

## Release Gate And Documentation Closure

| Option | Description | Selected |
| --- | --- | --- |
| Extend existing parity tooling | Use `tools/parity`, `just parity`, release-gate validation, release docs, and evidence records as one closure path. | yes |
| Add a new release checklist tool | Build a parallel release readiness mechanism for Phase 8. | no |
| Documentation-only summary | Add a final summary without command-backed validation. | no |

**Yolo choice:** Extend existing parity tooling.
**Notes:** The current code already validates safety-critical and release/OTA verified-row evidence shapes. Phase 8 should strengthen that path where needed.

## Deferred Scope And Gap Handling

| Option | Description | Selected |
| --- | --- | --- |
| Explicit bounded V1 | Keep non-205 boards, deferred ASIC families, Stratum v2, BAP, all-board images, and Angular UI rewrite outside V1 release closure unless separately evidenced. | yes |
| Expand V1 to all parity surfaces | Attempt to close every upstream board, ASIC, protocol, and UI surface before Ultra 205 release. | no |
| Hide release gaps | Leave gaps implicit in docs or checklist notes. | no |

**Yolo choice:** Explicit bounded V1.
**Notes:** OTAWWW may remain a REL-03 gap only if owner, impact, follow-up, and release impact are explicit across checklist, release docs, evidence, and release gate.

## the agent's Discretion

- Exact Phase 8 plan split.
- Evidence file names and final release summary shape.
- Specific validation helper names inside existing `tools/parity` modules.
- Whether a live HTTP blocker is closed by establishing a reachable URL or documented as pending.

## Deferred Ideas

- Future-board verification for Gamma 601/BM1370 and other upstream board families.
- Stratum v2 completion.
- BAP accessory parity.
- Angular AxeOS UI rewrite.
- All-board factory image matrix.
