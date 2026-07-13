# Phase 12: ASIC And Mining Hardware Evidence - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md. This log preserves the alternatives considered.

**Date:** 2026-06-30T00:14:49.245Z
**Phase:** 12-asic-and-mining-hardware-evidence
**Mode:** Yolo
**Areas discussed:** Hardware gate and recovery protocol, BM1366 init and work/result evidence, controlled mining smoke and soak, checklist promotion and evidence semantics

## Hardware Gate And Recovery Protocol

| Option | Description | Selected |
| --- | --- | --- |
| Strict detector plus recovery gate | Require `just detect-ultra205`, board-info success, documented recovery path, exact allowed commands, and redaction review before live ASIC/mining work. | yes |
| Detector only | Continue after board-info succeeds and rely on operator judgment for the rest. | |
| Manual bench notes only | Record hardware observations without command/evidence structure. | |

**Yolo choice:** Strict detector plus recovery gate.
**Notes:** This carries forward AGENTS.md repo-local hardware guidance and Phase 11's safety evidence protocol. It avoids ad hoc mining stress and keeps blocked or missing prerequisites visible.

## BM1366 Init And Work/Result Evidence

| Option | Description | Selected |
| --- | --- | --- |
| Tiered staged evidence ladder | Progress from detector/package/safe boot to chip-detect, staged init, diagnostic work/result, then mining smoke only after gates pass. | yes |
| One combined live mining proof | Treat one successful mining log as proof for init, work/result, and mining parity. | |
| Documentation-only closure | Update checklist notes without new live evidence. | |

**Yolo choice:** Tiered staged evidence ladder.
**Notes:** This preserves the Phase 3 semantic ASIC boundary and prevents one broad log from overclaiming multiple hardware behaviors.

## Controlled Mining Smoke And Soak

| Option | Description | Selected |
| --- | --- | --- |
| Controlled or fake-pool first, real-pool only through redacted procedure | Use deterministic/fake-pool and bounded controlled conditions first; allow real/public pool only with secret-safe evidence and stop conditions. | yes |
| Real public pool first | Connect directly to a public pool for first evidence. | |
| No mining run in Phase 12 | Leave all mining-loop hardware evidence pending for Phase 13. | |

**Yolo choice:** Controlled or fake-pool first, real-pool only through redacted procedure.
**Notes:** This satisfies STR-06/STR-07 direction while avoiding credentials in evidence and keeping controlled no-share conditions acceptable only when explicitly justified.

## Checklist Promotion And Evidence Semantics

| Option | Description | Selected |
| --- | --- | --- |
| Exact-claim promotion | Promote rows only when evidence covers the exact ASIC/mining claim; split or narrow broad rows when needed. | yes |
| Phase-level promotion | Mark all Phase 12-related rows verified after any successful live run. | |
| Prose-only governance | Rely on narrative evidence notes without parity guard updates. | |

**Yolo choice:** Exact-claim promotion.
**Notes:** This carries forward Phase 8 and Phase 11 evidence governance and keeps `tools/parity` as the canonical validator if new machine-checkable semantics are needed.

## the agent's Discretion

- Exact plan count, evidence file names, probe command shape, JSON schema details, helper module names, and first-smoke path are left to planning and execution.
- Any discretion is bounded by repo-owned tooling, read-only upstream reference policy, functional-core/imperative-shell architecture, safety gates, and secret-free evidence.

## Deferred Ideas

- Final release HTTP/OTA/recovery evidence remains Phase 13.
- Non-205 boards and additional ASIC families remain deferred.
- Long-running performance tuning and unbounded mining stress remain out of scope without a future phase-specific recovery and evidence plan.
