# Parity work result

- Parity row: `PWR-004`
- Final status: `verified`
- Implementation commit: `8266b68d01bf6a51d88153456fc7145ba1351223`
- Reference commit: `c1915b0a63bfabebdb95a515cedfee05146c1d50`

## Evidence and verification

- **Unit:** `crates/bitaxe-safety/src/tps546.rs` and its `config.rs` and
  `enable.rs` model upstream's ordered configuration writes, the
  `VOUT_MODE` encoding, the status word and the enable sequence, with a
  golden test of the writes against
  `crates/bitaxe-safety/fixtures/gamma-601-tps546-config.csv`.
- **The firmware:** it owns the controller through
  `firmware/bitaxe/src/safety_adapter/i2c_bus/tps546.rs`,
  `firmware/bitaxe/src/safety_adapter/gamma601.rs`,
  `firmware/bitaxe/src/gamma601/regulator.rs` and
  `firmware/bitaxe/src/gamma601/actuation.rs`.

Each run below used a repo-owned command on the Gamma 601, sealed its
contract, and was independently checked against its private captures:
- **The safe boot** (`docs/parity/evidence/20261022-gamma601-safe-boot.md`):
  the output commanded and confirmed off (10 mV), with VIN, IOUT, the
  regulator temperature and the status word read.
- **The output check** (`docs/parity/evidence/20261023-gamma601-vout-check.md`,
  the test-only `gamma601-vout-check` at `4a7c5717`):
  - the 32-register configuration written, every readback matching;
  - then 60 s at a 1.15 V target with the ASIC held in reset: VOUT 1144 to
    1146 mV over 240 samples, no faults before or after, and the output
    confirmed off.
- **Under load,** its VIN, IOUT and regulator temperature fed the safety
  limits:
  - the first local mining run;
  - the 10-minute and 4-hour runs at 525 MHz / 1150 mV, the regulator
    peaking at 80.0 C in the 4-hour soak;
  - the self-test;
  - the production smoke soak (regulator peak 74.5 C, VIN 5086 to
    5156 mV).
- **The overheat check**
  (`docs/parity/evidence/20261025-gamma601-overheat-check.md`, the
  test-only `gamma601-overheat-trip-test`): the restart ran at 1050 mV.
- **Production:** the promoted build (F4.4) runs the same configuration and
  enable path; its boot resume, self-test and smoke soak powered the BM1370
  through it.

## Deviations and deferrals

- **Two deliberate deviations bound the voltage:**
  - `VOUT_COMMAND` is written at the requested target, not upstream's
    interim 1.2 V;
  - `VOUT_MAX` is 1.25 V, not 2.0 V.

  The firmware also caps the target at 1.20 V.
- **Live voltage changes are deferred on the 601** (ADR-0041 item 15, owner
  decision 2026-10-26). A saved voltage applies at the next mining start.
- **A regulator overheat ends the run stopped** rather than recovering as
  upstream does (ADR-0041 item 11).

## Revision

Status `verified`, evidence `unit,golden,hardware-smoke,hardware-regression,soak`.
The repeated soak and self-test runs are repo-owned scripted probes. The
owned target becomes the files above and the golden fixture. The notes become:

On the Gamma 601, the TPS546 output is commanded and confirmed off through the safe boot. A bounded test-build check wrote its 32-register configuration with every readback matching, held 1144 to 1146 mV at a 1.15 V target for 60 s with no faults, and confirmed the output off again; see docs/parity/evidence/20261023-gamma601-vout-check.md. Its VIN, IOUT and regulator temperature then fed the safety limits through local mining, the 4-hour soak, the self-test and the production smoke soak (regulator peak 80.0 C, in the 4-hour soak), and a test build's overheat restart ran at 1050 mV. The promoted production build runs the same configuration and enable path. Two deliberate deviations bound the voltage: VOUT_COMMAND is written at the target and VOUT_MAX is 1.25 V, under a 1.20 V target cap. See docs/parity/work-plans/20261029T040602Z-PWR-004/RESULT.md. Live voltage changes are deferred on the 601 (ADR-0041 item 15). A regulator overheat ends the run stopped rather than recovering as upstream does (ADR-0041 item 11). The 105 C regulator throttle, injected faults, the output state from power-on until the safe boot's off write and after a panic, analog accuracy, other voltages and other TPS546 boards remain non-claims.

## Non-claims

- **The 105 C regulator throttle:** the highest reading was 80.0 C.
- **Injected faults:** the fault checks ran, but no fault was injected.
- **The output from power-on until the safe boot's off write:** how long it
  stays on is an open follow-up
  (`docs/parity/evidence/20261022-gamma601-follow-ups.md`).
- **The output after a panic:** it stays on until the next boot's off write
  (ADR-0041 item 7, accepted as upstream has it).
- **A regulator overheat recovery:** a regulator overheat ends the run
  stopped rather than recovering as upstream does (ADR-0041 item 11).
- **Analog accuracy beyond the controller's own readings,** including the
  IOUT offset (about -0.2 A) while the chip draws no load.
- **Regulation at other voltages:** only 1.15 V was measured; the 1.05 V
  restart was commanded, not measured.
- **Other TPS546 boards.**
