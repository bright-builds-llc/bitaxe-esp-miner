# Parity work log

## 2026-08-16T00:54:43Z | plan checkpoint

- Source commit: `edf01789abbf47654281b1b27635e823cf33dcae`
- Actions: Selected STAT-001 after recording the five higher-ranked hardware or
  accessory blockers, froze the sole detector-gated attempt-001 contract, and
  added the matching active-task promotion record.
- Verification: The mandatory Rust sequence, Bright Builds checks, firmware
  image build, repository tests, parity/progress, redaction, and pinned-reference
  checks passed before the immutable plan commit.
- Evidence: `PLAN.md` has SHA-256
  `32321e5916949d1e6e3b41454c8eb4d168b7cd1a42d3e006e78d231fc74bf07b`;
  plan commit `bb1b5184186d84389749cd79075643468869651a` was pushed to
  `origin/main`.
- Outcome: Plan and attempt contract persisted without touching the device.
- Blocker or next safe action: Implement and verify the closed hashrate quorum,
  projection, and independent validator before hardware admission.

## 2026-08-16T01:16:07Z | implementation checkpoint

- Source commit: `bb1b5184186d84389749cd79075643468869651a`
- Actions: Added HTTP and WebSocket hashrate observation counters to the private
  campaign evidence, a typed redacted projection, a separate Rust validator,
  command wiring, and pure plus real-child tests. Kept raw hashrates and all
  device, network, pool, and credential values out of the projection.
- Verification: Focused campaign, contract, generated-contract, automation,
  independent-validator, and real-child tests pass. During the loop, a stale
  v3 schema assertion was corrected, and the validator's Git commit admission
  was corrected to accept 40-character commit IDs while retaining 64-character
  SHA-256 checks for artifact digests. Reference review also found that the
  upstream comment labels `0x100000` as 2^24 even though the executable literal
  is 2^20; the unused Rust instantaneous-register conversion had followed the
  comment and was corrected from 2^24 to the literal's 2^20. The live counter
  path remains the independently matched 2^32 conversion.
- Evidence: `bazel test //tools/automation:automation_test //tools/flash:tests
  //crates/bitaxe-automation-contracts:tests
  //tools/automation:contracts_verified` passed with all targets green.
- Outcome: Focused software verification is complete; no device effect has
  occurred and attempt-001 remains unused.
- Blocker or next safe action: Run the full ordered pre-hardware gates, commit
  and push the implementation, build its exact package, then run only the two
  frozen detector and capture commands.

## 2026-08-16T01:38:45Z | pre-hardware admission

- Source commit: `bb1b5184186d84389749cd79075643468869651a`
- Actions: Completed the explicit simplification pass by moving the closed
  accumulator into `network/hashrate.rs` and isolating command/invocation
  adapters. Reviewed the complete diff and immutable plan/task bindings.
- Verification: `cargo fmt --all`; strict Clippy; all-target build; all-feature
  tests; Bright Builds; the ESP32-S3 firmware image; `just test`; `just parity`;
  `just parity-progress`; redaction; pinned-reference; generated-contract;
  immutable-plan; unique-task; file-length; and diff checks all passed.
- Evidence: The firmware package target produced the ELF, OTA application,
  SPIFFS, otadata, factory image, and typed package manifest. Focused Rust and
  Bazel suites include the 2^20 instantaneous-register regression, 2^32 counter
  regression, private evidence accumulator, actual independent validator, and
  real-child capture boundary.
- Outcome: The implementation is admitted for commit and push; attempt-001 is
  still unused and no device effect has occurred.
- Blocker or next safe action: Commit and push this exact implementation,
  rebuild and admit its exact package identity, then run detector command 1.
