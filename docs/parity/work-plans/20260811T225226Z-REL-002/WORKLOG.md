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
