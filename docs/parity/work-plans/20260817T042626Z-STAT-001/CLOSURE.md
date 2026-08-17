# Parity work closure

- Parity row: `STAT-001`
- Final status: `implemented`
- Outcome: `blocked`
- Verification claimed: `no`
- Plan SHA-256: `51c67bdd9657f077dccb7167bb32c5dd8d9202679d6606e12dc94c704b04e609`
- Active task: `task-parity-stat001-hashrate-monitor`

## Closure reason

Exact pushed source/package
`ae094a1d639fb7ae94905c6ba8977ca358db4f2e`, the pinned reference, every
focused and mandatory software/privacy/package gate, and one fresh detector
passed. The sole attempt-012 capture then failed closed after 13 of 20 windows
with `hardware_blocked` / `watchdog_unresponsive` /
`watchdog_feed_stale` while owner phase was `waiting_inbox`. The attempt-011
`watchdog_invalid_observation` race did not recur, so the pushed snapshot-
ordering correction objectively changed that boundary.

The v13 result and v7 network documents match their SHA-256 seals and required
private modes. Runtime identity and attestation were trusted, parse-failure
counts were zero, same-boot/package correlation, active state, and safety were
valid, terminal HTTP and WebSocket observations, pool persistence, safe stop,
and USB cleanup passed, redaction was true, and no public projection was
written. Work renewal is incomplete only because the primary watchdog failure
stopped the campaign.

Source tracing narrows the new failure to the owner’s readiness-bounded
`recv_timeout` phase. The loop feeds immediately before entering that wait and
derives its timeout from the next one-second readiness deadline, yet the
producer later reports a feed older than the compiled five-second watchdog
timeout. The current closed evidence cannot distinguish scheduler starvation,
an overlong or failed timed wait, or a related task-priority/runtime anomaly;
the phase alone has no entry timestamp or deadline-overrun discriminator.

Attempt-012 is consumed, no retry ran, and terminal outcome is
`stop_hardware_blocker`. STAT-001 remains `implemented`; checklist and progress
history are unchanged.

## Next safe action

Do not retry attempt-012 or authorize attempt-013 from this evidence. A fresh
software-only STAT-001 plan should add a coherent closed wait-entry/deadline/
overrun discriminator and production-shaped scheduler-delay regressions,
confirm the ESP-IDF pthread priority/runtime contract, and preserve value-free
sealed reporting. Hardware may be reconsidered only after a targeted fix or
new discriminator passes every mandatory gate under a fresh immutable
contract.

## Non-claims

This closure does not verify STAT-001, identify scheduler starvation versus a
timed-wait defect, authorize attempt-013, prove live BM1366 hashrate accuracy,
complete twenty windows or 600 active seconds, establish work-renewal
continuity or terminal zero after a complete campaign, or claim electrical
accuracy, profitability, arbitrary profiles or pools, other boards or ASICs,
update/recovery behavior, or release readiness.
