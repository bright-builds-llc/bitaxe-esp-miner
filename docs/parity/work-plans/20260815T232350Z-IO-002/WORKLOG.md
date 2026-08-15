# Parity work log

## 2026-08-15T23:29:00Z | implementation checkpoint

- Source checkpoint: immutable plan commit `15f474f3` plus the current
  implementation diff.
- Red proof: Adding only the missing
  `adc-observation-evidence.test.ts` import to the deployed `all.test.ts`
  entrypoint increased the suite from 328 to 334 tests and made the uncached
  Bazel target fail exactly at `checked-in ADC source semantics match the
  immutable contract` with `ADC source semantic fragment is not unique`.
- Root-cause fix: Replaced the broad upstream token
  `.bitwidth = ADC_BITWIDTH_DEFAULT` with the whitespace-normalized full
  `adc_oneshot_chan_cfg_t config` initializer context. The pinned reference
  contains that production channel configuration exactly once while retaining
  its other two legitimate bit-width initializers.
- Regression coverage: The registered suite now proves the checked-in source
  and runfiles pass, and separately rejects missing, duplicated, and drifted
  contextual initializers. All five named ADC source/task runfiles tests are
  present in the Bazel test log.
- Attempt binding: Rebound only the ADC workflow's task, immutable plan and
  SHA, protected wrapper/attempt paths, and public ordinal from consumed
  attempt-003 to attempt-004. Production firmware ADC, safety, mining, and
  control behavior remain unchanged.
- Verification: Ten focused Rust ADC contract/input tests pass; generated
  TypeScript contracts and both ADC validators build; the uncached deployed
  automation suite passes with the registered ADC module.
- Evidence: No credential was read, no detector or device command ran, no
  attempt ordinal was consumed, and no public ADC projection exists.
- Simplification review: One contextual breadcrumb expresses the exact
  upstream ownership boundary without weakening uniqueness. Registering the
  already-existing test module is sufficient; no new test runner or parallel
  validation path was added.
- Outcome: Both attempt-003 software boundaries are fixed at their real
  production/runfiles boundary. Mandatory verification, clean commit and push,
  package admission, and hardware remain pending.
- Next safe action: Run the full ordered pre-hardware matrix, review the diff,
  then commit and push before detector or device access.

## 2026-08-15T23:36:40Z | pre-hardware verification checkpoint

- Source checkpoint: immutable plan commit `15f474f3` plus the reviewed
  implementation diff.
- Verification: `cargo fmt --all`, strict all-target/all-feature Clippy,
  all-target/all-feature Cargo build, all-feature Cargo tests, the complete
  Bright Builds suite, the real ESP32-S3 package build, all 45 Bazel test
  targets, parity validation, parity progress (`69/94`, `73.4%`), public-
  evidence redaction, pinned-reference verification, immutable-plan hashing,
  and `git diff --check` all passed in a fail-fast run.
- Test-boundary proof: The deployed automation target now reports 337 tests and
  its log contains the checked-in ADC source/task tests plus the missing,
  duplicate, and drifted contextual-reference regressions. The prior omitted-
  module path is no longer possible through `all.test.ts`.
- Diff review: Changes are limited to ADC test registration, the unique
  upstream channel-config breadcrumb and regressions, attempt-004 contract
  bindings, generated contract projections, the plan runfiles declaration, and
  this task record. No firmware observation or control behavior changed.
- Evidence: No credential was read, no detector or device command ran, no
  attempt ordinal was consumed, and no public ADC projection exists.
- Outcome: The corrected source is ready for commit and push. Clean pushed
  package admission remains required before detector or device access.
- Next safe action: Commit this exact implementation, fetch and push without
  rewriting the immutable plan commit, then rebuild and admit the clean pushed
  package.
