# Parity work log

## 2026-08-12 19:39 UTC | selection and immutable plan

- Source commit: `24843096bf0750e481efe9a49b877c83a7fae8a1`
- Actions: Selected `PWR-002` as the first actionable row after temporarily
  unavailable `API-009`; audited the pinned reference, production preparation
  and rollback transactions, safety and ASIC adapters, accepted campaign,
  ASIC-002 projection, profile constants, and source-compatibility boundary;
  created the immutable plan and active task binding.
- Verification: Clean synchronized `main`; no open parity plan; selector order
  `API-009`, then `PWR-002`; reference commit pinned; accepted evidence proves
  the complete initialization terminal and successful downstream work.
- Evidence: `docs/parity/evidence/asic002-initialization/asic-initialization-projection.json`
  and its sealed accepted-attempt lineage.
- Outcome: A row-specific typed projection can reuse accepted hardware
  regression evidence without another device effect.
- Blocker or next safe action: Freeze, verify, commit, and push the immutable
  plan/task checkpoint, then implement only the closed projection and
  validators.

## 2026-08-12 19:45 UTC | immutable plan checkpoint

- Source commit: `24843096bf0750e481efe9a49b877c83a7fae8a1`
- Actions: Froze the PWR-002 plan with SHA-256
  `7ff2ca77e4967f2f823033ef68cfab264863fc20caad841a1ac30c8ecf5d14ff`
  and verified the unique active task, sole open plan, and absent final
  projection.
- Verification: `cargo fmt --all`, strict Cargo Clippy, all-target Cargo build,
  all-feature Cargo tests, Bright Builds with zero findings, all 41 Bazel test
  targets, parity report/progress, reference cleanliness, redaction, and diff
  checks passed.
- Evidence: Immutable plan and this worklog under
  `docs/parity/work-plans/20260812T193941Z-PWR-002/`.
- Outcome: The plan/task checkpoint is ready to commit and push before source
  work.
- Blocker or next safe action: Commit and push, then implement the typed closed
  projection without device interaction.

## 2026-08-12 20:08 UTC | implementation and focused verification

- Source commit: `b449a545` (immutable plan checkpoint)
- Actions: Added the Rust-owned
  `bitaxe-asic-power-initialization-evidence-v1` contract, independent
  validator, generated TypeScript binding, closed projector, command surface,
  redaction registration, and behavior regressions. The projector admits six
  byte-identical modules and requires unique matching profile, safety-routing,
  and active-low-enable semantics in both the accepted and current versions of
  three later-changed modules. Moved the existing contract-bundle tests from
  the saturated library root into a dedicated test module.
- Verification: All 52 contract-library tests and the full automation suite
  passed. The automation suite covers complete publication, every incomplete,
  validator, source-drift, semantic-drift, dirty-worktree, launch, task/plan,
  real-child, typed-failure, invocation, and redaction boundary. Bright Builds
  reports zero findings and all generated contract copies match.
- Evidence: Source and tests only. No public PWR-002 projection exists and no
  hardware, credential, network, USB, serial, or private-attempt input was
  accessed.
- Outcome: The implementation is ready for the full mandatory gate sequence
  and a clean pushed source checkpoint.
- Blocker or next safe action: Run every ordered repository gate, review the
  complete diff, then commit and push before invoking the projector.

## 2026-08-12 20:08 UTC | focused regression corrections

- Source commit: `b449a545` (immutable plan checkpoint)
- Actions: The first automation run rejected the safe field name
  `original_preparation_failure_primary` because the broad public denylist
  correctly matched the substring `origin`; renamed the field to
  `initial_preparation_failure_primary` without weakening the denylist. A
  separate package-wide Cargo run later stalled in the pre-existing zero-test
  `validate_network_scan_evidence` harness and was terminated; the scoped
  library run and Bazel automation suite then passed.
- Verification: The corrected public artifact passes the semantic redaction
  scanner, while an injected operational USB field still fails. The stalled
  process was not the new validator; the mandatory all-feature Cargo test will
  determine whether the host stall recurs.
- Evidence: Focused redaction regression and process observation only; no final
  evidence was published.
- Outcome: The redaction collision is fixed at the schema boundary; the host
  stall remains a verification observation, not a product failure.
- Blocker or next safe action: Continue to the ordered mandatory gates and stop
  if the all-feature Cargo suite reproduces the stall.

## 2026-08-12 20:15 UTC | clean implementation gates

- Source commit: `b449a545` (immutable plan checkpoint)
- Actions: Completed the ordered pre-commit gate sequence and the explicit
  simplification/diff review. The evidence layer remains a narrow aggregate
  projector over the already accepted ASIC-002 record; it adds no hardware
  command, private-attempt reader, or duplicate device-session machinery.
- Verification: `cargo fmt --all`, strict all-target/all-feature Clippy,
  all-target/all-feature Cargo build, all-feature Cargo tests, Bright Builds
  with zero findings, and all 41 Bazel tests passed. The Cargo suite completed
  successfully after unusually slow macOS dynamic-loader starts in several
  test binaries. `just parity` reports no validation errors, progress remains
  60/94, and redaction, pinned-reference, generated-contract, immutable-plan,
  unique-task, and diff checks pass.
- Evidence: Pre-publication verification only; no final PWR-002 projection
  exists and no hardware interaction occurred.
- Outcome: The complete implementation is ready for its clean pushed source
  checkpoint.
- Blocker or next safe action: Commit and push the implementation, then run
  the public projector from that exact clean commit.

## 2026-08-12 20:19 UTC | production-shaped source-admission correction

- Source commit: `ff021dcb` (first pushed implementation checkpoint)
- Actions: Invoked the sealed projector from the clean pushed checkpoint. It
  failed closed because the short stabilization fragment also appeared in the
  module import, so uniqueness could not be established. Replaced that guard
  with the complete 500-ms wait arm and made the regression fixture retain the
  production-shaped duplicate import token.
- Verification: The failed invocation returned `evidence_invalid`, reported
  `hardware_rerun_used: false` and `projection_published: false`, and left both
  public and candidate paths absent. All 18 focused tests passed. The complete
  ordered Cargo sequence, Bright Builds with zero findings, all 41 Bazel tests,
  parity, progress, redaction, pinned-reference, generated-contract,
  immutable-plan, unique-task, and diff checks then passed again.
- Evidence: Failure-category and absence proof only. No final projection was
  published and no hardware interaction occurred.
- Outcome: The real source shape is now represented by the regression, while
  the semantic admission remains stricter than the original short token.
- Blocker or next safe action: Commit and push the correction, then rerun the
  sealed public projector from the new exact clean commit.
