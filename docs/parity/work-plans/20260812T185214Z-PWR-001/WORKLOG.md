# Parity work log

## 2026-08-12 18:52 UTC | selection and immutable plan

- Source commit: `ade14353358c72809cd32e69341b3a676a22b02e`
- Actions: Selected `PWR-001` as the first actionable row after temporarily
  unavailable `API-009`; audited the pinned reference, reset implementation,
  deterministic preparation order, sealed attempt-007 facts, ASIC-002 public
  projection, and reset-path Git compatibility; created the immutable plan and
  active task binding.
- Verification: Clean synchronized `main`; no open parity plan; selector order
  `API-009`, then `PWR-001`; reference commit pinned; accepted-attempt reset
  implementation paths unchanged; existing projection independently valid at
  its prior checkpoint.
- Evidence: `docs/parity/evidence/asic002-initialization/asic-initialization-projection.json`
  and sealed ignored attempt `scratch/ultra205-accepted-pool-share/attempt-007`.
- Outcome: A row-specific typed projection can reuse the sealed hardware
  regression without another device effect.
- Blocker or next safe action: Commit and push the immutable plan/task
  checkpoint, then implement only the closed projection and validators.

## 2026-08-12 18:56 UTC | pre-freeze selector regression

- Source commit: `ade14353358c72809cd32e69341b3a676a22b02e`
- Actions: Ran the selector after creating the new plan and reproduced a false
  multi-row conflict. Inspected the functional core and confirmed it checks
  row diversity before retiring a latest terminal-closed continuation lineage.
- Verification: All eight API-009 continuation plans have valid closures; the
  clean selector reported no open plan immediately before PWR-001 creation;
  adding only the PWR-001 plan triggered the false conflict.
- Evidence: Exact error `multiple open parity plans span rows`; source boundary
  `tools/parity/src/parity_work.rs::reconcile_open_plans`.
- Outcome: Added the narrow reconciliation fix and behavior regression to the
  not-yet-frozen PWR-001 plan.
- Blocker or next safe action: Freeze, verify, commit, and push this expanded
  immutable plan before changing selector or evidence code.

## 2026-08-12 18:59 UTC | immutable plan checkpoint

- Source commit: `ade14353358c72809cd32e69341b3a676a22b02e`
- Actions: Froze the PWR-001 plan with SHA-256
  `3b3fb9ca3ae38156b006863a8b3ffded8ebfea43995fa3e3ef9cbec8e3911a79`
  and verified the unique active task and absent final projection.
- Verification: `cargo fmt --all`, strict Cargo clippy, all-target Cargo build,
  all-feature Cargo tests, Bright Builds (zero findings), all 41 Bazel tests,
  parity report/progress, reference cleanliness, redaction, and diff checks
  passed. The planned selector regression remains red by construction until
  implementation.
- Evidence: Immutable plan and this worklog under
  `docs/parity/work-plans/20260812T185214Z-PWR-001/`.
- Outcome: Plan/task checkpoint is ready to commit and push before source work.
- Blocker or next safe action: Commit and push, then fix the reproduced
  selector reconciliation defect and implement the typed projection.

## 2026-08-12 19:19 UTC | implementation and focused verification

- Source commit: `9a453443` (immutable plan checkpoint)
- Actions: Added the typed `bitaxe-asic-reset-evidence-v1` Rust contract,
  independent validator, generated TypeScript binding, closed projector,
  command surface, and behavior regressions. Changed selector reconciliation
  to retire the latest terminal-closed lineage for each row before detecting
  genuinely simultaneous active rows. Tightened semantic admission to bind
  the literal 100 ms low/high action, both GPIO delays, the fail-closed
  hold-low decision, safe shutdown, and the production reset-and-detect path.
- Verification: The focused automation aggregate passed, including the real
  child-validator boundary and all evidence-withholding cases; the exact
  selector regression passed; all three Rust reset-contract tests passed;
  generated bindings matched the Rust-owned contract; Bright Builds reported
  zero findings; diff whitespace checks passed.
- Evidence: Source and tests only. No public projection has been produced and
  no hardware interaction occurred.
- Outcome: The implementation is ready for the full mandatory gate sequence
  and a clean pushed source checkpoint.
- Blocker or next safe action: Run all mandatory gates, review the complete
  diff, then commit and push before invoking the projector.

## 2026-08-12 19:24 UTC | clean implementation gates

- Source commit: `9a453443` (immutable plan checkpoint)
- Actions: Completed the required pre-commit sequence and an explicit
  simplification/diff review. The projector reuses the validated ASIC-002
  boundary and existing process abstraction; it adds no device or duplicate
  hardware-session machinery.
- Verification: `cargo fmt --all`, strict all-target/all-feature Clippy,
  all-target/all-feature Cargo build, all-feature Cargo tests, Bright Builds
  with zero findings, and all 41 Bazel tests passed. The initial combined
  parity tail encountered transient macOS `os error 35` after rendering the
  report; isolated `just parity` and `just parity-progress` then passed with
  no validation errors at 59/94 verified. Redaction, pinned-reference,
  generated-contract, and diff checks also passed.
- Evidence: Pre-publication verification only; no final PWR-001 projection
  exists and no hardware interaction occurred.
- Outcome: The complete implementation is ready for its clean pushed source
  checkpoint.
- Blocker or next safe action: Commit and push the implementation, then run
  the public projector from that exact clean commit.

