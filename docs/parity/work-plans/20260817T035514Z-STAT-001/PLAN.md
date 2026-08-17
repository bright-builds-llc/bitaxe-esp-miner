# Parity work plan

- Run ID: `20260817T035514Z-STAT-001`
- Parity row: `STAT-001`
- Initial status: `implemented`
- Source commit: `cc409e3efb2913c794813d63f154fa3e1a301ad4`
- Reference commit: `c1915b0a63bfabebdb95a515cedfee05146c1d50`
- Active task: `task-parity-stat001-hashrate-monitor`

## Selection

The worktree and pinned reference are clean, `main` equals `origin/main`, and
the deterministic selector reports no open plan. It orders `SELF-001`,
`BAP-002`, then `STAT-001`. `SELF-001` remains dependency-blocked because no
production-safe firmware route exists for its hardware self-test submodes or
required hardware-regression evidence. `BAP-002` remains dependency- and
authority-blocked by unfinished `BAP-001` UART ownership/subscription work and
the absence of an authorized live accessory path.

STAT-001 is the first actionable row. Exact pushed attempt-011 reached a new
sealed signature after 5 of 20 windows:
`watchdog_invalid_observation` with owner phase `waiting_inbox`, while runtime
identity, attestation parsing, same-boot/package correlation, active state,
safety, terminal transports, pool persistence, safe stop, USB cleanup, modes,
seals, and redaction passed. This differs from attempts 008-010, which reached
14 windows with `watchdog_feed_stale` before the cadence/phase correction.

Source tracing identifies the matching race. The operator-snapshot caller
reads `runtime_uptime::millis()` before `runtime_health_adapter::collect`
copies supervisor and watchdog histories. The owner can record a newer feed
between those operations. The evaluator then correctly rejects the copied
feed as future relative to the stale caller time because checked subtraction
fails. The owner phase `waiting_inbox` rules out the prior publication stall
and makes snapshot ordering the targeted software boundary.

The active lessons exceed deterministic loading limits. Every heading was
inventoried. All global lessons and complete repository blocks for service
ownership, authorization, direct-UART prohibition, protected evidence,
earliest failure, runtime capacity, private classification, retry policy,
transport capability, evaluator identity, preflight exit handling,
operating-state telemetry, and legacy wire units were loaded within the
whole-block budget. Omitted lower-priority blocks are the GSD frontmatter
lesson and historical USB power/session, prearmed native capture, boot-replay
lifetime, silent-transport heartbeat, manual-removal ownership,
physical-identity, and cold-boot-observer lessons. Their active repo-local
equivalents remain controlling. The latest audit baseline has six later
lessons, is under 90 days old, and this plan appends no lesson, so no distinct
audit trigger is due.

## Scope and non-scope

Advance only `STAT-001` with software. Change the read-only firmware runtime-
health adapter so it atomically copies existing supervisor/watchdog facts and
the closed owner phase before sampling current monotonic time for evaluation.
Remove the stale caller-supplied time parameter and update the operator and
screen callers. Preserve every core sequence/time regression guard, timeout,
reason, phase spelling, HTTP/WebSocket/retained projection, campaign schema,
failure precedence, safety decision, hashrate calculation, and hardware
behavior.

Add a controlled regression that models the exact interleaving: an earlier
caller timestamp plus a later copied feed remains `invalid_observation`, while
sampling evaluation time after that same copied feed yields `feed_fresh` with
age zero. Add source-ownership assertions that production histories and owner
phase are read before firmware uptime and that callers cannot supply an older
time. Bind both `crates/bitaxe-core/src/runtime_health.rs` and
`firmware/bitaxe/src/runtime_health_adapter.rs` into the hashrate evidence
source inventory and Bazel runfiles; update the independent contract's source
path count and real-child fixtures.

This plan authorizes local source, fixtures, tests, builds, documentation, and
ordinary git operations only. Do not access protected attempts, credentials,
detector/device/network runtime, or public projection candidates. Do not
detect, flash, reset, monitor, mine, actuate, update, erase, inject faults,
manipulate power, use direct UART, or touch pins, pads, headers, GPIO, probes,
jumpers, solder, or signals. Attempt-011 remains consumed and attempt-012 is
not authorized.

## Implementation

- [ ] Move monotonic evaluation-time sampling after copied checkpoint,
      watchdog, and owner-phase observations; remove stale caller time input
      and update every production caller.
- [ ] Add behavior and ownership regressions for the controlled concurrent-feed
      interleaving, future-feed rejection, fresh zero-age evaluation, exact
      call ordering, and absence of caller-supplied evaluation time.
- [ ] Bind the runtime-health core and firmware adapter into the hashrate
      evidence source/runfiles graph; update source counts and real-child
      source drift tests without changing public evidence shape.
- [ ] Run focused and mandatory software, firmware-package, privacy,
      reference, immutable-plan, parity-invariance, and diff gates; commit and
      push a software-only closure without checklist transition or hardware.

## Verification and promotion

Run focused `bitaxe-core` runtime-health tests, firmware hashrate/source-
ownership tests, API runtime-health tests, flash watchdog tests, automation
real-child/source-admission tests, generated-contract verification, and
parity source guards. Run `just verify-redaction`, `just verify-reference`,
`just package`, and the mandatory sequence in order:

1. `cargo fmt --all`
2. `cargo clippy --all-targets --all-features -- -D warnings`
3. `cargo build --all-targets --all-features`
4. `cargo test --all-features`
5. `bun scripts/bright-builds-check.ts all`
6. `just test`
7. `just parity`
8. `just parity-progress`

Acceptance requires the controlled old ordering to produce
`invalid_observation`, the corrected post-copy timestamp to produce
`feed_fresh` at age zero, all other invalid/fresh/stale boundaries to remain
unchanged, exact production source ordering and call ownership, updated source
inventory identity, clean firmware packaging, redaction, and unchanged
checklist/progress. Commit and push the implementation as `SOURCE_COMMIT`.

Create `CLOSURE.md`, not `RESULT.md`: final status remains `implemented`,
outcome is `blocked`, and verification claimed is `no`. Do not change STAT-001
checklist fields, progress history, or README. This software correction cannot
prove the live twenty-window quorum. A future invocation may consider a new
attempt-012 contract only after this exact pushed correction passes every gate;
this plan itself authorizes no device use.

## Non-claims

This plan does not verify STAT-001, independently prove the attempt-011 race on
hardware, authorize attempt-012, change the watchdog timeout, weaken clock or
sequence validation, prove live BM1366 hashrate accuracy, complete twenty
windows or 600 active seconds, or claim electrical accuracy, profitability,
arbitrary profiles or pools, other boards or ASICs, update/recovery behavior,
or release readiness.
