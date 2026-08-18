# Parity work plan

- Run ID: `20260818T095707Z-STAT-003`
- Parity row: `STAT-003`
- Initial status: `implemented`
- Source commit: `26ec239d766af42049768f830378d05814d4c4af`
- Reference commit: `c1915b0a63bfabebdb95a515cedfee05146c1d50`
- Active task: `task-parity-stat003-scoreboard`

## Selection

The worktree and pinned reference are clean, `main` equals `origin/main`, and
the deterministic selector reports no open plan. Candidate order begins
`SELF-001`, `BAP-002`, then `STAT-003`.

`SELF-001` remains dependency- and safety-blocked because no production-safe
route exercises its complete hardware self-test lifecycle. `BAP-002` remains
blocked by not-started `BAP-001`; its remaining accessory path also requires
external UART/electrical attachment outside standing USB authorization.
`STAT-003` is first actionable because attempt-002 isolated one deterministic
software predicate with complete closed evidence.

Attempt-002 produced a sealed accepted 600-second campaign, 20/20 renewed
windows, real scoreboard candidates, accepted submit, trusted identity, fresh
safety, stable watchdog, no panic/mixed reset, terminal HTTP/WebSocket/pool
confirmation, final consumed serial state, safe stop, and cleanup. Network v12
also recorded `accepted_after_serial_close` and serial finished, but status
failed solely because `terminal_close_requested=false`. The serial analyzer had
naturally finished before the worker needed to request closure. The boolean is
therefore a closed initiator diagnostic, not acceptance truth.

Loaded guidance materially includes `AGENTS.md`, `AGENTS.bright-builds.md`,
managed architecture/code-shape/verification/testing/Rust standards, the active
task/checklist, and bounded whole lesson blocks for evidence privacy, earliest
failure, retry discipline, transitive evaluator identity, and agent-runtime
timing. The repository lesson ledger remains above startup budget; all headings
were inventoried, less-relevant whole blocks were omitted, and the existing
August audit baseline means no new audit trigger is due.

## Scope and non-scope

Advance only `STAT-003` through a software-only natural-closure correction.
Remove `terminal_close_requested` from the network acceptance predicate while
retaining the field as a mandatory closed boolean diagnostic. Acceptance must
still require no earlier failure, 20 complete windows, final terminal consumed,
serial finished, and `accepted_after_serial_close`; existing finish-time checks
continue to require pool persistence and terminal HTTP/WebSocket confirmation.

Update both hashrate and scoreboard consumers so either boolean value is valid,
but missing/non-boolean remains invalid. Add production-shaped Rust and real-
child TypeScript regressions for natural analyzer closure with false, plus
existing worker-requested true coverage and withholding for missing final
consumed state. Because the evaluator inventories already bind the reducer,
model, evidence, consumers, Rust validator, and validator entrypoint, these
source changes must rotate their digests without changing path membership.

This plan authorizes repository source, deterministic local child processes,
tests, firmware/package builds, docs, Git commit, and push only. It does not
authorize detector access, credentials, protected attempt artifacts, USB,
device/network runtime, flash, monitor, mining, restart, public projection,
attempt-003, recovery, external UART/BAP, pins, or electrical work. Do not
reinterpret or mutate attempt-002 evidence.

## Implementation

- [ ] Add a Rust regression proving natural analyzer closure accepts with
      `terminal_close_requested=false`, while keeping the field serialized and
      all final truth gates mandatory.
- [ ] Remove only the overconstrained acceptance predicate and update hashrate/
      scoreboard consumers to type-check rather than truth-require the closure
      initiator diagnostic.
- [ ] Add real-child consumer regressions for false acceptance and preserve
      rejection of missing/non-boolean diagnostics and missing final consumed
      state.
- [ ] Run focused and mandatory gates, commit/push the correction, and close
      without hardware, attempt-003, checklist transition, or progress sync.

## Verification and promotion

Focused tests must prove: worker-requested closure remains accepted; natural
analyzer closure is accepted with false; false is retained in v12 evidence;
missing/non-boolean remains invalid; final consumed and serial finished remain
mandatory; incomplete terminal transports or pool persistence still fail;
earlier failures win; both hashrate and scoreboard real-child workflows accept
the natural closure shape; source inventories remain complete and rotate.

Mandatory ordered gates are `cargo fmt --all`, `cargo clippy --all-targets
--all-features -- -D warnings`, `cargo build --all-targets --all-features`,
`cargo test --all-features`, `bun scripts/bright-builds-check.ts all`, `just
test`, `just parity`, and `just parity-progress`, plus real firmware/package,
redaction, reference, selector, sensitive-value, and diff checks.

Success is a clean pushed correction with deterministic reproduction and no
hardware effects. Create `WORKLOG.md` and `CLOSURE.md`; `STAT-003`, checklist,
progress history, and README remain unchanged. Only a future immutable hardware
plan may consider attempt-003 after this correction is package-bound. If the
same closed boundary recurs after that targeted fix, stop without another retry.

## Non-claims

This plan does not verify live scoreboard API/SPA behavior, persistence,
restart durability, arbitrary scheduling interleavings, hardware difficulty,
arbitrary profiles/pools, other ASICs/boards, unbounded mining, OTA, recovery,
or release readiness.
