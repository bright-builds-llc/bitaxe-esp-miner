# Accepted share and renewal on the final candidate

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

## Renewal on the candidate

Share-current-001 confirmed no renewal, so a renewal probe
(renew-current-001, ordinal 31, after restart008: boot 319 to 320) held its
generation for the Gate's renewals:

- **Renewals.** The Gate confirmed 2 renewals. It counts a renewal only
  after the device's validated, signed renewal acknowledgement, so the
  firmware's Renew path ran twice on `2bff1004`.
- **No share.** The ASIC returned 10 nonces at its own difficulty, but none
  met the fixture's 1024 target before the 45-second deadline (roughly an
  8% outcome at this rate). The probe therefore sealed `unverified` with
  first failure `share`. The contract allows no retry of a share timeout
  without a regression-backed change, so none was made.
- **Stop and accounting.** Normal Stop (`restoration_requested`,
  `fan_paused`); the device record admitted, dispatched, revoked, shut down
  and cooled; fresh recovery with no failures; ledger 32/31/4,080,000 ms;
  the final detector admitted one Ultra 205.
- **Seal.** Result `81ae7513e93a85c42c753a725356c0635176958203cdc63761a0fc26fc0fec89`,
  seal `998da7e265215bc5004b2d46f4824ca69936e70a780dbedd91c1986c96f8f91d`.

The share and the renewal are therefore proven on the candidate in two
separate runs, not in one.

## Non-claims

One generation on a local fixture. Not pool, sustained-mining or
parity-promotion evidence. Raw evidence stays in protected private roots
under `scratch/str005-accepted-share/share-current-001`.