## 2026-08-12 19:29 UTC | public projection and independent validation

- Source commit: `9cd2ec3741c09e3a3636c8358c0203de183805bb`
- Actions: Ran the repo-owned PWR-001 projector against the committed ASIC-002
  projection and accepted-attempt source commit. No detector, device, network,
  credential, protected campaign, or hardware surface was accessed.
- Verification: The projector returned `succeeded/complete`; the independent
  Rust validator accepted the final artifact; the file is mode `0644`, the
  private candidate is absent, and the public projection SHA-256 is
  `11bb816e6f6e2393b796b13c49ae7db5d181f719dc94898ca00e17ce384d469b`.
- Evidence: `docs/parity/evidence/pwr001-asic-reset/asic-reset-projection.json`
  and `RESULT.md` in this plan directory.
- Outcome: The complete closed quorum supports promoting only `PWR-001` to
  `verified` with `unit,workflow,hardware-smoke,hardware-regression` evidence.
- Blocker or next safe action: Commit and push the evidence checkpoint, then
  perform the audited checklist transition and synchronize progress.

## 2026-08-12 19:30 UTC | repository redaction coverage

- Source commit: `9cd2ec3741c09e3a3636c8358c0203de183805bb`
- Actions: Added the new reset schema to the repository semantic-evidence
  scanner. Its focused regression first exposed the broad `ip` key rule
  matching the safe `exactly_one_chip_detected_after_reset` field; admitted
  only that exact closed boolean and retained rejection for operational
  fields.
- Verification: The first focused run failed on that false positive. After
  the exact-key correction, the full automation test passed and repository
  redaction checked all 14 semantic artifacts. The ordered Cargo checks,
  Bright Builds with zero findings, all 41 Bazel tests, isolated parity and
  progress, reference cleanliness, and diff checks passed. The combined
  parity tail again encountered transient macOS `os error 35` only after
  rendering; the immediate isolated rerun reported no validation errors.
- Evidence: The committed PWR-001 projection is now included in the global
  semantic redaction scan rather than relying solely on projector tests.
- Outcome: Evidence, result, and redaction coverage are ready for the pushed
  source checkpoint required by the audited transition.
- Blocker or next safe action: Commit and push this checkpoint, retain its
  full hash as `SOURCE_COMMIT`, then transition only `PWR-001`.

## 2026-08-12 19:33 UTC | verified transition and progress synchronization

- Source commit: `312b512d437725b3de80e01d52f8aa9f6aef6724`
- Actions: Transitioned only `PWR-001` with transition ID
  `20260812T192855Z-PWR-001`, synchronized deterministic progress from the
  pushed evidence checkpoint, replaced stale pre-promotion checklist notes,
  and prepared the completed task for archival.
- Verification: The typed transition accepted the immutable plan and result,
  produced checklist digest
  `5614b1820d2b8ad228a1829f70b32a2a1c5698ea49752d579633a6bbbde8732b`,
  and appended progress at 60/94 verified (63.8%).
- Evidence: `docs/parity/checklist-transitions/20260812T192855Z-PWR-001.json`,
  updated checklist/progress/README, the typed result, and the closed public
  projection.
- Outcome: `PWR-001` is verified; the active task is complete and may be
  archived.
- Blocker or next safe action: Run the final ordered repository gates, review
  the finalization diff, commit, fetch, and push without force.

## 2026-08-12 19:36 UTC | transition metadata correction

- Source commit: `312b512d437725b3de80e01d52f8aa9f6aef6724`
- Actions: The final parity gate rejected the uncommitted first transition
  because the Rust-owned target lacked required Markdown code spans; a manual
  follow-up notes edit also correctly failed the transition-ledger digest.
  Restored the committed predecessor, removed the uncommitted receipt and
  progress record, and replayed one atomic transition with code-spanned targets
  and the complete source-bound notes.
- Verification: Final transition `20260812T193339Z-PWR-001` validates against
  its predecessor and result with checklist SHA-256
  `7f848fcb8c33f8c6a7110b8d2789457fa889a62e7625b0e7dedd69bd761d157c`;
  synchronized progress remains 60/94 verified (63.8%). The rejected first
  transition never entered Git history.
- Evidence: `docs/parity/checklist-transitions/20260812T193339Z-PWR-001.json`
  and the ledger-authored checklist/progress/README updates.
- Outcome: Checklist metadata, notes, transition ledger, and progress are now
  one atomic valid finalization.
- Blocker or next safe action: Rerun the final ordered gates, then commit,
  fetch, and push the validated finalization.

## 2026-08-12 19:39 UTC | final verified gates

- Source commit: `312b512d437725b3de80e01d52f8aa9f6aef6724`
- Actions: Ran the complete finalization sequence against the corrected atomic
  transition, synchronized progress, archived task, and final checklist notes.
- Verification: `cargo fmt --all`, strict all-target/all-feature Clippy,
  all-target/all-feature Cargo build, all-feature Cargo tests, Bright Builds
  with zero findings, all 41 Bazel tests, `just parity` with no validation
  errors, and `just parity-progress` at 60/94 verified (63.8%) all passed.
- Evidence: Final transition `20260812T193339Z-PWR-001`, closed projection,
  typed result, deterministic progress record, README summary, and archived
  task.
- Outcome: PWR-001 finalization is complete and ready to commit and push.
- Blocker or next safe action: Run final reference/redaction/diff/selector
  checks, commit the finalization, fetch, and push without force.
