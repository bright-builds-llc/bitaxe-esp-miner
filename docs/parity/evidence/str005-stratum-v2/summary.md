# STR-005 Stratum V2 software evidence

## Claim

`STR-005` is implemented, not hardware-verified. The Rust firmware and host
tools implement the pinned ESP-Miner Stratum V2 subset with bounded framing,
official SRI Noise NX, standard and extended channel messages, job/target/share
state, BM1366 work conversion, sole firmware ownership, task-gated safety, a
real local TCP pool fixture, and closed campaign evidence admission.

## Provenance

- Reference commit: `c1915b0a63bfabebdb95a515cedfee05146c1d50`
- Audit plan: `docs/parity/work-plans/20260822T040442Z-STR-005/PLAN.md`
- Reference sources:
  - `reference/esp-miner/components/stratum_v2/sv2_protocol.c`
  - `reference/esp-miner/components/stratum_v2/sv2_noise.c`
  - `reference/esp-miner/main/tasks/stratum_v2_task.c`
  - `reference/esp-miner/main/tasks/protocol_coordinator.c`
- Golden vectors:
  `crates/bitaxe-stratum/fixtures/stratum-v2-protocol-vectors.json`

## Implementation

- Pure protocol and work: `crates/bitaxe-stratum/src/v2`
- Sole firmware owner and transport:
  `firmware/bitaxe/src/stratum_v2_session.rs` and
  `firmware/bitaxe/src/stratum_v2_session/transport.rs`
- Typed private settings and admission:
  `firmware/bitaxe/src/settings_adapter/stratum_v2.rs` and
  `firmware/bitaxe/src/settings_adapter/production.rs`
- Deterministic pool fixture: `tools/stratum-v2-fixture/src/main.rs`
- Private campaign NVS and closed marker validation:
  `tools/flash/src/campaign/admission.rs` and
  `tools/flash/src/campaign/serial/stratum_v2.rs`

## Verification

- Ordered Cargo format, Clippy with warnings denied, all-target build, and
  all-feature tests passed.
- 23 focused V2 tests cover frame/message bounds, standard/extended wire
  behavior, provenance vectors, authority keys, target/job/work/share state,
  malformed inputs, debug redaction, official initiator/responder handshake,
  encrypted split frames, tamper poisoning, and nonce exhaustion.
- The host fixture test completes a real TCP Noise handshake, setup, standard
  channel, future job plus previous hash, target-qualified share, and success.
- All 393 flash tests pass, including V2 credential/NVS and ordered terminal
  safe-stop evidence rejection cases.
- Firmware ownership tests prove mutually exclusive V1/V2 startup and reuse of
  the retained ASIC, safety, watchdog, and complete safe-stop paths.
- `just build` and `just package` produce the canonical six-file ESP32-S3
  package with the official Noise/secp256k1 dependency compiled by the S3
  toolchain using long calls.
- Bright Builds, all 52 Bazel tests, parity validation, redaction, reference
  cleanliness, source inventory, sensitive-value review, and diff review pass.

## Evidence boundary

Evidence types: `unit,golden,workflow`.

No hardware, USB, external network, owner pool, share submission, or credential
effect occurred. The lower-level fixture and private `stratum-v2` flash campaign
stage are not the immutable plan's outer campaign transaction. Hardware remains
ineligible until `just stratum-v2-campaign` captures and proves exact restorable
settings/package state before its first write, supervises fixture/flash/cleanup,
restores exactly with `mineonboot=false`, independently validates the redacted
projection, and passes interruption/recovery tests. Therefore
`hardware-regression`, external production-pool interoperability, arbitrary
pools, mixed-protocol live fallback, unbounded mining, other boards, OTA, and
release readiness remain explicit non-claims.
