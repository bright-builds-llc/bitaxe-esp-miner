# Parity work plan

- Run ID: `20260804T150000Z-SYS-005`
- Parity row: `SYS-005`
- Initial status: `not-started`
- Source commit: `fbf316a403c1c0da3e87a844705dd0fa0f8fbe7a`
- Reference commit: `c1915b0a63bfabebdb95a515cedfee05146c1d50`
- Active task: `task-parity-sys005-runtime-orchestration`

## Selection

The deterministic selector reported no open plan after `IO-002` closed. The
implemented candidates before `SYS-005` retain their previously audited live
hardware, broad settings/network/API, mining, safety-effect, recovery,
other-board, or installed-UI evidence gaps. The in-progress display, input, and
statistics rows likewise need physical/UI or live mining observations that the
current evidence does not supply. None can be promoted by reclassifying an
existing artifact.

`SYS-005` is the first implementation-actionable candidate. Later phases have
already established strong separate owners for pool work construction, bounded
ASIC result polling/correlation, read-only operator observations, safety
supervision, and effect-gated hardware preparation. The remaining gap is an
explicit shared scheduling contract plus seam-level proof that those owners
preserve the pinned reference lifecycle, priority, bounded-wait, and fail-closed
behavior without relying on the upstream FreeRTOS task layout.

## Scope and non-scope

Add one pure periodic-deadline abstraction in `bitaxe-core` and consume it from
the firmware's operator-observation, safety-supervisor, and production-session
owners. Preserve event-driven wakeups while making the authoritative reread,
sensor sweep, supervisor yield, missed-deadline coalescing, and overflow
behavior explicit and testable.

Add focused behavioral coverage proving that fresh work dispatch outranks
result polling, each poll is bounded, timeouts remain non-terminal, stale
generations cannot submit, result correlation precedes submit intent, readiness
is reread after bounded waits, and fail-closed stop ordering invalidates work
before hardware shutdown. Add a narrow source-ownership regression proving one
boot-lifetime owner for each firmware seam and the intended startup notification
ordering.

Do not reproduce the upstream task-per-file layout, add a scheduler framework,
enable mining or hardware control, change authorization gates, read
credentials, contact a pool, touch hardware, change sensor/ASIC electrical
configuration, or claim live FreeRTOS timing and loaded-hardware behavior.

## Implementation

- [ ] Add a small pure periodic schedule type with deterministic initial,
      on-time, overrun, clock-regression, and overflow behavior.
- [ ] Replace local cadence arithmetic/constants where practical and bind the
      three runtime owners to the shared contract without widening their
      responsibilities.
- [ ] Add behavioral orchestration and source-ownership regressions covering
      work, result, readiness, safety, startup ordering, and bounded queues.
- [ ] Record focused and repository-wide verification in `WORKLOG.md`; create
      `RESULT.md` only if all acceptance criteria pass.

## Verification and promotion

Run focused `bitaxe-core`, `bitaxe-stratum`, firmware host-orchestration, and
real ESP32-S3 firmware build targets. Then run, in order, `cargo fmt --all`,
strict all-target/all-feature Clippy, all-target/all-feature build, all-feature
tests, Bright Builds checks, `just test`, `just parity`, `just
parity-progress`, redaction, reference cleanliness, and diff checks.

Transition only `SYS-005` to `verified` with `unit,workflow` evidence if the
pure deadline contract, end-to-end work/result ordering, bounded waits and
backpressure, authoritative rereads, fail-closed stop sequence, unique owners,
startup order, real firmware build, and all gates pass. Otherwise transition at
most to `implemented` and record the exact residual boundary. Hardware timing,
active mining, ASIC traffic, and safety-control effects remain non-claims.
