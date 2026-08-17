# Parity work closure

- Parity row: `STAT-001`
- Final status: `implemented`
- Outcome: `blocked`
- Verification claimed: `no`
- Plan SHA-256: `718099b6150b0ecf321e3ff81614cf35f4566f0867d84096ab084a38757f0a4d`
- Active task: `task-parity-stat001-hashrate-monitor`

## Closure reason

Exact pushed implementation `9604d145f92f6d1b93fa446ce24154e2ccb04e5f`
adds the closed evidence needed to distinguish attempt-012's waiting-inbox
boundary. Before publishing `waiting_inbox`, the owner now atomically writes a
wrap-aware absolute receive deadline and validity bit, then release-publishes
the phase. Runtime health copies that coherent observation before its clock
sample and derives only `not_waiting`, `within_deadline`, `deadline_overrun`,
or `invalid_observation`.

The state is projected through HTTP/WebSocket runtime health, exact retained
text, v8 network evidence, v14 result evidence, and seal-gated public watchdog
diagnostics. Public failure output carries only the closed state label; it does
not expose timestamps, deadlines, durations, identities, or private values.
Missing/unknown values fail closed and earliest failure precedence remains.

Exact deadline, one-millisecond overrun, missing/overflow, non-waiting,
production scheduler-delay, modulo-u32 wrap, deadline-before-phase, phase/
deadline-before-clock, one-second budget, schema/seal, redaction, and real-
child tests pass. The real Xtensa build proves the lock-free AtomicU32/validity
implementation. ESP-IDF pthread priority is explicitly pinned to 5 and bound
to upstream priority-5 coordinator/Stratum breadcrumbs. The evaluator identity
now covers 18 source paths. Package, format, lint, build, Rust tests, Bright
Builds checks, complete Bazel suite, privacy, reference, and parity gates pass.
No hardware, credentials, protected attempt, network runtime, or projection
candidate was accessed.

This diagnostic evidence cannot establish live BM1366 hashrate or twenty-
window continuity. STAT-001 remains `implemented`; no checklist/progress
transition is justified.

## Next safe action

A future invocation may consider attempt-013 only after creating and committing
a complete immutable detector-gated contract bound to exact implementation
`9604d145f92f6d1b93fa446ce24154e2ccb04e5f`. A live stale-feed result with
`deadline_overrun` would isolate an overlong timed wait or scheduler delay;
`within_deadline` would contradict the phase/deadline snapshot and require a
different producer investigation. Preserve v14/v8 seals, priority 5, the
18-source evaluator, protected evidence, cleanup, retry, and stop rules. This
closure authorizes no device use.

## Non-claims

This closure does not verify STAT-001, identify timed-wait overrun versus
scheduler starvation on hardware, authorize attempt-013, change priority or
watchdog timeout, prove live BM1366 hashrate accuracy, complete twenty windows
or 600 active seconds, or claim electrical accuracy, profitability, arbitrary
profiles or pools, other boards or ASICs, update/recovery behavior, or release
readiness.
