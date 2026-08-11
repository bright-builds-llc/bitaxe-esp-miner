# Parity work log

## 2026-08-11T21:59:04Z | Corrected attempt-002 plan checkpoint

- Source commit: `96d86057ff5af311a1cc6b4cea8800f3a561c8e3`
- Actions: Resumed only `REL-002` after the superseded post-FIN plan. Bound the
  new immutable design to a flush-without-FIN, 100-millisecond delivery grace,
  one forced reset, and observed local close, with the still-unused
  wrapper-002 and attempt-002 paths.
- Verification: Direct Node 24 traces deliver the exact test payload before a
  reset and make the peer observe reset/close without EOF. Branch, upstream,
  reference, selector, old-plan closure, and unused path preconditions pass.
- Evidence: Synthetic local TCP traces only. No detector, credentials, device
  effect, attempt root, or public projection exists.
- Outcome: The corrected immutable plan is ready for its plan-only gate.
- Blocker or next safe action: Archive the superseded task, run the complete
  plan gate, commit and push only the plan/task artifacts, then implement.

## 2026-08-11T22:02:35Z | Corrected immutable plan gate

- Source commit: `96d86057ff5af311a1cc6b4cea8800f3a561c8e3` plus planning and
  related task-history artifacts only.
- Actions: Archived the superseded task, admitted this plan as the sole open
  parity plan, and ran the complete ordered plan-only software gate.
- Verification: Cargo format, strict Clippy, all-target build, all-feature
  tests, Bright Builds, all 37 Bazel tests, parity/progress, redaction, pinned
  reference, selector, task uniqueness, fresh-path absence, and diff checks
  pass. Progress remains 45 of 94 active rows verified (47.9%).
- Evidence: Software and synthetic TCP evidence only. No hardware or public
  rollback projection exists.
- Outcome: The corrected plan/task commit is eligible to push.
- Blocker or next safe action: Commit and push these bytes without amendment,
  then implement the reset-before-FIN transport and focused regressions.
