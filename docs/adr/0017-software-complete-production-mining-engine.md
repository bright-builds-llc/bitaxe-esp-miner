# ADR-0017: Deepen the Production Mining Session into the mining engine

## Status

Accepted

## Context

ADR-0016 established one boot-lifetime Production Mining Session, but its
implementation still returned a recursive recovery-action protocol while V1
negotiation, framing, work dispatch, nonce correlation, submit classification,
and ASIC cadence lived in separately callable modules. The orphaned mining-loop
path duplicated admission and work ownership.

That split made the nominal owner shallow: tests could bypass it and prove
protocol or work behavior without proving the admitted production lifecycle.

## Decision

The Production Mining Session is one deep module with a typed event/effect
interface. It privately composes recovery, bounded line framing, request
correlation, the V1 runtime, the production-work registry, submit
classification, bridge cadence, one-shot campaign leases, session-owned
hardware lifecycle state, and immutable session snapshots.

The interface has exactly two adapters:

- deterministic settings, clock, pool, transport, ASIC, and projection fakes;
- the ordinary ESP firmware adapter.

The ordinary adapter may read authoritative, non-secret readiness facts, but it
supplies no campaign lease and always reports actuation as unqualified. It
performs no pool-secret read, connection, socket write, ASIC effect, or
hardware preparation. Deterministic accepted and rejected shares remain
software evidence only.

Hardware readiness is not an externally asserted Boolean. The session requests
a validated profile through `PrepareHardware`, accepts only the matching typed
preparation result, and withholds pool configuration until that succeeds. ASIC
dispatch and poll effects carry the pool generation and valid-job set used to
parse and correlate their results.

The mining-loop and direct fake/live-runtime interfaces have no compatibility
guarantee. Reusable logic is retained behind the Production Mining Session
interface; obsolete callers, tests, markers, and build wiring are removed.

## Consequences

- Callers and lifecycle tests use the same session interface.
- A connection is not running until V1 authorization succeeds, and work stays
  blocked until valid work is dispatched.
- Request identifiers and generations gate submit results; stale, duplicate,
  malformed, mismatched, and post-stop results cannot become accepted shares.
- Safe stop blocks submissions, invalidates generations, stops ASIC
  interaction, closes transports, requests hardware safe-stop, and withholds
  the terminal publication until the matching stop confirmation.
- Explicit fallback preference and automatic fallback restoration are distinct
  policies.
- Real networking, pool credentials, ASIC actuation, hardware evidence, and
  parity promotion require a separate qualification decision.
