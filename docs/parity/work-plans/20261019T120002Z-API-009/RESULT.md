# Parity work result

- Parity row: `API-009`
- Final status: `verified`
- Implementation commit: `f4cec1409474da3f324e425014e1e68de8c0b175`
- Reference commit: `c1915b0a63bfabebdb95a515cedfee05146c1d50`

## Evidence and verification

The redacted summaries are `docs/parity/evidence/20261017-ultra205-local-mining-soak-001.md`, `docs/parity/evidence/20261018-ultra205-local-mining-soak-002.md`, `docs/parity/evidence/20261018-ultra205-local-mining-soak-003.md`, `docs/parity/evidence/20261018-ultra205-lease-interruption-check.md`. Each run used a repo-owned command on an
exact installed Ultra 205 package, sealed `passed`, and was independently
reviewed against its private journal.

## Revision

The notes gain this passage:

Later evidence, outside the projection above, shows pause and resume driving ADR-0038 local mining with no Work Lease on exact Ultra 205 packages: in soaks 001 to 003 each HTTP resume started mining the saved pool within 16 s, and in those soaks and the lease interruption check each HTTP pause stopped it through the ordered safe-stop, confirmed within 30 s by two consecutive samples with fresh input power at most 5 W (soak 001 journaled only the second), `miningPaused` true, `startMiningOnBoot` false and no `pauseReason`; see docs/parity/evidence/20261017-ultra205-local-mining-soak-001.md, docs/parity/evidence/20261018-ultra205-local-mining-soak-002.md, docs/parity/evidence/20261018-ultra205-local-mining-soak-003.md and docs/parity/evidence/20261018-ultra205-lease-interruption-check.md. This adds the upstream meaning of resume as starting local mining; endurance, power-loss behavior and the remaining non-claims above still apply.

## Non-claims

Nothing beyond the redacted summaries is claimed. The status is unchanged.
