# Parity work plan

- Run ID: `20260817T090156Z-STAT-001`
- Parity row: `STAT-001`
- Initial status: `implemented`
- Source commit: `6f29fe1f8671c5c432f88291bce2f25cd1be0a5e`
- Reference commit: `c1915b0a63bfabebdb95a515cedfee05146c1d50`
- Active task: `task-parity-stat001-hashrate-monitor`

## Selection

The worktree/reference are clean, `main` equals `origin/main`, and the selector
has no open plan. It orders `SELF-001`, `BAP-002`, then `STAT-001`. SELF-001
remains blocked by the absent production-safe self-test route and required
hardware regression. BAP-002 remains blocked by unfinished BAP-001 firmware
UART/request/subscription ownership and no authorized live accessory path.

STAT-001 is first actionable. Attempt-015 produced the first trustworthy
coherent watchdog tuple: `watchdog_feed_stale`, read outcome `stable`, owner
phase `handling_inbox`, and wait state `not_waiting` after 364,110 active ms and
12/20 windows. This rules out receive waiting and store-read ambiguity, but the
phase currently spans inbox mapping, session evaluation, and every feedback
effect. Source tracing also shows that the owner feeds only after event or
effect completion. An otherwise bounded operation can therefore inherit most
of the watchdog age from preceding work before its boundary is recorded.

The active lesson set exceeds its loading budget; all headings were inventoried
and the full global file plus complete safety, authorization, evidence, retry,
unit, and watchdog-relevant blocks were loaded. Lower-priority historical
GSD/USB blocks were disclosed as omitted. The current audit baseline is under
90 days old, fewer than ten lessons are new, and no append is proposed, so no
audit trigger is due. Repo-local rules and Bright Builds architecture/testing
standards require typed boundary states, a thin firmware shell, and focused
Arrange/Act/Assert regressions.

## Scope and non-scope

Advance only STAT-001 through local software changes. Add one closed,
value-free `TaskWatchdogOwnerSubphase` that distinguishes unavailable work,
inbox-to-event mapping, session evaluation, and every current
`ProductionSessionEffect` category without retaining effect values. Publish
phase, subphase, and wait state as one sequence-bracketed coherent owner
context, including clearing subphase when the owner enters a top-level phase.

Feed at session-evaluation entry and immediately before each effect executes,
while retaining the existing completion feeds. This resets inherited watchdog
age at work boundaries but still lets a genuinely blocking handler or effect
become stale. Carry the subphase through runtime health, HTTP/WebSocket/retained
wire views, the earliest campaign watchdog tuple, sealed network/result
evidence, and the private-first wrapper. Rotate only private campaign result
v15/network v9 to v16/v10; retain the public
`bitaxe-hashrate-monitor-evidence-v1`, 18-source identity, safety and privacy
contracts, and checklist fields.

This plan authorizes source, tests, deterministic fixtures, documentation,
firmware/package builds, and ordinary Git operations only. Do not access
protected attempt-015 artifacts, credentials, detector/device/USB/network
runtime, private values, or a public projection. Do not flash, reset, monitor,
mine, actuate, update, erase, inject faults, manipulate power, use external
UART/BAP, touch electrical interfaces, retry attempt-015, or create/run
attempt-016. This plan cannot verify STAT-001.

## Implementation

- [ ] Add the exhaustive subphase vocabulary and sequence-bracketed firmware
      storage, including exact spelling, decoding, clearing, coherence, retry,
      and poison regressions.
- [ ] Record inbox mapping, session evaluation, and every value-free effect
      category; add entry feeds before evaluation/effect execution and prove
      ordering around production-shaped feedback cascades.
- [ ] Project subphase through runtime health and all wire/retained surfaces,
      with backward-compatible unavailable defaults and no free-text path.
- [ ] Parse and latch subphase with the earliest campaign watchdog failure;
      rotate private result/network evidence to v16/v10 and update Rust,
      TypeScript, generated-contract, fixture, seal, wrapper, source-inventory,
      redaction, and real-child consumers.
- [ ] Add an attempt-015-shaped regression that reproduces stale age inherited
      at the old post-completion-only boundary and proves the new entry feed,
      exact effect category, and immutable earliest diagnostic tuple.

## Verification and promotion

Focused checks:

1. `cargo test -p bitaxe-core runtime_health --all-features`
2. `bazel test //firmware/bitaxe:task_watchdog_observation_tests //firmware/bitaxe:hashrate_source_ownership_tests //crates/bitaxe-api:tests //tools/flash:tests //tools/automation:automation_test //tools/parity:tests`
3. `bazel build //tools/automation:contracts_verified //firmware/bitaxe:firmware`
4. `just verify-redaction`
5. `just verify-reference`
6. `just package`

Mandatory ordered gate:

1. `cargo fmt --all`
2. `cargo clippy --all-targets --all-features -- -D warnings`
3. `cargo build --all-targets --all-features`
4. `cargo test --all-features`
5. `bun scripts/bright-builds-check.ts all`
6. `just test`
7. `just parity`
8. `just parity-progress`

Acceptance requires exhaustive value-free subphases; one coherent
phase/subphase/wait snapshot; feeds before handler/effect execution without
masking a blocking operation; one earliest watchdog tuple retaining read
outcome, phase, subphase, wait, and failure through terminal observations;
v16/v10 only; generated contracts and 18-source evaluator identity current;
all focused and mandatory gates passing; immutable plan/reference clean; and no
hardware, public projection, checklist, progress, or README mutation. Create a
plan-bound `CLOSURE.md`, leave STAT-001 `implemented`, and require a separate
future plan before hardware.

## Non-claims

This plan does not identify which attempt-015 effect blocked, authorize a
hardware retry, prove scheduler/transport behavior, verify hashrate accuracy,
or claim arbitrary profiles/pools, other boards/ASICs, update/recovery,
profitability, or release readiness.
