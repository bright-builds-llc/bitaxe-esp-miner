# Parity work log

## 2026-08-04T22:41:28Z | attempt-007 normal-power remediation contract

- Source commit: `486d0718d3cb9089fff8300e2a54b15b4b61c4d4`.
- Actions: Resumed only `API-010` after the pushed attempt-006 detector stop
  and mapped its protected same-device unchanged-enumeration boundary to the
  repository-prescribed normal-connector power cycle.
- Verification: The 16 KiB observer fix, red/green regression, full software
  gate, exact package build, failed detector, and complete cleanup are already
  pushed. No attempt-007 hardware action has occurred.
- Evidence: Closed categories, booleans, bounded samples, and public source
  provenance only. Protected hardware and trace values remain private.
- Outcome: Attempt-007 is ineligible until this immutable plan/task checkpoint
  is clean, verified, committed, and pushed and the user reports the manual
  occurrence.
- Blocker or next safe action: Verify and push the contract, then request the
  physical normal barrel/USB power cycle as an occurrence checkpoint, not a
  new authorization decision.

## 2026-08-04T22:44:24Z | pre-hardware gate complete

- Source commit: `486d0718d3cb9089fff8300e2a54b15b4b61c4d4`.
- Actions: Ran the focused observer regression, canonical firmware build, and
  complete repository gate against the attempt-007 contract.
- Verification: The observer ownership target, firmware image, formatting,
  strict Clippy, all-target/all-feature build, all Cargo tests, Bright Builds,
  all 35 Bazel tests, parity validation/progress, semantic redaction, and
  pinned-reference cleanliness passed. The explicit continuation link makes
  the selector return this attempt-007 plan as the sole open lineage.
- Evidence: Public software outcomes only; no package for attempt-007,
  detector, device, credential, or hardware action occurred.
- Outcome: The immutable plan/task checkpoint is ready to commit and push.
- Blocker or next safe action: Commit and push, then wait for the user to
  report the full normal barrel/USB power-cycle occurrence.

## 2026-08-04T23:04:10Z | attempt-007 closed without promotion

- Source commit: `9bc7e03dbe18f5b0d317d2763e0ee25fff88fe50`.
- Actions: After the reported normal barrel/USB power cycle, built the exact
  pushed package, consumed the sole protected detector, and ran the one
  admitted capture. The detector passed and the exact-package flash completed;
  no theme mutation, software restart, or recovery flash occurred.
- Verification: The typed result is `evidence_invalid` at the
  `theme_durability` stage because terminal baseline classification was not
  admissible. Closed offline classification is `runtime_origin_missing`.
  The private trace contains 51 distinct boot sessions, 51 panic reset
  identities, 52 stack-overflow markers, and no runtime-origin or Wi-Fi-state
  marker. Startup ordering reaches the rendered operator display immediately
  before each overflow. ELF disassembly proves the operator-sensor runtime has
  an 8 KiB task stack, its own frame is 2 KiB, and its startup screen path
  reaches a full API-snapshot frame of 7,872 bytes before deeper calls.
- Evidence: Only closed categories, bounded counts, stack-frame sizes, safe
  booleans, and public source/ELF provenance are recorded. The detector,
  device, port, USB/network/process identities, credentials, origins, theme
  values, and raw traces remain private and untracked.
- Outcome: Attempt-007 is consumed. The previous 16 KiB boot-observer change
  targeted the wrong thread and did not affect the reproducible overflow.
  Withhold `RESULT.md` and public evidence; keep `API-010` at `implemented`.
- Blocker or next safe action: Do not retry this contract. Record and push the
  stop, then use a new immutable plan to replace the full API-snapshot screen
  dependency with a narrow screen-specific projection and a regression that
  prevents the oversized call path before one separately gated attempt.
