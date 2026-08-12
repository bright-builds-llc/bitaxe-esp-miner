# STR-006 worklog

## 2026-08-12T12:22:56Z | Fresh selection and retry design

- Source commit: `8789f99abc885f41f89cf07981661a367be06233`
- Actions: Ran the clean synchronized selector preflight, selected `STR-006`
  first, and inspected both legitimate ASIC-worker dispatch occurrences.
- Verification: The prior closure is accepted, no plan is open, the reference
  is clean at `c1915b0a63bfabebdb95a515cedfee05146c1d50`, and the current branch
  equals `origin/main`. The worker source has exactly one executor-consumption
  span and one effect-to-command mapping span.
- Evidence: No protected evidence or hardware was accessed.
- Outcome: A minimal software-only fix and production-shaped regression can
  repair the proof without weakening the existing evidence contract.
- Blocker or next safe action: Run the full plan-only gate, seal and push this
  immutable plan, then implement the two-span source guard.

## 2026-08-12T12:28:19Z | Plan gate and seal

- Source commit: `8789f99abc885f41f89cf07981661a367be06233`
- Actions: Ran the complete ordered repository gate plus redaction, reference,
  reference-cleanliness, immutable-plan, task-uniqueness, and diff checks for
  the plan-only change.
- Verification: All Cargo stages, Bright Builds, all 37 Bazel tests, parity,
  and progress pass. Progress remains 57 of 94 active rows verified (60.6%).
  Immutable plan SHA-256 is
  `d35415c87cc640f29749fcac4fa53132b7391e9e3e929b5ad2f2d0d1cb45f9da`.
- Evidence: No protected evidence or hardware was accessed.
- Outcome: The fresh bounded plan is ready to commit and push.
- Blocker or next safe action: Push the immutable plan before editing the
  projector or its regressions.

## 2026-08-12T12:30:29Z | Two-span guard implementation

- Source commit: `fc9c807359581b35c9af8dd7e03c6c116bdba10e`
- Actions: Replaced the broad worker dispatch token with the exact ordered
  executor-consumption and effect-to-command mapping spans, and reshaped the
  synthetic source fixture to match both legitimate production occurrences.
- Verification: The focused automation Bazel suite passes. New regressions
  reject missing, duplicate, reordered, and unbound spans; the production-shaped
  accepted fixture passes. The projector is 499 lines, the file-length check
  has no findings, and `git diff --check` passes.
- Evidence: No protected evidence or hardware was accessed. No projection was
  attempted or published.
- Outcome: The false uniqueness assumption is removed without weakening the
  source, lineage, validation, atomic-publication, or redaction gates.
- Blocker or next safe action: Run the complete implementation gate, then
  commit and push before the plan's one software-only projection attempt.

## 2026-08-12T12:35:59Z | Implementation gate

- Source commit: `fc9c807359581b35c9af8dd7e03c6c116bdba10e`
- Actions: Ran the complete ordered repository gate plus generated-contract,
  redaction, reference, reference-cleanliness, immutable-plan,
  task-uniqueness, and diff checks.
- Verification: All Cargo stages, Bright Builds, the expanded automation suite,
  all 37 Bazel tests, parity, and progress pass. Generated contracts verify,
  redaction checks 13 changed files, the pinned reference is clean, and the
  plan digest remains
  `d35415c87cc640f29749fcac4fa53132b7391e9e3e929b5ad2f2d0d1cb45f9da`.
- Evidence: No protected evidence or hardware was accessed. No projection was
  attempted or published.
- Outcome: The corrected implementation satisfies every pre-publication gate.
- Blocker or next safe action: Commit and push the fix, confirm exact clean
  synchronization, then run the plan's one software-only projection attempt.

## 2026-08-12T12:37:28Z | Accepted projection

- Source commit: `d6059a4330de070cca92b09346ac24a91ecd1300`
- Actions: Confirmed exact clean synchronization, ran the plan's single
  software-only projection attempt, then independently validated the published
  projection with the Rust validator and checked its mode and denylist.
- Verification: The projector completed with category `complete`; the direct
  absolute-path validator passed. The final file is mode 0644, its candidate is
  absent, its SHA-256 is
  `f008171f26b7a8ae6b08859e3cfef4f0c5bf88937c049dd66b6f868c9bbfd6f7`,
  and the explicit sensitive-value scan found no matches. An initial
  supplemental relative-path validator invocation found no file because Bazel
  runs from its runfiles directory; the corrected absolute-path invocation
  passed and did not regenerate or modify the projection.
- Evidence: The public projection contains only the closed contract. No
  protected evidence or hardware was accessed.
- Outcome: The evidence quorum supports promoting only `STR-006` to verified.
- Blocker or next safe action: Commit and push the evidence and result, then
  transition the checklist row and synchronize progress from this exact source
  commit.
