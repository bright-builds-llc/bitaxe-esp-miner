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

## 2026-08-11T22:05:26Z | Reset-before-FIN implementation checkpoint

- Source commit: `64015d94` plus the focused implementation diff.
- Actions: Replaced write-side FIN completion with one exact write, a
  100-millisecond fully open delivery grace, one forced TCP reset, and success
  only after local close. Replaced the cooperative child with an
  `allowHalfOpen` reset observer and made rollback fixtures assert one reset.
- Verification: The real child receives the exact 4,096-byte prefix beneath
  the declared image length while the helper remains pending, observes no EOF,
  then observes reset/close. Validation, refused connection, and timeout
  cleanup tests pass. The full automation target passes all 109 tests.
- Evidence: Synthetic process and socket evidence only; no hardware effect or
  public projection exists.
- Outcome: The root transport defect is implemented with a direct regression.
- Blocker or next safe action: Run the complete ordered software gate, review
  the full diff, commit and push, then build exact clean hardware artifacts.

## 2026-08-11T22:07:41Z | Implementation software gate

- Source commit: `64015d94` plus the reviewed transport/test diff.
- Actions: Ran the complete ordered implementation gate and verified the plan
  remained byte-identical, the selector resumes only this plan, private paths
  remain unused, and no parity/progress/public-evidence file changed.
- Verification: Cargo format, strict Clippy, all-target build, all-feature
  tests, Bright Builds, all 37 Bazel tests, parity/progress, redaction,
  reference, task/plan, cleanliness, sensitive-output, and diff checks pass.
- Evidence: Software regression evidence only. The hardware projection remains
  absent and progress remains 45 of 94 active rows verified (47.9%).
- Outcome: The implementation is eligible for its exact commit and push.
- Blocker or next safe action: Commit and push, rebuild normal and probe images
  from that clean source, verify provenance/isolation, then run one detector.

## 2026-08-11T22:26:06Z | Attempt-002 terminal closure

- Source commit: `9482dc49f9a5bfa2c09a22432703905b89ee79fe`.
- Actions: Built and admitted the exact normal and isolated probe artifacts,
  ran the sole detector, and ran the sole conditional attempt-002. The initial
  factory flash completed and the transaction later used its allowed exact-
  package recovery flash.
- Verification: The typed primary category is `evidence_invalid`. Initial
  monitor evidence retained one stable boot session, one runtime origin, and
  trusted passive runtime attestation but timed out after trusted output; the
  baseline HTTP artifact was not created. Recovery completed without secondary
  failure, private modes pass, no owned process remains, and public evidence is
  absent.
- Evidence: Private diagnostics remain in ignored wrapper-002/attempt-002.
  `CLOSURE.md` records only aggregate facts and non-claims.
- Outcome: Attempt-002 is consumed before the interrupted upload, probe, or
  rollback stages. `REL-002` remains `implemented`.
- Blocker or next safe action: Decouple initial monitor admission from the
  long device-session timeout, add bounded typed baseline HTTP readiness, push
  a fresh implementation, and use new wrapper/attempt-003 paths only.
