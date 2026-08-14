# Parity work log

## 2026-08-14T04:20:02Z | deterministic diagnosis checkpoint

- Source commit: `f8a0855713d65930ab9575804e93300d84b07678`.
- Actions: Closed and pushed attempt-015, restored a clean synchronized
  selector, joined its bounded serial diagnostics to the receive-only session
  and campaign fan-out source, and drafted one software-only fix contract.
- Verification: The first malformed marker is the first serial event and is
  shorter than every later accepted marker. The USB session directly forwards
  the first post-open bytes and resets no line boundary; the campaign callback
  then gives that same unadmitted chunk to both analyzers.
- Evidence: Public source and sealed category/count/length/boolean diagnostics.
  No credential, raw trace, detector, USB, device, network, display, mining,
  or hardware interface was accessed during this diagnosis.
- Outcome: Root cause is the receive-session line-admission gap, not package,
  flashing, runtime identity, protocol admission, safe stop, or cleanup.
- Blocker or next safe action: Verify, commit, and push this immutable
  software-only plan before adding the red regression. Attempt-016 remains
  unauthorized.

## 2026-08-14T04:24:55Z | immutable-plan verification

- Plan SHA-256:
  `b361b90301e2cf37a1b207aa0d457c43629466d8578b78bbfe9b3420a0ab8602`.
- Actions: Ran the complete plan-only gate sequence and queried the selector
  against the unique API-009 task binding.
- Verification: Formatting, strict Clippy, all-target build, all-feature tests,
  Bright Builds, all 44 canonical Bazel test targets, parity, parity-progress,
  focused device-session/flash/automation/real-process tests, redaction,
  reference cleanliness, the real ESP firmware build, task uniqueness,
  selector ownership, immutable digest, and diff checks pass.
- Evidence: Public plan/task/source and category-only gate outcomes. No
  credential, protected attempt artifact, detector, USB, device, network,
  display, mining, or hardware interface was accessed.
- Outcome: The software-only receive-ingress contract is ready to commit and
  push without changing API-009 from `implemented`.
- Blocker or next safe action: Push this checkpoint, confirm clean synchronized
  HEAD and the same open plan, then add the failing receive-session regression
  before production code.
