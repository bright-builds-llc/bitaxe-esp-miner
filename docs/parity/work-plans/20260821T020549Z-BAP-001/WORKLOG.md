# Parity work log

## 2026-08-21T03:12:39Z | software implementation

- Source commit: `9c7a386c19c373e6ee3e0df8e0557e808b8fbb42`
- Actions: added a pure BAP owner lifecycle, bounded frame accumulator,
  subscription scheduler, single ESP-IDF UART2 owner, startup wiring, runtime
  request/subscription projection, fail-closed setting boundary, and focused
  lifecycle/source-ownership tests.
- Verification: focused BAP/core/Bazel tests; canonical ESP32-S3 package;
  ordered format, strict Clippy, all-target build, and all-feature Cargo tests;
  Bright Builds; all 50 Bazel tests; parity/progress; redaction; reference
  cleanliness; sensitive-value/file-size/diff review.
- Evidence:
  `docs/parity/evidence/bap001-firmware-interface/summary.md`; implementation
  commit `80f88df1799be63e3a71c291ad015f89c65cd8ae`.
- Outcome: the firmware ownership and lifecycle surface supports advancing
  `BAP-001` from `not-started` to `implemented` with `unit,workflow` evidence.
- Blocker or next safe action: verified parity still requires a separate
  task-gated live accessory session on named hardware, request/response and
  subscription observations, cleanup, privacy review, and redaction. Setting
  mutation remains fail closed until its hardware/effect owners are separately
  authorized and verified.
