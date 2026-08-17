# Parity work plan

- Run ID: `20260817T045834Z-STAT-001`
- Parity row: `STAT-001`
- Initial status: `implemented`
- Source commit: `3670f446c3f64b0a6d72f616cebe14ff3c30ff2e`
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

STAT-001 is the first actionable row. Attempt-012 reached 13 of 20 windows
before sealed `watchdog_feed_stale` at owner phase `waiting_inbox`. Identity,
attestation, same-package correlation, active state, safety, terminal
transports, pool persistence, safe stop, cleanup, modes, and seals passed. The
owner feeds immediately before entering `recv_timeout`, whose requested wait
is derived from the one-second readiness deadline, yet the observed feed age
later exceeded the compiled five-second watchdog timeout.

Current evidence cannot distinguish an overlong/failed timed wait from
scheduler delay while the task is blocked. The phase store carries no armed
absolute deadline. The production thread uses ESP-IDF pthread defaults;
upstream's protocol coordinator and Stratum task use priority 5, so changing
priority without evidence is not justified. The next informational boundary
is an atomic wait-deadline discriminator plus an explicit priority-5 contract,
not an unchanged retry or speculative scheduling change.

The active lessons exceed deterministic loading limits. Every heading was
inventoried. All global lessons and complete repository blocks for service
ownership, authorization, direct-UART prohibition, protected evidence,
earliest failure, runtime capacity, private classification, retry policy,
transport capability, evaluator identity, operating-state telemetry, and
legacy wire units were loaded within the whole-block budget. Omitted lower-
priority blocks are the GSD frontmatter lesson and historical USB power/
session, prearmed native capture, boot-replay lifetime, silent-transport
heartbeat, manual-removal ownership, physical-identity, and cold-boot-observer
lessons. Their active repo-local equivalents remain controlling. The latest
audit baseline has six later lessons, is under 90 days old, and this plan
appends no lesson, so no distinct audit trigger is due.

## Scope and non-scope

Advance only `STAT-001` with software. Add a closed
`TaskWatchdogWaitState` vocabulary: `not_waiting`, `within_deadline`,
`deadline_overrun`, and `invalid_observation`. Before the owner publishes
`waiting_inbox`, atomically arm the absolute firmware-uptime deadline derived
from the actual requested receive duration. Publish the waiting phase only
after the deadline write, and clear applicability by publishing the next
non-waiting phase. Runtime health must copy phase/deadline before its post-copy
clock sample and derive the wait state without free text or private values.

Project the state additively through HTTP/WebSocket runtime health, retained
runtime-health text, campaign network/result evidence, and sealed private-first
watchdog diagnostics. Unknown/missing inputs fail closed. Bump
`mining-campaign-network-continuity-v7` to v8 and
`mining-campaign-result-v13` to v14 throughout writers, readers, fixtures,
seals, and tests. Public hardware-blocked output may carry only the closed
wait-state label, never timestamps, deadlines, durations, identities, or raw
values.

Add pure regressions for exact deadline inclusion, one-millisecond overrun,
non-waiting, missing deadline, future evaluation time, and overflow. Add
production source-ownership tests proving deadline-before-phase publication,
phase/deadline-before-clock collection, and one-second requested receive
budget. Pin `CONFIG_PTHREAD_TASK_PRIO_DEFAULT=5` in `sdkconfig.defaults`, bind
it to upstream priority-5 breadcrumbs and package/source checks, and do not
change the priority or watchdog timeout.

Preserve hashrate computation, campaign publication cadence, feedback,
readiness, all existing watchdog labels, owner phases, earliest failure,
safety, control, and hardware behavior. This plan authorizes local source,
fixtures, tests, builds, documentation, and ordinary git operations only. Do
not access protected attempts, credentials, detector/device/network runtime,
or public projection candidates. Do not detect, flash, reset, monitor, mine,
actuate, update, erase, inject faults, manipulate power, use direct UART, or
touch electrical interfaces. Attempt-012 remains consumed and attempt-013 is
not authorized.

## Implementation

- [ ] Add the closed wait-state domain evaluator and atomic firmware deadline
      store; arm deadline before `waiting_inbox` and derive state after copying
      producer observations but before serialization.
- [ ] Carry wait state through runtime-health wire/retained data, v14/v8
      campaign evidence, and seal-gated value-free watchdog diagnostics while
      preserving all prior labels and failure precedence.
- [ ] Add exact boundary, invalid/overflow, production ordering, one-second
      wait-budget, schema/seal, redaction, missing/unknown, and real-child
      regressions.
- [ ] Pin/test ESP-IDF pthread priority 5 against upstream coordinator/task
      breadcrumbs and bind every new producer/evaluator source in Bazel and the
      hashrate source inventory/contracts.
- [ ] Run focused and mandatory software, firmware-package, privacy,
      reference, immutable-plan, parity-invariance, and diff gates; commit and
      push a software-only closure without checklist transition or hardware.

## Verification and promotion

Run focused core runtime-health, firmware owner/source-ownership, API wire/
retained, flash network/watchdog/schema, automation real-child/seal, generated-
contract, sdkconfig/package, and parity source-guard tests. Run `just
verify-redaction`, `just verify-reference`, `just package`, and the mandatory
sequence in order:

1. `cargo fmt --all`
2. `cargo clippy --all-targets --all-features -- -D warnings`
3. `cargo build --all-targets --all-features`
4. `cargo test --all-features`
5. `bun scripts/bright-builds-check.ts all`
6. `just test`
7. `just parity`
8. `just parity-progress`

Acceptance requires exact closed wait-state boundaries, atomic deadline-before-
phase ownership, phase/deadline-before-clock evaluation, preserved watchdog
behavior and precedence, v14/v8 seal gating, value-free public output, pinned
priority 5 in package/source contracts, clean firmware packaging, redaction,
and unchanged checklist/progress. Commit and push implementation as
`SOURCE_COMMIT`.

Create `CLOSURE.md`, not `RESULT.md`: final status remains `implemented`,
outcome is `blocked`, and verification claimed is `no`. Do not change STAT-001
checklist fields, progress history, or README. Diagnostics cannot prove live
twenty-window parity. A future invocation may consider a targeted fix or
attempt-013 only after this exact pushed discriminator passes every gate; this
plan authorizes no hardware.

## Non-claims

This plan does not verify STAT-001, identify scheduler starvation versus a
timed-wait defect on hardware, authorize attempt-013, change thread priority or
watchdog timeout, prove live BM1366 hashrate accuracy, complete twenty windows
or 600 active seconds, or claim electrical accuracy, profitability, arbitrary
profiles or pools, other boards or ASICs, update/recovery behavior, or release
readiness.
