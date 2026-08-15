# Parity work plan

- Run ID: `20260815T034210Z-API-009`
- Parity row: `API-009`
- Initial status: `implemented`
- Source commit: `42a12d09ae15ff32c67fdce51b727fce9c6334a9`
- Reference commit: `c1915b0a63bfabebdb95a515cedfee05146c1d50`
- Active task: `task-parity-api009-command-effect-evidence-audit`
- Continues plan: `docs/parity/work-plans/20260815T024341Z-API-009/PLAN.md`

## Selection

Clean synchronized HEAD has no open plan and the deterministic selector ranks
API-009 first. Attempt-025 proves the resumable-reactivation repair on hardware:
the campaign recovered from stale safety to five fresh observations, returned
active, and produced an accepted share. Its sealed command-specific record then
isolates a later boundary: exactly one dismissal request was issued only after
reactivation, but the host did not observe the notification cleared or the
block count preserved before the easy-target campaign terminally safe-stopped.

The generic zero-of-twenty continuity counters in the shared network evidence
schema are non-applicable to command-effects completion; the command-specific
v6 quorum is authoritative. The current state machine unnecessarily places the
dismissal join in the unstable resumed-mining interval even though it already
holds a stable safe-stopped pause after observing the genuine notification.

## Scope and non-scope

Add a live-shaped red regression at the command-effects state-machine seam.
Move the one dismissal request and its exact notification-clear plus
block-count-preservation readback into the already-proven safe-stopped pause,
before the IDENTIFY transaction. After the live IDENTIFY clear, issue one
resume request; when active reactivation is confirmed, advance directly to
terminal validation without a second dismissal. Preserve the existing v6
evidence fields and complete quorum.

Add a succinct why-comment and regression making clear that generic soak
continuity-window counters do not decide command-effects completion. Do not
weaken genuine notification provenance, the pause/safe-stop join, exact request
counts, operator checkpoint ordering, natural IDENTIFY expiry, active resume,
terminal HTTP/pool truth, recovery, evidence sealing, or redaction.

This plan is software-only. It may modify source, tests, task records, worklog,
and closure; run deterministic loopback processes; and build firmware. It may
not read credentials or prior raw/protected traces; access the detector, USB,
device/network interfaces, or display; issue device HTTP commands; flash,
reset, restart, mine, or manipulate controls; create public parity evidence;
promote API-009; run attempt-026; use direct UART or pins/pads/GPIO; or perform
destructive, fault-injection, OTA, rollback, erase, factory-reset, or power-
cycle effects.

## Implementation and verification

- [ ] Commit and push this immutable plan/task checkpoint before editing
      implementation source.
- [ ] Add a red regression proving that a rapid terminal transition after
      resumed activity can strand the current post-resume dismissal join.
- [ ] Require this request order: genuine active notification; one pause;
      serial plus HTTP safe-stopped pause; one dismissal; exact clear and
      unchanged block-count readback; IDENTIFY ready/rendered/natural-clear;
      one resume; active reactivation; terminal validation.
- [ ] Preserve request-once, failure-precedence, recovery, private-evidence,
      schema, redaction, and no-replay-by-default behavior.
- [ ] Add a focused assertion/comment that generic continuity windows are not
      command-effects prerequisites and cannot be used to diagnose its quorum.
- [ ] Run focused state-machine, command-effects, campaign, CLI, automation,
      real-process, redaction, and source-shape tests plus every mandatory gate.
- [ ] Close the plan with API-009 still `implemented`; attempt-026 requires a
      separate immutable exact-package hardware plan after this fix is pushed.

Before plan commit and final source commit, run in order: `cargo fmt --all`,
`cargo clippy --all-targets --all-features -- -D warnings`,
`cargo build --all-targets --all-features`, `cargo test --all-features`,
`bun scripts/bright-builds-check.ts all`, `just test`, `just parity`, and
`just parity-progress`. Also run `just verify-redaction`,
`just verify-reference`, `just build`, immutable plan digest, unique task
binding, selector closure, sensitive-output review, `git diff --check`, and a
full diff review.

Promotion is prohibited in this software-only plan. Accepted completion is a
regression-backed root-cause fix with all gates green and no hardware access.
Any unresolved test, build, privacy, reference, or design contradiction closes
truthfully without attempt-026.
