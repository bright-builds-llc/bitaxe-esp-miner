# Parity work result

- Parity row: `STAT-001`
- Final status: `verified`
- Implementation commit: `f4cec1409474da3f324e425014e1e68de8c0b175`
- Reference commit: `c1915b0a63bfabebdb95a515cedfee05146c1d50`

## Evidence and verification

The redacted summaries are `docs/parity/evidence/20261018-ultra205-local-mining-soak-002.md`, `docs/parity/evidence/20261018-ultra205-local-mining-soak-003.md`. Each run used a repo-owned command on an
exact installed Ultra 205 package, sealed `passed`, and was independently
reviewed against its private journal.

## Revision

The notes gain this passage:

Later evidence, outside the projection above, shows the hashrate monitor under ADR-0038 local mining with no Work Lease, after the owning-generation fix (071b5af3): the current hashrate read nonzero on every mining sample except those right after a mining start, with a median of 433 GH/s over 2 hours (soak 002 attempt 002) and 432 GH/s over 4 hours (soak 003) on exact Ultra 205 packages; see docs/parity/evidence/20261018-ultra205-local-mining-soak-002.md and docs/parity/evidence/20261018-ultra205-local-mining-soak-003.md. Laboratory-calibrated accuracy, mining beyond 4 hours and the remaining non-claims above still apply.

## Non-claims

Nothing beyond the redacted summaries is claimed. The status is unchanged.
