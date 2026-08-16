# Parity work plan

- Run ID: `20260816T192025Z-STAT-001`
- Parity row: `STAT-001`
- Initial status: `implemented`
- Source commit: `e52832220f58638fe028fd14eeab98e3b9e73cf0`
- Reference commit: `c1915b0a63bfabebdb95a515cedfee05146c1d50`
- Active task: `task-parity-stat001-hashrate-monitor`

## Selection

The clean synchronized selector reported no open plan and returned SELF-001,
BAP-002, then STAT-001. SELF-001 is not actionable because its checklist row
still requires hardware-regression evidence from a production-safe self-test
route that does not exist. BAP-002 is not actionable because BAP-001 still
owns the unfinished firmware UART/request/subscription lifecycle and no
authorized compatible accessory path exists. STAT-001 is first actionable:
attempt-007 supplied the new closed discriminator `watchdog_feed_stale`, and
its active task requires software-only diagnosis before another ordinal.

Public source inspection, without reading protected attempt artifacts, found
the exact mismatch. `RuntimeHealthSnapshot` classifies a production task feed
as stale after 2,000 ms, while the exact firmware build compiles
`CONFIG_ESP_TASK_WDT_TIMEOUT_S=5` and subscribes/feeds the owner through
ESP-IDF. The pinned reference hashrate task uses an absolute one-second poll
schedule and does not add a separate two-second task-watchdog failure boundary.
The repo-local guidance, Bright Builds architecture/code-shape/testing/
verification rules, and Rust standard require a pure evaluator with the
compiled ESP-IDF policy supplied by the firmware boundary and focused tests.

## Scope and non-scope

Replace the unrelated hard-coded two-second task-watchdog freshness policy
with an explicit timeout supplied to the pure health evaluator. The firmware
adapter must derive that timeout from its compiled ESP-IDF sdkconfig binding,
so projected participation and the watchdog actually supervising the task
share one authoritative boundary. Preserve every existing closed observation,
sequence, overflow, invalid-order, serialization, and failure category.

This is local software, fixture, test, build, documentation, and ordinary git
work only. Do not access ignored credentials, detector output, attempt-007 or
any other protected evidence, device/USB/network runtime, private values, or a
public projection. Do not detect, flash, reset, monitor, mine, actuate, update,
erase, inject faults, manipulate power, use direct UART, or touch any
electrical pin/pad/header/GPIO/probe/jumper/solder/signal interface. This plan
does not authorize attempt-008 and cannot verify STAT-001.

## Implementation

- [ ] Parameterize task-watchdog freshness in the pure runtime-health
      evaluator and have the firmware adapter supply the compiled ESP-IDF
      timeout in milliseconds with checked conversion.
- [ ] Add focused boundary regressions proving the old 2,001-ms false stale
      result is now fresh under the five-second production policy, the exact
      configured boundary is accepted, and the first millisecond beyond it is
      stale while all closed failures remain unchanged.
- [ ] Prove the firmware boundary consumes the compiled sdkconfig value, build
      the exact firmware/package, and preserve checklist and progress history.

## Verification and promotion

Run focused runtime-health, API wire, firmware ownership, and production
firmware/package checks. Run `just verify-redaction`, `just verify-reference`,
`git diff --check`, immutable-plan hashing, and the mandatory sequence in
order:

1. `cargo fmt --all`
2. `cargo clippy --all-targets --all-features -- -D warnings`
3. `cargo build --all-targets --all-features`
4. `cargo test --all-features`
5. `bun scripts/bright-builds-check.ts all`
6. `just test`
7. `just parity`
8. `just parity-progress`

Commit and push the implementation only after all gates pass. Because this
software correction supplies no accepted live hardware evidence, do not
transition the checklist, append progress history, rewrite README status,
publish evidence, or archive the active task. Create `CLOSURE.md` with
`Verification claimed: no`, record the correction and remaining live boundary,
and leave STAT-001 `implemented`. A future invocation may consider a fresh
attempt-008 only after this targeted fix is pushed, independently gated, and
bound by a new immutable hardware plan.
