# Parity work log

## 2026-08-11T21:47:37Z | Attempt-002 plan checkpoint

- Source commit: `46f34ac101a15c2fabae3417e119cb0118afff0a`
- Actions: Resumed deterministic selection after the attempt-001 closure and
  selected only `REL-002`. Defined a bounded force-close transport fix, a
  non-cooperative real-child regression, and fresh wrapper-002/attempt-002
  hardware paths.
- Verification: The branch and pinned reference are clean, `origin/main` is
  synchronized, the selector reports no open plan, Node 24 exposes
  `Socket.resetAndDestroy`, and the old attempt-001 paths are not reused.
- Evidence: Planning and prior private attempt diagnostics only. No new
  detector, credential access, device effect, or public projection occurred.
- Outcome: The immutable attempt-002 contract is ready for its plan gate.
- Blocker or next safe action: Run the complete plan gate, commit and push the
  plan/task alone, then implement the socket closure boundary.

## 2026-08-11T21:52:00Z | Immutable plan software gate

- Source commit: `46f34ac101a15c2fabae3417e119cb0118afff0a` plus only the new
  plan, work log, and active task contract.
- Actions: Ran the ordered Rust, Bright Builds, Bazel, parity, progress,
  redaction, reference, selector, uniqueness, cleanliness, privacy, and diff
  checks required before the immutable plan commit. The first selector pass
  required the explicit link to the closed attempt-001 plan; that lineage was
  added before commit and the admission checks were rerun.
- Verification: Cargo format, strict Clippy, all-target build, all-feature
  tests, Bright Builds, all 37 Bazel tests, parity/progress, redaction, and
  pinned-reference verification pass. Progress remains 45 of 94 active rows
  verified (47.9%).
- Evidence: Software gate output only. The public rollback projection remains
  absent and both fresh private paths remain unused.
- Outcome: The attempt-002 plan and task are eligible for their immutable
  commit and push.
- Blocker or next safe action: Commit and push these planning artifacts without
  amendment, then implement and test the forced socket teardown.

## 2026-08-11T21:57:00Z | Post-FIN design disproved

- Source commit: `a736e315` with an unchanged immutable plan.
- Actions: Implemented the planned post-FIN reset experimentally, ran the real
  half-open child regression, stopped its bounded hang, and isolated the Node
  TCP event sequence in a minimal local trace. Removed every experimental code
  and test change after the design was disproved.
- Verification: Both the direct trace and the real test show that FIN followed
  by a delayed reset leaves an `allowHalfOpen` server's writable half live.
  A separate trace shows that writing the strict prefix without FIN and then
  resetting delivers the bytes and produces peer reset/close.
- Evidence: Local synthetic TCP behavior only. No detector, credentials,
  hardware effect, private attempt path, or public projection was used.
- Outcome: The frozen design is superseded before implementation or hardware.
  `REL-002` remains `implemented` and attempt-002 remains unconsumed.
- Blocker or next safe action: Close this plan, then create a fresh immutable
  continuation using flush-without-FIN followed by immediate forced reset and
  observed local close.
