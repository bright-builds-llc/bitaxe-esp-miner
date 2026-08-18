# Parity work closure

- Parity row: `STAT-003`
- Final status: `implemented`
- Outcome: `blocked`
- Verification claimed: `no`
- Plan SHA-256: `41ca445088dcf15c4c1c46e504a754c61260e7575eb16ccf68e0edb0fc742879`
- Active task: `task-parity-stat003-scoreboard`

## Closure reason

The attempt-003 rotation passed focused/full gates and was committed, pushed,
and packaged at `60a56d4935ced15eeb5ec6950b1ad4ea35fdf223`.
The sole detector admitted one Ultra 205 and the sole capture produced a valid
sealed accepted campaign.

The natural-closure correction passed its exact live boundary. Network v12 was
accepted with 20/20 renewed windows, `accepted_after_serial_close`,
`terminal_close_requested=false`, final consumed state, serial finish, terminal
HTTP/WebSocket/pool confirmation, stable watchdog, and no correlation failure.
The campaign observed real scoreboard candidates and an accepted submit with
trusted identity, fresh safety, confirmed safe stop, and ready cleanup. The
20-entry private scoreboard repeated identically and the live `/scoreboard`
route passed.

The verifier then issued one normal restart. The exact package returned with a
changed boot session, ordinal +1, `software_cpu`, and
`startMiningOnBoot=false`, but `miningActivity=paused`. The firmware's closed
model intentionally uses `paused` for an operator-paused state and
`safe_blocked` for non-operator blockers; both are non-active. The reference
implementation exposes `miningPaused` rather than this Rust extension and
initializes it independently, so it does not justify narrowing disabled boot
mining to only `safe_blocked`. The verifier's hardcoded spelling rejected the
otherwise valid safe restart as public `hardware_blocked`, withheld later
scoreboard reads and projection, and did not retry.

## Next safe action

Create a fresh software-only `STAT-003` plan. Add one pure closed predicate for
disabled boot mining: `startMiningOnBoot=false` and `miningActivity` equal to
either `paused` or `safe_blocked`. Use it in both post-restart admission and the
final evidence boolean. Add real-child tests for both accepted non-active
states and rejection of `active`, unknown strings, and enabled boot mining.
Keep exact package/session/ordinal/reset, scoreboard persistence, safety,
privacy, source identity, and first-failure gates unchanged.

Only after that targeted correction and all gates are clean, committed, pushed,
and package-bound may a separate immutable plan consider attempt-004. A repeat
of this exact post-restart signature after its fix must stop further retries.

## Non-claims

This closure does not verify post-restart scoreboard persistence or promote
`STAT-003`; the post-restart scoreboard was not read after the state predicate
failed. It does not claim arbitrary paused states are safe without disabled
boot intent, reinterpret attempt-003 as accepted public evidence, or verify
arbitrary profiles/pools, other ASICs/boards, unbounded mining, OTA, recovery,
or release readiness. It publishes no protected runtime values.
