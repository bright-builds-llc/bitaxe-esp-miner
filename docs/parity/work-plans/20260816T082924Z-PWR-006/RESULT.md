# Parity work result

- Parity row: `PWR-006`
- Final status: `verified`
- Implementation commit: `7a822b5c229d9f169fe22fe999202976980bed78`
- Reference commit: `c1915b0a63bfabebdb95a515cedfee05146c1d50`
- Hardware attempts: none; this correction re-evaluated the accepted read-only
  API-002 capture without accessing the device

## Evidence and verification

The corrected compatibility matrix is:

- legacy `voltage`: millivolts;
- legacy `current`: milliamps;
- `coreVoltage` and `coreVoltageActual`: millivolts;
- `power`: watts; and
- `nominalVoltage`: volts.

Rust sensor acquisition and safety logic remain in volts, amps, and watts.
Named conversions now apply only where system-info/WebSocket and statistics
data enter the legacy wire contract. Behavior tests prove that an internal
5.1 V / 2.25 A snapshot serializes as 5100 mV / 2250 mA while 11.5 W,
1198 mV core voltage, and 5 V nominal voltage retain their intended units.
Campaign tests prove that the unchanged physical 4.5-5.5 V safety range now
accepts exactly 4500-5500 mV and rejects values immediately outside it.

The committed
[unit-correction projection](../../evidence/pwr006-ina260/ina260-wire-units-projection.json)
uses schema `bitaxe-ina260-evidence-v2`, has SHA-256
`ddf94e029b55089bb0e9a86cac6a0ca0d737ef67509d850233e64b532260b7fb`,
and is mode `0644`. The independent Rust validator accepted it and repository
redaction passed across all 21 committed evidence artifacts.

The projector binds the immutable correction plan, the earlier hardware plan,
the exact historical source that produced the accepted read-only capture, the
corrected current source, and six pinned reference paths. Those reference paths
prove INA260 voltage/current milli-units, power conversion to watts, unchanged
system-info/statistics forwarding, and AxeOS division of legacy voltage,
current, and core-voltage fields by 1,000. The projection records no raw values
and explicitly records `hardware_rerun_used: false`.

Verification passed:

- `cargo fmt --all`
- `cargo clippy --all-targets --all-features -- -D warnings`
- `cargo build --all-targets --all-features`
- `cargo test --all-features`
- `bun scripts/bright-builds-check.ts all`
- `just test` (all 45 Bazel test targets)
- `just package`
- `just parity` (`validation_errors: none`)
- `just parity-progress` (`verified=73`, `active=94`, `total=99`)
- `just verify-reference`
- `just verify-redaction`
- `just validate-ina260-evidence` for the v2 projection
- generated-contract identity, immutable-plan digest, clean-source,
  source/reference ancestry, exact artifact mode/digest, and diff checks

## Conclusion

`PWR-006` remains verified with the false legacy-unit assumption removed. The
accepted Ultra 205 evidence still proves the production read-only INA260
register path and correlated fresh HTTP/WebSocket observation. The v2 evidence
now interprets that historical capture according to its exact source semantics
and independently proves that current Rust source restores the upstream
milli-unit compatibility boundary without changing internal SI safety logic.
The transition ledger forbids automatic edits out of `verified`; because every
corrected-source, evidence, privacy, and repository gate passed, no checklist
field change was required.

## Non-claims and residual risks

This result does not claim external-meter accuracy, calibration beyond the
admitted register scaling, long-duration drift, load response, control effects,
other-board behavior, mining behavior, or release readiness. It does not claim
that the corrected milli-unit JSON was freshly observed on hardware; it is
proved by current-source behavior tests and source-bound reuse of the earlier
read-only acquisition. No USB, serial, network, credential, flash, reset,
mining, voltage/frequency/fan/power actuation, OTA, recovery, direct UART, pin,
or fault-injection effect occurred during this correction.
