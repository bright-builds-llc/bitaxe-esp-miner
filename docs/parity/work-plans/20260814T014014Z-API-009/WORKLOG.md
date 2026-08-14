# Parity work log

## 2026-08-14T01:40:14Z | deterministic diagnosis checkpoint

- Source commit: `1c3f8d5bb180cbe3e1fff41010cc30e8233fb4df`.
- Actions: Resumed API-009 from the clean synchronized selector, built an
  ignored public-API Production Mining Session harness, reproduced the exact
  pre-active lease expiry three times, and probed both sides of 100- and
  200-millisecond boundaries.
- Verification: The harness fails deterministically because the resumable
  lease enters `SafeStopping` at its exact duration while hardware is still
  `Preparing`. Thresholds scale linearly. Source and sealed aggregate checks
  rule out wrong-stage admission, submit-response consumption, host-invented
  terminal state, and clock units.
- Evidence: Public source, closed attempt-013 aggregate facts, the ignored
  synthetic harness, and red/green category-only verdicts. No credential,
  detector, protected raw trace, USB, device, network, or hardware interface
  was accessed.
- Outcome: Root cause confirmed as an activation-clock/resumable-epoch
  ownership mismatch. The two-phase bounded software correction is ready for
  an immutable plan checkpoint.
- Blocker or next safe action: Verify, commit, and push this plan before any
  tracked regression or production-source edit. Attempt-014 remains
  unauthorized.

## 2026-08-14T01:47:00Z | immutable-plan verification

- Plan SHA-256:
  `858b4d0626dcccd9a5f691b52ecac843b025f34876acc9f6d9f072db74bd5ffa`.
- Actions: Ran the complete plan-only gate sequence and queried the selector
  against the drafted task binding.
- Verification: Formatting, strict Clippy, all-target build, all-feature tests,
  Bright Builds, canonical Bazel tests, parity, parity-progress, redaction,
  reference cleanliness, the real ESP firmware build, task uniqueness, and
  diff checks pass. The selector resumes only this API-009 plan with no
  alternate candidates.
- Evidence: Immutable plan digest, public task binding, selector result, and
  category-only pass/fail gate results. No hardware-capable command ran.
- Outcome: The software-only diagnosis/fix contract is ready to commit and
  push.
- Blocker or next safe action: Push this checkpoint, confirm clean synchronized
  HEAD, then add the tracked failing regression before production changes.
