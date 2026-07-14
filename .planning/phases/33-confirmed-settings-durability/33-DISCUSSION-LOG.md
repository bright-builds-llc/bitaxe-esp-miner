# Phase 33: Confirmed Settings Durability - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-07-14
**Phase:** 33-confirmed-settings-durability
**Mode:** Yolo
**Areas discussed:** Storage confirmation transaction, compatibility and publication, normal reboot durability proof

***

## Storage Confirmation Transaction

| Option | Description | Selected |
| --- | --- | --- |
| Single serialized confirmation transaction | Always write and commit, including same-value requests; independently reload, reconcile, and atomically publish before success. | ✓ |
| Reload-confirmed same-value no-op | Avoid flash wear for same-value requests while still reloading and reconciling, but diverge from the literal write-and-commit success contract. | |
| Optimistic writers with rollback | Allow concurrent writes with generation checks and compensating commits, at the cost of ambiguous ordering and unprovable rollback. | |

**User's choice:** Auto-selected the recommended single serialized confirmation transaction.
**Notes:** Public readers retain the prior confirmed value until atomic publication. A post-commit confirmation failure must not claim rollback or unchanged durable storage.

***

## Compatibility and Publication

| Option | Description | Selected |
| --- | --- | --- |
| Compatibility-first response with exact-set authority | Preserve broad response behavior, allow only exact hostname to write, and publish only reloaded truth. | ✓ |
| Reject every unsupported field set | Return explicit failures for unknown, unsupported, empty, or mixed inputs, breaking established compatibility. | |
| Persist hostname from mixed patches | Ignore other fields while updating hostname, creating partial-change and excluded-field ambiguity. | |

**User's choice:** Auto-selected the recommended compatibility-first response with exact-set authority.
**Notes:** Valid compatibility-only inputs remain empty-success no-ops. Invalid known input stays a generic atomic error. Requested-write overlays and secret-bearing diagnostics are prohibited.

***

## Normal Reboot Durability Proof

| Option | Description | Selected |
| --- | --- | --- |
| Operator restart plus one preflight detector and passive same-board lock | Put detector resets before the proof window, then correlate one application restart through stable physical identity and passive capture. | ✓ |
| Detector before and after restart | Reuse board-info twice, but add a second reset and weaken the exactly-one-normal-reboot claim. | |
| HTTP-only reacquisition | Use only service recovery and API readback, which is insufficient same-board evidence by itself. | |

**User's choice:** Auto-selected the recommended operator restart with one detector preflight and passive same-board lock.
**Notes:** The proof excludes raw reset/power, flash, OTA, fault reset, direct UART/pins, archived lineage work, and Phase 35 promotion. Raw local traces remain protected and uncommitted.

## the agent's Discretion

- Exact transaction and confirmation-state type names.
- Exact repo-owned evidence helper or narrow flash-tool extension.
- Exact category-only log labels and digest encodings.

## Deferred Ideas

None.
