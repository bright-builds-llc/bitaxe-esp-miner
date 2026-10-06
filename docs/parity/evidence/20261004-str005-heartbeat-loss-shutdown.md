# Heartbeat-loss revocation and bounded shutdown: verified

> Superseded as candidate evidence: `60e344e2` failed the internal-heap
> check after this run. Heartbeat010 repeated the check on the final
> candidate `2bff1004` ([evidence](20261005-str005-internal-heap-exhaustion.md)).
> This record stays as history for its own image.

Heartbeat008 sealed `complete=true` on the then-current lineage image `60e344e2`,
which includes the movsp-safe queue workaround and the BBPLL setting. The
Worker suppressed heartbeats once during a live, dispatching generation. The
device then revoked authority on its own, shut down within the native bounds,
cooled and restored, with exact accounting and complete host cleanup.

| Boundary     | Direct evidence                                                                                                                                                              |
| ------------ | ---------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Image        | Source `60e344e2`, ELF `3f01a5f4dea1676b505ea686a58d16f1bd2122138556b7ac1a1bbb4c43f4ac3d`, installed by usb-bbpll-install attempt-001; the native audits passed at preflight |
| Lineage      | Lineage head install, re-based by the sealed current recovery017: boot 301, ledger next 27, last 26, 3,180,000 ms                                                            |
| Start        | One attempt at ordinal 27, generation 4, zero renewals, local V2 fixture                                                                                                     |
| Fault        | One `suppressHeartbeats` during the dispatching generation, with 49,520 ms lease and 164,252 ms work-gate headroom                                                           |
| Revocation   | Native `revocation_reason=heartbeat_timeout`; the gate closed 2,803 ms after the last valid heartbeat (bound 2,800–3,000 ms)                                                 |
| Shutdown     | Initiated 2,814 ms after the last valid heartbeat (bound ≤ 3,000 ms), inside the 15,550-ms reserve                                                                           |
| Stop/cooling | `safe_stop_stage=fan_paused`, `safe_stop_complete=true`, 33 °C, `mine_on_boot=false`, not running                                                                            |
| Recovery     | Fresh recovery on the same boot 301 with no errors; ledger next 28, last 27, 3,360,000 ms (exactly +180,000), not pending                                                    |
| Observer     | Passive cadence observer: 38 messages, 8,280-ms tail, 47-ms cleanup                                                                                                          |
| Release      | Server, fixture, signer and serial owners released; port free; final detector admitted one Ultra 205                                                                         |
| Seal         | Result `fcbb3ce7767778183584a36dc23a1efcd31874b11ded3e518d562b015a0cbdae`, seal `68621dc4406ec506e4b067b0c1cdd5447439e206db331b50c7ce4cc215005b92`                           |

## History

- Heartbeat002 (old image `654338d0`) met every native bound (2,805/2,823 ms)
  but sealed unverified on a judge defect, which was fixed.
- Heartbeat005 and heartbeat006 were stopped by the idle-review panic. That
  panic was traced to a toolchain stack-realignment hazard and corrected
  ([evidence](20261004-str005-idle-panic-capture.md)).
- Heartbeat007 passed every criterion on the corrected image `7ca3e29c`
  (2,802/2,823 ms; ordinal 26; result
  `565857c2547b6b0af9ebd3bc483ec4997440b27038ed29895cd60d486a3acfae`).
- The queue workaround and the BBPLL setting then changed scheduling and
  queues, so heartbeat008 re-ran the check on `60e344e2` and passed.

## Non-claims

- This covers one generation on a local fixture. It is not evidence for
  sustained mining, a pool, an accepted share or parity promotion.
- Parity stays 90/95.

Raw evidence stays in protected private roots under
`scratch/str005-heartbeat-shutdown/heartbeat007` and `heartbeat008`. This summary carries only
timings, categories and digests.
