# Parity work closure

- Parity row: `STAT-001`
- Final status: `implemented`
- Outcome: `blocked`
- Verification claimed: `no`
- Plan SHA-256: `da3c3eb4fa4d4a9f949307db2b0e6e905f4e905ad31352a271a0b52ff1096205`
- Active task: `task-parity-stat001-hashrate-monitor`

## Closure reason

Exact clean pushed source/package
`1892800bbaf4eba2dd1d5c076699b41ed09908a1`, the pinned reference, all focused
and mandatory software/privacy/package gates, and one protected detector passed.
The sole attempt-015 campaign stopped after 364,110 accumulated active
milliseconds and 12 of 20 credited windows with the sealed v15/v9 signature
`hardware_blocked` / `watchdog_unresponsive` / `watchdog_feed_stale` / read
outcome `stable` / owner phase `handling_inbox` / wait state `not_waiting`.

This is the first trustworthy failure-time tuple after the attempt-014
diagnostic correction. The stable read outcome rules out coherent-store retry
exhaustion, history poison, and genuine uninitialized state. The owner was not
inside its bounded receive wait; it was processing an inbox message before a
later successful watchdog feed. Runtime/package identity and attestation were
trusted; active safety and same-package state were valid; terminal HTTP,
reconstructed WebSocket, and pool persistence passed; safe stop was confirmed;
USB cleanup was ready; private modes and result/network digests matched; and
redaction/projection withholding passed. No public evidence was written.

The current phase spans inbox-to-event mapping, pure session evaluation, and
the complete feedback/effect cascade. It cannot identify which boundary held
the owner beyond the compiled five-second timeout, so selecting a source fix
would still be guesswork. Attempt-015 is consumed, no retry ran, terminal
outcome is `stop_hardware_blocker`, and STAT-001/checklist/progress remain
unchanged.

## Next safe action

Do not retry attempt-015 or authorize attempt-016. A fresh software-only
STAT-001 plan should add one closed value-free owner subphase spanning at least
inbox mapping, session evaluation, effect execution, and completed-effect
progress. It should carry the subphase and closed effect category through
runtime health and sealed v16/v10 evidence, reproduce the post-window stable-
read stale-feed transition at the production boundary, and apply the minimum
regression-backed correction. Hardware requires a separate complete contract
after that work is pushed and fully gated.

## Non-claims

This closure does not identify a specific inbox message or effect, prove
scheduler starvation, transport blocking, or an ESP-IDF watchdog defect;
authorize attempt-016; verify STAT-001 or live BM1366 hashrate accuracy;
complete twenty windows or 600 active seconds; establish work renewal or
terminal zero; or claim arbitrary profiles/pools, other boards/ASICs,
update/recovery, profitability, or release readiness.
