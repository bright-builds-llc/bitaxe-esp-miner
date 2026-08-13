# Parity work log

## 2026-08-13T16:39:21Z | attempt-012 contract checkpoint

- Source commit: `f9add8e29e47806baa79bef398d1a951437e1dad`.
- Actions: Closed and pushed the lost-intent ownership fix, re-ran the clean
  synchronized selector, selected API-009 first, and bound exactly one fresh
  attempt-012 to that verified production-boundary change.
- Verification: The exact attempt-011 pause-loss interleaving now has a typed
  owner and behavioral/source-ownership regressions. Focused campaign and real
  process tests plus every mandatory gate and real ESP build passed before
  selection.
- Evidence: Current pushed source, public closures, active task, and this
  redaction-safe contract only. No credential, package, detector, protected
  attempt, USB, device, or network interface was accessed.
- Outcome: Attempt-012 can become effect-eligible only after this immutable
  plan/task checkpoint is verified, committed, pushed, clean, synchronized,
  and every named post-push gate passes.
- Blocker or next safe action: Run plan-only gates and push this checkpoint
  before any hardware-capable command.

## 2026-08-13T16:48:00Z | immutable-plan verification

- Plan SHA-256:
  `5cf3553f0ea1580cb2108542772c3a62daaf5a61af831c6d9a23af39f54e7384`.
- Actions: Replayed the complete plan-only gate sequence with terse durable
  results after the original terminal output was truncated. Queried the parity
  selector against the drafted contract.
- Verification: Formatting, strict Clippy, all-target build, all-feature
  tests, Bright Builds, `just test`, parity, parity progress, redaction,
  reference cleanliness, the real ESP firmware build, and `git diff --check`
  all passed. The selector reports this API-009 plan and zero candidates.
- Evidence: The immutable plan digest, public task binding, selector result,
  and pass/fail gate results only. No hardware-capable command ran.
- Outcome: The single-attempt contract is ready to commit and push.
- Blocker or next safe action: Commit and push only this plan, work log, and
  task checkpoint; then verify clean synchronized HEAD before packaging.

## 2026-08-13T17:12:00Z | attempt-012 terminal closure

- Source commit: `76d6ff3153b1bc345784bc90d6b72f01f7f8014f`.
- Package app ELF SHA-256:
  `0bafae7a02bbdf9818980bd46008a12458266732c7c91b298fb05b197d12287c`.
- Actions: Passed all post-push gates, independently admitted the exact clean
  pushed package, ran the one private detector, and consumed the one authorized
  attempt-012. The USB transaction reached both post-flash recovery boundaries
  and monitor admission. No IDENTIFY checkpoint was emitted or confirmed.
- Verification: The public wrapper closed with primary category `timeout`.
  Its safe-stop, cleanup, and recovery booleans are false because the parent
  killed the child before its closing result was written. The campaign root is
  private but empty, the public projection is absent, all private modes remain
  valid, no related child process remains, and the admitted port is holder-free.
- Diagnosis: The host timeout is fixed at 810 seconds. The Rust child reserves
  600 seconds for observation plus 180 seconds of terminal grace, while the
  preceding bounded USB package admission, factory flash, NVS flash, and
  monitor admission also consume the same parent budget. This run needed about
  four minutes to reach monitor admission, making the parent deadline strictly
  shorter than the complete child transaction.
- Evidence: Redaction-safe wrapper fields, package identity facts, private
  mode checks, holder/process checks, and safe USB recovery-phase categories
  only. No raw trace or sensitive value was published.
- Outcome: `blocked`. API-009 remains `implemented`; evidence is withheld and
  attempt-013 is not authorized.
- Blocker or next safe action: Close and push attempt-012, then create a
  software-only plan to derive a parent deadline above the complete bounded
  child envelope and preserve the primary timeout through recovery reporting.

## 2026-08-13T17:26:00Z | closure verification

- Actions: Stabilized the shared real-process evidence test helper after the
  overloaded host made its five-second test-only deadline expire. Production
  timeout behavior is unchanged.
- Verification: The focused real-process group passed concurrently. The full
  ordered format, strict Clippy, all-target build, all-feature tests, Bright
  Builds, canonical Bazel tests, parity, parity-progress, redaction, reference,
  real ESP firmware build, and diff checks all pass. Closure metadata and
  evidence withholding remain intact.
- Outcome: The attempt-012 blocked closure and test reliability adjustment are
  ready to commit and push.
- Blocker or next safe action: Push this closure. Continue only with a new
  software-only timeout-contract plan; do not run attempt-013.
