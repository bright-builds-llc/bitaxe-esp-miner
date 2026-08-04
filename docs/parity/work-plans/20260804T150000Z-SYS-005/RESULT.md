# SYS-005 work result

- Parity row: `SYS-005`
- Final status: `verified`
- Implementation commit: `5c3866760830bf911a6be8eefe61bfc08ddfc99b`
- Reference commit: `c1915b0a63bfabebdb95a515cedfee05146c1d50`
- Hardware attempts: none

## Evidence and verification

The Rust runtime now shares one checked absolute-deadline contract across the
operator-observation, safety-supervisor, and production-readiness owners. The
contract starts at an explicit boundary, coalesces elapsed slots without drift,
preserves boundaries across clock regression, and fails without mutating state
on zero cadence or deadline overflow.

The production owner no longer restarts a relative one-second timeout after
every inbox message. It waits only until the next absolute readiness boundary,
handles any transport or ASIC message, and then performs the authoritative
reread when due. Continuous traffic therefore cannot starve readiness changes.
Category wakeups satisfy the same read without duplicating it. Deadline
overflow requests a fail-closed shutdown.

The firmware owner graph is bound by source-ownership regressions: safety starts
before the production owner, networking starts after the production owner, and
the network notification follows network admission. The production inbox and
ASIC command path are bounded channels. Existing bridge tests prove fresh work
dispatch outranks result polling, each poll slice is at most 100 ms, timeout is
non-terminal, stale generations cannot submit, and replacement work re-arms
polling. The strengthened lifecycle test proves no submit exists before a
correlated ASIC result. Existing recovery tests prove submissions and work are
invalidated before ASIC stop, connection close, hardware safe stop, and final
publication.

Focused verification passed:

- four `bitaxe-core` absolute-deadline tests;
- the production result-before-submit lifecycle test;
- the firmware runtime source-ownership target;
- the real ESP32-S3 release firmware build.

The following repository-wide gates passed on the implementation commit:

- `cargo fmt --all`
- `cargo clippy --all-targets --all-features -- -D warnings`
- `cargo build --all-targets --all-features`
- `cargo test --all-features`
- `bun scripts/bright-builds-check.ts all`
- `just test` (all 29 Bazel test targets passed)
- `just parity`
- `just parity-progress`
- `just verify-redaction`
- `just verify-reference`
- `git diff --check`

## Conclusion

The bounded software orchestration contract is verified with `unit,workflow`
evidence. Rust does not reproduce the upstream task-per-file layout; it proves
the equivalent observable priority, lifecycle, bounded-wait, ownership, and
fail-closed behavior through its deeper event-driven owners.

## Non-claims

This result does not verify live FreeRTOS timing under load, production mining,
pool connectivity, ASIC traffic, accepted or rejected shares, active voltage,
fan or power effects, fault injection, soak behavior, credentials, OTA,
recovery, other boards, direct UART, or pins. Those remain owned by their
separate hardware and parity rows.
