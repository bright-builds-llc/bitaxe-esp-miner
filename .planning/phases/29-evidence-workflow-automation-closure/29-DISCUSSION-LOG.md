# Phase 29: Evidence Workflow Automation Closure - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md; this log preserves the alternatives considered.

**Date:** 2026-07-12
**Phase:** 29-evidence-workflow-automation-closure
**Mode:** Yolo
**Areas discussed:** Canonical root profiles, validation ordering, Phase 28 consolidation

______________________________________________________________________

## Canonical Root Profiles

| Option | Description | Selected |
| --- | --- | --- |
| Typed phase profiles with native inventories | Make phase identity explicit while preserving each wrapper's current partial artifact shape. | |
| Path/content marker heuristics | Extend the current implicit Phase 23/28 classification with more markers. | |
| Shared canonical builder with generic validation | Normalize all workflows to eleven slots but use one generic policy. | |
| Typed profiles plus shared canonical builder | Use explicit phase policy and a shared schema-driven eleven-slot builder/validator. | ✓ |

**User's choice:** Typed profiles plus a shared canonical builder.
**Notes:** Yolo selected the advisor recommendation. It avoids fail-open marker guessing and prevents builder/validator schema drift while preserving phase-specific non-claims.

______________________________________________________________________

## Validation Ordering

| Option | Description | Selected |
| --- | --- | --- |
| Single top-level terminal finalizer | Accumulate workflow status and run redaction, mining-allow, and operator validation once in fixed order. | ✓ |
| Finalizer in every terminal branch | Keep branch-local exits but duplicate finalization calls. | |
| Armed `EXIT` trap finalizer | Use a shell trap to catch normal and unexpected exits. | |

**User's choice:** Single top-level terminal finalizer.
**Notes:** Yolo selected the advisor recommendation. The finalizer runs operator validation exactly once and last, while preserving any earlier detector or workflow failure.

______________________________________________________________________

## Phase 28 Consolidation

| Option | Description | Selected |
| --- | --- | --- |
| Atomic managed-root regeneration | Build and validate a complete staging root, then replace the dedicated destination atomically. | ✓ |
| In-place declarative upsert | Update managed files individually while preserving unrelated destination content. | |
| Immutable generations plus promoted index | Keep every generated root and select the current one through an index. | |

**User's choice:** Atomic managed-root regeneration.
**Notes:** Yolo selected the advisor recommendation. The source and destination are explicit, distinct, and non-nested; output uses stable cross-links/category labels only and retains the prior valid destination on failure.

## the agent's Discretion

- Exact module, type, helper, Bazel target, and generated-file names.
- Stable deterministic wording for typed blocked/deferred slots.
- Internal placement of the shared schema descriptor, provided generation and validation consume the same contract.

## Deferred Ideas

- Phase 30 verified checklist promotion.
- Further live nonce/share diagnosis.
- New hardware, safety, recovery, board-family, protocol, UI, and stress claims.
