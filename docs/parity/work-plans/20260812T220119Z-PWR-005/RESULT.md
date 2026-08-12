# Parity work result

- Parity row: `PWR-005`
- Final status: `verified`
- Implementation commit: `a2fefad3b5863b0162747d98cdd1033878745a7a`
- Reference commit: `c1915b0a63bfabebdb95a515cedfee05146c1d50`

## Evidence and verification

The committed
[core-voltage-control projection](../../evidence/pwr003-core-voltage-control/core-voltage-control-projection.json)
uses schema `bitaxe-core-voltage-control-evidence-v1` and has SHA-256
`11dd1abbf6fda86d203fdcff49b420ab5139e1d29c35f4d17000c61c3112ae68`.
The independent Rust validator accepted it, the final file remains mode
`0644`, and repository redaction passed across all 16 committed evidence
artifacts.

The projection binds board 205, exact source and pinned-reference identities,
trusted package/runtime admission, fresh safety, and the source-compatible
typed DS4432U transaction:

- device address `0x48` and output-zero register `0xf8`;
- conservative 1100 mV register code `0xe1`;
- exactly one typed register write routed through the sole runtime I2C owner;
- 500 ms stabilization before active-low ASIC enable;
- successful BM1366 initialization and accepted downstream work; and
- confirmed safe stop, lease and USB cleanup, no hardware rerun, and passed
  redaction.

The existing
[PWR-003 result](../20260812T212218Z-PWR-003/RESULT.md) has SHA-256
`9f9e260411e983ba7b23748df4210b21a532b39f8772c1feaf6d453af9a54c36`,
matching the committed typed transition receipt. That result records the sole
projection command, source projection lineage, independent validation,
production-file regression, safe cleanup, and explicit non-claims. The
evidence implementation commit is an ancestor of current `main`; all current
DS4432U write, I2C ownership, voltage orchestration, projector, and validator
paths are byte-compatible from that admitted commit through the PWR-005
reconciliation checkpoint.

Focused verification included the Rust core-voltage evidence validator, its
three contract tests, the canonical TypeScript automation suite, exact digest
and mode checks, source/reference ancestry and path compatibility, pinned
reference cleanliness, and repository redaction. The complete ordered Cargo,
Bright Builds, Bazel, parity, progress, reference, privacy, task-binding, and
diff gates also passed before the immutable plan checkpoint.

## Conclusion

PWR-005's stale note said no DS4432U hardware write had been evidenced. The
subsequently accepted PWR-003 campaign and closed projection directly prove the
exact Ultra 205 DS4432U address, output register, conservative code, typed
single-write route, successful downstream operation, and safe stop. These are
row-independent DS4432U facts, so the existing immutable evidence satisfies
`unit,workflow,hardware-regression` verification for PWR-005 without another
hardware attempt or a redundant evidence schema.

## Non-claims and residual risks

This result reuses only closed facts from immutable committed evidence. It does
not claim direct analog voltage measurement, setpoint accuracy, rail timing or
waveform, DS4432U reads, output-one behavior, arbitrary or dynamic voltage
targets, unsafe-voltage fault injection or recovery, INA260 correlation,
non-conservative profiles, another board, or another ASIC family. It performs
no detector, package, flash, reset, USB/serial, network, credential, mining,
voltage, fan, power, GPIO, I2C, direct UART, pin, or fault-injection action.
Future changes to the owning DS4432U paths require fresh compatible evidence.
