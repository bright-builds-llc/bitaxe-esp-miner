# Parity work result

- Parity row: `PWR-006`
- Final status: `verified`
- Implementation commit: `bff0e54708b95951409cb18dda0d38f2da097a11`
- Reference commit: `c1915b0a63bfabebdb95a515cedfee05146c1d50`
- Hardware attempts: none; the accepted API-002 capture was projected without
  a device rerun

## Evidence and verification

The committed
[INA260 projection](../../evidence/pwr006-ina260/ina260-projection.json) uses
schema `bitaxe-ina260-evidence-v1` and has SHA-256
`c9624b3c77e4021137a375de2a70c2bf7425bc947af6ba59c4e42fbceb25634d`.
The independent Rust validator accepted it, the final file is mode `0644`, and
repository redaction passed across all 17 committed evidence artifacts.

The typed projector admits only the exact protected API-002 attempt and its
already accepted public source projection. It closes the following facts
without publishing raw readings or device identifiers:

- one detector-admitted Ultra 205 booted the exact admitted package while
  mining and hardware control remained disabled;
- the production read-only sensor owner uses INA260 address `0x40` and the
  current, bus-voltage, and power registers `0x01`, `0x02`, and `0x03`;
- HTTP and WebSocket each contained a complete fresh three-field sample;
- both views had identical finite safe-range values, states, typed acquisition
  stamps, boot session, package identity, and the expected revision relation;
- all nine current sensor-to-API production paths are byte-compatible with the
  accepted hardware source commit; and
- cleanup completed, public redaction passed, and no hardware rerun occurred.

Regression coverage proves success with semantically identical reordered stamp
objects and large firmware u64 boot-session counters. It also proves typed
withholding for stale or uncorrelated samples, digest, source, semantic, dirty-
path, plan, task, private-mode, validator, and launch failures; candidate
cleanup; sensitive-output exclusion; and a real child-process/file boundary.

The implementation content passed:

- `cargo fmt --all`
- `cargo clippy --all-targets --all-features -- -D warnings`
- `cargo build --all-targets --all-features`
- `cargo test --all-features`
- `bun scripts/bright-builds-check.ts all`
- `just test` (all 41 Bazel test targets passed)
- `just parity` (`validation_errors: none`)
- `just parity-progress`
- `just verify-redaction`
- `just verify-reference`
- independent source and final evidence validation, generated-contract
  identity, immutable-plan and unique-task binding, source/reference ancestry
  and compatibility, candidate absence, exact digest/mode, and diff checks

## Conclusion

`PWR-006` has closed hardware-regression evidence for the production Ultra 205
INA260 read path and its correlated HTTP/WebSocket projection. The accepted
live capture proves the complete read-only current, bus-voltage, and power
sample; current source compatibility proves the same production path remains
in force. A duplicate hardware attempt would add risk without strengthening
this claim.

## Privacy, non-claims, and residual risks

Raw power, voltage, current, acquisition stamps, boot sessions, origins,
hostnames, ports, USB and network identifiers, credentials, retained logs, and
traces remain only in the ignored protected attempt. The committed projection
contains closed schemas, commits, digests, fixed register/address constants,
counts, categories, and booleans.

This result does not claim INA260 calibration beyond the admitted production
conversion, long-duration drift, out-of-envelope behavior, write or control
effects, fan, voltage, ASIC, mining, pool, thermal, OTA, recovery, other-board,
release-readiness, direct-UART, pin, or fault-injection parity. No USB, serial,
network, credential, flash, reset, mining, hardware-control, recovery, direct
UART, or pin effect occurred during this plan.
