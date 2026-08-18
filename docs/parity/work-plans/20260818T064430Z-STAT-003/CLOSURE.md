# Parity work closure

- Parity row: `STAT-003`
- Final status: `implemented`
- Outcome: `blocked`
- Verification claimed: `no`
- Plan SHA-256: `ca37ecdbdb7d789d1712af9f2c21dc6afa72d9a7b87c96c908e9c03179c34a65`
- Active task: `task-parity-stat003-scoreboard`

## Closure reason

The immutable plan, exact pushed implementation/package `a337babc`, focused
and mandatory gates, one detector, and the sole attempt-001 command passed
admission. The campaign reached 600,320 active milliseconds, all 20 windows,
204 valid scoreboard candidate outcomes, an accepted submit, trusted identity,
fresh safety, stable watchdog, no panic or mixed reset, confirmed safe stop,
ready USB cleanup, and terminal HTTP/WebSocket/pool joins.

It nevertheless failed closed as `terminal_state_unconfirmed`. The latest
serial marker had the closed terminal reason `campaign_lease_consumed` and no
campaign failure diagnostic, but its campaign state was not yet `consumed`.
The network handoff therefore refused to equate a blocker/reason with the
authoritative consumed lifecycle state. The scoreboard verifier correctly did
not proceed to private API, SPA, restart, or durability observations, and no
public projection was written.

The wrapper initially flattened the typed `hardware_blocked` error to
`process_failed` because `ScoreboardEvidenceError` was missing from the shared
typed-failure union. This plan corrects and regresses that reporting defect;
it does not change or reinterpret the consumed campaign evidence.

## Next safe action

Do not rerun attempt-001 or create attempt-002 from unchanged source. A fresh
software-only immutable `STAT-003` plan must reproduce the exact terminal
tuple—lease-consumed reason, non-consumed campaign state, safe stop confirmed,
complete terminal HTTP/WebSocket/pool joins, and no underlying campaign
failure—at the production status-publication/network-handoff boundary. It must
apply a targeted regression-backed correction that publishes the authoritative
consumed state atomically or keeps capture open for that later publication,
without accepting reason-only terminal state or weakening safe stop. Only a
separate later hardware plan may authorize a new ordinal after that correction
is clean, pushed, fully gated, and package-bound.

## Non-claims

This closure does not verify a live scoreboard API or page, NVS persistence,
post-restart boot load, exact hardware difficulty ordering, browser rendering,
arbitrary profiles/pools, other ASICs/boards, unbounded mining, OTA, recovery,
or release readiness. It does not publish scoreboard entries, difficulties,
jobs, extranonce values, times, nonces, version bits, credentials, endpoints,
identities, exact sensors/hashrates, HTTP bodies, serial, commands, PIDs, or
protected traces.
