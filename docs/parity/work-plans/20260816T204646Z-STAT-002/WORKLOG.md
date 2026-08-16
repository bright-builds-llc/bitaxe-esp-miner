# Parity work log

## 2026-08-16T21:10:20Z | implementation checkpoint

- Source commit: `866439889406978cc397789e35b79acb899e296d`
- Actions: Added the closed Rust statistics-history evidence schema and
  validator, matching generated TypeScript contract, detector-only command
  surface, private-first exact-package orchestration, one-field settings
  mutation/restoration, bounded recovery, protected publication, source/task
  binding, Bazel/Just wiring, and behavior-focused regressions.
- Verification: `cargo test -p bitaxe-automation-contracts` passed 95 contract
  tests; `bazel test //tools/automation:automation_test` passed after resolving
  two compile-only interface findings. The workflow file was split at the
  contract/observation boundary during the simplification pass.
- Evidence: Software-only focused verification; no detector, credential,
  device, network, flash, settings effect, or hardware attempt has occurred.
- Outcome: Implementation is ready for the ordered repository gates and exact
  ESP32-S3 package build.
- Blocker or next safe action: Run every pre-hardware gate, review the diff,
  commit and push the exact source, then use only the immutable detector and
  attempt-001 sequence.

## 2026-08-16T21:19:10Z | pre-hardware verification

- Source commit: pending implementation commit from plan base
  `866439889406978cc397789e35b79acb899e296d`.
- Actions: Completed the simplification pass, corrected aggregate file-length
  findings without altering behavior, enforced exact timeout/reference-clean
  preflight, added bounded zero-setting clear polling, and removed the only
  post-publication fallible operation.
- Verification: The ordered Rust format/Clippy/build/test sequence, Bright
  Builds checks, real ESP32-S3 package build, all 45 Bazel tests, parity report,
  parity progress, redaction, reference cleanliness, generated-contract,
  immutable-plan, source-fragment, file-length and diff checks passed.
- Evidence: Software verification only. The package build proves the current
  tree builds but will be repeated after the implementation commit is pushed so
  its manifest binds the exact clean source.
- Outcome: Implementation and command contract are ready to commit and push.
- Blocker or next safe action: Commit and push, rebuild and inspect the exact
  package, then execute the single detector-gated attempt-001.

## 2026-08-16T21:29:54Z | hardware attempt-001

- Source commit: `01e48e12f7b063f923fdfc589c129448cb559064`.
- Actions: Rebuilt the exact clean pushed package, admitted exactly one Ultra
  205 through the protected detector, and launched the sole immutable
  statistics-history capture command with a 360-second bound.
- Verification: The typed public envelope reports `timeout` at
  `initial_flash_monitor`; the flash effect result is valid and completed; no
  projection was published; no settings mutation, restoration, or recovery
  occurred. Owner-only wrapper/attempt modes, clean synchronized Git state,
  absent projection/candidate, and post-attempt USB/tool-holder cleanup passed.
- Evidence: Protected private attempt at the immutable plan path; public
  aggregate evidence was correctly withheld.
- Outcome: Terminal non-verification. The equal supervisor and child 360-second
  boundaries prevented the monitor from returning before the supervisor fired.
- Blocker or next safe action: Close this plan without transition or progress
  sync. A future immutable attempt-002 requires a verified source correction
  that gives the child monitor a bounded cleanup grace or sole timeout ownership
  plus a real-child boundary regression.
