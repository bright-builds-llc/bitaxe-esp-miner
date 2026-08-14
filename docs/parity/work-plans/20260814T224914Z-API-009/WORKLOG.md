# Parity work log

## 2026-08-14T22:49:14Z | immutable-plan draft

- Source commit: `420c22369ad16025b0ce47ba6748144637217d6b`.
- Actions: Confirmed clean synchronized `main`, selected API-009 first, checked
  the ignored Wi-Fi input only for non-empty and ignored status, confirmed all
  attempt-022 paths are fresh, and drafted one replay-capable hardware plan.
- Verification: Plan-only mandatory, focused, privacy, reference, firmware,
  selector, task, immutable-digest, and diff gates are pending before commit.
- Evidence: Public source, selector output, and categorical preflight checks
  only. No credential content, protected attempt content, detector, USB,
  device, network, display, mining, or hardware interface was accessed.
- Outcome: Attempt-022 remains ineligible until this immutable contract and all
  named plan gates are committed and pushed at clean synchronized HEAD.
- Blocker or next safe action: Run the complete plan-only gate sequence, review
  the diff, commit and push, then build and validate the exact package before
  the sole detector run.

## 2026-08-14T22:56:37Z | immutable-plan verified

- Source commit: `420c22369ad16025b0ce47ba6748144637217d6b`.
- Actions: Kept the plan immutable, reviewed the complete plan/task diff, and
  bound the selector to this single open API-009 plan and active task.
- Verification: Focused replay, exact effect-deadline, human checkpoint,
  automation real-process, Cargo format, strict Clippy, all-target build,
  all-feature tests, Bright Builds, `just test`, `just parity`,
  `just parity-progress`, redaction, reference cleanliness, real firmware
  build, unique task, selector, plan digest, and diff checks passed. The first
  `just parity` invocation encountered transient host error 35; one immediate
  retry passed with no validation errors. Immutable plan SHA-256 is
  `43b3913f651a1a16b66f7a761c69e6608a98da2ed230f08080c97eaf5edd00b6`.
- Evidence: Plan-only public checks. Credential contents and protected attempt
  artifacts were not read, and detector, USB, device, network, display,
  mining, and hardware-control interfaces were not accessed.
- Outcome: The single-attempt contract is ready to commit and push. No hardware
  attempt has been consumed.
- Blocker or next safe action: Commit and push this immutable checkpoint, then
  build and validate the exact pushed package before the sole detector run.

## 2026-08-14T23:06:31Z | exact-package attempt and terminal closure

- Source commit: `79756b9a21a1de5ed2b8ad1fedbb84b1f2f4fb89`.
- Actions: Built and validated the exact package, ran the fresh detector once,
  and invoked the sole attempt-022 campaign once. Consumed ready and rendered
  only after their matching live user inputs; no replayed or cleared signal was
  sent because the campaign terminated at the rendered boundary.
- Verification: One board-205 session, trusted runtime identity, genuine
  notification, positive block count, pause, and stopped hardware passed. The
  unbounded ready wait lasted 28 seconds. The rendered checkpoint opened one
  second after ready consumption, but the confirmed response was consumed 34
  seconds after it opened and was correctly rejected beyond the exact
  30-second evidence window. The terminal result is
  `operator_checkpoint_expired` / `operator_paused`; one recovery pause proves
  terminal safe stop, USB cleanup is ready, recovery has no secondary failure,
  private modes pass, attempt processes are absent, and projection withholding
  passes.
- Evidence: Only closed categorical fields, booleans, counts, modes, bounded
  durations, and source identities were inspected. Credential, port,
  USB/network identity, origin, hostname, sensor values, and raw traces remain
  protected.
- Outcome: Attempt-022 is consumed. API-009 remains `implemented`; the user's
  live physical observation is retained as diagnostic truth but is not timely
  host-bound evidence. No replay, clear, resume, dismissal, restart, public
  evidence, or promotion is claimed.
- Blocker or next safe action: Close this immutable plan without attempt-023.
  A fresh software-only plan should add an unbounded post-expiry replay-choice
  checkpoint before any later hardware plan.

## 2026-08-14T23:10:32Z | terminal closure verified

- Actions: Added the plan-bound non-verifying closure and updated only the
  active API-009 task block. The immutable plan digest remains unchanged.
- Verification: Cargo format, strict Clippy, all-target build, all-feature
  tests, Bright Builds, `just test`, `just parity`, `just parity-progress`,
  redaction, reference cleanliness, real firmware build, selector closure,
  unique task, projection withholding, protected modes, process cleanup, plan
  digest, and complete diff review pass. One transient host error 35 occurred
  during the first final `just parity` invocation; its single retry passed with
  no validation errors.
- Outcome: The closure is ready to commit and push. API-009 remains first in
  the unfinished selector with no open plan, and no additional hardware action
  is authorized by this plan.
- Blocker or next safe action: Publish this truthful closure. Any continuation
  begins with a fresh software-only plan for the post-expiry replay choice.
