# Parity work log

## 2026-08-04T14:09:33Z | selection and plan

- Source commit: `2c587ce4a3d5f1e05563575e8686d053af65f0db`.
- Actions: Continued deterministic selection, traced the pinned ADC call into
  the power-management loop and public `coreVoltageActual` field, and compared
  it with the Rust producer and API projection.
- Verification: Clean synchronized branch, no open plan, ADC1 channel 1 maps
  to ESP32-S3 GPIO2, and Rust currently has no ADC owner or stamped core-voltage
  observation.
- Evidence: Immutable plan and active task record.
- Outcome: Bounded read-only ADC observation work is ready for its planning
  gate.
- Blocker or next safe action: Commit the plan before implementation.

## 2026-08-04T14:22:00Z | implementation and focused verification

- Source commit: `b6417579`.
- Actions: Added pure stamped ADC reduction, exact ESP-IDF ADC1 channel-1
  curve-calibrated ownership, I2C-independent startup admission, 500 ms producer
  sampling, fresh-only API/statistics projection, and a companion truth field.
- Verification: Four focused core-voltage reducer tests, all API tests, the
  sensor source-ownership target, and the real ESP32-S3 release firmware build
  passed. One first build exposed and resolved the exact generic channel type
  and module visibility; no behavior workaround was retained.
- Evidence: Successful zero remains fresh truth; read failures retain last-good
  provenance without advancing sequence; unavailable, invalid, stale, and
  fault states stay distinct; non-fresh numerics are suppressed.
- Outcome: The bounded read-only software behavior is implemented without
  adding core voltage to mining authorization or enabling any effect.
- Blocker or next safe action: Run mandatory repository-wide gates, record the
  implementation result, commit, and transition only `IO-002` to `implemented`.
