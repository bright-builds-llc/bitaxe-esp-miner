# Parity work log

## 2026-08-13T07:33:53Z | selection and safe-stimulus checkpoint

- Source commit: `4af352a38f828c2ba0c3b3fe3754d3d0cf5a2fad`.
- Actions: Skipped API-009 at its consumed authority boundary, selected
  THR-001, audited the authoritative hardware-regression rule, the three prior
  closures, the read-only projection, production EMC2101 owner, typed reducer,
  API truth model, NVS modes, and safety-allow fault-stimulus policy.
- Verification: A physical overheat route is neither needed nor acceptably
  bounded. The production reducer already has a lossless, recoverable invalid-
  sample boundary, and a consume-before-use one-shot NVS admission can exercise
  it on the real device while all physical controls and mining stay disabled.
- Evidence: Committed source, policy, read-only projection, and redacted prior
  conclusions only. No protected attempt, credential, package effect,
  detector, device, network session, sensor mutation, or hardware action ran.
- Outcome: THR-001 is actionable through one injected acquisition-fault
  hardware regression that explicitly does not claim physical overheat or an
  electrical sensor fault.
- Blocker or next safe action: Complete the plan-only gates, commit and push
  the immutable contract, then implement the admission, state machine, host
  transaction, and independent evidence validator before any hardware use.

## 2026-08-13T07:52:00Z | immutable-plan verification checkpoint

- Plan SHA-256:
  `806a75411a98ccb242c631c7f7176fed6d94cd60c06c65163705aec3ab512f60`.
- Actions: Reviewed the exact task binding, attempt ordinal, permitted and
  prohibited effects, single-attempt boundary, restoration transaction,
  privacy policy, terminal categories, verification quorum, and promotion
  rule. Ran the complete plan-only gate sequence without accessing USB or the
  device.
- Verification: `cargo fmt --all`, `cargo clippy --all-targets --all-features
  -- -D warnings`, `cargo build --all-targets --all-features`, `cargo test
  --all-features`, `bun scripts/bright-builds-check.ts all`, `just test`,
  `just parity`, `just parity-progress`, `just verify-redaction`, `just
  verify-reference`, and `git diff --check` passed. Parity remains 67 verified
  of 94 active rows with no validation errors.
- Evidence: The plan digest above, clean mandatory command results, and the
  task-local contract only. The active lesson audit baseline remains valid;
  no new lesson block or audit trigger was introduced.
- Outcome: The plan is ready to become immutable in its own pushed commit.
- Blocker or next safe action: Commit and push only the plan, worklog, and
  targeted THR-001 task update; then implement and separately verify the
  transaction before any detector or hardware command.

## 2026-08-13T08:25:35Z | implementation verification checkpoint

- Actions: Added the strict private attempt-004 intent, internal physical
  package/plan admission, consume-before-use NVS tuple, bounded five-sample
  reducer, sole sensor-owner integration, private-first host restoration
  transaction, v1 public evidence contract, independent validator, and real-
  child regression coverage. No detector or hardware command ran.
- Verification: Focused safety, automation-contract, flash, and complete
  automation tests passed. The real ESP32-S3 firmware image built. The ordered
  Cargo format, strict Clippy, all-target build, and all-feature test sequence
  passed; the test command used an isolated generated target directory after
  the default directory exposed a host `syspolicyd`/dynamic-loader stall before
  Rust execution. Bright Builds, all 42 Bazel tests, parity validation,
  parity progress, semantic redaction, reference cleanliness, and diff checks
  passed. Parity remains 67 verified of 94 active rows with no validation
  errors.
- Evidence: Software tests prove one-shot erase-before-validation, exactly five
  overlays only after successful real reads, typed fault observation, fresh
  recovery, ordinary exact-package restoration, primary-failure precedence,
  protected modes, sensitive-field exclusion, and no invented child artifact.
- Outcome: The implementation is ready for its separate clean pushed commit.
  THR-001 remains `implemented`; no hardware-regression claim exists yet.
- Blocker or next safe action: Commit and push the implementation, rebuild the
  exact clean package, admit one Ultra 205 with the detector, and consume the
  single attempt-004 capture. Never start attempt-005.
