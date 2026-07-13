# Phase 31: Operator Claim and Telemetry Contract - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md; this log preserves the alternatives considered.

**Date:** 2026-07-13
**Phase:** 31-operator-claim-and-telemetry-contract
**Mode:** Yolo
**Areas discussed:** Observation truth states, Producer-owned freshness, Settings and claim admission

## Observation Truth States

| Option | Description | Selected |
| --- | --- | --- |
| State-carrying enum per observation | Encode fresh/stale/unavailable/fault with variant-specific sample and reason data so invalid combinations are unrepresentable. | ✓ |
| Flat record with validated constructors | Keep one fixed record shape and rely on constructors to prevent contradictory optional fields. | |
| Current truth plus separate history record | Store current state and last-good history separately, requiring consumers to join them. | |

**Yolo choice:** State-carrying enum per observation.
**Notes:** This best fits OBS-01, independent fact availability, functional-core design, and the repository's illegal-states-unrepresentable rule. Compatibility numeric values remain a separate projection.

## Producer-Owned Freshness

| Option | Description | Selected |
| --- | --- | --- |
| Boot-scoped stamped observation stored by the producer | Successful acquisitions atomically own value, source sequence, monotonic time, and boot/session identity; consumers only clone state. | ✓ |
| Producer events folded by one snapshot reducer | Introduce the full coherent operator-snapshot event/reducer model now. | |
| Query-time freshness from cached metadata | Let each consumer classify age at request time without changing the acquisition stamp. | |

**Yolo choice:** Boot-scoped stamped observation stored by the producer.
**Notes:** This prevents request traffic from minting freshness and avoids pulling Phase 34's coherent-snapshot machinery into the Phase 31/32 boundary.

## Settings And Claim Admission

| Option | Description | Selected |
| --- | --- | --- |
| Closed v1.2 admission types behind the compatibility parser | Preserve public compatibility handling while granting write/promotion authority only through hostname-only and narrow eligible-claim types. | ✓ |
| Filter the generic patch and keep string admission | Retain broad representable settings and claim strings, then reject them late. | |
| Central versioned policy registry | Build a generic registry for multiple milestone and board policies now. | |

**Yolo choice:** Closed v1.2 admission types behind the compatibility parser.
**Notes:** The narrow type prevents later schema growth or broad compatibility parsing from silently widening v1.2 writes or promotion.

## the agent's Discretion

- Exact type and module names, serialization tags, reason variants, and migration order within the locked contract.

## Deferred Ideas

- Actual sensor acquisition, settings durability, coherent snapshot composition, hardware evidence, and promotion remain in Phases 32-35.
- Every excluded active, credential, archived-lineage, OTA, other-board, mining, and broad-claim category remains outside Phase 31.
