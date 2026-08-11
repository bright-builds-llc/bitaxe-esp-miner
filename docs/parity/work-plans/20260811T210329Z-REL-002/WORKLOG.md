# Parity work log

## 2026-08-11T21:07:00Z | Plan and implementation checkpoint

- Source commit: `a6c47ce55da9404f376780cf028b6d20d4e38628`
- Actions: Selected only `REL-002`, committed and pushed the immutable plan and
  active one-attempt hardware contract, and began the isolated rollback-probe
  and typed host-transaction implementation.
- Verification: Plan gate passed Cargo format, clippy, build, and tests; Bright
  Builds; all Bazel tests; parity; progress; redaction; pinned reference;
  selector open-plan validation; task uniqueness; and diff checks.
- Evidence: Immutable plan
  `docs/parity/work-plans/20260811T210329Z-REL-002/PLAN.md` at commit
  `a6c47ce5`.
- Outcome: Planning is complete; no device effect has occurred.
- Blocker or next safe action: Implement and test the probe build, interrupted
  upload, two device-session transactions, recovery, and closed projection.

## 2026-08-11T21:17:00Z | Implementation checkpoint

- Source commit: `a6c47ce55da9404f376780cf028b6d20d4e38628`
- Actions: Added a build-isolated pending-validation firmware probe, optional
  admitted postcondition ELF identity in the canonical device session, one
  raw bounded interrupted-upload transport, the two-session rollback capture,
  exact-package recovery, closed evidence contract, CLI/Just/Bazel wiring, and
  validator.
- Verification: Focused Rust and Bazel tests pass. The real TCP child-process
  regression proves declared-versus-transmitted body behavior; orchestration
  tests cover success, every non-ready device-session category, missing and
  malformed projections, primary-failure precedence, recovery, private modes,
  and sensitive-value exclusion. The normal/probe ELF isolation target passes.
- Evidence: No device effect has occurred and no public hardware evidence has
  been produced.
- Outcome: Implementation is ready for the mandatory pre-commit software gate.
- Blocker or next safe action: Run the full ordered gates, review the diff,
  commit and push the exact implementation, then use the one detector gate.

## 2026-08-11T21:28:00Z | Implementation software gate

- Source commit: `a6c47ce55da9404f376780cf028b6d20d4e38628` plus the reviewed
  implementation diff.
- Actions: Ran the complete ordered gate, continuation-aware selector,
  immutable-plan, generated-contract, task-uniqueness, tracked-scratch,
  public-output absence, reference-cleanliness, privacy, and diff checks.
- Verification: Cargo format, strict Clippy, all-target build, all-feature
  tests, Bright Builds, all Bazel tests, parity/progress, redaction, and
  reference verification pass. Progress remains 45 of 94 verified (47.9%).
  The selector resumes only this `REL-002` plan and its committed bytes remain
  unchanged.
- Evidence: Software and private synthetic test evidence only. No detector,
  credential access, device effect, or public projection has occurred.
- Outcome: The implementation is eligible for its immutable commit and push.
- Blocker or next safe action: Commit and push, rebuild the exact clean normal
  package and rollback probe, then run the one protected detector.

## 2026-08-11T21:43:49Z | Attempt-001 terminal closure

- Source commit: `89fd198cf262f4755a5846206da0e122985f92c6`
- Actions: Built and admitted the exact clean normal package and isolated
  rollback probe, ran the sole detector, and ran the sole conditional hardware
  attempt. The initial exact-package flash completed and the transaction sent
  its one bounded partial OTA request.
- Verification: Ten same-origin checks retained the exact normal application,
  unchanged boot session and ordinal, and `factory` partition, but none found
  the required retained protocol-abort marker. The typed result is
  `interruption_not_observed`; probe and rollback sessions did not start,
  recovery was unnecessary, no owned process remains, all private directories
  are mode `0700`, all private files are mode `0600`, and no public projection
  exists.
- Evidence: Private attempt diagnostics remain under the ignored protected
  attempt root. `CLOSURE.md` records only closed aggregate diagnosis and
  non-claims; no `RESULT.md` or public evidence was created.
- Outcome: Attempt-001 is consumed and `REL-002` remains `implemented`.
- Blocker or next safe action: A fresh continuation must first force and await
  full local socket teardown after the admitted prefix and add a child-process
  regression whose peer stays open. Do not retry attempt-001.
