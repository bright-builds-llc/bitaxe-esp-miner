# STAT-001 worklog

## 2026-08-16T18:08:39Z | Watchdog-classifier diagnosis and plan checkpoint

- Source commit: `3632659c7a32033153b322e947cfaf64a820b35f`
- Actions: Ran the clean synchronized selector, skipped the first two concrete
  dependency/authorization blockers, audited the consumed attempt-006 closure,
  and traced the coarse discriminator through the runtime evaluator and
  campaign classifier.
- Verification: The core evaluator owns a closed reason vocabulary, while the
  campaign classifier checks coarse participation first and its test fabricates
  a production-inconsistent state. No protected attempt or hardware input was
  accessed.
- Evidence: Current source and deterministic evaluator semantics prove that
  specific non-participating causes cannot reach the existing specific reason
  branches.
- Outcome: STAT-001 is the first actionable row and a software-only classifier
  correction is eligible; no hardware ordinal is authorized. Ordered Cargo,
  Bright Builds, canonical tests, parity/progress, redaction, and reference
  gates passed. The selector admitted this plan after its required metadata
  shape was corrected, and `git diff --check` passed.
- Immutable plan SHA-256:
  `7c9130657a23fb7fbc5993be885652864c7cf50d79d8253b239ddfbaa3045fc0`.
- Blocker or next safe action: Commit and push the immutable plan/task
  continuation, then implement the closed reason classifier and schema
  rotation.

## 2026-08-16T18:20:52Z | Closed reason classifier and schema rotation

- Plan commit: `a8e5597628e1f5748629c476a4e5c060dc48c264` (pushed and
  synchronized before implementation).
- Actions: Classified every evaluator-owned watchdog reason before the generic
  participation check; added separate missing, unknown, and inconsistent
  failures; and rotated campaign-result v11 to v12 and network-continuity v5
  to v6 across Rust and TypeScript producers, consumers, fixtures, and gates.
- Verification: Ten focused Rust watchdog tests, the complete
  `//tools/flash:tests` target, and the hashrate-filtered
  `//tools/automation:automation_test` target passed. Active source contains no
  old schema or collapsed discriminator values; historical task evidence was
  intentionally preserved.
- Evidence: Production-shaped reason/participation pairs now distinguish
  `unproved`, invalid observations, subscription/feed/unsubscription failures,
  clean unsubscription, stale feed, missing/unknown reason, and inconsistent
  `feed_fresh` participation. Earliest-failure and value-free serialization
  regressions remain green.
- Outcome: The source-owned diagnostic collapse is corrected without changing
  firmware, mining, hardware control, parity fields, or progress history.
- Blocker or next safe action: Run the complete mandatory implementation and
  package gates, review the diff for simplification and private-value safety,
  then commit and push the source correction before closure.
