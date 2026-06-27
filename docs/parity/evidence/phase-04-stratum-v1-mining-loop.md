# Phase 04 Stratum V1 Mining Loop Evidence

## Scope

This evidence covers Phase 04 Stratum v1 parser, fake-pool lifecycle, mining job construction, work queue behavior, share-submission mapping, and the fail-closed first Ultra 205 mining-loop gate.

It does not record a live pool socket run, real BM1366 production work submission, accepted live share, or long-running soak. No pool credentials, wallet usernames, or secret-bearing logs are included.

## Commands

| Command | Result | Evidence Boundary |
| --- | --- | --- |
| `cargo test -p bitaxe-stratum --all-features` | passed during Phase 04 Plan 04 verification | Unit and fake-pool coverage for Stratum v1 protocol, lifecycle, mining jobs, queue, mining-loop gate, and share-submission mapping. |
| `bazel test //crates/bitaxe-stratum:tests` | passed during Phase 04 Plan 04 verification | Bazel-visible Stratum crate coverage and fixture wiring. |
| `just parity` | passed during Phase 04 Plan 04 verification | Checklist parse and safety-critical evidence checks. |
| `just verify-reference` | passed during Phase 04 Plan 04 verification | Read-only reference tree guard. |

Additional task-level checks run during Plan 04:

- `cargo test -p bitaxe-stratum mining_loop --all-features`
- `cargo build -p bitaxe-firmware --target xtensa-esp32s3-espidf`
- `cargo test -p bitaxe-asic --all-features`

## Pure/Fake-Pool Evidence

- Stratum v1 raw JSON is parsed into typed messages before mining state can consume pool data.
- Deterministic fake-pool transcripts cover subscribe, authorize, notify, set-difficulty, accepted submit, rejected submit, reconnect, fallback, timeout, and unexpected client behavior.
- Mining job construction turns typed notify/extranonce data into `Bm1366WorkFields`; Stratum code does not create raw ASIC `JobFrame` or `CommandFrame` values.
- `MiningWorkQueue` covers bounded capacity 12, FIFO dequeue, clean-jobs clearing, and `Bm1366ValidJobIds` reset.
- `MiningLoopGate::default().decision()` blocks with `hardware_evidence_ack_missing`.
- Ready gates require ASIC initialization, safety evidence, and hardware-evidence acknowledgment before dispatch planning.
- Firmware logs `mining_loop_status=blocked reason=hardware_evidence_ack_missing work_submission=disabled` at boot.

## Hardware Smoke

Conclusion: not run - hardware evidence pending.

Required smoke record before live mining can move beyond implemented:

- Board: Ultra 205 BM1366.
- Command: `just flash-monitor board=205 port=<redacted-port-or-safe-device-path>`.
- Firmware commit: record short commit.
- Reference commit: `c1915b0a63bfabebdb95a515cedfee05146c1d50`.
- Pool target: redact usernames, passwords, tokens, wallet labels, and any secret-bearing connection strings.
- Logs: boot identity, ASIC gate status, mining-loop gate status, pool lifecycle, accepted/rejected share counters, fallback/reconnect observations when exercised.
- Result: observed accepted or rejected share outcome, or explicit safe-blocked conclusion.

## Soak Criteria

Conclusion: not run - hardware evidence pending.

Required soak record before live mining stability can be claimed:

- Start and end timestamps.
- Firmware commit and reference commit.
- Board and port.
- Pool target with secrets redacted.
- Accepted share count, rejected share count, rejected reasons, best/share difficulty when available.
- Reconnect and fallback observations.
- Thermal, fan, voltage, and power safety status once later safety phases own those surfaces.

## Conclusion

Phase 04 has pure and fake-pool evidence for Stratum v1 parsing, lifecycle state, mining job construction, work queue behavior, share-submission mapping, and the fail-closed first mining-loop gate.

Live pool socket use and real BM1366 production work submission remain disabled in firmware main. Live hardware mining smoke and soak are not run - hardware evidence pending.

## Residual Risk

- Fake-pool and unit tests do not prove ESP-IDF socket behavior, pool DNS/TCP/TLS handling, accepted live shares, or reconnect timing against real services.
- Firmware currently publishes a blocked mining-loop status only; it does not start a Stratum task or submit BM1366 work.
- Hardware-control surfaces for power, voltage, thermal, and fan remain below live parity proof until their own evidence-gated phases complete.
