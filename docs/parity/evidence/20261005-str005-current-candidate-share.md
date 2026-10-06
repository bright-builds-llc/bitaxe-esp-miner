# Accepted share on the final candidate: verified

Share-current-001 sealed `complete=true` with `accepted_share_verified=true`
on the PSRAM-first candidate. It repeats Share001's check after the queue,
reply, clock and allocation changes, under the
[accepted-share amendment](../../hardware/str005-accepted-share-amendment.md),
"Current-image re-run".

| Boundary | Direct evidence                                                                                                                                            |
| -------- | ---------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Image    | Source `2bff1004`, ELF `9783dc74dfb369e633f9f3295021c34eefbd36fb50859a0d58a235447189b33a`, installed by psram-default-install attempt-002; Gate `86fc62d7` |
| Lineage  | Heartbeat010 (ordinal 29), then restart007: boot 318 to 319, ledger 30/29/3,720,000 ms unchanged                                                           |
| Start    | One `normal` attempt at ordinal 30, generation 3, local V2 fixture, conservative profile; Start reply after about 9.7 s                                    |
| Share    | One BM1366 result submitted over the encrypted V2 channel and accepted: 1 submitted, 1 accepted, 0 rejected; the judge's independent joins held            |
| Stop     | Worker-requested Stop about 17.2 s after the reply: `restoration_requested`, `fan_paused`, 33 °C, `mine_on_boot=false`                                     |
| Renewals | 0 confirmed: the acknowledgement arrived before the first 20-second renewal                                                                                |
| Recovery | Fresh recovery on boot 319 with no failures; ledger 31/30/3,900,000 ms (exactly +180,000), not pending                                                     |
| Release  | Server, fixture, signer and serial owners released; port free; final detector admitted one Ultra 205                                                       |
| Seal     | Result `0227a65e35eb7e11778d39ec44dc2520d9e4e23f5772a45db77d498eae29fe00`, seal `2d0d31b9976acdd7d1374d4ad3b389fca140d3189d1b66eefa13144836869116`         |

## Gap

The Renew path is not covered on this image: no renewal was confirmed.
Share001 (`654338d0`) covered one renewal on its own image. The integration
review keeps this as an explicit gap.

## Non-claims

One generation on a local fixture. Not pool, sustained-mining or
parity-promotion evidence. Raw evidence stays in protected private roots
under `scratch/str005-accepted-share/share-current-001`.
