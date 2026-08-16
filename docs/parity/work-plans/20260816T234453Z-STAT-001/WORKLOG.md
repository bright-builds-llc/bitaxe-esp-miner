# Parity work log

## 2026-08-16T23:44:53Z | plan checkpoint

- Source commit: `a63e0243e68f7cc904fc4cad889a29925cf2ec5d`
- Actions: Loaded the required policy, standards, active lesson inventory,
  current tracker/checklist, canonical selector, and prior STAT-001 closure;
  traced the repeated watchdog discriminator across firmware and campaign
  ownership boundaries.
- Verification: The worktree and pinned reference are clean; `main` equals
  `origin/main`; no open plan exists; STAT-001 is the first actionable row.
- Evidence: The firmware classifies feed freshness against the compiled
  ESP-IDF timeout, while the production campaign independently reclassifies
  ages above 2,000 ms.
- Outcome: Immutable software-only correction plan ready for mandatory
  pre-commit verification.
- Blocker or next safe action: Commit and push this exact plan/task checkpoint,
  then turn the contradictory campaign threshold into a red regression before
  editing production logic.

## 2026-08-16T23:51:00Z | plan verification

- Source commit: `a63e0243e68f7cc904fc4cad889a29925cf2ec5d`
- Actions: Ran the mandatory ordered repository gates plus privacy, reference,
  and exact firmware-package verification.
- Verification: Cargo format, clippy, build, and tests; Bright Builds; all 46
  Bazel tests; redaction; reference cleanliness; firmware package; parity; and
  progress passed. The first parity launch hit transient host resource
  exhaustion after rendering its report; one bounded rerun passed with
  `validation_errors: none`, and progress remains 75 verified of 94 active.
- Evidence: Immutable plan SHA-256
  `3a82785aede895c12890cb3ee363ae97e06a05c369f53b0819c9c5b9b8664ff1`.
- Outcome: The plan/task checkpoint is verified for commit and push.
- Blocker or next safe action: Push the immutable checkpoint, then reproduce
  the obsolete host threshold in a focused red campaign test.
