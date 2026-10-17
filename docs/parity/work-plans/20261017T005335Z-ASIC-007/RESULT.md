# Parity work result

- Parity row: `ASIC-007`
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

Later soak evidence (ADR-0037), outside the projection above, narrows the default-profile, dynamic-retuning and soak-stability non-claims for these cases only: 600 s soaks prepared through the BM1366 ramp to 485 MHz ([soak 007](evidence/20261016-ultra205-soak-007.md)) and to 575 MHz, whose ordered shutdown completed in about 9.3 s against the 22,550 ms bound ([soak 009](evidence/20261016-ultra205-soak-009.md)); during a lease, live retuning from 485 to 400 MHz and back applied without revocation, with the counter hashrate following each frequency ratio ([soak 008](evidence/20261016-ultra205-soak-008.md)). Other targets, overclock behavior and the remaining non-claims above still apply.

## Non-claims

Nothing beyond the redacted soak summaries is claimed. The status is
unchanged.
