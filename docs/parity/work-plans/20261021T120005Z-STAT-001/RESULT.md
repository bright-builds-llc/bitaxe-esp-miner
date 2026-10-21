# Parity work result

- Parity row: `STAT-001`
- Final status: `verified`
- Implementation commit: `d95d7002980a72f178ef94821d30508c08af3093`
- Reference commit: `c1915b0a63bfabebdb95a515cedfee05146c1d50`

## Evidence and verification

The redacted summaries are `docs/parity/evidence/20261021-ultra205-local-mining-soak-006.md`. Each run used a repo-owned command on
an exact installed Ultra 205 package, sealed `passed`, and was
independently reviewed against its private journal.

## Revision

The notes gain this passage:

Later evidence adds a median of 432 GH/s (p5-p95 367-493 GH/s) over the first 4 hours of soak 006, on firmware whose Rust atomics avoid S32C1I (ADR-0039), matching soak 005 before that change; see docs/parity/evidence/20261021-ultra205-local-mining-soak-006.md. Laboratory-calibrated accuracy, mining beyond 4 hours and the remaining non-claims above still apply.

## Non-claims

Nothing beyond the redacted summaries is claimed. The status and the
evidence tokens are unchanged.
