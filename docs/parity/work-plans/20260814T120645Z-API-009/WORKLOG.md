# Parity work log

## 2026-08-14T12:06:45Z | immutable-plan draft

- Source commit: `8e8cf59e930f63c8fffd132c4e0ffd5ab1d1bc22`.
- Actions: Selected API-009 first and drafted a software-only plan for a
  paused, safe-stopped asynchronous readiness signal window.
- Verification: Plan-only mandatory, privacy, reference, selector, task,
  immutable-digest, and diff gates remain pending.
- Evidence: Public attempt-016 category and lifecycle facts only. No credential,
  protected attempt content, USB, device, network, display, mining, or hardware
  interface was accessed.
- Outcome: Implementation and attempt-017 remain ineligible until this plan is
  verified, committed, and pushed at clean synchronized HEAD.
- Blocker or next safe action: Run the complete plan gate sequence, review the
  diff, commit and push, then implement only the software contract above.

## 2026-08-14T12:10:06Z | immutable-plan verification

- Plan SHA-256:
  `fad42c79d54d18789ec8647ffabf8cfab5804afa2b135f1c960f50a3f73c2236`.
- Actions: Ran the complete plan-only gate sequence and confirmed the selector
  resumes this unique API-009 plan.
- Verification: Formatting, strict Clippy, all-target build, all-feature tests,
  Bright Builds, canonical Bazel tests, parity, parity-progress, redaction,
  reference cleanliness, the real ESP firmware build, task uniqueness, open-
  plan selection, immutable digest, and diff checks pass.
- Evidence: Public plan/task/source and category-only gate outcomes. No
  credential, protected attempt content, USB, device, network, display, mining,
  or hardware interface was accessed.
- Outcome: The software-only signal-window plan is ready to commit and push
  without changing API-009 from `implemented`.
- Blocker or next safe action: Push this immutable checkpoint, confirm clean
  synchronized HEAD and the same open plan, then implement the bounded paused
  readiness transaction and sender.

## 2026-08-14T12:28:25Z | software implementation verified

- Source commit: `d0beee70c4815d23672e2fe353492550f6ea52dd`.
- Actions: Moved ready arming after the logical-pause/serial-safe-stop join,
  moved resume behind one-shot ready consumption, and moved the first IDENTIFY
  request after same-session active recovery. Added a one-hour operator-ready
  budget to the Rust child and TypeScript parent/fixture envelopes plus the
  explicit `signal-api-command-identify` command.
- Verification: Focused flash tests pass with 268 cases; the automation target,
  checkpoint real child, timeout/budget/source contracts, strict Clippy,
  all-target build, all-feature tests, Bright Builds, canonical Bazel tests,
  parity, parity-progress, redaction, reference cleanliness, and the real ESP
  firmware build pass. File-length findings were resolved by moving the
  command-effects tests into their natural submodule.
- Evidence: Deterministic software tests and public source only. The sender
  retains mode-checked private files, one-shot rename consumption, ordered
  checkpoint schema, and path-free public output. No credential, protected
  attempt content, USB, device, network, display, mining, or hardware interface
  was accessed.
- Outcome: The software contract is complete with API-009 still `implemented`.
  No checklist field or public parity evidence changed.
- Blocker or next safe action: Commit and push this implementation as the
  source checkpoint, then create the truthful software closure and rerun final
  repository gates. Attempt-017 remains unauthorized.
