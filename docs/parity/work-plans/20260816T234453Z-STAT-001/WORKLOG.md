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

## 2026-08-17T00:01:00Z | campaign policy correction

- Source commit: pending exact implementation commit
- Actions: Added a production campaign regression for a producer-classified
  fresh 2,001-ms feed, observed it fail as `WatchdogFeedStale`, removed the
  campaign's duplicate numeric freshness policy, and rebound stale fixtures to
  the producer's typed `feed_stale` verdict at 5,001 ms.
- Verification: The 107 campaign-network tests, 16 runtime-health tests, and
  focused Bazel `tools/flash` plus `bitaxe-core` targets pass. Producer verdicts
  at 2,001 and 5,000 ms are accepted; 5,001 ms remains stale; missing fields,
  inconsistent participation, unknown reasons, stagnant per-transport
  checkpoint/feed sequences, earliest-failure precedence, and value-free
  serialization remain fail-closed. A source guard rejects any new numeric
  feed-age comparison in the campaign evaluator.
- Evidence: The implementation deletes three policy lines and changes no
  firmware, API schema, campaign schema, hardware-control, mining, sensor, or
  private-evidence behavior.
- Outcome: Root cause corrected at the contradictory producer/consumer policy
  boundary; ready for complete repository verification.
- Blocker or next safe action: Run all plan-mandated gates, then commit and push
  the exact software correction without a checklist transition.

## 2026-08-17T00:08:00Z | implementation verification

- Source commit: pending exact implementation commit
- Actions: Ran the complete focused, privacy, reference, firmware-package,
  mandatory repository, parity-invariance, plan-immutability, and diff gates.
- Verification: Cargo format, clippy, build, and tests; Bright Builds; all 46
  Bazel tests; redaction; reference cleanliness; exact firmware package;
  parity; and progress passed. The first parity launch hit transient host
  resource exhaustion after rendering its report; one bounded rerun passed
  with `validation_errors: none`. Progress remains 75 verified of 94 active.
- Evidence: The immutable plan SHA-256 remains
  `3a82785aede895c12890cb3ee363ae97e06a05c369f53b0819c9c5b9b8664ff1`;
  checklist and deterministic progress inputs are unchanged.
- Outcome: Exact software implementation ready for source commit and push.
- Blocker or next safe action: Push the implementation, then write a truthful
  non-promotion closure and complete the active task subsection.

## 2026-08-17T00:13:00Z | non-promotion closure

- Source commit: `812bcd45b284e44d7a1d5ec3bc35f2148b14b7ff`
- Actions: Pushed the exact root-cause correction and recorded the conservative
  software-only closure without changing checklist or progress inputs.
- Verification: The source commit is on `origin/main`; the immutable plan hash
  is unchanged; no hardware or protected runtime input was accessed.
- Evidence: `CLOSURE.md` binds the red failure, corrected producer/consumer
  ownership, preserved closed boundaries, complete gates, and explicit
  non-claims.
- Outcome: `software_correction_complete`; STAT-001 remains `implemented`.
- Blocker or next safe action: Run the final ordered gates on the closed state,
  commit and push the closure, then end this one-row invocation. Any attempt-010
  requires a fresh complete immutable plan.
