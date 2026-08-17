# Parity work plan

- Run ID: `20260817T020911Z-STAT-001`
- Parity row: `STAT-001`
- Initial status: `implemented`
- Source commit: `da06b3635c3bcaddd7cfd7a5d663497a80553592`
- Reference commit: `c1915b0a63bfabebdb95a515cedfee05146c1d50`
- Active task: `task-parity-stat001-hashrate-monitor`

## Selection

The worktree and pinned reference are clean, `main` equals `origin/main`, and
the deterministic selector reports no open plan. It orders `SELF-001`,
`BAP-002`, then `STAT-001`. `SELF-001` remains blocked because no production-
safe firmware route exists for its hardware self-test modes. `BAP-002` remains
blocked by unfinished `BAP-001` UART/subscription ownership and the absence of
an authorized live accessory path.

STAT-001 is the first actionable row for software-only work. Attempts 008,
009, and 010 all stopped after 14 of 20 windows with producer-owned
`watchdog_feed_stale`; attempt-010 proved the prior host 2,000-ms threshold was
not the remaining cause and selected `stop_repeated_boundary`. Its closure
explicitly permits a software-only plan that adds a new closed discriminator,
production-shaped regression, and targeted verified correction while
prohibiting attempt-011.

Source tracing now identifies one deterministic owner boundary: the production
owner feeds at loop start, after completed event/effect progress, and after
campaign-status publication. It publishes a large synchronous retained/serial
`mining_campaign_status` record on every loop, including high-rate transport
and ASIC inbox events, even though continuity evidence requires only bounded
periodic markers. That unbounded per-event logging runs immediately before the
final feed and can create serial-output backpressure while HTTP/WebSocket tasks
continue to report the last successful feed. A one-second publication cadence
is already the firmware readiness and hashrate cadence and remains comfortably
inside the host's five-second marker-gap requirement.

The active lesson ledgers exceed deterministic loading limits. Every heading
was inventoried. All global lessons and complete repository blocks for safety,
privacy, authorization, retry policy, real-process boundaries, earliest-
failure precedence, transport capability, evaluator identity, runtime
capacity, and telemetry units were loaded within the whole-block budget.
Omitted lower-priority blocks are the GSD frontmatter lesson and historical
USB power/session, prearmed native capture, boot-replay lifetime, silent-
transport heartbeat, manual-removal ownership, physical-identity, and cold-
boot-observer lessons; equivalent active repo-local rules remain controlling.
The latest audit baseline has six later lessons, is under 90 days old, and
this work appends no lesson, so no distinct audit trigger is due.

## Scope and non-scope

Advance only STAT-001 without hardware. Add a closed
`TaskWatchdogOwnerPhase` vocabulary and lock-free firmware observation that
records the owner immediately before inbox wait, message/observation/readiness
drive, campaign-status publication, hashrate service, and shutdown. Project the
phase through pure runtime health, additive HTTP/WebSocket wire data, retained
runtime-health text, campaign network/result evidence, and the sealed
private-first public failure diagnostic. Unknown or missing phase values must
fail closed or render `unavailable`; no phase may contain values, identifiers,
durations, addresses, or free text.

Add a pure one-second campaign-status publication schedule. State tracking,
actuation qualification, feedback handling, safety sampling, terminal
convergence, and hashrate service remain per-event; only redundant status
serialization and synchronous retained/serial emission are coalesced. The
first status and terminal/consumed status remain prompt, periodic active
markers have gaps no greater than 1,000 ms, and time regression/overflow fails
closed. A production-shaped 600-second high-event-rate regression must prove
publication count is cadence-bounded rather than event-count-bound.

Because the evidence shape gains a materially new discriminator, bump
`mining-campaign-network-continuity-v6` to v7 and
`mining-campaign-result-v12` to v13 throughout writers, readers, fixtures,
seals, generated contracts, and tests. Preserve every prior closed watchdog
label, earliest-failure precedence, schema/seal gating, value-free public
error, source/reference admission, and projection withholding.

This plan authorizes local source, fixtures, tests, builds, documentation, and
ordinary git operations only. Do not access protected prior-attempt artifacts,
credentials, detector/device/network runtime, or public projection candidates.
Do not detect, flash, reset, monitor, mine, actuate, update, erase, inject
faults, manipulate power, use direct UART, or touch pins, pads, headers, GPIO,
test points, probes, jumpers, solder, or signals. Attempt-010 remains consumed;
attempt-011 is not authorized.

## Implementation

- [ ] Add and test the closed owner-phase domain type, lock-free firmware
      observation, phase instrumentation, runtime-health/wire/retained
      projection, and unavailable/unknown fail-closed behavior.
- [ ] Add and test the one-second campaign-status publication schedule with
      first/periodic/terminal behavior, clock guards, and a 600-second high-
      event-rate regression that bounds emissions independently of event count.
- [ ] Bump campaign network/result schemas to v7/v13 and carry the phase only
      through sealed watchdog failures, preserving all prior labels, seals,
      precedence, redaction, value-free public output, and projection
      withholding across real-child tests.
- [ ] Bind all newly reachable producer/reducer/evaluator sources into Bazel
      and the hashrate evidence source inventory; update typed/generated
      contracts and ownership tests without changing mining, watchdog timeout,
      hardware control, or hashrate calculations.
- [ ] Run every focused and mandatory software, firmware-package, privacy,
      reference, immutable-plan, parity-invariance, and diff gate; commit and
      push a non-verifying closure with no checklist/progress transition.

## Verification and closure

Run focused `bitaxe-core` runtime-health, firmware owner-progress/campaign-
status/source-ownership, `bitaxe-api` wire/retained, flash campaign network/
watchdog/schema, automation real-child, contract/generated-contract, and Bazel
tests. Run `just verify-redaction`, `just verify-reference`, `just package`, and
the mandatory sequence in order:

1. `cargo fmt --all`
2. `cargo clippy --all-targets --all-features -- -D warnings`
3. `cargo build --all-targets --all-features`
4. `cargo test --all-features`
5. `bun scripts/bright-builds-check.ts all`
6. `just test`
7. `just parity`
8. `just parity-progress`

Acceptance requires all prior watchdog behaviors plus exact closed owner-phase
coverage, one-second maximum periodic publication gap, cadence-bounded 600-
second event-storm behavior, prompt terminal publication, v13/v7 seal-gated
diagnostics, no secret/value-bearing output, unchanged package behavior, and
an immutable plan digest. Commit and push the implementation as
`SOURCE_COMMIT`.

Create `CLOSURE.md`, not `RESULT.md`: final status remains `implemented`,
outcome is `blocked`, and verification claimed is `no`. Do not change STAT-001
checklist fields, progress history, or README. This software work cannot prove
live twenty-window hashrate parity. A future invocation may consider a new
complete attempt-011 hardware contract only after this exact pushed correction
and discriminator pass every gate; this plan itself authorizes no device use.

## Non-claims

This plan does not verify STAT-001, prove the inferred serial-backpressure
cause on hardware, authorize a retry, change the five-second ESP task-watchdog
timeout, guarantee arbitrary log transports, prove live BM1366 hashrate
accuracy, complete twenty windows or 600 seconds, prove terminal zero, or
claim mining, pool, electrical, profitability, other-board, update/recovery,
or release behavior.
