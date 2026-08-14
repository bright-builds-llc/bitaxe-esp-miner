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

## 2026-08-14T02:05:58Z | red-green implementation checkpoint

- Source commit: `59508aba0dbba1166e13140bf9ae5cfface4d3c6`.
- Actions: Added the production-session regression before implementation; it
  failed because a delayed pre-active wake changed the resumable campaign from
  `Preparing` to `Consumed`. Replaced the ambiguous wall-clock lease with
  separately bounded activation and resumable-active phases, added a typed
  activation-timeout blocker and host terminal category, and expanded the Rust
  capture plus TypeScript parent/fixture budgets to cover both phases.
- Verification: The exact ignored harness changed from deterministic RED to
  GREEN and was removed from the workspace. Four focused boundary and
  pause/resume tests, all 273 Stratum tests, all 265 flash-tool tests, three
  affected firmware host targets, the automation test target, and the real
  child-process Stratum fixture target pass.
- Evidence: Public source, category-only red/green verdicts, typed snapshots,
  and process exit status. No credential, detector, protected raw trace, USB,
  device, network, or hardware interface was accessed.
- Outcome: The confirmed clock-ownership defect is fixed at the Production
  Mining Session owner. Activation expiry no longer remains live after the
  first active state; pause/resume continues to consume the single active
  epoch; the host can distinguish activation timeout from ordinary lease
  consumption.
- Blocker or next safe action: Run the full mandatory software gate sequence,
  review public output and the complete diff, then close and push this plan
  with API-009 still `implemented`.

## 2026-08-14T02:32:47Z | software closure checkpoint

- Source commit: `59508aba0dbba1166e13140bf9ae5cfface4d3c6` plus the uncommitted
  changes described by this work log.
- Actions: Completed the structural simplification pass, splitting campaign
  timing, runtime campaign-state transitions, timing regressions, and command-
  effects terminal assessment into focused modules. Updated the active task
  and created the terminal software closure without changing parity status.
- Verification: Formatting, strict Clippy, all-target build, all-feature tests,
  Bright Builds, all 44 Bazel test targets, parity, parity-progress, redaction,
  reference cleanliness, real ESP firmware build, immutable-plan digest,
  selector ownership, task uniqueness, and diff checks pass.
- Evidence: Public source and category-only command results. No credential,
  detector, protected attempt trace, USB, device, network, or hardware
  interface was accessed.
- Outcome: The software plan closes `blocked` with API-009 still
  `implemented`; the blocker is now only the absent complete live command-
  effects quorum, not the reproduced pre-active clock defect.
- Blocker or next safe action: Commit and push this checkpoint, require a clean
  synchronized selector, and let that selector decide whether a separately
  bounded attempt-014 plan is eligible.
