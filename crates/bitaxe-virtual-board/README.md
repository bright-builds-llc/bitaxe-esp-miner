# Functional Ultra 205 model

This crate produces device observations and byte exchanges without physical I/O,
threads or host files. The production runtime owns authority, safety, packet
construction and accounting policy. Model reservations describe an explicit
predicted heap; they are not real ESP-IDF allocator measurements.

## Provenance

Register addresses, units and wire framing are behavioral facts from the pinned
read-only reference at `c1915b0a63bfabebdb95a515cedfee05146c1d50`:

- `components/asic/bm1366.c` and `components/asic/crc.c` describe BM1366 framing,
  registers, PLL, baud transitions and response layout.
- `main/power/INA260.h` and `.c` describe big-endian measurement registers.
- `main/power/DS4432U.c` gives the adjustable current DAC transfer constants.
- `main/thermal/EMC2101.h` and `.c` give temperature/tach/PWM register units.

The discovery reply is the retained Phase 28.1 J3 frame documented by
`crates/bitaxe-asic/src/bm1366/result.rs`. Endpoint CRCs are independently computed;
conformance tests consume the existing GPL-compatible upstream golden fixtures
without copying their source expression into this crate. The independent nonce
oracle reconstructs header words and checks double SHA-256 against the fixed
public Bitcoin genesis hash. Nonce fixtures must contain explicit provenance and
matching work/header/target; ordinary submitted work never manufactures a nonce.

## Calibration and unsupported coverage

Voltage slew, fan acceleration, ambient temperature, heating/cooling rates,
sampling interval, I2C delay and input current are **uncalibrated functional
assumptions**. They are injectable and cannot establish electrical or thermal
parity. Fixed one-millisecond integration makes observations independent of
caller advance granularity. The pressure heap reproduces retained totals
(3,451 bytes free, 2,176 bytes largest block); its two segments are not a
reconstruction of the original device free-list. Default allocation routing
threshold/reserve are configurable and must be bound to the candidate SDK config.

Unknown I2C registers, display commands, ASIC operations and partitions return
`Unsupported`. Missing nonce fixtures produce no nonce response. Network and
control adapters transport opaque bytes: they perform no authentication, fixture
ACK or encryption on the caller's behalf. NVS/accounting helpers are model state;
shared runtime scenarios must still use production replay and ledger decisions.

ASIC hashing rate, physical brownout/RF behavior, real USB timing, full SSD1306
rendering and unimplemented sensor configuration side effects remain outside
validated fidelity. Qualification must list unsupported required operations,
not promote an unsupported result to success.

## Reset and sample ownership

Samples carry their publishing boot ordinal. Reset retains the old sample as
historical data, marks it unavailable, and forbids measurement-register success
until a new publication belongs to the new boot. Physical fan speed, temperature
and voltage decay retain inertia across reset. Sensor publication faults remain
scenario inputs rather than being erased by a reboot.

Reboot releases boot-owned heap reservations, old transport records and pending
I2C host claims independently of ordinary Close rejection. Heap tickets include a
boot epoch so an old ticket cannot release a newly allocated block with a reused
numeric ID. NVS identity, replay high water, charges and pending ordinals persist.
The original heap profile is restored as a model assumption; it does not assert
that physical post-boot allocator measurements equal modeled capacity.

The internal EMC2101 register is the calibrated snapshot minus the physical
Ultra 205 +5 degree driver offset, quantized to its signed whole-degree format.
The external register preserves its separately documented raw eighth-degree
domain. Tests use the real production decoder and offset function at the
45-degree boundary; calibrated observations are not offset twice.
