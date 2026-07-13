# Phase 26: Telemetry And Parity Closure - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md. This log preserves the alternatives considered.

**Date:** 2026-07-05T03:48:38.522Z
**Phase:** 26 - Telemetry And Parity Closure
**Mode:** Yolo
**Areas discussed:** Runtime event source and counter invariants, API projection shape, WebSocket session correlation and redaction, parity checklist and evidence promotion

## Runtime Event Source And Counter Invariants

| Option | Description | Selected |
| --- | --- | --- |
| Typed v1.1 event log plus pure projections | Single source for API, WebSocket, statistics, scoreboard, and parity; deterministic replay; encodes accepted/rejected shares only from matching submit intent, parsed pool response, and current generation. | yes |
| Existing state with typed counter/evidence gates | Smaller change that builds on `MiningRuntimeState`, `SubmitClassification`, and existing stale/fake-pool guards, but source of truth remains fragmented. | no |
| Redacted runtime evidence ledger as projection input | Strong audit trail for exact parity claims, but heavier for firmware and risks coupling runtime design to artifact lifecycle. | no |

**User's choice:** Yolo selected the shared typed runtime-event projection.
**Notes:** This best fits Phase 26 because API, WebSocket, statistics, scoreboard, and parity claims must derive from one exact runtime source. Counter advancement must require current-generation submit intent plus parsed pool response.

## API Projection Shape

| Option | Description | Selected |
| --- | --- | --- |
| Shared exact-evidence projection feeding AxeOS DTOs | Keeps public endpoints derived from one typed runtime snapshot while preserving upstream field names and compatibility shapes. | yes |
| Endpoint-local promotion | Smallest patch, but high drift risk across `/api/system/info`, statistics, scoreboard, and WebSocket state. | no |
| AxeOS-compatible fields plus Rust-only diagnostic extensions | Makes operator diagnostics explicit, but risks contract drift from upstream AxeOS clients. | no |
| Separate parity/debug projection with conservative public endpoints | Useful if evidence needs more detail than AxeOS fields can carry, but creates another surface that must not drift. | partial |

**User's choice:** Yolo selected shared exact-evidence projection feeding AxeOS-compatible DTOs.
**Notes:** Public route shape should remain compatible. Richer evidence semantics can live in internal types, tests, or evidence artifacts.

## WebSocket Session Correlation And Redaction

| Option | Description | Selected |
| --- | --- | --- |
| Shared redacted mining telemetry projection feeding WebSocket fan-out | Sanitizes before fan-out, preserves `/api/ws/live` full/diff cadence, and gives safe-stop one authoritative reset point. | yes |
| Endpoint-local adapters over same runtime event store | Keeps `/api/ws` raw-log behavior conservative, but risks policy drift unless shared helpers and tests enforce redaction. | partial |
| Session-scoped event ledger with sequence IDs and sanitized snapshots | Strongest reconnect/replay evidence, but more stateful than current snapshot/diff loop. | no |

**User's choice:** Yolo selected shared redacted projection, while preserving `/api/ws` raw-log compatibility.
**Notes:** Safe-stop must reset active-mining state before connect-time full updates or cadence diffs can serialize another frame.

## Parity Checklist And Evidence Promotion

| Option | Description | Selected |
| --- | --- | --- |
| Checklist-first exact delta with Phase 26 evidence | Minimal churn and row-by-row review for a narrow closure. | yes |
| Evidence-root promotion manifest plus parity validator | Best machine-checkable trace if Phase 26 becomes a reusable closure gate. | partial |
| Governance-only freeze with explicit non-claims | Safest if live artifacts are blocked, but does not close API/WebSocket/statistics/scoreboard beyond limited evidence. | fallback |

**User's choice:** Yolo selected checklist-first exact promotion, with manifest/tooling only if planning finds it necessary.
**Notes:** EVD-08 requires exact claim promotion. If Phase 25 live share proof remains blocked, Phase 26 must preserve that non-claim instead of promoting accepted/rejected share behavior.

## Claude's Discretion

- Choose exact module names, projection/event type names, evidence filenames, and whether the shared projection sits in `bitaxe-stratum`, `bitaxe-api`, or a narrow bridge.
- Choose whether a promotion manifest is needed after inspecting `tools/parity` and the final evidence shape.
- Keep public API compatibility, redaction, detector gating, and exact evidence semantics fixed.

## Deferred Ideas

- Live accepted/rejected share proof if detector-gated Phase 25 evidence remains blocked.
- Full active voltage/fan/thermal/fault/self-test closure, non-205 boards, non-BM1366 ASICs, OTA/recovery, display/input, BAP, Stratum v2, and unbounded stress mining.
