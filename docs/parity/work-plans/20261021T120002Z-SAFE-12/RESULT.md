# Parity work result

- Parity row: `SAFE-12`
- Final status: `verified`
- Implementation commit: `d95d7002980a72f178ef94821d30508c08af3093`
- Reference commit: `c1915b0a63bfabebdb95a515cedfee05146c1d50`

## Evidence and verification

The redacted summaries are `docs/parity/evidence/20261021-ultra205-local-mining-overheat-check.md`, `docs/parity/evidence/20261021-ultra205-local-mining-overheat-stays-stopped.md`. Each run used a repo-owned command on
an exact installed Ultra 205 package, sealed `passed`, and was
independently reviewed against its private journal.

## Revision

The notes gain this passage:

Later evidence narrows the exact_non_claims entry automatic thermal or fan fault recovery, for the overheat case only: from a test-only lowered trip (55 C, ADR-0040) on exact Ultra 205 packages, a local overheat safe stop was followed by the ADR-0038 recovery, restarting 100 MHz and 100 mV lower after the firmware's 30 s cooling window at 45 C or less (cooling bounds consistent with the samples of 40.2 s for the restart and 32.4 s for the pause), or staying stopped with the device paused by itself when that pair would leave the catalog; see docs/parity/evidence/20261021-ultra205-local-mining-overheat-check.md and docs/parity/evidence/20261021-ultra205-local-mining-overheat-stays-stopped.md. The production 75 C trip, the VR temperature trip, a second trip after a restart, fan fault recovery, recovery across a reboot and the remaining exact_non_claims above still apply.

## Non-claims

Nothing beyond the redacted summaries is claimed. The status and the
evidence tokens are unchanged.
