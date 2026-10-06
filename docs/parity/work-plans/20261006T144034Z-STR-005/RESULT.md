# Parity work result

- Parity row: `STR-005`
- Final status: `verified`
- Implementation commit: `2bff100442a7c5cbd4b806bdbbceff00f0c06a79`
- Reference commit: `c1915b0a63bfabebdb95a515cedfee05146c1d50`

## Evidence and verification

The final candidate is firmware `2bff1004`, app ELF
`9783dc74dfb369e633f9f3295021c34eefbd36fb50859a0d58a235447189b33a`, Gate
`86fc62d7`, on one Ultra 205 (BM1366) against the local Stratum V2 fixture.
The piecewise checkpoints of ADR-0029 cover it:

| Behaviour                                           | Evidence                                                                                                                                                                           |
| --------------------------------------------------- | ---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Install and update continuity, Noise authentication | psram-default-install attempt-002: five installs, four verified cycles, Noise exchange, restoration, cleanup ([projection](../../evidence/psram-default-install/attempt-002.json)) |
| Signed Start, dispatch, charged ledger              | heartbeat010 and renew-current-002, each exactly +180,000 ms                                                                                                                       |
| Accepted share with encrypted V2 submission         | renew-current-002: 1 submitted, 1 accepted, 0 rejected, independent joins verified                                                                                                 |
| Lease renewal                                       | renew-current-002: 2 renewals in the run proof and the recovered state                                                                                                             |
| Worker-requested normal Stop                        | renew-current-002 and share-current-001: `restoration_requested`, `fan_paused`                                                                                                     |
| Heartbeat-loss revocation and bounded shutdown      | heartbeat010: gate closed 2,808 ms and shutdown started 2,813 ms after the last heartbeat                                                                                          |
| Memory headroom                                     | idle and 59 minutes after heartbeat loss: at least 52,707 bytes free, 31,744-byte largest block                                                                                    |
| Native resources                                    | nine ELF-bound audits ([inventory](../../evidence/20261006-str005-negative-and-native-coverage.md))                                                                                |
| Negative paths                                      | about 480 host-side negative tests ([inventory](../../evidence/20261006-str005-negative-and-native-coverage.md))                                                                   |

Summaries: [integration review](../../evidence/20261005-str005-integration-review.md),
[share and renewal](../../evidence/20261005-str005-current-candidate-share.md),
[internal-heap correction](../../evidence/20261005-str005-internal-heap-exhaustion.md).

Commands run for this result (see `WORKLOG.md`): sealed-root inventory and
digest re-verification for every cited root, `just str005-lineage show`,
`just stratum-v2-noise-serial review` on install attempt-002, the ordered
Cargo checks, Bright Builds, `just verify-redaction` and `just parity`.

## Conclusion

Every STR-005 obligation transferred by ADR-0029 has complete,
independently reviewed evidence on the exact final candidate, or a recorded
owner decision: Share002's retained resource proof is a permanent non-claim
(ADR-0032), and single-run, single-board, local-fixture evidence qualifies
as `hardware-regression`. The row moves to `verified` with
`unit,golden,workflow,hardware-regression`.

## Non-claims and residual risks

- External or production pools, arbitrary pools, mixed-protocol live
  fallback, sustained or unbounded mining, other boards, OTA and release
  readiness are not claimed.
- Evidence is one run per checkpoint on one board against a local fixture.
- On hardware, only heartbeat loss exercises a negative path; rejected
  shares and refused renewals are covered by host-side tests only.
- Channel006 stays private; Share002's pre-reset resource release is never
  claimed; the original Share001/Share002 panics are "likely the same
  compiler bug, not proven".
- The candidate keeps the diagnostic end-of-stack watchpoint, per-command
  heap check and idle heap-sample line; removing them makes a new candidate.
