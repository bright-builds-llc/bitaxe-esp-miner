# Parity work log

## 2026-08-11T22:52:26Z | Attempt-004 plan checkpoint

- Source commit: `f83d38443192446fc58178385e4dbb3af46fae7d`.
- Actions: Resumed only `REL-002` after attempt-003 closure. Compared the
  firmware status sink, API-visible retained buffer, host predicate, and all
  ten attempt-003 snapshots. Defined the canonical marker correction and fresh
  attempt-004 paths without changing hardware effects.
- Verification: The selector reports no open plan. Branch, upstream,
  reference, predecessor closure, source ownership, and fresh-path
  preconditions pass. All ten private snapshots contain the canonical retained
  protocol-error status and none contain the UART-only spelling.
- Evidence: Source inspection plus aggregate private attempt-003 diagnosis.
  No new detector, credentials, hardware effect, or public evidence exists.
- Outcome: The attempt-004 plan is ready for its plan-only gate.
- Blocker or next safe action: Run the complete plan gate, commit and push the
  immutable plan/task, then implement the one retained-marker predicate fix.

## 2026-08-11T22:55:36Z | Immutable plan software gate

- Source commit: `f83d38443192446fc58178385e4dbb3af46fae7d` plus only the
  attempt-004 plan, work log, and active task contract.
- Actions: Admitted this as the sole open parity plan and ran the complete
  ordered plan-only gate.
- Verification: Cargo format, strict Clippy, all-target build, all-feature
  tests, Bright Builds, all 37 Bazel tests, parity/progress, redaction,
  reference, selector, task uniqueness, fresh-path absence, and diff checks
  pass. Progress remains 47.9%.
- Evidence: Software planning only. No new hardware use or public projection.
- Outcome: The immutable plan/task commit is eligible to push.
- Blocker or next safe action: Commit and push without amendment, then
  implement the canonical retained-marker predicate and focused regressions.

## 2026-08-11T22:57:58Z | Retained-marker implementation checkpoint

- Source commit: `3a5aa7fc` plus the focused implementation diff.
- Actions: Replaced the UART-only retained-log search with an exact complete-
  line match for the canonical API-visible OTA status. Updated the production-
  shaped device fixture and added positive, malformed, unrelated, and UART-
  only negative coverage.
- Verification: The canonical automation target passes all 112 tests. The
  ready transaction advances through probe and rollback with the canonical
  marker. The UART-only spelling holds the unchanged normal runtime through
  all ten checks, launches no probe or rollback child, returns
  `interruption_not_observed`, and publishes no evidence.
- Evidence: Software/process fixtures plus the already-closed aggregate
  attempt-003 diagnosis. No attempt-004 hardware use or public projection.
- Outcome: The retained evidence contract now matches the firmware producer
  without weakening identity, reset, recovery, or privacy gates.
- Blocker or next safe action: Run the complete software gate, review, commit
  and push, then build exact clean hardware artifacts before the one detector.

## 2026-08-11T23:00:14Z | Implementation software gate

- Source commit: `3a5aa7fc` plus the reviewed retained-marker implementation.
- Actions: Ran the complete ordered gate and verified immutable plan bytes,
  single-plan selector admission, fresh attempt-004 paths, clean reference,
  privacy, no public output, and unchanged parity/progress files.
- Verification: Cargo format, strict Clippy, all-target build, all-feature
  tests, Bright Builds, all 37 Bazel tests including 112 automation tests,
  parity/progress, redaction, reference, task/plan, cleanliness, and diff
  checks pass. Progress remains 47.9%.
- Evidence: Software regression evidence only; no attempt-004 device effect or
  public rollback projection exists.
- Outcome: The implementation is eligible for its exact commit and push.
- Blocker or next safe action: Commit and push, rebuild normal and probe images
  from the clean source, verify provenance and isolation, then run one detector.
