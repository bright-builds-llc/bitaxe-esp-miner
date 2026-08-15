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

## 2026-08-15T23:17:11Z | attempt-003 accepted stop

- Source checkpoint: clean pushed commit
  `9f48a1dbc07d7df83b05452e10edee4ff8989d12`; pinned reference
  `c1915b0a63bfabebdb95a515cedfee05146c1d50`.
- Admission: The exact package rebuilt from the synchronized commit, protected
  attempt and projection paths were absent, local Wi-Fi input passed the
  non-reading presence check, and the exact detector command admitted one
  Ultra 205.
- Hardware outcome: The sole attempt-003 capture observed the exact-package
  boot and same origin, kept mining and hardware control disabled, completed
  cleanup, and passed redaction. Protected system-info evidence and the ADC
  input validator both passed; the latter proves finite integer millivolt-domain
  values plus fresh, monotonic, coherent HTTP/WebSocket observation state.
- Earliest blocker: Final source-provenance admission returned typed category
  `evidence_invalid` with safe summary `ADC source semantic fragment is not
  unique`. The breadcrumb `.bitwidth = ADC_BITWIDTH_DEFAULT` appears three
  times in pinned `reference/esp-miner/main/adc.c`, while the admission helper
  requires every breadcrumb to occur exactly once.
- Publication: The ADC candidate was removed and the public projection remains
  absent. IO-002 stays `implemented`; attempt-003 is consumed and was not
  retried.
- Root-cause conclusion: The stop does not indicate volts-versus-millivolts
  confusion. Both Rust and upstream use calibrated millivolts at this boundary;
  the blocker is an over-broad upstream provenance fragment and a regression
  input/cache gap that failed to expose its multiplicity before hardware.
- Next safe action: Under a fresh task and immutable plan, replace the ambiguous
  breadcrumb with exact initializer context and make the pinned reference file
  an explicit regression input. Re-run software gates before separately
  authorizing any fresh hardware ordinal.
