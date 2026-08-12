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
