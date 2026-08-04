# IO-002 work result

- Parity row: `IO-002`
- Final status: `implemented`
- Implementation commit: `4d7c8486526984ea9a44c60c9d8c65d7151da44d`
- Reference commit: `c1915b0a63bfabebdb95a515cedfee05146c1d50`
- Hardware attempts: none

## Evidence and verification

The Ultra 205 firmware now owns ESP32-S3 ADC1 channel 1 on GPIO2 through the
ESP-IDF oneshot API with 12 dB attenuation, default resolution, and curve
calibration. ADC admission and I2C admission are independent; the existing sole
operator sensor producer samples calibrated millivolts on its 500 ms cadence
without introducing another telemetry writer.

The pure reducer stamps each successful value with boot session, source-local
sequence, and monotonic acquisition time. A successful zero remains fresh
truth rather than becoming indistinguishable from failure. Read failures retain
the last good stamp behind `adc_read_failed`, initialization absence is
`core_voltage_unavailable`, invalid samples are distinct, and elapsed samples
become stale without advancing provenance.

Only fresh stamped truth reaches `coreVoltageActual` and statistics. Stale,
faulted, and unavailable states expose numeric zero while the new
`coreVoltageActualStatus` companion preserves their typed truth. Core-voltage
observation is deliberately absent from the mining-safety predicate and grants
no new effect authority.

The following gates passed on the implementation commit:

- focused core-voltage reducer and public-projection tests;
- sensor source-ownership tests;
- real `xtensa-esp32s3-espidf` release firmware build through `just build`;
- `cargo fmt --all`;
- `cargo clippy --all-targets --all-features -- -D warnings`;
- `cargo build --all-targets --all-features`;
- `cargo test --all-features`;
- `bun scripts/bright-builds-check.ts all`;
- `just test` (all 29 Bazel test targets passed);
- `just parity` and `just parity-progress`;
- `just verify-redaction`, `just verify-reference`, and `git diff --check`.

## Conclusion

The reference ADC configuration, calibrated read path, cadence ownership,
typed failure behavior, and fresh-only public projection are implemented with
unit and workflow evidence.

## Non-claims and residual risks

`IO-002` remains `implemented`, not `verified`. No connected device was used,
so calibration accuracy, physical millivolt values, live cadence, failure
behavior, and HTTP/WebSocket/statistics correlation are unproved. No hardware,
credentials, network request, mining, voltage/fan/power effect, OTA, recovery,
direct UART, or pin action ran or is claimed.
