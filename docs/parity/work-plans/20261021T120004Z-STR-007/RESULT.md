# Parity work result

- Parity row: `STR-007`
- Final status: `verified`
- Implementation commit: `d95d7002980a72f178ef94821d30508c08af3093`
- Reference commit: `c1915b0a63bfabebdb95a515cedfee05146c1d50`

## Evidence and verification

The redacted summaries are `docs/parity/evidence/20261021-ultra205-local-mining-soak-006.md`, `docs/parity/evidence/20261021-ultra205-local-mining-soak-005-rejudged.md`. Each run used a repo-owned command on
an exact installed Ultra 205 package, sealed `passed`, and was
independently reviewed against its private journal.

## Revision

The notes gain this passage:

Later evidence adds soak 006 on firmware whose Rust atomics avoid S32C1I (ADR-0039): 4 hours of hashing in one unbroken span, judged by hashing rather than status, with no outages, 424 accepted shares and 0 rejected, within the live limits and ending in a powered-down stop, independently reviewed; see docs/parity/evidence/20261021-ultra205-local-mining-soak-006.md. Rejudged by hashing, soak 005, before that fix, accumulated 7 h 59 min 47 s of hashing across one 47 s outage, 12,948 ms short of 8 hours; see docs/parity/evidence/20261021-ultra205-local-mining-soak-005-rejudged.md. Mining for 8 hours, more than 4 hours on the ADR-0039 firmware, arbitrary pools and the remaining non-claims above still apply.

## Non-claims

Nothing beyond the redacted summaries is claimed. The status and the
evidence tokens are unchanged.
