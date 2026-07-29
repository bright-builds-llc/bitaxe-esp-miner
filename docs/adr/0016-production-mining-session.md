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
Stratum V1 support, fresh safety prerequisites, a validated one-shot campaign
lease, and explicit actuation qualification. The session then requests typed
hardware preparation and owns the resulting preparing, ready, safe-stopping,
and stopped state. Pool configuration is loaded lazily only after hardware
preparation succeeds. The ordinary ESP implementation supplies no lease and
reports actuation as unqualified, so real pool, ASIC, and hardware effects
remain unreachable in this change.

Campaign leases carry a validated hardware profile and one bounded stop
condition: the first accepted or rejected submit response with a timeout, or
an authorized active-mining duration. A consumed lease identifier cannot be
reused during the boot-lifetime session.

Retired GSD-era mining runtimes, compile modes, scripts, build targets, tests,
and compatibility shims may be removed together with their active callers.
Historical `.planning/` archives and formal evidence documents remain
unchanged.

## Consequences

- Pause and resume mutate typed operator intent only; the session publishes the
  resulting activity and pool lifecycle.
- Safe stop has one idempotent ordering: block submissions, invalidate work and
  generations, stop ASIC interaction, close transports, request hardware
  safe-stop, and publish the terminal snapshot only after confirmation.
- Production recovery policy is virtual-time testable without hardware or pool
  access.
- ASIC dispatch and polling effects carry their pool generation and valid-job
  context across the adapter boundary.
- Current development no longer preserves executable compatibility with
  phase-named mining workflows.
- Real TCP/TLS and ASIC/hardware actuation require later adapter qualification
  and hardware evidence.
