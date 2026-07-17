# Phase 35: Detector-Gated Correlated Evidence and Exact Parity Promotion - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-07-17
**Phase:** 35-detector-gated-correlated-evidence-and-exact-parity-promotion
**Mode:** Yolo
**Areas discussed:** Detector admission and evidence-root structure; Correlated capture, reboot, cleanup, and redaction; Exact allowlist promotion and deterministic non-promotion

***

## Detector Admission and Evidence-Root Structure

| Option | Description | Selected |
| --- | --- | --- |
| Typed three-gate orchestrator with content-addressed staging | Pure package/current-head gate, detector capability gate, then effect/capture gate; one digest-bound staged root and atomic verdict. | ✓ |
| Hardened shell wrapper plus final Rust validator | Smaller change surface but weaker transition typing and greater cross-process drift risk. | |
| In-toto-style DSSE attestations | Strong cross-actor provenance but unnecessary signing/key machinery for one local host and device. | |

**User's choice:** Auto-selected the recommended typed three-gate orchestrator.
**Notes:** Preserve the existing single-host trust boundary while borrowing digest-linked step principles; do not add signature infrastructure.

***

## Correlated Capture, Reboot, Cleanup, and Redaction

| Option | Description | Selected |
| --- | --- | --- |
| Thin Phase 35 supervisor plus typed two-epoch admission | Reuse proven Phase 33 shell mechanics while Rust validates two same-session epoch bundles and their continuity join. | ✓ |
| Dedicated Rust evidence-runner state machine | Strongest typed orchestration but would replace proven serial/process behavior before the sole qualifying run. | |
| Single shell runner with protected ledger | Smallest change but expands an oversized shell and leaves key invariants as conventions. | |

**User's choice:** Auto-selected the recommended thin supervisor plus typed two-epoch admission.
**Notes:** Boot A and boot B remain independently coherent; the reboot join is explicit and never weakens Phase 34’s same-session validation.

***

## Exact Allowlist Promotion and Deterministic Non-Promotion

| Option | Description | Selected |
| --- | --- | --- |
| Closed typed decision matrix over dedicated v1.2 parity rows | Exhaustive evidence-to-row decisions, purpose-built narrow rows, staged validation, and atomic admission. | ✓ |
| Hash-locked existing-row patches | Small mutation surface but broad legacy rows risk semantic over-promotion. | |
| Immutable artifact plus manual checklist edits | Familiar Phase 30 precedent but risks split-brain manual state. | |
| Structured registry regenerating the checklist | Exhaustive but a broad historical checklist migration outside Phase 35. | |

**User's choice:** Auto-selected the recommended closed typed decision matrix.
**Notes:** Dedicated exact rows are preferred when existing rows are broader; every excluded category and non-allowlisted row receives an explicit non-promotion/unchanged decision.

## the agent's Discretion

- Exact module/type/file names and helper factoring.
- Exact digest-chain representation and dedicated v1.2 row identifiers within the locked boundaries.
- Exact fixture organization and simulation command names.

## Deferred Ideas

- Signed cross-organization attestations.
- A reusable all-Rust HIL runner.
- Full checklist-registry migration and regeneration.
