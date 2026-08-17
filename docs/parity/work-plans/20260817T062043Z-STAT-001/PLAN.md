# Parity work plan

- Run ID: `20260817T062043Z-STAT-001`
- Parity row: `STAT-001`
- Initial status: `implemented`
- Source commit: `4945871d937f508d536cac11dcecef0146564bd4`
- Reference commit: `c1915b0a63bfabebdb95a515cedfee05146c1d50`
- Active task: `task-parity-stat001-hashrate-monitor`

## Selection

The clean synchronized selector returned `SELF-001`, `BAP-002`, then
`STAT-001`. `SELF-001` is not actionable because its checklist record requires
a production-safe self-test route and hardware regression that do not exist.
`BAP-002` is dependency-blocked by `BAP-001`: firmware UART ownership,
request/subscription lifecycle, and live accessory interoperability remain
unfinished, while direct external UART or electrical access is unauthorized.

`STAT-001` is the first actionable row. Attempt-013 closed after 12 of 20
windows with `watchdog_feed_stale`, owner phase `waiting_inbox`, and wait state
`within_deadline`. The producer feeds immediately before arming that bounded
wait, so those facts cannot describe one coherent owner instant. Source tracing
shows runtime health copies mutex-protected feed history separately from the
atomic phase/deadline tuple, allowing an old-feed/new-wait mixed snapshot.

## Scope and non-scope

Implement one firmware-owned, bounded coherent observation snapshot spanning
task-watchdog history, owner phase, and wait deadline. Use a single-writer
sequence protocol around every producer publication and bounded reader retries
that fail closed to unavailable/non-waiting facts. Preserve the current closed
domain types, wrap-aware low-32-bit deadline representation, task priority 5,
compiled five-second watchdog timeout, one-second owner wait/cadence, v14/v8
value-free diagnostics, evaluator source identity, and all existing failure
precedence.

This is software-only. Authorized work is local source, tests, deterministic
fixtures, firmware/package builds, documentation, and ordinary Git operations.
Do not access credentials, detector/device/USB/network runtime, protected
attempt artifacts, private values, or a public projection. Do not flash, reset,
monitor, mine, actuate, update, erase, inject faults, manipulate power, use
direct UART, touch pins/pads/headers/GPIO/probes/jumpers/solder/signals, retry
attempt-013, or create/run attempt-014. This plan cannot verify STAT-001 or
change its checklist fields.

## Implementation

- [ ] Replace the independently read global history/phase/deadline stores with
      one typed task-watchdog observation store and coherent snapshot result.
- [ ] Sequence every history, phase, and wait publication; accept a reader
      snapshot only when one even sequence value brackets every copied fact.
- [ ] Bound retries and return the closed default observation when a coherent
      snapshot cannot be obtained, including poisoned-history handling.
- [ ] Update runtime-health collection to consume only the combined snapshot
      before sampling evaluation time.
- [ ] Add a deterministic store-level regression that forces the exact old-
      feed/new-wait interleaving and proves the reader retries to the new feed
      plus new wait, along with stable and retry-exhaustion coverage.
- [ ] Strengthen production source-ownership checks and the Bazel test graph so
      the coherent protocol, one static owner, publication ordering, priority,
      and runtime-health consumption cannot drift.

## Verification and promotion

Focused verification:

1. `cargo test -p bitaxe-core runtime_health --all-features`
2. `bazel test //firmware/bitaxe:task_watchdog_observation_tests //firmware/bitaxe:hashrate_source_ownership_tests //firmware/bitaxe:runtime_health_tests`
3. `bazel build //firmware/bitaxe:firmware`
4. `just package`
5. `just verify-redaction`
6. `just verify-reference`

Mandatory ordered gate:

1. `cargo fmt --all`
2. `cargo clippy --all-targets --all-features -- -D warnings`
3. `cargo build --all-targets --all-features`
4. `cargo test --all-features`
5. `bun scripts/bright-builds-check.ts all`
6. `just test`
7. `just parity`
8. `just parity-progress`

Acceptance requires the exact mixed interleaving to be rejected and retried,
stable observations to preserve their existing values, bounded contention or
poison to fail closed, all focused/mandatory gates to pass, the previous plan
hash to remain unchanged, the pinned reference to remain clean, and no
checklist/progress/README or public-evidence change. Record the implementation
commit and a plan-bound `CLOSURE.md`; leave STAT-001 `implemented`. Hardware
eligibility, a fresh ordinal, and promotion require a separate future immutable
plan after this source correction is pushed.
