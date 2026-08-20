# Parity work closure

- Parity row: `STAT-003`
- Final status: `implemented`
- Outcome: `blocked`
- Verification claimed: `no`
- Plan SHA-256: `43d13ec599e9f46988f0ebb44607dc000eff95db78c37fdc340fe52e14365684`
- Active task: `task-parity-stat003-scoreboard`

## Closure reason

The readiness implementation and attempt-005 rotation passed every focused and
mandatory gate and were pushed at exact source
`a31af2873e6b2d41fe47aa18a57626f33aaf099b`. The sole protected readiness
transaction completed three consecutive configure/subscribe/authorize
sessions, recorded a source/reference-bound private `ready` result, exposed no
protected pool values, and submitted no shares. That objective signal changed
attempt-004's `network_unavailable` boundary before any device effect.

The sole detector admitted one Ultra 205 and the sole attempt-005 campaign then
completed successfully: 600,306 active milliseconds, 20/20 network windows,
19 qualified candidates, 19 accepted and zero rejected shares, trusted runtime
identity, fresh safety, healthy watchdog, work renewal, terminal HTTP/
WebSocket/pool settlement, `mineonboot=false`, confirmed safe stop, and ready
USB cleanup. Its result and seal match, protected modes pass, and the campaign
is redacted.

The evidence workflow proceeded through stable 20-entry pre-restart reads, the
live SPA route, one valid same-device software restart, and stable 20-entry
post-restart reads. It then closed `hardware_blocked` at
`scoreboard restart persistence is invalid` because every non-difficulty field
survived in the same order but all 20 difficulty values changed representation.

Pinned `reference/esp-miner/main/tasks/scoreboard.c` proves this is expected
upstream behavior: `scoreboard_add` retains the full in-memory `double` while
its NVS record uses `%.1f`; `scoreboard_init` reloads that one-decimal record.
The existing Rust `persisted_projection_rounds_only_the_durable_difficulty`
test records the same contract. The attempt-005 verifier nevertheless required
the full pre-restart and post-restart JSON digests to be identical. The public
projection was correctly withheld, and the immutable plan's explicit identity
criterion is not weakened after observing the result.

## Next safe action

Create a fresh software-only `STAT-003` plan that derives restart persistence
from the pinned codec: require every non-difficulty field and ordering to remain
exact, require each post-restart difficulty to equal the independently computed
one-decimal durable projection of its pre-restart value, and retain both
immediate-repeat checks. Add production-shaped red/green tests and bind the
reference `%.1f`/`%lf` semantics uniquely.

That future plan must decide explicitly whether the sealed, mode-correct,
source-bound attempt-005 artifacts may be re-evaluated without another hardware
effect. This closure authorizes neither protected re-evaluation nor attempt-006.
No unchanged mining rerun is warranted: network readiness, mining, shares,
safety, safe stop, cleanup, SPA, restart, and the exact persisted field set were
already observed.

## Non-claims

This closure does not verify `STAT-003`, publish scoreboard evidence, or claim
exact difficulty equality across restart. It does not expose or validate raw
scoreboard entries, pool values, device/network identity, or sensor values. It
does not verify arbitrary profiles or pools, absolute difficulty calibration,
other boards/ASICs, unbounded mining, OTA/recovery, or release readiness.
