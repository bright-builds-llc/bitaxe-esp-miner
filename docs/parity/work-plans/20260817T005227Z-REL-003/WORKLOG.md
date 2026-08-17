# Parity work log

## 2026-08-17T00:52:27Z | plan checkpoint

- Source commit: `4bf594cf9f0cabd391881a6cee0e4ab0024a9151`
- Actions: Selected REL-003 after four concrete dependency/retry skips and
  froze one typed large-erase recovery attempt.
- Verification: Worktree/reference are clean, `main` equals `origin/main`, the
  selector has no open plan, and wrapper/attempt/projection paths are absent.
- Evidence: Existing accepted artifacts now satisfy release-gate, provenance,
  package workflow, failed/interrupted update, rollback, and recovery terms;
  large erase recovery remains the sole explicit REL-003 gap.
- Outcome: Immutable plan checkpoint ready for repository gates.
- Blocker or next safe action: Verify, commit, and push this plan/task before
  implementing the task-bound erase and evidence workflow.

## 2026-08-17T00:56:00Z | plan digest

- Source commit: `4bf594cf9f0cabd391881a6cee0e4ab0024a9151`
- Actions: Bound the checkpoint to immutable PLAN SHA-256
  `042e6e11fa69c44c4cde59c680755ce757193de74cb5a7910d763af819b7a6df`.
- Verification: The canonical selector reports this exact REL-003 plan as its
  only open plan and `git diff --check` passes.
- Evidence: No implementation file or hardware state changed; the active task
  records the exact erase, restoration, recovery, privacy, and stop contract.
- Outcome: Plan digest recorded before pre-commit verification.
- Blocker or next safe action: Run all plan-checkpoint gates and push without
  amending or rewriting the plan.

## 2026-08-17T01:00:00Z | release-gate red baseline

- Source commit: `4bf594cf9f0cabd391881a6cee0e4ab0024a9151`
- Actions: Built the current package and exercised its manifest through the
  canonical release gate before any implementation.
- Verification: Redaction, reference, and package commands pass. The release
  gate correctly fails because the generated partition-table artifact path is
  an absolute workspace path rather than the required canonical repository-
  relative `firmware/bitaxe/partitions-ultra205.csv`.
- Evidence: The other five required artifact identities remain canonical; the
  failure is isolated to package-manifest path normalization and reproduces
  before the planned large-erase workflow exists.
- Outcome: Red baseline captured. This is a necessary minimal REL-003 package
  correction discovered by the plan's acceptance gate, not hardware evidence.
- Blocker or next safe action: Finish repository-mandatory plan checks, commit
  the immutable plan, then add a regression and correct path normalization
  before implementing or running the destructive workflow.

## 2026-08-17T00:59:59Z | plan verification

- Source commit: `4bf594cf9f0cabd391881a6cee0e4ab0024a9151`
- Actions: Completed every repository-mandatory plan-checkpoint gate after
  preserving the expected release-gate red baseline.
- Verification: Ordered Cargo format/clippy/build/test, Bright Builds,
  `just verify-redaction`, `just verify-reference`, `just package`, all 46
  Bazel tests, parity, and progress pass. The initial parity renderer hit the
  known transient host resource boundary; its one bounded retry passed.
- Evidence: Parity reports `validation_errors: none`; progress remains
  `verified=75 active=94 total=99 deferred=5 completion=79.8%`; PLAN SHA-256
  remains `042e6e11fa69c44c4cde59c680755ce757193de74cb5a7910d763af819b7a6df`.
- Outcome: Immutable plan/task checkpoint is ready to commit and push.
- Blocker or next safe action: Push this checkpoint, then correct the package
  artifact path red baseline before implementing the task-bound erase flow.

## 2026-08-17T01:27:28Z | release recovery implementation

- Source commit: `3a76d92ca5bb1514bf7e5fa687d23d7fd701e626`
- Actions: Corrected package-manifest partition-table path normalization and
  added the plan-bound `rel003-large-erase` command, supervised erase effect,
  exact factory/Wi-Fi restore, qualified runtime proof, conditional factory-
  precompletion recovery, typed contract, independent validator, generated
  TypeScript surface, Bazel/runfiles wiring, and thin `just` interface.
