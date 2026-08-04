# Parity work plan

- Run ID: `20260804T140933Z-IO-002`
- Parity row: `IO-002`
- Initial status: `not-started`
- Source commit: `2c587ce4a3d5f1e05563575e8686d053af65f0db`
- Reference commit: `c1915b0a63bfabebdb95a515cedfee05146c1d50`
- Active task: `task-parity-io002-adc-observation`

## Selection

The deterministic selector reported no open plan. All earlier `implemented`
candidates retain documented live hardware, network, mining, safety, release,
other-board, or browser evidence gaps and have no new admissible evidence in
this run. `UI-001` through `UI-003` require live display/input UAT, while
`STAT-002` and `STAT-003` require live mining statistics. Earlier
`not-started` rows are broader task-orchestration, SoftAP/network, or non-205
ASIC surfaces. `IO-002` is the first narrow software-actionable row: the
pinned reference configures calibrated ADC1 channel 1 and samples it into
`coreVoltageActual`, while Rust currently publishes a permanent compatibility
zero with no ADC producer.

## Scope and non-scope

Add an Ultra 205 ESP-IDF oneshot ADC adapter for ADC1 channel 1 on GPIO2 using
12 dB attenuation, default resolution, and ESP32-S3 curve calibration. Admit
the adapter at startup without coupling its availability to I2C sensors, and
sample calibrated millivolts on the existing 500 ms operator-sensor cadence.

Extend the pure producer reduction and API observation model so successful
samples carry boot-session, sequence, and acquisition-time truth; read failure
preserves the last good sample behind a typed fault; and missing initialization
is explicitly unavailable. Project only fresh stamped values to
`coreVoltageActual` and expose a truth-only companion status. Keep this fact
out of the mining-safety predicate: this work adds read-only observation, not
permission to actuate or mine.

Do not flash or probe hardware, read credentials, make network requests,
change voltage/fan/power behavior, enable mining, add raw ADC logging, use OTA,
or invoke direct UART or pins. Do not claim calibration accuracy, physical
millivolts, cadence on device, or API correlation without a separately gated
hardware attempt.

## Implementation

- [ ] Add pure core-voltage acquisition reduction, stamped observation state,
      stale/fault handling, and behavior-focused tests.
- [ ] Add the exact ESP-IDF ADC1/channel-1/GPIO2 calibrated adapter and integrate
      it into startup and the sole operator sensor producer with fault isolation.
- [ ] Project fresh core-voltage truth through the API wire contract, update
      fixtures/source-ownership guards, and build the real firmware target.
- [ ] Record `RESULT.md` only after focused and repository-wide gates pass.

## Verification and promotion

Run focused safety/API/firmware host tests and the real ESP-IDF firmware build,
then `cargo fmt --all`, strict Clippy, all-target build, all-feature tests,
Bright Builds checks, `just test`, parity/progress, redaction, reference
cleanliness, and diff checks. Transition only `IO-002` to `implemented` with
`unit,workflow` evidence. `verified` requires a future detector-gated capture
that proves calibrated live millivolts, stamped freshness/cadence, failure
behavior, and correlation with the public API on the same admitted Ultra 205.
No hardware or recovery contract exists in this plan.
