# Parity work result

- Parity row: `THR-002`
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

Later soak evidence (ADR-0037), outside the projection above, narrows the closed-loop-settling and soak non-claims for these cases only: under the automatic fan setting with a 60 C target, the chip settled at about 60 C, within 35-61 C overall, at 2,420-5,775 RPM during 600 s soaks at 485 and 575 MHz ([soak 007](evidence/20261016-ultra205-soak-007.md), [soak 009](evidence/20261016-ultra205-soak-009.md)); a third soak peaked at 60 C and ran 53-56 C once settled at 400 MHz ([soak 008](evidence/20261016-ultra205-soak-008.md)). Tuning quality, faults, overheat behavior and the remaining non-claims above still apply.

## Non-claims

Nothing beyond the redacted soak summaries is claimed. The status is
unchanged.
