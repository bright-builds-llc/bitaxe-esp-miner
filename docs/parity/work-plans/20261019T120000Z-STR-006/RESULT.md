# Parity work result

- Parity row: `STR-006`
- Final status: `verified`
- Implementation commit: `f4cec1409474da3f324e425014e1e68de8c0b175`
- Reference commit: `c1915b0a63bfabebdb95a515cedfee05146c1d50`

## Evidence and verification

The redacted summaries are `docs/parity/evidence/20261017-ultra205-local-mining-soak-001.md`, `docs/parity/evidence/20261018-ultra205-local-mining-soak-002.md`, `docs/parity/evidence/20261018-ultra205-local-mining-soak-003.md`, `docs/parity/evidence/20261018-ultra205-lease-interruption-check.md`. Each run used a repo-owned command on an
exact installed Ultra 205 package, sealed `passed`, and was independently
reviewed against its private journal.

## Revision

The notes gain this passage:

Later evidence, outside the projection above, adds the ADR-0038 local owner: with no Work Lease, the same production coordinator mined the owner's saved pool after an operator resume on exact Ultra 205 packages for 30 continuous minutes (soak 001), 2 hours accumulated (soak 002) and 4 hours accumulated in 241 minutes (soak 003), each in one boot session with no device pause, fault or overheat, and each pause ended in the ordered stop with the ASIC powered down. Every outage observed (about 40-47 s each; cause unconfirmed) ended with local mining stopping safely and a new run starting a new pool session, without a reset. In the lease interruption check (attempt 004), a refused Gate Connect stopped local mining; a 600 s Work Lease then ran to its budget, and local mining resumed 20 s after the Gate soak's result; see docs/parity/evidence/20261017-ultra205-local-mining-soak-001.md, docs/parity/evidence/20261018-ultra205-local-mining-soak-002.md, docs/parity/evidence/20261018-ultra205-local-mining-soak-003.md and docs/parity/evidence/20261018-ultra205-lease-interruption-check.md. This narrows the reconnect non-claim: recovery by safe-stop and restart after outages up to 47 s is now shown. Fallback-pool switching, a lease starting without a refused Connect first, mining beyond 4 hours, a native panic seen once during a lease handover (still undiagnosed), and the remaining non-claims above still apply.

## Non-claims

Nothing beyond the redacted summaries is claimed. The status is unchanged.
