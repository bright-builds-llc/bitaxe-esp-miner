# Parity work plan

- Run ID: `20260816T225404Z-STAT-001`
- Parity row: `STAT-001`
- Initial status: `implemented`
- Source commit: `2c1c88e05bb02b2ef623573921d8eeaad8eb00fb`
- Reference commit: `c1915b0a63bfabebdb95a515cedfee05146c1d50`
- Active task: `task-parity-stat001-hashrate-monitor`

## Selection

The worktree and pinned reference are clean, `main` equals `origin/main`, and
the deterministic selector reports no open plan. It orders `SELF-001`,
`BAP-002`, then `STAT-001`. `SELF-001` remains dependency-blocked because no
production-safe firmware route exists for its hardware submodes and required
hardware-regression evidence. `BAP-002` remains dependency- and
authority-blocked by the unfinished `BAP-001` UART/task/subscription lifecycle
and the absence of an authorized compatible accessory path.

`STAT-001` is the first actionable row. Attempts 007 and 008 both crossed
exact-package admission, trusted runtime identity, active hashrate/network
windows, safe stop, cleanup, seals, protected modes, and redaction before the
production task-watchdog feed became stale. Attempt 008 included the compiled
five-second ESP-IDF timeout correction, so the repeated boundary disproves the
former two-second policy hypothesis and requires a source-level lifecycle
diagnosis before another hardware ordinal.

Source inspection identifies a scheduling mismatch with the pinned reference.
Upstream runs `hashrate_monitor_task` as an independent periodic FreeRTOS task
that yields with `vTaskDelay` and `vTaskDelayUntil`. The Rust firmware instead
subscribes the broader production-owner task to the ESP task watchdog and
records its only recurring feed after a complete receive, session dispatch,
feedback-effect cascade, campaign publication, and hashrate-service pass.
Consequently, completed cooperative progress inside a long dispatch pass is
not visible to the watchdog, while one genuinely blocked effect must still be
allowed to exceed the configured timeout and fail closed.

The two active lesson ledgers total 30,773 bytes with a summed conservative
estimate of 10,259 tokens, above both deterministic loading limits. All lesson
headings were inventoried. Complete safety, privacy, authorization, evidence,
transport, retry, ESP-IDF ownership, operating-state unit, and diagnostic
guardrail blocks relevant to this work were loaded. The unrelated repository
blocks for GSD frontmatter, native USB capture/replay, boot replay,
heartbeat/silent transport, manual removal, physical USB identity, cold-boot
observation, and ESP-IDF main-task runtime capacity were not loaded. The
2026-08-03 audit baseline is under 90 days old and only six active lessons have
been added across both ledgers, so no distinct audit trigger is due.

## Scope and non-scope

Advance only `STAT-001`. Make task-watchdog feeding represent bounded,
cooperative production-owner progress: feed before the bounded receive wait,
between completed session events/effects in a feedback cascade, and at the
remaining outer-loop progress boundaries. Keep the ESP-IDF subscription and
feed owned by the monitored production task. Preserve the compiled watchdog
timeout and the existing closed observation vocabulary.

Add a production-shaped, host-testable regression that proves a multi-event
feedback cascade records recurring progress checkpoints while a single
unfinished effect does not receive a synthetic feed. Strengthen source
ownership guards so a future refactor cannot move watchdog ownership to a
helper task or reduce the health evaluator to a timer-only heartbeat.

This plan is software-only. It authorizes local source, fixtures, tests,
builds, documentation, and ordinary git operations. Do not access protected
attempt artifacts, ignored credentials, detector/device/network runtime,
private endpoints, or a public projection. Do not detect, flash, reset,
monitor, mine, actuate, update, erase, inject faults, manipulate physical
power, use direct UART, or touch any pin, pad, header, GPIO, probe, jumper,
solder, or signal interface. Attempt 008 remains consumed, and this plan does
not authorize attempt 009.

Do not change hashrate arithmetic, network policy, ASIC commands, safety
limits, voltage units, fan behavior, campaign duration, package identity,
evidence schemas, the task-watchdog timeout, or checklist fields. Input safety
continues to use volts; the ASIC core-voltage command continues to use
millivolts. Hardware responsiveness, twenty-window continuity, hashrate
accuracy, shares, electrical accuracy, extended mining, other pools, boards,
or ASICs, updates/recovery, and release readiness remain non-claims.

## Implementation

- [ ] Introduce the minimum testable cooperative-progress boundary needed to
      feed the existing production-task watchdog from the owner task itself.
- [ ] Feed at bounded owner progress checkpoints, including between completed
      feedback events/effects, without feeding during an unfinished blocking
      effect or from any helper task.
- [ ] Add behavior-focused cascade, blocked-effect, ordering, ownership, and
      configured-timeout regressions against the production-shaped path.
- [ ] Run focused and complete verification, record a truthful software-only
      closure, and leave hardware promotion to a separate immutable plan.

## Verification and promotion

Focused verification must cover the pure cooperative-progress policy, the
production owner-loop and watchdog ownership tests, the runtime-health timeout
boundary, and the Phase 34 source guard. Build the canonical firmware package
to prove the ESP-IDF calls and owner task still compile together. Run
`just verify-redaction`, `just verify-reference`, and `just package`, then run
the mandatory sequence in order:

1. `cargo fmt --all`
2. `cargo clippy --all-targets --all-features -- -D warnings`
3. `cargo build --all-targets --all-features`
4. `cargo test --all-features`
5. `bun scripts/bright-builds-check.ts all`
6. `just test`
7. `just parity`
8. `just parity-progress`

Acceptance requires tests to prove that each completed unit of cooperative
owner progress can feed before the next unit starts; no feed is issued for an
unfinished effect; the monitored production task remains the sole ESP-IDF
watchdog subscriber/resetter/unsubscriber; the compiled timeout remains the
health threshold; all focused and mandatory gates pass; the pinned reference
stays clean; and the final diff contains no unrelated changes.

This software correction cannot verify `STAT-001`. Create `CLOSURE.md` bound
to the immutable plan, append `WORKLOG.md`, keep the row `implemented`, make no
checklist transition or progress synchronization, and prohibit hardware until
a separate immutable plan binds the pushed correction to a new exact package
and defines a fresh detector/evidence/privacy/safety/recovery/retry contract.
