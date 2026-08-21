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

## 2026-08-21T03:18:00Z | checklist transition

- Source commit: `294b5cf0a998f53ccfe5a6537ad8cf39dfcd6fff`
- Actions: transitioned only `BAP-001` from `not-started` to `implemented`
  with `unit,workflow` evidence and synchronized deterministic progress.
- Verification: transition receipt
  `docs/parity/checklist-transitions/20260821T031800Z-BAP-001.json` binds the
  immutable plan, source/reference context, status, evidence, and updated
  notes.
- Evidence: checklist digest
  `bf50d0fae2cf3deec986b8727b60d831c4a225db05f07e39f7e8f359b8405088`;
  progress remains 89 of 94 active rows verified (94.7%).
- Outcome: `BAP-001` is conservatively `implemented`, not `verified`.
- Blocker or next safe action: unchanged; live accessory electrical behavior,
  Wi-Fi-password subscription delivery, setting effects, and detector-gated
  hardware verification remain unclaimed.
