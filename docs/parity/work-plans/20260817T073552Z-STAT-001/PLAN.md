# Parity work plan

- Run ID: `20260817T073552Z-STAT-001`
- Parity row: `STAT-001`
- Initial status: `implemented`
- Source commit: `dcfddb13a657e516c66e72470697ae277ced43b9`
- Reference commit: `c1915b0a63bfabebdb95a515cedfee05146c1d50`
- Active task: `task-parity-stat001-hashrate-monitor`

## Selection

The worktree/reference are clean, `main` equals `origin/main`, and the selector
has no open plan. It orders `SELF-001`, `BAP-002`, then `STAT-001`. SELF-001
remains blocked by the absent production-safe self-test route and required
hardware regression. BAP-002 remains blocked by unfinished BAP-001 firmware
UART/request/subscription ownership and no authorized live accessory path.

STAT-001 is first actionable. Attempt-014 stopped at the distinct sealed
`watchdog_unproved` boundary after 302,436 active ms. Current evidence cannot
classify why the firmware projected no latest watchdog observation. Source
tracing also exposes a concrete diagnostic defect: `record_active_sample`
stores phase/wait before latching the first watchdog failure, but later
`record_terminal_sample` overwrites phase/wait even when the failure remains
latched. Therefore `watchdog_unproved` and `waiting_inbox/within_deadline` are
not guaranteed to describe one sample. This plan corrects that real evidence
boundary before any new hardware ordinal.

The active lesson set exceeds its loading budget; all headings were inventoried
and the full global file plus complete safety, authorization, evidence, retry,
unit, and watchdog-relevant blocks were loaded. Lower-priority historical
GSD/USB blocks were disclosed as omitted. The current audit baseline is under
90 days old, fewer than ten lessons are new, and no append is proposed, so no
audit trigger is due. Repo-local rules and Bright Builds architecture/testing
standards require typed boundary states, a thin firmware shell, and focused
Arrange/Act/Assert regressions.

## Scope and non-scope

Advance only STAT-001 through local software changes. Add a closed
`TaskWatchdogReadOutcome` domain with `stable`, `uninitialized`,
`retry_exhausted`, and `history_poisoned`. The firmware coherent store must
return the exact outcome instead of collapsing retry exhaustion or mutex poison
to the default. Runtime health must project the outcome and map read failures
to precise fail-closed reason labels rather than generic `unproved`.

Carry the field through HTTP/WebSocket/retained runtime-health wire surfaces,
the campaign network accumulator, result evidence, and private-first wrapper.
Latch read outcome, owner phase, and wait state only with the earliest watchdog
failure so later terminal observations cannot construct a mixed diagnostic
tuple. Rotate only the private campaign schemas from result v14/network v8 to
v15/v9; retain the public `bitaxe-hashrate-monitor-evidence-v1`, source-path
count, all safety/unit/identity/seal/redaction behavior, and checklist fields.

This plan authorizes source, tests, deterministic fixtures, documentation,
firmware/package builds, and ordinary Git operations only. Do not access
protected attempt-014 artifacts, credentials, detector/device/USB/network
runtime, private values, or a public projection. Do not flash, reset, monitor,
mine, actuate, update, erase, inject faults, manipulate power, use external
UART/BAP, touch electrical interfaces, retry attempt-014, or create/run
attempt-015. This plan cannot verify STAT-001.

## Implementation

- [ ] Add the closed read-outcome enum and make stable/uninitialized/retry-
      exhausted/history-poisoned store exits explicit and unit tested.
- [ ] Project the outcome through runtime health and wire/retained records;
      map retry exhaustion and poison to distinct fail-closed watchdog reasons.
- [ ] Make the campaign parse a closed outcome, latch it atomically with the
      first watchdog failure plus phase/wait, and preserve that tuple through
      terminal observations and earliest-failure handling.
- [ ] Rotate campaign result/network evidence to v15/v9 and update every Rust,
      TypeScript, generated-contract, fixture, seal, wrapper, source-inventory,
      redaction, and real-child consumer.
- [ ] Add an attempt-014-shaped regression proving the old mixed diagnostic
      fails before the fix, later terminal samples cannot overwrite the tuple,
      and each read outcome yields its exact value-free failure.

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

Acceptance requires exact outcome spellings and defaults; retry/poison no
longer mapping to generic `unproved`; one earliest watchdog failure atomically
retaining its outcome/phase/wait despite later terminal samples; v15/v9 only;
generated contracts and 18-source evaluator identity current; all focused and
mandatory gates passing; immutable plan/reference clean; and no hardware,
public projection, checklist, progress, or README mutation. Create a plan-bound
`CLOSURE.md`, leave STAT-001 `implemented`, and require a separate future plan
before hardware.

## Non-claims

This plan does not determine attempt-014's actual read outcome, pre-authorize a
hardware retry, prove scheduler/lock/transport behavior, verify hashrate
accuracy, or claim arbitrary profiles/pools, other boards/ASICs,
update/recovery, profitability, or release readiness.
