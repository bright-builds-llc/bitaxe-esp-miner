# Parity work plan

- Run ID: `20260815T010813Z-API-009`
- Parity row: `API-009`
- Initial status: `implemented`
- Source commit: `0fbcf9c041b38d28a68b3343806912a91b979f6b`
- Reference commit: `c1915b0a63bfabebdb95a515cedfee05146c1d50`
- Active task: `task-parity-api009-command-effect-evidence-audit`
- Continues plan: `docs/parity/work-plans/20260814T234917Z-API-009/PLAN.md`

## Selection and diagnosis

The clean synchronized selector reports no open plan and ranks API-009 first.
Attempt-023 proved the complete latency-tolerant IDENTIFY transaction, then
failed immediately after its one resume request. Closed evidence records
`resume_request_count=1`, no resume confirmation, no dismissal, no restart,
and `network_correlation_failed` after the fixed 15-second automated phase plus
one polling interval. Recovery issued one pause, but the network owner stopped
without joining an API/serial safe-stop confirmation; the wrapper consequently
reported a secondary recovery failure and no confirmed terminal safe stop.

The host currently represents notification, resume, dismiss, and terminal with
one shared 15-second deadline. Resume has no typed intermediate fact for the
API-visible run intent or production re-preparation, so a delayed transition
and an unapplied transition are indistinguishable. On any failure it sends one
best-effort pause, requests network stop, and breaks immediately. Separately,
the firmware's resumable campaign duration is measured from the first active
epoch across operator-paused wall time, contradicting the public unbounded
operator-wait contract and the plan's 600-second active-time budget.

## Scope and non-scope

This is a software-only root-cause repair. Split resume into typed intent and
reactivation phases. Require the API projection to show run intent before the
host waits for active mining. Give production re-preparation and pool recovery
their own bounded activation contract rather than the unrelated 15-second
command deadline. Preserve exact package, same-boot, safety, and identity
validation throughout.

Make `ResumableActiveEpoch` consume only accumulated active mining time. Paused
human time remains unbounded and must not silently consume the 600-second active
budget. Preserve expiration, overflow, clock-regression, higher-lease, and
terminal-safe-stop behavior.

On any post-start failure, preserve the earliest terminal category, issue at
most one recovery pause, and continue the network/serial join for a bounded
recovery interval. Confirm safe stop only from both API paused state and the
authoritative serial hardware-stop marker. Stop the network worker only after
that join succeeds or expires; recovery remains secondary and cannot replace
the primary failure.

Update closed command-effects evidence and host validation to distinguish
resume request, intent confirmation, reactivation confirmation, recovery-pause
request, and recovery safe-stop result without exposing timing origins,
network/device identities, credentials, sensor values, or traces. Preserve the
latency-tolerant IDENTIFY semantics, explicit single replay, request-count
quorum, dismissal, restart transaction, cleanup, and evidence withholding.

This plan authorizes source, tests, deterministic fixtures, documentation,
tracker, worklog, closure, and ordinary firmware builds only. It does not
authorize credentials, protected attempt artifacts, detector, USB,
device/network access, device HTTP, display claims, mining, restart, hardware
control, direct UART, pins/pads/GPIO, public parity evidence, checklist
promotion, attempt-024, or any hardware attempt.

## Implementation and verification

- [ ] Add pure resumable active-budget accounting that pauses the budget while
      hardware is stopped for an operator-resumable pause and resumes without
      resetting already consumed active time.
- [ ] Split host resume into request, API intent confirmation, and bounded
      active reactivation phases with separate typed failures and evidence.
- [ ] Replace immediate post-failure shutdown with a one-shot recovery-pause
      join over API paused state and authoritative serial stopped-hardware state.
- [ ] Bump and validate closed evidence schemas; preserve earliest-failure,
      duplicate/count, privacy, cleanup, restart, and evidence-withholding
      invariants.
- [ ] Add behavior-focused pure engine, firmware owner, host state-machine,
      delayed transition, never-transition, recovery success/timeout, CLI,
      evidence mismatch, and real-child regressions.

Run focused targets first. Then run in order: `cargo fmt --all`;
`cargo clippy --all-targets --all-features -- -D warnings`;
`cargo build --all-targets --all-features`; `cargo test --all-features`;
`bun scripts/bright-builds-check.ts all`; `just test`; `just parity`; and
`just parity-progress`. Also run `just verify-redaction`,
`just verify-reference`, `just build`, selector, unique-task, immutable-plan
digest, reference cleanliness, sensitive-output, `git diff --check`, and full
diff review.

Success closes this software-only plan with API-009 still `implemented`. It
proves deterministic resume/recovery orchestration only. A fresh immutable
hardware plan is required before detector admission or attempt-024.
