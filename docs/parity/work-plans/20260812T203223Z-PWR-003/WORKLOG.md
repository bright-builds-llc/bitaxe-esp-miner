# Parity work log

## 2026-08-12T20:32:23Z | selection and immutable-plan checkpoint

- Source commit: `96d18ba5ec7d4e33c7806a5c4cfac54869934f41`
- Actions: Temporarily skipped API-009 at its fresh physical-observer gate;
  selected PWR-003; audited pinned upstream VCORE behavior, the current typed
  DS4432U actuation route, and the sealed accepted PWR-002 projection; created
  the immutable plan and active task contract.
- Verification: Clean synchronized `main`; selector order confirmed; pinned
  reference commit confirmed; no hardware command issued.
- Evidence: Planning and source audit only. No PWR-003 projection exists yet.
- Outcome: PWR-003 is actionable as a software-only typed projection of
  already accepted hardware evidence.
- Blocker or next safe action: Verify, commit, and push this plan/task
  checkpoint before implementation.

## 2026-08-12T20:39:00Z | immutable-plan verification

- Source commit: `96d18ba5ec7d4e33c7806a5c4cfac54869934f41`
- Actions: Froze the PWR-003 plan with SHA-256
  `7aff33c814262fc32ceeb082778093a055609711655ffd87d568aba37c7e2c5b`
  and verified the unique active task, sole open plan, and absent final
  projection.
- Verification: `cargo fmt --all`, strict Cargo Clippy, all-target Cargo
  build, all-feature Cargo tests, Bright Builds with zero findings, all Bazel
  test targets, parity report/progress, reference cleanliness, redaction, and
  diff checks passed.
- Evidence: Immutable plan and this worklog under
  `docs/parity/work-plans/20260812T203223Z-PWR-003/`.
- Outcome: The plan/task checkpoint is ready to commit and push before source
  work.
- Blocker or next safe action: Commit and push, then implement the typed
  core-voltage-control projection without device interaction.

## 2026-08-12T20:58:00Z | implementation and focused verification

- Source commit: `06abd057` (immutable plan checkpoint)
- Actions: Added the Rust-owned
  `bitaxe-core-voltage-control-evidence-v1` contract, independent validator,
  generated TypeScript binding, projector command, human command surface,
  redaction registration, and behavior regressions. The projector validates
  the exact PWR-002 projection, three byte-identical voltage-owning paths, two
  semantically compatible paths, and both pinned upstream voltage files.
- Verification: All 55 contract tests passed; the focused nine-test projector
  suite passed including its real `/usr/bin/stat` child boundary; the complete
  270-test automation suite passed after restarting the owned Bazel server.
  The first full attempt exposed the known macOS host-policy stall: newly
  launched binaries waited at `_dyld_start`, and every existing real-child test
  timed out while in-process tests passed. A one-shot sample confirmed the
  boundary; only owned check processes were terminated, the stale Bazel server
  was shut down, and the clean server rerun passed without code or timeout
  relaxation.
- Evidence: Source and tests only. No public PWR-003 projection exists and no
  hardware command ran.
- Outcome: The typed implementation is ready for the complete pre-commit gate
  sequence.
- Blocker or next safe action: Run every mandatory gate, commit and push the
  clean implementation, then invoke the projector once from that exact HEAD.

## 2026-08-12T21:11:58Z | complete implementation gate

- Source commit: `06abd057` (immutable plan checkpoint)
- Actions: Simplified typed failure classification into the existing typed
  failure module so the central CLI remains within its enforced file-length
  budget. No final projection was created and no hardware command ran.
- Verification: `cargo fmt --all`, strict Cargo Clippy, all-target Cargo
  build, all-feature Cargo tests, Bright Builds with zero findings, all 41
  Bazel test targets, parity report/progress, reference cleanliness,
  redaction, generated-contract, source-projection, immutable-plan hash, and
  diff checks passed. The existing `target/debug` tree intermittently blocked
  macOS executable and directory admission; bounded samples isolated the
  stalls outside test code, owned processes were cleaned up, and the complete
  Cargo suite then passed from a fresh `/tmp` target directory without source,
  assertion, or timeout changes.
- Evidence: Source and tests only. The admitted PWR-002 source projection
  independently validates with SHA-256
  `0668c274d09b3e39d7d5edfea4b2e66c97248ff77de9192981f3af00e547ddfe`.
- Outcome: The implementation is ready to commit and push before the one-shot
  projection command.
- Blocker or next safe action: Commit and push the implementation, require a
  clean exact HEAD, then produce and independently validate the PWR-003
  projection once.
