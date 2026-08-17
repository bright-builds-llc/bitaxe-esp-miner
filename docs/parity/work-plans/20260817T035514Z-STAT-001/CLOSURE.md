# Parity work closure

- Parity row: `STAT-001`
- Final status: `implemented`
- Outcome: `blocked`
- Verification claimed: `no`
- Plan SHA-256: `0f9981aff313745a07bdce48edfe1fa7c81da0fe111f8175acc666e4abf3b857`
- Active task: `task-parity-stat001-hashrate-monitor`

## Closure reason

Exact pushed implementation `0b5338f6c1224dbdae6e664cd286e114ad611c6c`
corrects the cross-task snapshot ordering consistent with attempt-011's sealed
`watchdog_invalid_observation` boundary. The read-only firmware adapter now
copies supervisor checkpoints, watchdog history, and owner phase before it
samples current monotonic time. Operator and screen callers can no longer pass
an earlier timestamp into that evaluation.

A controlled regression proves the exact boundary: evaluating a copied feed
against the stale pre-copy caller time yields `invalid_observation`, while
evaluating the same feed at the post-copy time yields participating
`feed_fresh` with age zero. Production source guards prove all observations
precede the clock read and both callers use the zero-argument adapter.
Existing sequence regression, feed-time regression, fresh/stale timeout,
closed reason, phase, wire, retained, and campaign watchdog tests remain green.

The hashrate evidence evaluator now binds 15 source paths, adding the pure
runtime-health core and firmware adapter; Bazel runfiles, the independent Rust
contract, and real-child source drift fixtures agree. Package, formatting,
lint, build, Rust tests, Bright Builds checks, the complete Bazel suite,
privacy, reference, generated-contract, source-ownership, and parity
invariance gates pass. No hardware, credential, detector, protected-attempt,
network-runtime, or public-projection access occurred.

This software evidence cannot establish live BM1366 hashrate or twenty-window
continuity. STAT-001 therefore remains `implemented`; no checklist or progress
transition is justified.

## Next safe action

A future invocation may consider a new attempt-012 only after creating and
committing a complete immutable detector-gated hardware contract bound to
exact pushed implementation `0b5338f6c1224dbdae6e664cd286e114ad611c6c`.
That contract must preserve v13/v7 seals, owner phase, the 15-source evaluator,
protected evidence, cleanup, retry bounds, and accepted stop conditions. This
closure itself authorizes no device use or retry.

## Non-claims

This closure does not verify STAT-001, prove the corrected interleaving was the
only live failure, authorize attempt-012, prove live BM1366 hashrate accuracy,
complete twenty windows or 600 active seconds, prove arbitrary scheduler
behavior, or claim electrical accuracy, profitability, arbitrary profiles or
pools, other boards or ASICs, update/recovery behavior, or release readiness.
