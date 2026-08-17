# Parity work log

## 2026-08-16T23:59:08Z | plan checkpoint

- Source commit: `b19a011dd6c1ea89e797864d1594ef5f05f8c7d0`
- Actions: Reconfirmed the clean selector and froze the corrected immutable
  STAT-001 software-only plan after the prior malformed checkpoint was
  recoverably reverted.
- Verification: `main` equals `origin/main`, no open plan exists, the pinned
  reference is clean, and STAT-001 remains the first actionable row.
- Evidence: Firmware owns configured-timeout classification; campaign source
  still contains the contradictory 2,000-ms consumer threshold.
- Outcome: Plan checkpoint ready for mandatory verification.
- Blocker or next safe action: Verify, commit, and push this exact plan/task
  checkpoint before reapplying the proven red/green correction.

## 2026-08-17T00:08:00Z | plan verification

- Plan SHA-256:
  `011496a29cd12b738b2cee81b525f87cbfda03ffd0aa75e24509f7281ad0ebee`
- Verification: `just verify-redaction`, `just verify-reference`,
  `just package`, the ordered Cargo format/clippy/build/test gates, the full
  Bright Builds check, `just test`, `just parity`, and
  `just parity-progress` passed.
- Evidence: All 46 Bazel tests passed; parity reported
  `validation_errors: none`; progress remained `verified=75 active=94
  total=99 deferred=5 completion=79.8%`.
- Outcome: The immutable plan and active-task checkpoint are ready to commit
  without changing STAT-001 status or progress.
- Blocker or next safe action: Push the plan checkpoint, reproduce the real
  campaign boundary failure, and apply only the planned consumer-policy fix.

## 2026-08-17T00:05:39Z | implementation and focused verification

- Plan commit: `f208c207faa6626c720a67ae236d6c89100cf459`
- Failure signal: The production-campaign regression rejected a producer-
  classified `feed_fresh` sample at 2,001 ms as `WatchdogUnresponsive`;
  `cargo test -p bitaxe-flash
  producer_classified_fresh_sample_after_legacy_boundary_is_accepted --
  --nocapture` failed with left `Some(WatchdogUnresponsive)`, right `None`.
- Correction: Removed only the campaign's `feed_age_millis > 2_000`
  reclassification. Feed-age presence remains mandatory; producer reason,
  participation, sequences, transport-window advancement, earliest-failure
  precedence, and closed value-free evidence remain enforced.
- Regression guard: Producer-classified `feed_fresh` is accepted at 2,001 and
  5,000 ms, producer-classified `feed_stale` is rejected at 5,001 ms, and a
  source guard rejects numeric campaign feed-age comparisons.
- Focused verification: `cargo test -p bitaxe-flash campaign::network` passed
  107 tests; `cargo test -p bitaxe-core runtime_health` passed 16 tests;
  `bazel test //tools/flash:tests //crates/bitaxe-core:tests` passed both
  targets.
- Outcome: The root-cause software correction is complete with no firmware,
  schema, hardware, checklist, or progress change.
- Blocker or next safe action: Run the complete package, privacy, reference,
  mandatory ordered, immutable-plan, and diff gates before committing and
  pushing the implementation.

## 2026-08-17T00:10:11Z | implementation verification

- Verification: `just verify-redaction`, `just verify-reference`, and
  `just package` passed. The ordered Cargo format, clippy-with-warnings-denied,
  all-target build, and all-feature test gates passed. The Bright Builds check
  passed with zero findings, and `just test` passed all 46 Bazel tests.
- Parity: The initial renderer process exhausted a transient host resource
  after the full report and successful tests. The single bounded
  `just parity && just parity-progress` retry passed with
  `validation_errors: none` and unchanged progress at `verified=75 active=94
  total=99 deferred=5 completion=79.8%`.
- Plan integrity: PLAN SHA-256 remains
  `011496a29cd12b738b2cee81b525f87cbfda03ffd0aa75e24509f7281ad0ebee`.
- Outcome: Implementation satisfies every software acceptance criterion and
  is ready to commit as the source commit.
- Blocker or next safe action: Commit and push the implementation, then create
  a `CLOSURE.md` recording that STAT-001 remains implemented and live quorum
  evidence is still required.
