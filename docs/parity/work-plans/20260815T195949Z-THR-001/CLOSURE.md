# Parity work closure

- Parity row: `THR-001`
- Final status: `implemented`
- Outcome: `blocked`
- Verification claimed: `no`
- Plan SHA-256: `c3dfb3219e73e8c4fd1d1c88e4fe52db06bc02a0a1721b94cdb5ac9d2adf65be`
- Active task: `task-parity-thr001-emc2101-live-thermal`

## Closure reason

The pushed software correction at `9fa31503` reproduced the consumed hardware
signature with production-shaped in-process and real-child fixtures, then
closed the mismatch with an exact two-origin INFO allowlist. The host accepts
only direct `bitaxe_firmware` and retained
`bitaxe_firmware::boot_evidence` envelopes. It still rejects arbitrary or
nested tags, wrong modules, non-INFO levels, malformed timestamps, extra
payload text, missing states, and wrong marker order.

Focused automation and the complete repository verification sequence passed,
including the real firmware build, all 45 Bazel tests, parity validation,
redaction, reference cleanliness, live-plan selection, and diff review. No
hardware was used and no evidence projection or parity promotion was produced.

## Next safe action

Create a distinct immutable hardware plan that binds a fresh attempt-007 to the
clean pushed correction, exact package identity, detector admission, one
bounded thermal transaction, ordinary recovery, cleanup, and protected
evidence handling. Never reuse attempt-006 or infer hardware success from the
software regression.

## Non-claims

This closure does not publish hardware-regression evidence, verify or promote
THR-001, or claim physical overheat, electrical sensor failure, calibration,
mining, controls, other boards, or release readiness. It authorizes no USB,
serial, HTTP, device, NVS, sensor, reset, OTA, erase, or attempt-007 effect.
