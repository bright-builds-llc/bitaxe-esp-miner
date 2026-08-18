# Parity work closure

- Parity row: `STAT-003`
- Final status: `implemented`
- Outcome: `blocked`
- Verification claimed: `no`
- Plan SHA-256: `80aa52a698a3b1e583117c6330ef6d19d47e1710517c7b449fd5711e48a6653f`
- Active task: `task-parity-stat003-scoreboard`

## Closure reason

The software-only plan completed at source commit
`9da1d2c33b3a2c7d2a200f03b19e682f476b87e4`. Network acceptance now requires
no earlier failure, 20 complete windows, final consumed state, serial finish,
and `accepted_after_serial_close`, while finish-time checks still require pool
persistence and terminal HTTP/WebSocket confirmation. It no longer requires
the worker itself to have initiated serial closure.

`terminal_close_requested` remains a serialized closed boolean diagnostic.
Hashrate and scoreboard consumers require its presence and type but accept both
values. Rust and real-child regressions prove worker-requested true and natural
analyzer false acceptance, missing/non-boolean rejection, missing final consumed
withholding, and unchanged failure precedence. All mandatory gates, firmware/
package builds, redaction, and reference checks passed.

No hardware or protected runtime input was used. `STAT-003` remains
`implemented` because deterministic software evidence cannot establish live
scoreboard API/SPA visibility, persistence, or restart durability.

## Next safe action

Create a fresh immutable hardware verification plan that rotates the scoreboard
workflow, Rust/TypeScript contract, paths, plan binding, and ordinal to
attempt-003. After all gates, commit/push, and exact package binding, it may run
one detector and one conditional bounded scoreboard capture under the existing
safety, privacy, recovery, cleanup, and promotion quorum.

If network v12 again records `accepted_after_serial_close`, final consumed, and
serial finished but status failed from the same closure-request signature, stop
without another retry because the targeted fix did not change the real
boundary. Any different failure requires its own closed diagnosis and plan.

## Non-claims

This closure does not verify live scoreboard API/SPA behavior, NVS persistence,
restart durability, hardware difficulty ordering, arbitrary profiles/pools,
other ASICs/boards, unbounded mining, OTA, recovery, or release readiness. It
does not reinterpret or mutate attempt-002 evidence and publishes no protected
runtime values.