- Verification: The package-path regression failed red on the absolute host
  path and passes green with the canonical repository-relative path. Focused
  Cargo and Bazel targets pass for xtask, device-session, flash, contracts,
  generated contracts, and the independent validator.
- Evidence: Tests prove one fixed ESP32-S3 erase vector, one successful erase
  plus factory/NVS restore, exact safe-state projection, one recovery attempt
  after factory precompletion failure, no reflash after completed transfer or
  Wi-Fi-seed/runtime-proof failure, closed private failure facts, mode `0644`
  public output, and plan/task/path/source binding. PLAN SHA-256 remains
  `042e6e11fa69c44c4cde59c680755ce757193de74cb5a7910d763af819b7a6df`.
- Outcome: Software implementation is complete; no hardware command has run.
- Blocker or next safe action: Run every mandatory software/package/privacy
  gate, commit and push the implementation, then rebuild the clean package and
  require the release gate to turn green before detector access.

## 2026-08-17T01:36:51Z | pre-hardware implementation verification

- Source commit: pending exact implementation commit.
- Actions: Ran every focused and mandatory software, firmware, package,
  privacy, reference, immutable-plan, generated-contract, and parity-invariance
  gate on the completed implementation tree.
- Verification: Ordered Cargo format/clippy/build/test, Bright Builds,
  `just verify-redaction`, `just verify-reference`, `just package`, and all 46
  Bazel tests pass. The dirty-tree release gate now fails only on its required
  clean-provenance check; the original partition-table path error is absent.
  The initial parity renderer hit the known transient host resource boundary;
  its one bounded retry passed.
- Evidence: Parity reports `validation_errors: none`; progress remains
  `verified=75 active=94 total=99 deferred=5 completion=79.8%`; PLAN SHA-256
  remains `042e6e11fa69c44c4cde59c680755ce757193de74cb5a7910d763af819b7a6df`.
- Outcome: Exact implementation is ready to commit and push. The clean package
  release gate remains a mandatory post-commit pre-hardware check.
- Blocker or next safe action: Push this implementation, rebuild its clean
  package, require `release_gate: passed`, then run only the frozen detector.

## 2026-08-17T01:49:29Z | attempt-001 complete

- Source commit: `70493a51249df2f82eb5b046be7dc95b137c7e97`
- Actions: Rebuilt the exact clean package, passed the manifest release gate,
  ran the one frozen detector, verified protected modes/fresh paths/nonempty
  opaque Wi-Fi input, and ran the sole large-erase recovery capture.
- Verification: The effectful command and independent validator exited zero.
  The public projection revalidates independently and has SHA-256
  `6c712fd14e2dfc666a78602855efa825b9c104178efeee889e05b6f6b76f5b12`.
- Evidence: One board 205 was admitted; full erase, exact factory and Wi-Fi/
  default-NVS restore, `mineonboot=false`, trusted runtime identity, SPIFFS
  readiness, passive safe state, cleanup, protected modes, and redaction pass.
  No recovery reflash ran and no private operational value was published.
- Outcome: `complete`; the new large-erase evidence plus accepted Phase 18/19
  and verified REL-002 artifacts satisfy every REL-003 release verifier term.
- Blocker or next safe action: Commit this evidence as `SOURCE_COMMIT`, then
  transition only REL-003 to verified, synchronize progress, archive the task,
  run final gates, and push.

## 2026-08-17T01:58:10Z | verified transition and archival

- Source commit: `fa8f06a73ed708a6036e91e61fde7516a6918267`
- Actions: Transitioned only REL-003 under
  `20260817T015623Z-REL-003`, synchronized progress from the exact evidence
  source commit, and moved the completed native task block to
  `TASKS.archive.md`.
- Verification: The transition validator accepted the plan/result hashes,
  complete evidence vocabulary, and positive rollback, recovery, large erase,
  failed update, interrupted-update, release-gate, provenance, and package-
  workflow terms.
- Evidence: REL-003 is `verified` with
  `workflow,api-compare,hardware-smoke,hardware-regression,release-gate`;
  progress appended at `verified=76 active=94 completion=80.9%`; the stable
  task ID exists exactly once across active and archive trackers.
- Outcome: REL-003 verified finalization is ready for the mandatory closed-tree
  gate and push.
- Blocker or next safe action: Run every final gate in order, review the diff,
  commit and push finalization, then confirm a clean selector with no open plan.
