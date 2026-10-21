# Parity work result

- Parity row: `API-009`
- Final status: `verified`
- Implementation commit: `d95d7002980a72f178ef94821d30508c08af3093`
- Reference commit: `c1915b0a63bfabebdb95a515cedfee05146c1d50`

## Evidence and verification

The redacted summaries are `docs/parity/evidence/20261020-ultra205-boot-resume-check.md`. Each run used a repo-owned command on
an exact installed Ultra 205 package, sealed `passed`, and was
independently reviewed against its private journal.

## Revision

The notes gain this passage:

Later evidence adds restart with mining on boot: on exact Ultra 205 packages, boot-resume check attempt 002 sent `POST /api/system/restart` twice, and each answered and came back as a `software_cpu` boot with the next boot ordinal, which the check enforced; with `startMiningOnBoot` on, the device mined again with no resume command and accumulated 5 minutes, and with it off the next boot stayed paused for 2 minutes; attempt 001 had found a panic in the restart path, root-caused to Rust atomics on PSRAM and fixed (ADR-0039); see docs/parity/evidence/20261020-ultra205-boot-resume-check.md. Both restarts were requested while paused, so a restart while mining, power-loss behavior, the crash-loop guard on hardware and the remaining non-claims above still apply.

## Non-claims

Nothing beyond the redacted summaries is claimed. The status and the
evidence tokens are unchanged.
