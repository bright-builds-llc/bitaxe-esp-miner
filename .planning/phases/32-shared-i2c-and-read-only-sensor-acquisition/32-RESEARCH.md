---
generated_by: gsd-phase-researcher
lifecycle_mode: yolo
phase_lifecycle_id: 32-2026-07-13T23-12-34
generated_at: 2026-07-13T23:24:00.000Z
phase: 32
requirements:
  - OBS-02
  - OBS-03
  - OBS-04
  - OBS-05
---

# Phase 32: Shared I2C and Read-Only Sensor Acquisition - Research

## Summary

Phase 32 should replace the current normal-boot split—where `display_adapter` consumes and drops `i2c0`, while the Phase 27 diagnostic branch consumes it elsewhere—with one normal-runtime owner. The smallest robust shape is a bounded `BitaxeI2cBus` created once in `main`, temporarily borrowed by startup display rendering, then moved into one producer thread that performs a 500 ms read-only sweep and replaces the existing observation store from producer-owned results.

The exact pinned `esp-idf-hal 0.46.2` APIs matter. `I2cDriver::{read,write,write_read}` accept FreeRTOS ticks, not milliseconds. The current `I2C_TIMEOUT_MS` value is passed as raw ticks, and the `embedded-hal` trait implementation used by `ssd1306` hard-codes `BLOCK`. The implementation therefore needs `TickType::new_millis(...).ticks()`, an atomic bounded `write_read` for register reads, and a narrow display adapter implementing the required I2C trait while forwarding every operation with the finite timeout.

The existing INA260 and EMC2101 files contain useful decoding breadcrumbs but cannot be reused wholesale. `emc2101::init` and `set_fan_duty_percent` write control registers, while generic `BitaxeI2cBus::write_register` makes prohibited effects reachable. Phase 32 needs explicit read-only sensor ports or command methods with a closed address/register allowlist and source-level regression guards proving the normal runtime does not call active helpers.

## Recommended Stack

| Concern | Existing choice | Phase 32 use |
| --- | --- | --- |
| Firmware I2C | `esp-idf-svc 0.52.1` re-exporting `esp-idf-hal 0.46.2` | Keep one `I2cDriver<'static>` behind a repo-owned bounded facade. |
| Timing | FreeRTOS/ESP-IDF ticks and `runtime_uptime::millis()` | Convert finite transaction timeouts with `TickType::new_millis`; use monotonic milliseconds for observation stamps and cadence decisions. |
| Producer task | Existing `std::thread`/FreeRTOS-backed firmware pattern | One named long-lived producer task, no independent sensor threads and no request-time acquisition. |
| Truth model | `bitaxe_safety::observation::{Observation, StampedSample, ...}` | Reuse the four-state contract and advance source-local sequences only on successful validation. |
| Store | `safety_adapter::observation_store` | Replace one complete `TelemetryObservations` snapshot after each sweep; consumers remain clone-only. |
| Display | `ssd1306` + `embedded-graphics` | Preserve the current startup text through a bounded borrowed-bus adapter; do not give the display ownership of the driver. |
| Verification | Cargo/Bazel/just + host unit tests | Pure decoder/reducer tests, source reachability guards, firmware compile/package, then detector-gated read-only hardware smoke if the plan requires it. |

No new third-party dependency is needed.

## Architecture Pattern

### 1. One lifecycle, two sequential borrowers

The normal path should have this ownership flow:

```text
Peripherals::take()
  -> BitaxeI2cBus::new(i2c0, gpio47, gpio48)
  -> display_adapter::render_startup_debug_text(&mut bounded_bus, ...)
  -> operator_sensor_runtime::start(bounded_bus, boot_session)
```

The display call is bounded and best effort. Regardless of render success, ownership returns to `main`, which moves the bus into the producer. The Phase 27 diagnostic path remains isolated and must not be called or broadened by Phase 32.

### 2. Pure sweep reducer, thin firmware shell

Separate effects from truth transitions:

- Firmware shell owns the bus, task sleep/deadline, monotonic clock, logging, and store replacement.
- Read ports return typed raw results per logical source: atomic INA260 triple, temperature, and tachometer.
- Pure decoders validate raw bytes and engineering-unit plausibility.
- A pure reducer takes the previous `TelemetryObservations`, source results, session, sequences, and acquisition time; it returns the next observations and next sequences.

This keeps failure-isolation, stamp preservation, and stale transitions host-testable without ESP-IDF.

### 3. Source-local atomicity

- INA260 current, bus voltage, and power form one acquisition. All three reads and validations must succeed before the power sequence advances and the triple becomes fresh.
- EMC2101 external temperature and tachometer are independent acquisitions. Each has its own sequence and can succeed while the other faults.
- After all attempts, publish one complete store snapshot. Never publish intermediate partial state while the sweep is in progress.

