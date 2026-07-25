# Phase 32: Shared I2C and Read-Only Sensor Acquisition - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md; this log preserves the alternatives considered.

**Date:** 2026-07-13
**Phase:** 32-shared-i2c-and-read-only-sensor-acquisition
**Mode:** Yolo
**Areas discussed:** Shared I2C0 ownership, producer cadence and failure isolation, read-only sensor validation

***

## Shared I2C0 ownership

| Option | Description | Selected |
| --- | --- | --- |
| Dedicated owner task | One task owns both display initialization and periodic sensors. | |
| Borrow then move one bus | Startup borrows one bounded bus, then the bus moves into the sole producer. | ✓ |
| Shared mutex | Display and independent sensor clients share `Arc<Mutex<_>>`. | |

**User's choice:** Auto-selected recommended default: borrow then move one bus.
**Notes:** This is the smallest fit for the existing startup-only display while preserving one physical and logical lifecycle. Display failure must return ownership intact.

***

## Producer cadence and failure isolation

| Option | Description | Selected |
| --- | --- | --- |
| Fixed single-owner sweep | One 500 ms bounded sweep with independent fact reduction. | ✓ |
| Staggered deadline wheel | One owner schedules facts on separate deadlines. | |
| Independent mutex producers | Each sensor owns its cadence behind a shared mutex. | |

**User's choice:** Auto-selected recommended default: fixed single-owner sweep.
**Notes:** Successful sources alone advance their stamps. Failures preserve prior stamps, continue the sweep, and become typed fault/stale states under producer control.

***

## Read-only sensor validation

| Option | Description | Selected |
| --- | --- | --- |
| Typed read-only commands | Closed register allowlists, atomic INA260 admission, independent EMC2101 facts. | ✓ |
| Reuse legacy initialization | Invoke existing EMC2101 initialization and sensor helpers unchanged. | |
| Generic raw register access | Let the producer choose arbitrary read/write registers. | |

**User's choice:** Auto-selected recommended default: typed read-only commands.
**Notes:** Existing EMC2101 configuration/fan writes and DS4432U voltage writes remain unreachable. Invalid or partial samples never become fresh.

## the agent's Discretion

- Exact module and task names.
- Exact finite transaction deadline, stale threshold, and overrun policy within the 500 ms cadence.
- Exact validation ranges and pure helper organization, subject to upstream register semantics and Phase 31 truth rules.

## Deferred Ideas

- Runtime display updates.
- Phase 33 settings durability, Phase 34 coherent snapshots, and Phase 35 correlated evidence/promotion.
- All active hardware effects, mining, credentials, direct UART/pins, OTA, other boards, and archived diagnostic work.
