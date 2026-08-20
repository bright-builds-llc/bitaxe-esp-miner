# Parity work plan

- Run ID: `20260820T171138Z-STAT-003`
- Parity row: `STAT-003`
- Initial status: `implemented`
- Source commit: `395dbc0e2046a0c6cee234db9a53bdeffb707a70`
- Reference commit: `c1915b0a63bfabebdb95a515cedfee05146c1d50`
- Active task: `task-parity-stat003-scoreboard`

## Selection

The worktree and pinned reference are clean, `main` equals `origin/main`, and
the deterministic selector reports no open plan. The user selected the
recommended software-only `STAT-003` correction after attempt-005 isolated a
restart-persistence verifier error.

The pinned upstream scoreboard keeps full runtime `double` difficulty in RAM,
serializes it to indexed NVS with `%.1f`, and reloads that durable one-decimal
value with `%lf`. The Rust scoreboard intentionally matches this behavior with
`format!("{:.1}")`, parse-on-load, and a focused test proving that only the
durable difficulty changes. The evidence verifier currently compares the full
pre-restart digest to the full post-restart digest, so it rejects valid
upstream-compatible persistence whenever a runtime difficulty has more than
one decimal place.

## Scope and non-scope

Advance only `STAT-003` through a software-only restart-persistence verifier
correction. Extend the closed scoreboard view with a second digest formed by
projecting only each finite positive difficulty through the pinned one-decimal
durable codec. Keep entry count, order, job ID, extranonce2, ntime, nonce, and
version bits exact. Require the raw pre-restart reads to match, the raw
post-restart reads to match, the post-restart view to be an idempotent durable
projection, and the pre-restart durable digest to equal the raw post-restart
digest.

Add pure coverage for projection semantics and full real-child coverage where
full-precision pre-restart difficulty becomes one-decimal post-restart
difficulty. Add negative regressions for wrong durable difficulty, changed
non-difficulty data, entry reordering, and post-restart repeat drift. Bind both
the Rust `"{:.1}"` codec and upstream `"%.1f"`/`"%lf"` codec in the existing
source inventory without changing its 31-path membership.

This plan authorizes source, deterministic local child processes, tests,
firmware/package builds, docs, Git commit, and push only. It does not authorize
credentials, protected attempt-005 artifacts, detector access, USB/device or
external network runtime, flash, monitor, mining, share submission, device
restart, evidence projection or promotion, attempt-006, recovery, external
UART/BAP, pins, or electrical work. Do not read, reinterpret, mutate, or
publish attempt-005 evidence.

## Implementation

- [ ] Add a source-bound one-decimal durable difficulty projection and closed
      exact/durable scoreboard digests.
- [ ] Compare restart persistence against the durable projection while keeping
      same-boot repeats and every non-difficulty field/order exact.
- [ ] Add positive and negative pure/real-child regressions for the complete
      restart contract.
- [ ] Run focused and mandatory gates, commit/push, and close without hardware,
      protected evidence, attempt-006, checklist transition, or progress sync.

## Verification and promotion

Focused tests must prove that full-precision pre-restart values accept only the
matching one-decimal durable post-restart values; changes to any other field,
order, count, or repeated read withhold projection. They must also prove that
the durable projection is finite, positive, and idempotent for admitted wire
values. Existing wire-shape, bounded-field, descending-order, source identity,
privacy, safety, restart identity, and first-failure gates remain unchanged.

Mandatory ordered gates are `cargo fmt --all`, `cargo clippy --all-targets
--all-features -- -D warnings`, `cargo build --all-targets --all-features`,
`cargo test --all-features`, `bun scripts/bright-builds-check.ts all`, `just
test`, `just parity`, and `just parity-progress`, plus relevant firmware/package,
redaction, reference, selector, sensitive-value, file-size, and diff checks.

Success is a clean pushed verifier correction with deterministic regressions
and no protected or hardware effects. Create `WORKLOG.md` and `CLOSURE.md`;
`STAT-003`, the checklist, progress history, and README remain unchanged. Any
future re-evaluation or attempt-006 requires a separate immutable evidence plan
that binds the corrected pushed source and explicitly authorizes its inputs and
effects.

## Non-claims

This plan does not verify live post-restart persistence, promote `STAT-003`,
re-evaluate attempt-005, authorize another device attempt, or verify arbitrary
profiles/pools, other ASICs/boards, unbounded mining, OTA, recovery, or release
readiness.