### 4. Closed read-only capability

Do not pass a generic write-capable `BitaxeI2cBus` into the normal sensor producer. Prefer a narrow API such as:

```rust
trait ReadOnlySensorBus {
    fn read_ina260(&mut self, register: Ina260ReadRegister, output: &mut [u8; 2]) -> Result<()>;
    fn read_emc2101(&mut self, register: Emc2101ReadRegister, output: &mut [u8]) -> Result<()>;
}
```

The exact type names are discretionary, but valid constructors must encode only INA260 registers `0x01`, `0x02`, `0x03` and EMC2101 registers `0x01`, `0x10`, `0x46`, `0x47`. Display writes remain a separate startup-only capability and must not leak into the periodic producer.

## Important Exact Behaviors

### Bounded transactions

`esp-idf-hal` uses a repeated-start `I2cDriver::write_read` for atomic register-pointer-plus-read transactions. Prefer that over the current separate `write` then `read`, and pass a finite `TickType_t` produced from a real millisecond duration.

The initial timeout should be materially less than the 500 ms sweep and the sum of worst-case transaction timeouts must remain bounded. A 50 ms per-transaction starting point is reasonable for planning, but the executor should keep the value named and pure-tested rather than burying it in calls. There should be no same-sweep retry loop; the next scheduled sweep provides the retry opportunity.

### Cadence and overrun

Use a deadline-based 500 ms schedule, not `sleep(500 ms)` after work, so transaction time does not permanently stretch cadence. When one sweep overruns, log a redaction-safe category and advance to the next future deadline; do not run unbounded catch-up sweeps.

### Failure transitions

- A read error becomes `FaultReason::ReadFailed` or the existing more specific stable reason, preserving `maybe_last_good` and its original stamp.
- A decoded invalid value becomes `FaultReason::InvalidSample` or the existing stable source-specific reason, preserving the last good stamp.
- Producer-owned time evaluation may move retained data to `StaleReason::ProducerCadenceExpired` after the chosen stale threshold.
- An absent/not-yet-seen source remains unavailable until an attempt yields a typed fault or a successful sample.
- API request frequency must not change any state, stamp, or sequence.

### Decoding pitfalls

- INA260 current is a signed 16-bit two's-complement register; the existing Rust code decodes it as `u16`, so the Phase 32 decoder must use `i16::from_be_bytes` before applying 1.25 mA/bit. Bus voltage and power remain unsigned.
- The three INA260 fields must be rejected as one acquisition when any field is incomplete or invalid.
- EMC2101 external temperature is an 11-bit signed value with 0.125 °C resolution. Preserve explicit open/short/fault and plausibility handling rather than converting sentinel values into ordinary temperatures.
- Tachometer raw zero must avoid division by zero. Preserve the upstream no-spin/sentinel behavior as explicit valid zero or typed unavailable according to the chosen decoder contract; never let an invalid raw count overflow or become a fresh arbitrary RPM.
- Narrow integer conversion for RPM must be checked before constructing `TachometerReading`.

## Do Not Hand-Roll

- Do not create a second I2C driver or a global shared bus mutex; Rust move/borrow ownership can prove the lifecycle structurally.
- Do not build custom binary transaction framing; use pinned `I2cDriver::write_read`.
- Do not add a scheduler dependency; one deadline-based producer loop is sufficient.
- Do not duplicate observation-state or stamp models; extend/reuse Phase 31 types.
- Do not use upstream fallback-cache behavior as freshness proof. The Rust producer preserves last-good values only inside explicit stale/fault variants.
- Do not invoke upstream or legacy initialization/control writes merely to enable tachometer reads. If hardware requires a control write, Phase 32 must report tachometer unavailable and defer actuation rather than widen scope.

## Common Pitfalls

1. Passing milliseconds directly to a tick-based timeout, producing a much longer bound than intended.
2. Passing raw `I2cDriver` to `ssd1306`, which selects the embedded-hal implementation that uses `BLOCK`.
3. Letting a display error drop the only driver before the producer starts.
4. Updating INA260 fields one at a time and thereby mixing acquisition epochs.
5. Reusing the temperature sequence/stamp for tachometer data.
6. Returning early on one sensor error and leaving later facts unattempted.
7. Holding the observation-store mutex during I2C transactions; acquire hardware first, then briefly replace the completed snapshot.
8. Treating zero compatibility projections or upstream cached values as fresh samples.
9. Making `write_register`, `emc2101::init`, fan duty, DS4432U, or Phase 27 bring-up reachable from the normal producer.
10. Claiming hardware parity from host tests; Phase 35 owns final evidence admission.

