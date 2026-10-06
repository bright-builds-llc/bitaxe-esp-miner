# STR-005 integration review

Status: in progress. The obligation map and change-impact review are
complete. The first candidate (`60e344e2`) failed: its restart before the
share check found internal-heap exhaustion. The corrected PSRAM-first
candidate (`2bff1004`) has passed install continuity, idle and
post-shutdown headroom, heartbeat-loss shutdown, an accepted share with a
normal Stop, and an accepted share with two renewals in one complete run.
The independent evidence review found the numbers correct; the remaining
promotion blockers are listed below. Parity stays 90/95; any transition belongs to
`task-str005-evidence-promotion`.

Policy: [ADR-0029](../../adr/0029-piecewise-str005-qualification.md), with
prospective diagnostics under
[ADR-0031](../../adr/0031-prospective-panic-diagnostics.md). Checklist row:
STR-005 in [the parity checklist](../checklist.md), currently `implemented`.

## Final candidate

The candidate is the verified lineage head (`scripts/str005-lineage/head.json`):

| Item     | Identity                                                                  |
| -------- | ------------------------------------------------------------------------- |
| Firmware | `2bff100442a7c5cbd4b806bdbbceff00f0c06a79`                                |
| App ELF  | `9783dc74dfb369e633f9f3295021c34eefbd36fb50859a0d58a235447189b33a`        |
| Gate     | `86fc62d7a9d75da1affa2d51bc3b9eab41d86031`                                |
| Fixture  | The canonical stamped V2 fixture built from the same clean source         |
| Install  | `psram-default-install` attempt-002 (five installs, four verified cycles) |

The candidate keeps the diagnostic end-of-stack watchpoint, the
per-command heap check and the idle `internal_heap_sample` line. Every
candidate result was measured with them, so removing them would be a new
candidate and would reopen this review.

## Obligations and covering results

| Obligation                                      | Covering result on the candidate                                                                                                         | Older results (kept on their own identities)               |
| ----------------------------------------------- | ---------------------------------------------------------------------------------------------------------------------------------------- | ---------------------------------------------------------- |
| Noise authentication, encrypted proof           | psram-default-install attempt-002 (all criteria true)                                                                                    | usb-bbpll-install (`60e344e2`), Noise-serial (`ad629679`)  |
| Channel and job dispatch over V2                | Heartbeat010 (`work_ready`, `asic_dispatch`, same generation)                                                                            | heartbeat008 (`60e344e2`), Share001 (`654338d0`)           |
| Fresh admission, signed Start, charged ledger   | Heartbeat010 (ordinal 29, exactly +180,000 ms, not pending)                                                                              | heartbeat008, startup003, Share001                         |
| Heartbeat revocation, bounded shutdown, cooling | Heartbeat010 (2,808/2,813 ms)                                                                                                            | [heartbeat008](20261004-str005-heartbeat-loss-shutdown.md) |
| Internal-memory headroom                        | [Idle and post-shutdown captures](20261005-str005-internal-heap-exhaustion.md)                                                           | the diagnosis image (fails)                                |
| ASIC result, encrypted submission, device ACK   | [Share-current-001](20261005-str005-current-candidate-share.md) (1 submitted, 1 accepted, 0 rejected)                                    | [Share001](20261003-str005-accepted-share.md) (`654338d0`) |
| Renewal                                         | [Renew-current-002](20261005-str005-current-candidate-share.md) (complete: 2 renewals plus an accepted share and normal Stop in one run) | Share001 (one renewal); renew-current-001 (`unverified`)   |
| Worker-requested normal Stop while mining       | Share-current-001 (`restoration_requested`, `fan_paused`)                                                                                | startup003 (`361425b9`), Share001                          |
| Restoration, settings and identity preservation | Heartbeat010 recovery; psram-default-install attempt-002                                                                                 | every complete result                                      |
| Install/update continuity                       | psram-default-install attempt-002                                                                                                        | usb-bbpll-install, Channel006, Noise-serial                |
| Host, serial, signer and fixture cleanup        | Heartbeat010; psram-default-install attempt-002                                                                                          | every complete result                                      |
| Privacy                                         | `just verify-redaction` (JSON only) plus a manual review of the Markdown summaries                                                       | —                                                          |
| Fixture correctness                             | Share-current-001's independent header, target, nonce and ACK joins (`accepted_share_verified=true`)                                     | Share001 (`654338d0`)                                      |
| Negative tests                                  | **Not yet audited:** software tests exist per probe (share, heartbeat, normal stop, recovery), without an inventory                      | not hardware evidence                                      |

## Change impact since each result

- **`60e344e2` → candidate:** the idle heap sample (`f7999faf`), the
  PSRAM-first allocation policy (`92f58abb`) and a host fixture argv fix
  (`2bff1004`). Install continuity and heartbeat-loss shutdown re-ran on the
  exact candidate, so nothing from `60e344e2` is reused.
- **Share001 (`654338d0`) → candidate:** the stack-realignment fix
  (`24af10be`), the diagnostic watchpoint and heap check (`c634cc20`), the
  queue workaround (`00f84eae`), `#[inline(never)]` on the mining-session
  start (`9386999d`), the BBPLL setting (`2b784c49`) and the changes above.
  The ASIC result, V2 transport and session queues on the submit and ACK
  path changed, and ordinary allocations moved to PSRAM. `bitaxe-stratum`,
  `bitaxe-asic`, `bitaxe-worker-control`, `Cargo.lock` and the toolchain did
  not. The impact was **uncertain**, so the checks were repeated on the
  candidate: share-current-001 (submission, acknowledgement, normal Stop)
  and renew-current-001 (Renew).
