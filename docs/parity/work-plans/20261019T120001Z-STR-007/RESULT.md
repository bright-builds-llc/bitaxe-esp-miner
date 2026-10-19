# Parity work result

- Parity row: `STR-007`
- Final status: `verified`
- Implementation commit: `f4cec1409474da3f324e425014e1e68de8c0b175`
- Reference commit: `c1915b0a63bfabebdb95a515cedfee05146c1d50`

## Evidence and verification

The redacted summaries are `docs/parity/evidence/20261017-ultra205-local-mining-soak-001.md`, `docs/parity/evidence/20261018-ultra205-local-mining-soak-002.md`, `docs/parity/evidence/20261018-ultra205-local-mining-soak-003.md`. Each run used a repo-owned command on an
exact installed Ultra 205 package, sealed `passed`, and was independently
reviewed against its private journal.

## Revision

The notes gain this passage:

Later evidence, outside the projection above, adds current accepted-share soaks of ADR-0038 local mining through the repo-owned `just local-mining-soak` on exact Ultra 205 packages: 30 continuous minutes with 87 accepted and 0 rejected shares (soak 001), 2 hours accumulated with 336 accepted and 1 rejected (soak 002 attempt 002), and 4 hours accumulated in 241 minutes with 477 accepted and 1 rejected (soak 003). Each was judged on live limits with fresh readings, one boot session and a powered-down stop, and each was independently reviewed; see docs/parity/evidence/20261017-ultra205-local-mining-soak-001.md, docs/parity/evidence/20261018-ultra205-local-mining-soak-002.md and docs/parity/evidence/20261018-ultra205-local-mining-soak-003.md. This narrows the share-soak non-claim; mining beyond 4 hours, arbitrary pools and the remaining non-claims above still apply.

## Non-claims

Nothing beyond the redacted summaries is claimed. The status is unchanged.
