# Parity work plan

- Run ID: `20260818T112336Z-STAT-003`
- Parity row: `STAT-003`
- Initial status: `implemented`
- Source commit: `f709b8237d744f03d9a8c3db72c0945dcb94fa4c`
- Reference commit: `c1915b0a63bfabebdb95a515cedfee05146c1d50`
- Active task: `task-parity-stat003-scoreboard`

## Selection

The worktree and pinned reference are clean, `main` equals `origin/main`, and
the deterministic selector reports no open plan. Candidate order begins
`SELF-001`, `BAP-002`, then `STAT-003`.

`SELF-001` remains blocked without a production-safe complete hardware self-
test route. `BAP-002` remains blocked by not-started `BAP-001` and a remaining
external UART/electrical dependency outside standing USB authorization.
`STAT-003` is first actionable because attempt-003 isolated a pure verifier
predicate after proving every preceding campaign, network, scoreboard, SPA, and
restart identity boundary.

Attempt-003 produced accepted network v12, 20/20 windows, candidates, accepted
submit, identity, safety, watchdog, terminal state, safe stop, cleanup, a stable
20-entry scoreboard, live `/scoreboard`, and an exact session-changing ordinal-
plus-one software-CPU restart with `startMiningOnBoot=false`. The verifier then
rejected the closed non-active `miningActivity=paused` because it hardcodes only
`safe_blocked`. Rust intentionally uses `paused` for an operator-paused blocker
and `safe_blocked` for other blockers; both are non-active. Upstream exposes the
separate `miningPaused` boolean, not this Rust extension, so it does not require
one internal non-active spelling.

Loaded guidance materially includes `AGENTS.md`, `AGENTS.bright-builds.md`,
managed architecture/code-shape/verification/testing/Rust standards, the active
task/checklist, and bounded whole lesson blocks for privacy, diagnostic
completeness, failure precedence, retry discipline, qualified transports,
transitive evaluator identity, and agent-runtime timing. The repository lesson
ledger remains above startup budget; all headings were inventoried, less-
relevant whole blocks were omitted, and the existing August audit baseline
means no new audit trigger is due.

## Scope and non-scope

Advance only `STAT-003` through a software-only stopped-state verifier
correction. Add one pure closed predicate for disabled boot mining:
`startMiningOnBoot=false` and `miningActivity` equal to either `paused` or
`safe_blocked`. Use that predicate in both post-restart admission and the final
`boot_mining_disabled` evidence boolean so the same truth cannot drift.

Add pure table coverage for both accepted states and rejection of `active`, an
unknown string, and enabled boot intent. Extend the real-child scoreboard server
to return `paused` after restart and prove the entire workflow publishes; retain
the existing `safe_blocked` success and restart-drift failure. Keep exact
package, boot session, ordinal +1, `software_cpu`, scoreboard repeat/persistence,
source identity, privacy, safety, and first-failure gates unchanged.

The evaluator source inventory already binds the scoreboard evidence contract
and orchestrator; their source changes must rotate the 31-path digest without
membership drift.

This plan authorizes source, deterministic local child processes, tests,
firmware/package builds, docs, Git commit, and push only. It does not authorize
detector access, credentials, protected attempt artifacts, USB/device/network
runtime, flash, monitor, mining, restart, public projection, attempt-004,
recovery, external UART/BAP, pins, or electrical work. Do not reinterpret or
mutate attempt-003 evidence.

## Implementation

- [ ] Add one pure disabled-boot-mining predicate with accepted paused/safe-
      blocked and rejected active/unknown/enabled table coverage.
- [ ] Use the predicate for post-restart admission and evidence projection so
      truth is defined once.
- [ ] Add a full real-child paused-after-restart success regression while
      retaining safe-blocked success and existing failure cases.
- [ ] Run focused and mandatory gates, commit/push, and close without hardware,
      attempt-004, checklist transition, or progress sync.

## Verification and promotion

Focused tests must prove both closed non-active states require false boot intent;
active, unknown, and enabled shapes fail; a paused exact restart continues to
post-restart scoreboard reads and public projection; safe-blocked remains valid;
restart drift still withholds; source inventory remains complete and rotates.

Mandatory ordered gates are `cargo fmt --all`, `cargo clippy --all-targets
--all-features -- -D warnings`, `cargo build --all-targets --all-features`,
`cargo test --all-features`, `bun scripts/bright-builds-check.ts all`, `just
test`, `just parity`, and `just parity-progress`, plus firmware/package,
redaction, reference, selector, sensitive-value, file-size, and diff checks.

Success is a clean pushed correction with deterministic reproduction and no
hardware effects. Create `WORKLOG.md` and `CLOSURE.md`; `STAT-003`, checklist,
progress history, and README remain unchanged. Only a future immutable hardware
plan may consider attempt-004 after this correction is package-bound. A repeat
of the attempt-003 stopped-state signature after its targeted fix must stop
further retries.

## Non-claims

This plan does not verify post-restart hardware persistence, claim any active or
unknown state safe, verify arbitrary profiles/pools, other ASICs/boards,
unbounded mining, OTA, recovery, or release readiness.
