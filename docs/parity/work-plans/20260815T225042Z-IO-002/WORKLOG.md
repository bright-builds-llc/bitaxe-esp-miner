# Parity work log

## 2026-08-15T23:04:00Z | implementation checkpoint

- Source checkpoint: plan commit `1f935cd8` plus the current implementation
  diff.
- Actions: Rebound the closed ADC evidence contract to attempt-003 and added a
  production-owned task/plan preflight before the system-info transaction. The
  checked-in plan is now a declared Bazel/runfiles input, and the same validator
  serves production plus the real-artifact regression. Production firmware ADC,
  API, safety, mining, and control paths remain unchanged.
- Verification: The new regression first failed to compile because the
  production preflight did not exist. After implementation, the real checked-in
  task/plan passes from Bazel runfiles, removing its schema binding fails before
  any child process runs, ten focused Rust ADC contract/input tests pass, the
  filtered real-child automation test passes, and the canonical generated-
  contract/capture/validator targets build.
- Evidence: No credential was read, no detector or device command ran, no
  attempt ordinal was consumed, and no public projection exists.
- Simplification review: One exported async preflight reads the two authoritative
  documents and delegates to the existing pure validator. Production invokes it
  once before effects; no duplicate post-effect check or reusable retry
  parameter was added.
- Outcome: The attempt-002 task-contract boundary is fixed and guarded in
  software. Mandatory verification, clean commit/push, package admission, and
  hardware remain pending.
- Next safe action: Run the full ordered pre-hardware matrix, review the diff,
  then commit and push before detector or device access.

## 2026-08-15T23:12:00Z | pre-hardware verification checkpoint

- Source checkpoint: plan commit `1f935cd8` plus the reviewed implementation
  diff.
- Verification: `cargo fmt --all`, strict all-target/all-feature Clippy,
  all-target/all-feature Cargo build, all-feature Cargo tests, the complete
  Bright Builds suite, the ESP32-S3 package build, all 45 Bazel test targets,
  parity validation, parity progress (`69/94`, `73.4%`), public-evidence
  redaction, pinned-reference verification, and `git diff --check` all passed
  in a fail-fast run.
- Diff review: Changes are limited to attempt-003 contract bindings, the
  fail-before-effect task/plan preflight, its runfiles declaration and
  regression coverage, generated contract projections, and this task record.
  No firmware observation or control behavior changed.
- Evidence: No credential was read, no detector or device command ran, no
  attempt ordinal was consumed, and no public ADC projection exists.
- Outcome: The corrected source is ready for commit and push. Clean pushed
  package admission remains required before detector or device access.
- Next safe action: Commit this exact implementation, fetch and push without
  rewriting the immutable plan commit, then rebuild and admit the clean pushed
  package.
