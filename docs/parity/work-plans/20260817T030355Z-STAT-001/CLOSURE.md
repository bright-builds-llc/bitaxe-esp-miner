# Parity work closure

- Parity row: `STAT-001`
- Final status: `implemented`
- Outcome: `blocked`
- Verification claimed: `no`
- Plan SHA-256: `815bd7c9ee11bc6ac10051b7136678cf5aec6831e354333f85e665a39fb1f402`
- Active task: `task-parity-stat001-hashrate-monitor`

## Closure reason

Exact pushed source/package
`43acffd3972e85a9a2c5ef30d3063fd6a887e622`, the pinned reference, every
focused and mandatory software/privacy/package gate, and one fresh detector
passed. The sole attempt-011 capture then failed closed after 5 of 20 required
windows with `hardware_blocked` / `watchdog_unresponsive` /
`watchdog_invalid_observation` while the closed owner phase was
`waiting_inbox`. This is objectively distinct from attempts 008-010, which
reached 14 windows with `watchdog_feed_stale` at the undifferentiated
publication boundary.

The v13 result and v7 network documents are independently hash-bound and
mode-protected. Runtime identity and attestation were trusted, parse-failure
counts were zero, same-boot/package correlation, active state, and safety were
valid, terminal HTTP and WebSocket observations, terminal pool persistence,
safe stop, and USB cleanup passed, redaction was true, and no public projection
was written. Work-renewal evidence is incomplete because the primary watchdog
failure stopped the campaign; it is not a second cause.

Source tracing identifies a concrete cross-task snapshot race consistent with
the sealed category. `collect_operator_snapshot_candidate` reads the current
monotonic millisecond before `runtime_health_adapter::collect` locks and copies
the watchdog observation history. The owner can record a newer successful feed
between those operations. The evaluator then receives a copied feed timestamp
newer than its earlier evaluation timestamp, so checked age subtraction fails
closed as `invalid_observation`. Attempt-011 does not independently prove that
interleaving, but the new discriminator rules out the prior publication stall
and makes this the targeted next software boundary.

Attempt-011 is consumed, no retry ran, and terminal outcome is
`stop_hardware_blocker`. STAT-001 remains `implemented`; checklist and progress
history are unchanged.

## Next safe action

Do not retry attempt-011 or authorize attempt-012 from this evidence. A fresh
software-only STAT-001 plan should make runtime-health evaluation time no
earlier than the atomically copied producer observations, add a controlled
interleaving regression that fails before and passes after the correction,
preserve every existing age/sequence/clock guard and owner phase, and run all
mandatory gates. Hardware may be reconsidered only after that exact targeted
fix is independently verified and a new complete immutable attempt contract
is committed and pushed.

## Non-claims

This closure does not verify STAT-001, prove the inferred interleaving on the
device, authorize attempt-012, prove live BM1366 hashrate accuracy, complete
twenty windows or 600 active seconds, establish work-renewal continuity or
terminal zero after a complete campaign, prove arbitrary scheduler behavior,
or claim electrical accuracy, profitability, arbitrary profiles or pools,
other boards or ASICs, update/recovery behavior, or release readiness.
