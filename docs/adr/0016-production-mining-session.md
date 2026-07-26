# ADR-0016: Make the Production Mining Session the sole mining owner

## Status

Accepted

## Context

Mining behavior was distributed across phase-named compile modes, evidence
loops, HTTP-derived activity state, and firmware branches. Those surfaces made
the lifecycle hard to reason about, coupled current development to retired GSD
work, and allowed settings or projections to imply mining behavior without one
owner enforcing readiness and safe-stop ordering.

The ordinary firmware must remain fail-closed before reading pool secrets or
causing network, ASIC, or hardware effects. Production-shaped behavior still
needs deterministic software verification before those effects are qualified.

## Decision

Introduce one boot-lifetime Production Mining Session. Its functional core owns
operator intent, readiness evaluation, pool selection and recovery, work and
submission generations, safe stop, and the derived public mining lifecycle.
A thin ESP implementation supplies authoritative snapshots and category-only
notifications.

Readiness is evaluated in this order: operator intent, network availability,
Stratum V1 support, fresh safety prerequisites, production ASIC readiness, and
explicit actuation qualification. Pool configuration is loaded lazily only
after every gate passes. The ordinary ESP implementation reports actuation as
unqualified, so real pool, ASIC, and hardware effects remain unreachable in
this change.

Retired GSD-era mining runtimes, compile modes, scripts, build targets, tests,
and compatibility shims may be removed together with their active callers.
Historical `.planning/` archives and formal evidence documents remain
unchanged.

## Consequences

- Pause and resume mutate typed operator intent only; the session publishes the
  resulting activity and pool lifecycle.
- Safe stop has one idempotent ordering and invalidates work and submissions
  before external resources are stopped.
- Production recovery policy is virtual-time testable without hardware or pool
  access.
- Current development no longer preserves executable compatibility with
  phase-named mining workflows.
- Real TCP/TLS and ASIC actuation require a later qualification decision and
  hardware evidence.
