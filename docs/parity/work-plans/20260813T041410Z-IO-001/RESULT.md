# Parity work result

- Parity row: `IO-001`
- Final status: `verified`
- Implementation commit: `265b573cfc42be841a5c16fcaf0cd4dd1cbe25a4`
- Reference commit: `c1915b0a63bfabebdb95a515cedfee05146c1d50`
- Hardware attempts: none; existing typed post-retry evidence was reconciled
  without another device effect

## Evidence and verification

The Rust firmware retains one I2C0 owner for the Ultra 205 startup display,
runtime display, read-only safety sensors, and typed actuators. It uses the
pinned GPIO 47/48, internal pull-ups, and 400 kHz bus speed. All display reads,
writes, write-reads, and transactions; INA260 and EMC2101 register reads; and
EMC2101 and DS4432U register writes enter the same retry helper with the pinned
500 ms driver timeout, exactly three transfer attempts, and a 10 ms delay after
every failed attempt, including the terminal failure. Address and operation
types remain closed to the four supported devices.

The source comparison matches `I2C_MASTER_TIMEOUT_MS=500`,
`I2C_RETRY_COUNT=3`, `I2C_RETRY_DELAY_MS=10`, and
`I2C_BUS_SPEED_HZ=400000` in the pinned `i2c_bitaxe.c` and
`i2c_bitaxe.h`. The focused host regressions prove immediate success, recovery
after one and two transient failures, exactly three failed calls, and the
terminal delay. Source-ownership regressions prove all six supported transfer
shapes retain the shared retry owner and no safety adapter constructs a second
I2C driver.

The immutable plan SHA-256 is
`1796d9ccf478a595557762e9197e811afefc68a35c2e7c8a87c2743f626f9c12`.
The retry source SHA-256 is
`e0104a0e578c677dab303cec090f7cf12ff01616b589caa2291a46e4b1b4c806`;
the shared-bus source SHA-256 is
`793d8441460e104107a26a3ddad8e38cd0fd7b2b53b4012e539b6d319273ac7d`;
the focused retry test SHA-256 is
`bd528c33cf469deb5af93276fd932677e8b65d4af85c60119e2cf32c3923c897`;
and the source-ownership test SHA-256 is
`c594f51236828229658f418a8fe3bbd6354c7ed39ed3195472e2e65e3f49de91`.
The retry and bus files are byte-identical from implementation commit
`b15073c9` through this result.

The independently validated committed `bitaxe-ina260-evidence-v1` projection
at `docs/parity/evidence/pwr006-ina260/ina260-projection.json` has mode `0644`
and SHA-256
`c9624b3c77e4021137a375de2a70c2bf7425bc947af6ba59c4e42fbceb25634d`.
Its board-205 attempt source is after the retry implementation and proves the
exact INA260 address and complete register 1/2/3 read set through matching
fresh same-package HTTP and WebSocket samples, with mining and hardware
control disabled, cleanup complete, and redaction passed.

The independently validated committed `bitaxe-emc2101-thermal-evidence-v1`
projection at
`docs/parity/evidence/thr001-emc2101-thermal/thermal-projection.json` has mode
`0644` and SHA-256
`d599357460b8b26431e8e362a3ff4c4f68572856f5cf1960631aa84046a345b5`.
Its board-205 source commit is after the retry implementation and proves the
exact EMC2101 address/register read through matching fresh same-package HTTP
and WebSocket samples, with mining and hardware control disabled, cleanup
complete, and redaction passed. The I2C and EMC2101 adapter paths are
byte-identical from that capture through this result.

The existing board-named SSD1306 hardware-smoke record remains the functional
display boundary: the Phase 14 ledger records the exact startup SSD1306/I2C
subclaim and the Phase 20 display-input log records the rendered 128x32 panel
at the closed display address. The latter log SHA-256 is
`81142ccdaed0e4fa4232a3224b7b4795e42de3d4b40b439fd2048d6076a58014`.
The user's fresh report of visible new display information is consistent with
this boundary but is not treated as typed evidence.

The physical fan/DS4432U semantics remain independently validated by the
mode-`0644` PWR-002 and PWR-003 projections with SHA-256 values
`0668c274d09b3e39d7d5edfea4b2e66c97248ff77de9192981f3af00e547ddfe`
and
`11dd1abbf6fda86d203fdcff49b420ab5139e1d29c35f4d17000c61c3112ae68`.
Those artifacts prove the typed fan-before-voltage transaction, exact
DS4432U address/register/code, successful downstream initialized work, safe
stop, and cleanup, but their physical run predates the retry owner and is not
used as post-retry proof.

The sealed post-retry API-009 attempt-007 campaign closes that time-order gap.
Its protected result and diagnostics have SHA-256 values
`0d17c5980a86536217ec86ba01e50d23e8f6f496d25d937a8003091ac4a1b744`
and
`a6efe832cf16f3cd422e95da9744f0b378cb00ba57f6e39be64f41a248a14164`;
the mode-`0700` root and mode-`0600` artifacts pass, the result seal matches,
and its private intent binds the exact post-retry source and pinned reference.
Only closed aggregate facts were consumed: exact package/runtime admission,
clean serial outcome, all 18 preparation events accepted through
`retain_production_uart/completed`, fresh required safety observations,
accepted work, confirmed safe stop, ready USB cleanup, redaction, and no
automatic promotion. The retry, bus, DS4432U, mining-actuation, and display
paths are byte-identical from that attempt through this result. The later
API-009 network-correlation failure remains authoritative for API-009 and does
not invalidate the earlier completed preparation transaction.

The following gates passed on the final evidence source:

- `cargo fmt --all`
- `cargo clippy --all-targets --all-features -- -D warnings`
- `cargo build --all-targets --all-features`
- `cargo test --all-features`
- `bun scripts/bright-builds-check.ts all`
- `just test` (all 42 Bazel test targets passed)
- `just parity`
- `just parity-progress`
- focused I2C retry, sensor ownership, display ownership, DS4432U actuation,
  and EMC2101 acquisition targets
- normal and rollback-probe ESP32-S3 firmware builds
- all four independent Rust evidence validators
- protected campaign seal/mode/source/fact validation
- exact reference comparison, source ancestry and compatibility, redaction,
  reference cleanliness, task/plan uniqueness, sensitive-output, and diff
  checks

## Conclusion

IO-001 has a closed composed proof. Exact unit/workflow evidence proves the
pinned transfer contract, and board-205 hardware evidence proves successful
display, INA260, EMC2101, fan, and DS4432U behavior on the retained post-retry
production paths. The required
`unit,workflow,hardware-smoke,hardware-regression` evidence is complete without
another flash, reset, mining campaign, or hardware effect.

## Non-claims and residual risks

This result does not prove physically injected electrical I2C faults, forced
arbitration loss, live terminal retry exhaustion, electrical waveform or bus
timing measurement, simultaneous multi-owner access, raw probing or scanning,
arbitrary addresses/registers/values, unsupported devices, non-205 boards, or
release readiness. The deterministic host regressions remain authoritative for
transient recovery and terminal exhaustion. No detector, package capture,
flash, reset, USB/serial/network session, credentials, mining rerun,
voltage/fan/power effect, HTTP command, raw I2C, fault injection, OTA,
recovery, direct UART, pin, pad, or GPIO action occurred during this plan.
