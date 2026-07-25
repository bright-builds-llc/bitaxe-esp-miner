# Phase 36: Substantive Evidence Admission and Exact Re-Promotion - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in `36-CONTEXT.md`; this log preserves alternatives considered.

**Date:** 2026-07-23T15:26:57.039Z
**Phase:** 36-substantive-evidence-admission-and-exact-re-promotion
**Mode:** Yolo
**Areas discussed:** substantive sensor/runtime-health admission, independently observed runtime identity, independent no-actuation proof, claim-specific re-promotion and rerun gate

## Substantive Sensor and Runtime-Health Admission

| Option | Description | Selected |
| --- | --- | --- |
| Typed substantive fact bundle with explicit boot-provenance join | Parse every sensor and health fact into strict domain types and retain enough information for claim-specific review. | ✓ |
| Canonical substantive-facts digest | Validate API/WebSocket facts and bind a compact, versioned canonical digest into retained evidence. | |
| Shared canonical firmware boot identity | Migrate sensor and operator snapshots to one source-level boot identity before admission. | |

**Agent choice:** Typed substantive fact bundle with an explicit provenance bridge.
**Notes:** Keep the existing identity validator unchanged and layer stricter claim-specific parsing around it. Canonical digests remain a representation option only when exhaustively typed and mutation-tested.

## Independently Observed Runtime Identity

| Option | Description | Selected |
| --- | --- | --- |
| Replay full device-session ledger | Recompute same-device and Boot B identity from immutable request, event ledger, private result, and public projection. | ✓ |
| Admit terminal result and projection | Parse the paired result/projection without replaying the full observation chain. | fallback |
| New on-device running-partition measurement | Add stronger firmware measurement and capture it in a separately authorized run. | deferred |

**Agent choice:** Replay the complete immutable device-session chain when present, with a narrowly typed terminal-result fallback.
**Notes:** Package fields cannot authenticate observation. Missing immutable provenance yields a typed insufficiency result.

## Independent No-Actuation Proof

| Option | Description | Selected |
| --- | --- | --- |
| Retrospective Attempt 31 classifier | Prove complete independent observation from immutable legacy artifacts or fail closed. | ✓ first |
| Capability-enforced effect broker | Own every permitted effect through an exclusive typed ledger in a future run. | ✓ if needed |
| External process/network observer | Trace host effects independently through platform facilities. | |
| Broker plus firmware-owned effect counters | Add device-side defense in depth to host ledgering. | deferred |

**Agent choice:** Classify Attempt 31 first; if insufficient, require an exclusive broker-ledger contract before a later authorized run.
**Notes:** The supervisor Boolean, chronology, source guards, and cleanup cannot prove absence independently. Structural guards must close direct-command bypasses.

## Claim-Specific Re-Promotion and Rerun Gate

| Option | Description | Selected |
| --- | --- | --- |
| Phase 36 successor schema with one-pass re-evaluation | Preserve history, fingerprint the prior generation, and atomically publish per-claim corrections. | ✓ |
| Mutate Phase 35 v1 validation and generation | Retrofit new meaning into the existing schema and destination. | |
| Conservative demotion then later re-promotion | Publish an intermediate correction before all successor validators complete. | |
| Fresh Phase 36 hardware root | Capture all new roles natively after explicit authorization. | only after typed insufficiency |

**Agent choice:** A successor schema with common root prerequisites and explicit per-claim gates.
**Notes:** The matrix may demote unsupported rows and must not require four promotions. Hardware is not an automatic fallback.

## the agent's Discretion

- Exact Rust type/module/schema names and canonical fact encoding.
- Exact supersession metadata and atomic generation layout.
- Exact future broker implementation, within the locked exclusivity, privacy, and non-bypass requirements.

## Deferred Ideas

- On-device running-partition measurement.
- Firmware-owned actuator counters as defense in depth.
- Cross-platform external observation and a reusable all-Rust HIL runner.
