# Parity work closure

- Parity row: `STAT-003`
- Final status: `implemented`
- Outcome: `blocked`
- Verification claimed: `no`
- Plan SHA-256: `43c88184aeb7a41e1fe169ae22aee94283df1a72934a7cc4d1a431834c7cb86a`
- Active task: `task-parity-stat003-scoreboard`

## Closure reason

The attempt-002 contract rotation and 31-path transitive evaluator identity
passed focused/full gates and was committed, pushed, and packaged at
`e9034ea11d4d9de5ebea9c34198a2f26b49a387b`. The sole detector admitted one
Ultra 205 and the sole capture produced a valid sealed campaign result.

The campaign itself reached accepted submit after 600,148 active milliseconds,
20/20 renewed windows, real scoreboard candidates, trusted identity, fresh
safety, stable watchdog, no panic or mixed reset, terminal HTTP/WebSocket/pool
confirmation, final consumed serial state, confirmed safe stop, and ready USB
cleanup. Network settlement reached the closed state
`accepted_after_serial_close` with final consumed and serial-finished facts.

Network status nevertheless failed because `terminal_close_requested` was
false. The worker did not need to request closure: the serial analyzer naturally
finished and handed off authoritative consumed state first. The current
acceptance model incorrectly treats the optional closure initiator diagnostic as
a mandatory truth fact. The verifier therefore returned public
`evidence_invalid`, correctly withheld API/SPA/restart work and the public
projection, and did not retry.

## Next safe action

Create a fresh software-only `STAT-003` plan. Add a production-shaped regression
where serial is already finished with final consumed state and complete terminal
transports before the worker requests closure. Remove `terminal_close_requested`
from acceptance truth while retaining it as a required closed boolean diagnostic;
continue to require `accepted_after_serial_close`, final consumed state, serial
finish, HTTP/WebSocket/pool confirmation, 20/20 windows, work renewal, safety,
watchdog, identity, and first-failure precedence. Rotate both hashrate and
scoreboard consumers so false is valid but missing/non-boolean still fails.

Only after that exact fix and all gates are clean, committed, pushed, and
package-bound may a separate immutable hardware plan consider attempt-003. The
same boundary recurring after its targeted fix must stop without another retry.

## Non-claims

This closure does not verify live scoreboard API or SPA rendering, scoreboard
NVS persistence, restart durability, or `STAT-003`. It does not claim the
attempt-002 network document is accepted or reinterpret failed evidence. It
does not verify arbitrary profiles/pools, other ASICs/boards, unbounded mining,
OTA, recovery, or release readiness, and publishes no protected scoreboard,
credential, endpoint, device, network, sensor, hashrate, HTTP, serial, process,
or command values.
