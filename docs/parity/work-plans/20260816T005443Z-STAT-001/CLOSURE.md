# Parity work closure

- Parity row: `STAT-001`
- Final status: `implemented`
- Outcome: `blocked`
- Verification claimed: `no`
- Plan SHA-256: `32321e5916949d1e6e3b41454c8eb4d168b7cd1a42d3e006e78d231fc74bf07b`
- Active task: `task-parity-stat001-hashrate-monitor`

## Closure reason

Detector command 1 admitted exactly one Ultra 205, but the sole attempt-001
capture command stopped before its campaign child launched. The typed terminal
category was `evidence_invalid`; the public closed facts were
`stage=hashrate_monitor_capture` and `projection_published=false`. The safe
summary identified pinned-reference semantic admission. Its root cause was an
overbroad `update_hash_counter` fragment: the token legitimately occurs eight
times in the pinned reference while the wrapper required exactly one.

The attempt root and public projection remained absent, so the capture command
performed no flash or mining effect. Starting command 2 nevertheless consumed
attempt-001. The plan prohibits an unchanged retry or attempt-002, so this plan
ends without a checklist transition or progress synchronization.

## Next safe action

The preflight now binds the unique upstream function definition and the unique
production worker emission site, and a regression validates the complete
current task/plan/source/reference admission surface. Commit and push that
correction and this closure. A future immutable STAT-001 plan may authorize a
fresh attempt-002 only after full gates pass and a new exact clean package is
built and detector-admitted.

## Non-claims

This closure does not verify live hashrate values, BM1366 counter accuracy,
HTTP or WebSocket hashrate coherence, rolling-window behavior, terminal zero
telemetry, safe stop after mining, share behavior, profitability, thermal or
electrical accuracy, other profiles, ASICs, or boards. It does not promote
STAT-001 and supplies no reusable hardware evidence.
