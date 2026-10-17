# Parity work result

- Parity row: `PWR-002`
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

Later soak evidence (ADR-0037), outside the projection above, narrows the arbitrary-profile, automatic-fan and soak-stability non-claims for these cases only: the device-settings profiles 485 MHz / 1200 mV ([soak 007](evidence/20261016-ultra205-soak-007.md)) and 575 MHz / 1200 mV ([soak 009](evidence/20261016-ultra205-soak-009.md)) each prepared with the fan at 100% and then mined 600 s under the automatic fan; a third soak mined 485/1200 with 200 s at 400/1100 ([soak 008](evidence/20261016-ultra205-soak-008.md)). The 1300 mV corner, other profiles and fan settings, and the remaining non-claims above still apply.

## Non-claims

Nothing beyond the redacted soak summaries is claimed. The status is
unchanged.
