# ADR-0032: Accept Share002's retained resource proof as a permanent non-claim

Accepted 2026-10-05 by explicit owner decision ("accept Share002 as a
non-claim"). It amends [ADR-0029](0029-piecewise-str005-qualification.md) and
complements [ADR-0031](0031-prospective-panic-diagnostics.md).

## Context

Share002 (firmware `cf7a3f03`, Gate `e20c0fd5`) sealed unverified after a
Start failure and a panic reset. Ordinal 18 was issued and its delivery was
attempted, but whether the device consumed it is unknown
([report](../parity/evidence/20260927-str005-v2-share-start-unverified.md)).

`task-str005-failure-recovery-accounting` built a failure-only collector and
measured the device's authenticated accounting and restoration
([evidence](../parity/evidence/20260927-str005-failure-recovery-accounting.md)).
The one item it could not supply is Share002's retained resource record. The
device reports idle status with `record:null`; the reset cleared the record,
and the firmware keeps nothing that could reconstruct it. Every later Start has
since charged its exact ledger increment from that measured baseline.

ADR-0031 admitted new diagnostics despite the gap, but it did not complete the
recovery task or allow promotion. The STR-005 integration review depends on
that task, so the gap blocked promotion indefinitely.

## Decision

Share002's retained resource proof is a permanent non-claim:

- It is never supplied, inferred or reconstructed. Current idle state, later
  restoration and host release do not prove that Share002's socket, worker or
  fence were released before the reset.
- Share002's seal, its failed outcome and Channel006's private status stay
  unchanged. Neither is relabelled as evidence for any later image.
- `task-str005-failure-recovery-accounting` completes on what it measured:
  authenticated ledgers, restoration, inactive leases, preservation and host
  release. Its last checkbox is closed by this decision, not by evidence.
- The integration review treats the dependency as satisfied with this
  non-claim recorded. STR-005 parity claims rest on later, complete results on
  the final candidate, which do not depend on Share002.

## Consequences

Promotion is no longer blocked by an unrecoverable historical record. The
STR-005 evidence must list this non-claim wherever it describes Share002, and
any future claim about pre-reset resource release on `cf7a3f03` remains
unsupported.
