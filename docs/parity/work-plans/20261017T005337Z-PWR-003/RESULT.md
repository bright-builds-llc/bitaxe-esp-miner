# Parity work result

- Parity row: `PWR-003`
- Final status: `verified`
- Implementation commit: `5a127e413193150af73374b27831203203d9aba9`
- Reference commit: `c1915b0a63bfabebdb95a515cedfee05146c1d50`

## Evidence and verification

The redacted summaries are in `docs/parity/evidence/20261016-ultra205-soak-007.md`,
`-008.md` and `-009.md`. Each soak ran 600 s of mining under a signed Work
Lease, on an exact installed package, from device settings. Each sealed
`passed`, and an independent review recomputed its criteria from the private
records.

## Revision

The notes gain this sentence:

Later soak evidence (ADR-0037), outside the projection above, narrows the other-profiles and arbitrary-targets non-claims for 1100 and 1200 mV only: during a lease, live core-voltage changes from 1200 to 1100 mV and back moved the board's ADC core-voltage reading by -107 mV and +108 mV ([soak 008](evidence/20261016-ultra205-soak-008.md)). That reading is not a calibrated analog measurement; setpoint accuracy, waveforms, other targets and the remaining non-claims above still apply.

## Non-claims

Nothing beyond the redacted soak summaries is claimed. The status is
unchanged.
