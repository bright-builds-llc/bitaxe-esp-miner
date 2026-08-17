# Parity work closure

- Parity row: `STAT-001`
- Final status: `implemented`
- Outcome: `blocked`
- Verification claimed: `no`
- Plan SHA-256: `5744893e269547d247a89d4b15022630f99902f884fb4e0394be05b60225df2c`
- Active task: `task-parity-stat001-hashrate-monitor`

## Closure reason

Exact pushed source/package
`43cc417822ec64bdf862d1f48a081c1f33c52a9a`, pinned reference, all focused/
mandatory software/privacy/package gates, and one detector passed. The sole
attempt-013 capture failed closed after 12/20 windows with
`hardware_blocked` / `watchdog_unresponsive` / `watchdog_feed_stale`, owner
phase `waiting_inbox`, and wait state `within_deadline`.

The v14/v8 documents match their SHA-256 seals and private modes. Runtime
identity/attestation, parse counts, same-package correlation, active state,
safety, terminal transports, pool persistence, safe stop, cleanup, and
redaction passed; no public projection was written. Work renewal is incomplete
only because the primary watchdog failure stopped the campaign.

The closed combination is contradictory for one coherent owner instant: a
`within_deadline` wait was armed recently, and the owner feeds immediately
before arming it, yet runtime health reported a feed older than five seconds.
Source tracing explains the contradiction: runtime health first copies the
mutex-protected feed history, then separately reads atomic phase/deadline.
The owner can feed and arm a new wait between those reads, yielding old feed
plus new wait facts. Individual stores are coherent, but the combined snapshot
is not.

Attempt-013 is consumed, no retry ran, terminal outcome is
`stop_hardware_blocker`, STAT-001 remains `implemented`, and checklist/progress
are unchanged.

## Next safe action

Do not retry attempt-013 or authorize attempt-014. A fresh software-only
STAT-001 plan should add one bounded coherent snapshot protocol—preferably a
single-writer sequence lock spanning feed and phase/deadline publications with
bounded fail-closed reader retries—then regression-test the exact old-feed/new-
wait interleaving. Preserve lock-free owner writes where supported, priority 5,
v14/v8 value-free reporting, and all existing guards. Hardware requires a
separate complete contract after that fix passes every mandatory gate.

## Non-claims

This closure does not verify STAT-001, prove a timed-wait or scheduler defect,
authorize attempt-014, prove live BM1366 hashrate accuracy, complete twenty
windows/600 active seconds, establish work-renewal or terminal-zero completion,
or claim electrical accuracy, profitability, arbitrary profiles/pools, other
boards/ASICs, update/recovery, or release readiness.
