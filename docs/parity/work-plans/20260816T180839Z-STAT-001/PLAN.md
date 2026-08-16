# Parity work plan

- Parity row: `STAT-001`
- Initial status: `implemented`
- Reference breadcrumb: `reference/esp-miner/main/tasks/hashrate_monitor_task.c`
- Rust-owned target: `tools/flash/src/campaign/network/watchdog.rs`,
  `tools/flash/src/campaign/network/model.rs`,
  `tools/automation/src/hashrate-monitor-evidence.ts`
- Source commit: `3632659c7a32033153b322e947cfaf64a820b35f`
- Reference commit: `c1915b0a63bfabebdb95a515cedfee05146c1d50`
- Active task: `task-parity-stat001-hashrate-monitor`

## Selection inputs

The clean synchronized selector reported no open plan and ordered SELF-001,
BAP-002, then STAT-001. SELF-001 is blocked because the production-safe route
and its fan, voltage, power, thermal, and ASIC diagnostic hardware submodes are
not implemented or hardware-regression verified. BAP-002 is blocked on
not-started BAP-001 firmware UART/task ownership and live accessory/electrical
interoperability, which standing USB authorization does not cover. STAT-001 is
the first actionable row because its consumed attempt-006 produced verified
new information: campaign-result v11 failed as `watchdog_unresponsive` with
the closed discriminator `watchdog_not_participating` while identity, terminal
transport state, safe stop, cleanup, modes, and seal passed.

Current-source inspection proves why that discriminator is still too coarse.
`RuntimeHealthSnapshot` maps `unproved`, `invalid_observation`,
`subscription_failed`, `feed_failed`, `unsubscription_failed`, `unsubscribed`,
and `feed_stale` into a non-participating projection with a separate closed
reason. The campaign classifier tests participation before that reason, so all
production-shaped non-participating causes collapse into
`watchdog_not_participating`; its later stale/reason branches are unreachable.
The existing test instead constructs the inconsistent pair
`not_participating` plus `feed_fresh` and therefore misses the real boundary.

## Scope

Correct only the source-owned closed watchdog classifier and the schemas and
tests that carry it. Map every evaluator-owned closed reason to a distinct
value-free failure enum before applying the generic participation-consistency
guard. Preserve supervisor/checkpoint precedence, feed sequence/age checks,
per-window advancement checks, earliest-failure precedence, privacy, and all
campaign behavior. Bump the sealed network and campaign-result schemas because
the failure vocabulary changes, and update the hashrate wrapper to accept only
the new schemas and complete closed vocabulary.

This is a software-only diagnostic correction. It does not change firmware
watchdog subscription, feed cadence, mining behavior, hashrate computation,
hardware control, or any checklist field. It must not access ignored attempt-
006 artifacts, credential files, detector/device/network runtime, or private
values. It must not flash, reset, mine, actuate, update, erase, inject faults,
use direct UART or electrical interfaces, create a public projection, or start
attempt-007.

## Implementation

1. Replace primitive check ordering with a reason-aware closed classifier that
   distinguishes every current `TaskWatchdogObservation` projection, missing
   and unknown reason values, and an inconsistent participation/reason pair.
2. Advance network continuity v5 to v6 and campaign-result v11 to v12; update
   every producer, consumer, fixture, seal/schema gate, and closed TypeScript
   diagnostic union without widening public values.
3. Add production-shaped table regressions built from the real runtime-health
   evaluator vocabulary. Prove every label is lowercase/value-free, stale and
   lifecycle causes remain distinct, inconsistent/unknown inputs fail closed,
   successful `feed_fresh` still reaches sequence/age checks, and the earliest
   category/discriminator remains immutable through terminal cleanup.
4. Run focused Rust and automation tests, the canonical firmware/package
   build, all mandatory repository gates, redaction/reference checks, and a
   simplification/diff review. Record a non-promotion `CLOSURE.md`; do not
   transition or synchronize progress because checklist fields remain exact.

## Verification and evidence

- Focused: `cargo test -p bitaxe-flash watchdog` and the complete
  `//tools/flash:tests` target.
- Cross-boundary: the hashrate-filtered `//tools/automation:automation_test`
  real-child/schema/seal suite and the current-source admission test.
- Firmware/package: `just package` plus manifest source/reference validation.
- Mandatory: ordered Cargo format, strict Clippy, all-target build, all-feature
  tests, Bright Builds, `just test`, `just parity`, and
  `just parity-progress`.
- Privacy and provenance: `just verify-redaction`, `just verify-reference`,
  immutable-plan diff, sensitive-value review, and clean final diff.
- Evidence is current source, deterministic production-shaped tests, schema
  gates, and build output only. Synthetic tests do not verify live watchdog or
  STAT-001 hardware parity.

## Safety, stop, and promotion criteria

Stop on unrelated worktree changes, source/reference drift, an unclosed or
value-bearing failure label, a stale schema consumer, changed campaign/mining
semantics, any failed gate, or any need for protected/private/hardware input.
Do not inspect or reuse attempt-006 and do not allocate attempt-007.

Successful completion proves only that future failures preserve the exact
closed source-owned watchdog boundary. STAT-001 remains `implemented` with
`unit,workflow`; no `RESULT.md`, public projection, checklist transition,
progress-history append, README rewrite, or task archive is permitted. A
future immutable hardware plan may authorize attempt-007 only after this
correction is committed, pushed, fully gated, and objectively changes the
previous diagnostic boundary.

## Standards and lesson audit

Repo-local task, safety, privacy, hardware, evidence, and GSD-sunset guidance;
`AGENTS.bright-builds.md`; the architecture, code-shape, testing, verification,
and Rust standards; and the diagnostic-completeness, real-boundary,
protected-root, earliest-failure, retry-new-information, evaluator-identity,
standing-authorization, preflight-exit, telemetry-state, and legacy-unit
lessons materially govern this plan. The active lesson set is 30,773 bytes
with a 10,259-token conservative estimate, so headings were inventoried and
only complete priority blocks were loaded. Unloaded repository blocks were
GSD frontmatter, espflash reset effects, power/USB distinction, native capture,
boot replay, silent transport, manual removal, physical USB identity, cold-
boot observation, time-bounded physical checkpoints, and live-checkpoint
invitation. The 2026-08-03 audit baseline has six later active lessons, below
the ten-new trigger; no lesson audit is due.
