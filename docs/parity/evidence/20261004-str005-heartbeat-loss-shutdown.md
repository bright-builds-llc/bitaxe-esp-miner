# Heartbeat-loss revocation and bounded shutdown: verified

Heartbeat007 sealed `complete=true` on the corrected image. The Worker
suppressed heartbeats once during a live, dispatching generation. The device
then revoked authority on its own, shut down within the native bounds, cooled
and restored, with exact accounting and complete host cleanup.

| Boundary     | Direct evidence                                                                                                                                                                 |
| ------------ | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Image        | Source `7ca3e29c`, ELF `227bc380ec2d2171d187f8561390259fae464b92164d4d30c2a80354135eb3f0`, installed by realignment-fix attempt-001; the four native audits passed at preflight |
| Lineage      | Install-only: no earlier Start on this image; baseline on boot 5 with ledger next 26, last 25, 3,000,000 ms                                                                     |
| Start        | One `normal` attempt at ordinal 26, generation 12, zero renewals, local V2 fixture, conservative profile                                                                        |
| Fault        | One `suppressHeartbeats` after same-generation `work_ready` and `asic_dispatch`, with 49,011 ms lease and 163,711 ms work-gate headroom                                         |
| Revocation   | Native `revocation_reason=heartbeat_timeout`; the gate closed 2,802 ms after the last valid heartbeat (bound 2,800–3,000 ms)                                                    |
| Shutdown     | Initiated 2,823 ms after the last valid heartbeat (bound ≤ 3,000 ms), inside the 15,550-ms reserve; no work after revocation                                                    |
| Stop/cooling | `safe_stop_stage=fan_paused`, `safe_stop_complete=true`, 32 °C, `mine_on_boot=false`, not running                                                                               |
| Recovery     | Fresh recovery on the same boot 5 with no failures; ledger next 27, last 26, 3,180,000 ms (exactly +180,000), not pending                                                       |
| Observer     | Passive cadence observer: 38 messages, 8,328-ms tail, 136-ms cleanup; device, state, suppression and tail joins matched                                                         |
| Release      | Server, fixture, signer and serial owners released; port free; final detector admitted one Ultra 205                                                                            |
| Seal         | Result `565857c2547b6b0af9ebd3bc483ec4997440b27038ed29895cd60d486a3acfae`, seal `68350c6fc7d2521c37255db83d39e88e5c39d789b6d986e8f4a976af834bc11c`                              |

## History

- Heartbeat002 (old image `654338d0`) met every native bound (2,805/2,823 ms)
  but sealed unverified on a judge defect, which was fixed.
- Heartbeat005 and heartbeat006 were stopped by the idle-review panic. That
  panic was traced to a toolchain stack-realignment hazard and corrected
  ([evidence](20261004-str005-idle-panic-capture.md)).
- Heartbeat007 is the first attempt on the corrected image, and it passed
  every judge and inspector criterion.

## Non-claims

- This covers one generation on a local fixture. It is not evidence for
  sustained mining, a pool, an accepted share or parity promotion.
- Parity stays 90/95.

Raw evidence stays in protected private roots under
`scratch/str005-heartbeat-shutdown/heartbeat007`. This summary carries only
timings, categories and digests.
