# Parity work result

- Parity row: `PWR-003`
- Final status: `verified`
- Implementation commit: `a2fefad3b5863b0162747d98cdd1033878745a7a`
- Reference commit: `c1915b0a63bfabebdb95a515cedfee05146c1d50`

## Evidence and verification

The sole software-only projection command ran from clean synchronized pushed
implementation commit `a2fefad3b5863b0162747d98cdd1033878745a7a`:

`just project-core-voltage-control-evidence --source-projection docs/parity/evidence/pwr002-asic-power-initialization/power-initialization-projection.json --attempt-source-commit 3e0966a140edbff1a14d2a48ca63d140649762c0 --projection docs/parity/evidence/pwr003-core-voltage-control/core-voltage-control-projection.json`

The command completed with typed category `complete` and atomically published
`docs/parity/evidence/pwr003-core-voltage-control/core-voltage-control-projection.json`
with SHA-256
`11dd1abbf6fda86d203fdcff49b420ab5139e1d29c35f4d17000c61c3112ae68`,
mode `0644`, and no surviving candidate. The independent Rust validator
accepted the final document. Its admitted PWR-002 source SHA-256 is
`0668c274d09b3e39d7d5edfea4b2e66c97248ff77de9192981f3af00e547ddfe`,
and the immutable retry-plan SHA-256 is
`dbd5d3a620726f270fd2827d4c8f53f0301834cea4999107964c22c711cb277e`.

The closed projection binds board 205, the exact admitted source/reference
commits, trusted package/runtime identity, target 1100 mV, I2C address `0x48`,
output register `0xf8`, register code `0xe1`, exactly one typed write, 500 ms
stabilization before active-low ASIC enable, upstream-compatible zero-voltage
disable semantics, successful initialized work, an accepted submit, safe stop,
lease and USB cleanup, disabled mine-on-boot, no hardware rerun, and passed
redaction.

Verification included the production-file regression that proves the former
token occurs twice while the exact multiline sleep expression is uniquely
admitted, focused automation and Rust contract tests, generated contracts,
independent source/final validators, the ordered Cargo checks, Bright Builds,
all 41 Bazel tests, parity and progress, source compatibility, pinned-reference
cleanliness, redaction, immutable/task binding, file mode, candidate absence,
and diff checks.

## Conclusion

The accepted Ultra 205 PWR-002 campaign already proved the exact production
power transaction reached initialized work and an accepted submit before safe
stop. This source-bound PWR-003 projection independently joins that accepted
hardware lineage to the unchanged production core-voltage command route and
its pinned upstream semantics. The complete typed quorum therefore supports
promoting only `PWR-003` from `implemented` to `verified`.

## Non-claims and residual risks

This result does not claim direct analog voltage measurement, setpoint
accuracy, rail timing or waveform, arbitrary or dynamic voltage targets,
INA260 correlation, injected fault recovery, non-conservative profiles,
another board, another ASIC family, or any new device effect. It reuses the
sealed accepted PWR-002 hardware projection and performs no detector, package,
flash, reset, USB/serial, network, credential, mining, voltage, fan, power,
GPIO, I2C, UART, or pin action. Future changes to the production voltage paths
must produce fresh compatible evidence rather than inherit this result.