- **startup003 (`361425b9`) → candidate:** superseded by Share001 and
  heartbeat010. Its one uncovered path, a Worker-requested Stop while
  mining, ran in share-current-001.
- **Channel006, Noise-serial and reflash preservation → candidate:**
  USB-adjacent startup clocking (BBPLL) and the allocation policy changed;
  ADR-0029 requires a continuity reassessment, which psram-default-install
  attempt-002 supplies on the exact candidate. The USB serial writer gained
  the idle heap-sample line (`f7999faf`); the USB controller, descriptors,
  partitions and packaging did not change.

## Resolved: internal-heap exhaustion on `60e344e2`

Restart006 panic-rebooted on a Worker connection after heartbeat008. The
[diagnosis](20261005-str005-internal-heap-exhaustion.md) found persistent
internal-RAM exhaustion, not a leak. The PSRAM-first candidate keeps about
52 KB free with a 31 KB largest block, idle and for an hour after a
heartbeat-loss session.

## Provenance

| Result                | Result digest                                                      | Seal digest                                                        |
| --------------------- | ------------------------------------------------------------------ | ------------------------------------------------------------------ |
| Install (attempt-002) | `45db405cf1ad54e6fb9a75dff3c92cc2b46e7a8c41ac61027f9d741a504b38c8` | `3998bc370ec653be0cbf943808b0ad3052d7762885eb94b80ba58588e08cb035` |
| Heartbeat010          | `8fc48d2f283b10aef8f6ceca0d3d284ac9170a44d834426c263702a8283049de` | `0e23e10f91fb539e433a0204ee841cd410674b01aceaad544d2bddf4d0d8c958` |
| Restart007            | `ef87f9a9e68c5936c954489022b47aab766023dacd75e7129e7e05e2a1051b00` | `b87302e90848062eff3b673d872e501591c340950122b7625ad29dbbfdc28620` |
| Share-current-001     | `0227a65e35eb7e11778d39ec44dc2520d9e4e23f5772a45db77d498eae29fe00` | `2d0d31b9976acdd7d1374d4ad3b389fca140d3189d1b66eefa13144836869116` |
| Restart008            | `003683ef76032c87ba2ca632e9988dec4dd6d6f824c03341e364d70f4b5d9195` | `764cbaa7026f253cfad5453c58586e5125098b953c22a51ac21ff1b47ed99f70` |
| Renew-current-001     | `81ae7513e93a85c42c753a725356c0635176958203cdc63761a0fc26fc0fec89` | `998da7e265215bc5004b2d46f4824ca69936e70a780dbedd91c1986c96f8f91d` |
| Restart009            | `f736af76046fb12a57777647cab3794d810540862f991ebecd1da667e1ee5451` | `6cee41efcdfd6492640a02fa75cc0e1e305e39c6a2b419a27e83d704b30538af` |
| Renew-current-002     | `3e2c282b7b8d5cf93b0b0ababcd9ee0d9c262afba1c83c2a3e07b0559af758e4` | `7f691b80ad694112bcc2233f42ac783053c4feb56698ffbe45842e21cb85cf82` |

## Independent review

An independent review (2026-10-05) re-derived every digest, ordinal,
generation, ledger, boot, timing, heap statistic and ELF/manifest identity
above from the sealed private roots and found them correct. It also
confirmed the change-impact commit lists and found no private values in the
committed evidence. Its wording findings are fixed here and in the linked
summaries.

Eligible to hand to promotion: both install projections, share-current-001,
heartbeat010, restart007 and restart008, and the candidate heap captures as
summarised facts. Heartbeat008 and Share001 count only as older-image
history. Must stay private: Channel006, every failed or unverified raw root
(including psram-default-install attempt-001) and Share002.

## Promotion blockers

1. **Renewal coverage.** Resolved 2026-10-05: renew-current-002 sealed
   complete with an accepted share, two renewals and a normal Stop in one
   run, under an 80-second window that stays inside the renewed lease.
2. **Share002 retained resource proof.** Resolved 2026-10-05: the owner
   accepted it as a permanent non-claim
   ([ADR-0032](../../adr/0032-share002-resource-proof-non-claim.md)).
   `task-str005-failure-recovery-accounting` is complete and archived on what
   it measured; nothing about Share002's pre-reset resource release is
   claimed.
3. **Evidence class.** Promotion would add `hardware-regression`, but the
   evidence is one run per checkpoint, on one board, against a local
   fixture. Whether that meets the class is an owner decision.
4. **Negative-test and native-coverage audit.** ADR-0029 requires an
   inventory of the negative tests and native resource checks behind each
   probe; it is not done yet.

## Open items

1. **Channel006 publication.** Its result stays private. The candidate does
   not depend on it.
2. **Panic attribution.** The original Share001 and Share002 panics are
   "likely the same compiler bug, not proven", as accepted by the owner.
   The restart006 panic site is inferred from heap evidence, not captured.

## Naming

"Share001" names two results. The original failed attempt on `f000872f`
panicked. The accepted result on `654338d0` (seal `abde26a6…`) is the one
cited here.

## Non-claims

No pool, sustained or unbounded mining. Software negative tests are not
hardware evidence. Raw evidence stays in protected private roots.
