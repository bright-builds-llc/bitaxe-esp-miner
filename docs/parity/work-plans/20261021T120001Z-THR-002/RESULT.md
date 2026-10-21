# Parity work result

- Parity row: `THR-002`
- Final status: `verified`
- Implementation commit: `d95d7002980a72f178ef94821d30508c08af3093`
- Reference commit: `c1915b0a63bfabebdb95a515cedfee05146c1d50`

## Evidence and verification

The redacted summaries are `docs/parity/evidence/20261021-ultra205-local-mining-overheat-check.md`, `docs/parity/evidence/20261021-ultra205-local-mining-overheat-stays-stopped.md`. Each run used a repo-owned command on
an exact installed Ultra 205 package, sealed `passed`, and was
independently reviewed against its private journal.

## Revision

The notes gain this passage:

Later evidence narrows the overheat-behavior non-claim for a test-only lowered trip (55 C, ADR-0040) only: on exact Ultra 205 packages, a local overheat stop set the fan to manual 100% as upstream's power management does, the chip cooled to 45 C or less, and mining either restarted 100 MHz and 100 mV lower or, when that pair would leave the catalog, the device paused itself; see docs/parity/evidence/20261021-ultra205-local-mining-overheat-check.md and docs/parity/evidence/20261021-ultra205-local-mining-overheat-stays-stopped.md. The production 75 C trip, the VR temperature trip, tuning quality, faults and the remaining non-claims above still apply.

## Non-claims

Nothing beyond the redacted summaries is claimed. The status and the
evidence tokens are unchanged.
