# Parity work log

## 2026-08-04T15:00:00Z | selection and plan

- Source commit: `fbf316a403c1c0da3e87a844705dd0fa0f8fbe7a`.
- Actions: Ran deterministic selection and traced the pinned work-creation,
  ASIC-result, and power-management loops into the Rust production-session,
  ASIC-worker, operator-observation, safety-supervisor, and startup owners.
- Verification: The branch is clean and synchronized, no plan is open, work
  dispatch/result correlation already have strong pure coverage, and cadence
  arithmetic plus cross-owner startup/ownership proof remain fragmented.
- Evidence: Pinned reference task sources and current Rust owner boundaries.
- Outcome: `SYS-005` is bounded to an explicit software orchestration contract;
  no hardware or effectful verification is authorized by this plan.
- Blocker or next safe action: Commit the immutable plan and active task before
  implementation.

## 2026-08-04T15:18:00Z | implementation and focused verification

- Source commit: `86214262`.
- Actions: Added a pure checked periodic-deadline contract, wired the operator
  observation, safety-supervisor, and production reread cadences to it, made
  deadline overflow halt the observation producer so stale truth fails closed,
  and strengthened the owner/startup plus result-before-submit seams.
- Verification: Four deadline reducer tests, the focused production lifecycle
  test, source-ownership target, and real ESP32-S3 firmware build passed.
- Evidence: Dispatch-before-poll and bounded timeout behavior remain in the
  bridge reducer; stale-generation and fail-closed shutdown ordering remain in
  the production-session suite; new seams bind those behaviors to one bounded
  firmware owner graph.
- Outcome: The implementation is ready for mandatory repository-wide gates.
- Blocker or next safe action: Run the ordered gate, review the diff for a
  simpler equivalent, and record the bounded parity result.

## 2026-08-04T15:34:00Z | simplification and starvation fix

- Source commit: `86214262`.
- Actions: The explicit simplification pass found that the relative production
  `recv_timeout` restarted after every inbox message, so continuous transport
  or ASIC traffic could postpone the authoritative readiness reread without a
  bound. Replaced it with the same absolute periodic deadline used by the other
  owners and advanced the safety supervisor from scheduled boundaries rather
  than sleep-after-work drift.
- Verification: Focused deadline, lifecycle, and source-ownership tests passed
  again; the real ESP32-S3 firmware rebuilt successfully.
- Evidence: The production owner now checks an absolute due boundary after
  every received message and on timeout, while category wakeups can satisfy the
  same read without duplication.
- Outcome: The implementation fixes the root scheduling gap instead of merely
  documenting the existing owners.
- Blocker or next safe action: Rerun the complete mandatory gate against the
  final implementation diff.
