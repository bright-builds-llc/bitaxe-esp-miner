# Parity work plan

- Run ID: `20260817T104623Z-STAT-001`
- Parity row: `STAT-001`
- Initial status: `implemented`
- Source commit: `223a0193af0fd35f94020e08fa4e4ba63be2a678`
- Reference commit: `c1915b0a63bfabebdb95a515cedfee05146c1d50`
- Active task: `task-parity-stat001-hashrate-monitor`

## Selection

The worktree/reference are clean, `main` equals `origin/main`, and the selector
has no open plan. It orders `SELF-001`, `BAP-002`, then `STAT-001`. SELF-001
remains blocked by the absent production-safe self-test route and required
hardware regression. BAP-002 remains blocked by unfinished BAP-001 firmware
UART/request/subscription ownership and no authorized live accessory path.

STAT-001 is first actionable. Attempt-016 produced a precise sealed v16/v10
failure after 364,314 active ms and 4/20 windows:
`watchdog_snapshot_retry_exhausted/retry_exhausted/unavailable/unavailable/
not_waiting`. Identity, attestation, safety, terminal state, safe stop, cleanup,
modes, digests, and redaction passed. Source tracing shows two concrete causes:
owner entry progress publishes subphase and feed as adjacent seqlock writes,
and all eight reader retries use only `spin_loop`, so they never yield a
scheduling opportunity when the writer is preempted with an odd sequence.

The active lesson set exceeds its loading budget; all headings were inventoried
and the full global file plus complete safety, authorization, evidence, retry,
cross-process, legacy-unit, and watchdog-relevant blocks were loaded. Lower-
priority historical GSD and older USB-session blocks were disclosed as omitted.
The current audit baseline is under 90 days old, fewer than ten lessons are new,
and no append is proposed, so no audit trigger is due. Repo-local hardware and
privacy policy plus Bright Builds architecture, testing, code-shape, and
verification rules govern this plan.

## Scope and non-scope

Advance only STAT-001 through local software changes. Add one store operation
that updates an owner entry subphase and the resulting watchdog observation
inside one sequence-bracketed publication. Refactor `ProductionTaskWatchdog`
so entry progress uses that fused publication after the ESP Task WDT reset,
while top-level and completion feeds retain their existing observation-only
publication.

Keep exactly eight coherent-read attempts, but replace CPU-only retry spins
with a scheduler yield between attempts. A preempted finite writer must be able
to complete before the next attempt; a publication that remains odd or changes
on every attempt must still end as `retry_exhausted`. Preserve poison handling,
single-writer assertions, phase/subphase/wait coherence, v16/v10 fields,
earliest tuple, 18-source evaluator identity, and all public/private schemas.

This plan authorizes source, tests, deterministic thread/hook fixtures,
documentation, real firmware/package builds, and ordinary Git operations only.
Do not access protected attempts, credentials, detector/device/USB/network
runtime, private values, or public projection candidates. Do not detect,
flash, reset, monitor, mine, actuate, update, erase, inject faults, manipulate
power, use external UART/BAP, touch electrical interfaces, retry attempt-016,
or create/run attempt-017. This plan cannot verify STAT-001 or change checklist,
progress-history, or README fields.

## Implementation

- [ ] Reproduce a writer preempted with an odd publication sequence and prove
      the current immediate retry budget exhausts without a scheduling handoff.
- [ ] Fuse owner-entry subphase plus watchdog observation into one publication;
      yield between the same eight bounded read attempts.
- [ ] Prove finite contention recovers to one coherent snapshot while a stuck
      odd writer and continuous sequence changes remain exact retry exhaustion.
- [ ] Preserve poison, stable, uninitialized, phase-clearing, wait, owner
      progress ordering, source inventory, v16/v10, and value-free evidence
      regressions; run every focused and mandatory gate.

## Verification and promotion

Focused checks:

1. `bazel test //firmware/bitaxe:task_watchdog_observation_tests //firmware/bitaxe:production_owner_progress_tests //firmware/bitaxe:hashrate_source_ownership_tests //crates/bitaxe-api:tests //tools/flash:tests //tools/automation:automation_test //tools/parity:tests`
2. `bazel build //tools/automation:contracts_verified //firmware/bitaxe:firmware`
3. `just verify-redaction`
4. `just verify-reference`
5. `just package`

Mandatory ordered gate:

1. `cargo fmt --all`
2. `cargo clippy --all-targets --all-features -- -D warnings`
3. `cargo build --all-targets --all-features`
4. `cargo test --all-features`
5. `bun scripts/bright-builds-check.ts all`
6. `just test`
7. `just parity`
8. `just parity-progress`

Acceptance requires one fused entry publication; a scheduler handoff between
the same eight attempts; deterministic recovery after finite odd-sequence
contention; fail-closed exhaustion for a writer that stays odd and for sequence
change on every attempt; unchanged poison and stable outcomes; exact owner
progress ordering; generated contracts and 18-source identity current; real
firmware/package, privacy/reference, file-length, immutable-plan, and diff gates
passing; and no hardware, public projection, checklist, progress, or README
mutation. Create a plan-bound `CLOSURE.md`, leave STAT-001 `implemented`, and
require a separate future plan before hardware.

## Non-claims

This plan does not prove attempt-016's exact scheduler interleaving, authorize
attempt-017, prove all contention impossible, verify hashrate accuracy, or
claim arbitrary profiles/pools, other boards/ASICs, update/recovery,
profitability, or release readiness.
