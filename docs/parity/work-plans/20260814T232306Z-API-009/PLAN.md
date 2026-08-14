# Parity work plan

- Run ID: `20260814T232306Z-API-009`
- Parity row: `API-009`
- Initial status: `implemented`
- Source commit: `5bff772e03c7aae128addd15a894d22ac4993a11`
- Reference commit: `c1915b0a63bfabebdb95a515cedfee05146c1d50`
- Active task: `task-parity-api009-command-effect-evidence-audit`
- Continues plan: `docs/parity/work-plans/20260814T224914Z-API-009/PLAN.md`

## Selection

The clean synchronized selector reports no open plan and ranks API-009 first.
Attempt-022 proved that the exact IDENTIFY frame renders on the real Ultra 205,
but the host consumed the matching operator response 34 seconds after the
rendered checkpoint opened. The device effect correctly retained its upstream
30-second duration; the evidence protocol incorrectly required the human report
to arrive during that same device interval. The user requested a simpler,
latency-tolerant interaction. This exact orchestration boundary is actionable
without another hardware attempt.

## Scope and non-scope

Separate the bounded device effect from the unbounded operator report. Keep
each IDENTIFY activation exactly 30 seconds. A `confirmed` rendered or replayed
response attests that the operator observed the exact expected frame during the
checkpoint's uniquely bound device-effect interval; consuming that report
after the interval must not invalidate the observation. The checkpoint remains
attempt-local, single-use, mode-`0600`, and bound to one required file, so a
report cannot cross an attempt, checkpoint, or replay ordinal.

After a confirmed observation, wait until the current IDENTIFY effect is
conservatively inactive and then open the unbounded cleared checkpoint. Do not
issue a second IDENTIFY toggle merely to clear the frame. One initial observed
effect therefore requires one identify request; an explicit replay requires
exactly one additional request. Preserve the existing explicit replay choice,
inactive-before-replay gate, pause ownership, safe failure, cleanup, redaction,
late duplicate rejection, and maximum replay count of one.

This plan authorizes source, tests, documentation, tracker, deterministic
fixtures, and ordinary firmware builds only. It does not authorize credentials,
protected attempt artifacts, detector, USB, device/network access, HTTP to a
device, display claims, mining, restart, hardware control, direct UART,
pin/pad/GPIO work, public evidence, checklist promotion, attempt-023, or any
other hardware attempt.

## Implementation

- [ ] Replace the response-time expiry result with an unbounded, attempt-bound
      observed-during-effect attestation for rendered and replayed checkpoints.
- [ ] Replace the second toggle-to-clear request with conservative natural
      expiry before the cleared checkpoint and update the closed request-count
      quorum to `1 + replay_count`.
- [ ] Preserve explicit replay, inactive-before-replay, pause, decline,
      recovery, cleanup, privacy, and duplicate/cross-checkpoint rejection.
- [ ] Update public checkpoint semantics so the operator is told to report an
      observation made during the bounded effect, with no human response
      deadline.
- [ ] Add behavior-focused unit, campaign, CLI, evidence-binding, and real-child
      regressions for prompt and delayed confirmation, natural clear, replay,
      decline, duplicates, and request-count mismatch.

## Verification and promotion

Run focused flash and automation tests first, including the former exact-expiry
regression as a delayed-attestation acceptance test. Then run, in order:
`cargo fmt --all`; `cargo clippy --all-targets --all-features -- -D warnings`;
`cargo build --all-targets --all-features`; `cargo test --all-features`;
`bun scripts/bright-builds-check.ts all`; `just test`; `just parity`; and
`just parity-progress`. Also run `just verify-redaction`,
`just verify-reference`, `just build`, selector, unique-task, immutable-plan
digest, reference cleanliness, sensitive-output, `git diff --check`, and full
diff review.

Success closes this software-only plan with API-009 still `implemented`. It
proves only the deterministic latency-tolerant orchestration contract. A fresh
immutable hardware plan is required to prove the complete pause, identify,
natural clear, optional replay, resume, dismissal, restart, same-device,
safe-stop, cleanup, and redaction quorum. No hardware evidence or promotion is
permitted here.
