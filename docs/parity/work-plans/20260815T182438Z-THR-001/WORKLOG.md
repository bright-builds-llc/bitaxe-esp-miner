# Parity work log

## 2026-08-15T18:24:38Z | attempt-005 contract drafted

- Prerequisite: The production-order fault-projection correction is committed
  and pushed at `4fdd17db`; its software-only plan is closed without promotion.
- Scope: Advance the consumed attempt-004 bindings to one fresh attempt-005,
  then separately verify, commit, and push before exact packaging or hardware.
- Effects: Retain the existing single exact-package one-shot fault campaign and
  ordinary restoration. No new stimulus, hardware-control surface, protocol,
  or retry is introduced.
- Stop: Starting the sole campaign consumes attempt-005. Any non-ready or
  incomplete quorum withholds the projection and forbids attempt-006.

## 2026-08-15T18:30:00Z | immutable-plan verification

- Plan SHA-256:
  `8e8049fd6fbb19575f6abe593afcdd9ac2303eee0204b5f188d4b65aa7607d58`.
- Verification: Ordered Cargo format, strict Clippy, all-target build, and
  all-feature tests passed. Bright Builds, parity/progress, redaction,
  pinned-reference cleanliness, and diff checks passed without device access.
- Outcome: The attempt-005 contract is ready for its own commit and push before
  any ordinal/path/generated-binding implementation or hardware effect.

## 2026-08-15T18:45:00Z | attempt-005 binding implementation

- Changes: Advanced the strict flash intent, host transaction, independent
  evidence contract, generated TypeScript contract, protected attempt/wrapper
  roots, public projection path, plan path/digest, Rust fixtures, real-child
  fixture, and Bazel runfiles from consumed ordinal 4 to fresh ordinal 5.
- Focused verification: Flash intent/NVS tests pass. The complete real-process
  automation target passes with plan/task admission, both flash epochs, Git
  identity, validator boundaries, restoration, and evidence publication owned
  by child processes.
- Fixture hardening: The real-child test now asserts each emulated Git identity
  and cleanliness response before entering the transaction and accepts the
  executable script path in its argument vector. Production behavior is
  unchanged.
- Safety: No package, detector, USB, serial, HTTP, device, NVS, or hardware
  command ran. The new public projection path remains absent.

## 2026-08-15T19:05:00Z | attempt-005 implementation verified

- Verification: Focused flash and complete real-process automation tests pass.
  Ordered Cargo format, strict Clippy, all-target build, and all-feature tests
  pass. The real ESP32-S3 firmware build, Bright Builds, all 45 Bazel tests,
  parity/progress, semantic redaction, pinned-reference cleanliness, and diff
  checks pass.
- Simplification: The implementation changes no campaign lifecycle logic; it
  advances only the closed ordinal and its exact bindings. The added child
  assertions make the existing process boundary explicit without introducing
  an in-process artifact substitute.
- Outcome: The complete attempt-005 binding change is ready to commit and push.
  Packaging, detection, and the one authorized hardware invocation remain
  gated on the resulting clean synchronized HEAD.

## 2026-08-15T19:25:00Z | attempt-005 consumed without promotion

- Admission: Clean synchronized implementation `38968ff4`, exact source and
  reference package, absent fresh roots/projection, protected wrapper streams,
  and one detector-admitted board-205 device passed.
- Result: The sole campaign returned `evidence_invalid` and withheld its public
  projection. Protected allowlisted inspection found one `fault_observed`, one
  `recovered`, no `baseline_ready`, and no closed stimulus abort reason.
- Root cause boundary: Captured production marker payloads begin after the
  canonical ESP logging prefix, but the host validator requires byte-zero bare
  markers. The deterministic child fixture invented bare lines. The early
  baseline marker was additionally absent from the post-flash monitor capture.
- Recovery and cleanup: Ordinary exact-package recovery completed through a
  recovery flash with no secondary failure. Child cleanup and USB holder
  cleanup passed. Candidate and final projections remain absent.
- Disposition: Attempt-005 is consumed. THR-001 remains `implemented`; no
  attempt-006 is authorized. A future software-only plan must fix both the real
  log-payload seam and the late-attachment baseline witness before another
  ordinal can be considered.
