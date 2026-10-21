# Parity work result

- Parity row: `STR-006`
- Final status: `verified`
- Implementation commit: `d95d7002980a72f178ef94821d30508c08af3093`
- Reference commit: `c1915b0a63bfabebdb95a515cedfee05146c1d50`

## Evidence and verification

The redacted summaries are `docs/parity/evidence/20261021-ultra205-local-mining-soak-006.md`, `docs/parity/evidence/20261020-ultra205-boot-resume-check.md`. Each run used a repo-owned command on
an exact installed Ultra 205 package, sealed `passed`, and was
independently reviewed against its private journal.

## Revision

The notes gain this passage:

Later evidence adds 4 hours of continuous local mining on firmware whose Rust atomics avoid S32C1I (ADR-0039): soak 006 hashed 14,402,050 ms in one unbroken span with no outages and 424 accepted shares; and with `startMiningOnBoot` on, mining resumed after a software restart without a resume command; see docs/parity/evidence/20261021-ultra205-local-mining-soak-006.md and docs/parity/evidence/20261020-ultra205-boot-resume-check.md. A restart panic found on the way was root-caused to Rust atomics on PSRAM and fixed; the lease handover panic fits that cause, but its dump could not be decoded, so that link stays unproven. Mining beyond 4 hours and the remaining non-claims above still apply.

## Non-claims

Nothing beyond the redacted summaries is claimed. The status and the
evidence tokens are unchanged.