## Suggested Plan Decomposition

1. **Pure acquisition contract and tests:** typed read registers, raw decoders, atomic/independent reducers, sequence/failure/stale tests, and no-actuation source guards.
2. **Bounded shared bus and display handoff:** real tick conversion, bounded `write_read`, bounded display adapter, and one-driver ownership wiring.
3. **Producer/store integration:** 500 ms deadline loop, boot/session and per-source sequence ownership, failure-isolated sweep, complete snapshot replacement, and API consumer regression tests.
4. **Firmware/package and optional detector-gated read-only smoke:** compile/package, preserve startup display marker, prove repeated stamped observations and no prohibited-effect markers without promoting Phase 35 claims.

## Validation Architecture

### Test layers

| Layer | Purpose | Fast command |
| --- | --- | --- |
| Pure unit | Decode exact register vectors, validate sentinel/range behavior, reduce success/failure/stale states, prove sequence ownership and source independence. | `cargo test -p bitaxe-safety` plus the pure host crate that owns new acquisition types. |
| Firmware source guard | Prove the normal producer has no reachable write/init/control identifiers and the store remains acquisition-free. | Targeted firmware unit/source tests under `cargo test --all-features`. |
| API/store integration | Repeated request reads preserve observations; producer replacement changes only supplied state; failed sources leave unaffected facts usable. | `cargo test -p bitaxe-api` and relevant firmware-host tests. |
| Full host regression | Protect all existing Rust behavior. | `cargo test --all-features`. |
| Firmware compile/package | Prove ESP-IDF ownership, trait, thread, and linker integration. | Repo-owned Bazel/just firmware build and package commands from current project guidance. |
| Hardware smoke | Validate the one physical owner, startup display observability, fresh stamped data, failure tolerance where non-destructive, and absence of prohibited effects. | Only a phase-plan command beginning with `just detect-ultra205`, then a repo-owned ≥360-second flash/monitor evidence run. |

### Per-task sampling

- Every implementation task should run its narrowest affected unit target.
- Every plan should finish with `cargo fmt --all`, `cargo clippy --all-targets --all-features -- -D warnings`, `cargo build --all-targets --all-features`, and `cargo test --all-features` in that order.
- The integration plan must also run the canonical Bazel/just build and package verification required by the repo.
- No three consecutive tasks may lack an automated behavior check.

### Required regression cases

- Exactly one I2C0 driver construction exists on the normal runtime path.
- Startup display success and failure both hand the bus to the producer.
- All display I2C operations use finite timeouts.
- INA260 current/voltage/power are admitted together or not at all; current negative raw vectors decode as signed.
- Temperature success with tach failure and tach success with temperature failure preserve the successful fact.
- Read and validation failures preserve last-good stamps and do not advance sequences.
- Stale transition is producer/time driven and repeated API reads are effect-free.
- A failed early acquisition does not prevent later acquisitions or complete store publication.
- The periodic producer source contains no calls to EMC2101 init/fan writes, DS4432U writes, voltage/reset/power/ASIC/mining/fault-stimulus/self-test paths, credentials, direct UART/pins, OTA, or archived lineage code.
- Hardware evidence, if run, is read-only, board-205 detector-gated, bounded to ≥360 seconds, redaction-safe for promotion, and explicitly non-promotional outside Phase 35.

### Hardware gate

Host and firmware compile checks are mandatory. Real-device verification is appropriate because OBS-02 through OBS-05 describe observable hardware behavior, but it must remain read-only. Before any hardware command, run `just detect-ultra205` and continue only on exactly one validated board `205`. Do not perform fault injection, erase, raw writes, direct UART/pin work, or active control. If the detector fails or the phase plan lacks a redacted evidence contract, record hardware evidence pending rather than improvising.

## Sources

- `firmware/bitaxe/src/main.rs`, `display_adapter.rs`, `safety_adapter/i2c_bus.rs`, `ina260.rs`, `emc2101.rs`, `observation_store.rs`, and `runtime_snapshot.rs` — current integration and ownership boundaries.
- `crates/bitaxe-safety/src/observation.rs`, `power.rs`, and `thermal.rs` — Phase 31 truth, stamp, validation, and independent-fact contracts.
- Pinned local source `esp-idf-hal-0.46.2/src/i2c.rs` and `delay.rs` — exact finite-timeout, repeated-start, tick-conversion, and embedded-hal `BLOCK` behavior.
- `reference/esp-miner/main/i2c_bitaxe.c`, `power/INA260.c`, and `thermal/EMC2101.c` — read-only upstream behavioral evidence and active-write exclusions.
- `.planning/phases/31-operator-claim-and-telemetry-contract/31-CONTEXT.md` — locked cross-phase decisions.
