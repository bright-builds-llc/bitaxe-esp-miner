# STR-005 integration review

Status: in progress. The obligation map and change-impact review are
complete. The first candidate (`60e344e2`) failed: its restart before the
share check found internal-heap exhaustion. The corrected PSRAM-first
candidate (`2bff1004`) has passed install continuity, idle and
post-shutdown headroom, heartbeat-loss shutdown and an accepted share with
a normal Stop. One gap remains on the candidate: no renewal was exercised.
The independent evidence and publication review is next. Parity stays
90/95; any transition belongs to `task-str005-evidence-promotion`.

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

| Obligation                                      | Covering result on the candidate                                                                      | Older results (kept on their own identities)               |
| ----------------------------------------------- | ----------------------------------------------------------------------------------------------------- | ---------------------------------------------------------- |
| Noise authentication, encrypted proof           | psram-default-install attempt-002 (all criteria true)                                                 | usb-bbpll-install (`60e344e2`), Noise-serial (`ad629679`)  |
| Channel and job dispatch over V2                | Heartbeat010 (`work_ready`, `asic_dispatch`, same generation)                                         | heartbeat008 (`60e344e2`), Share001 (`654338d0`)           |
| Fresh admission, signed Start, charged ledger   | Heartbeat010 (ordinal 29, exactly +180,000 ms, not pending)                                           | heartbeat008, startup003, Share001                         |
| Heartbeat revocation, bounded shutdown, cooling | Heartbeat010 (2,808/2,813 ms)                                                                         | [heartbeat008](20261004-str005-heartbeat-loss-shutdown.md) |
| Internal-memory headroom                        | [Idle and post-shutdown captures](20261005-str005-internal-heap-exhaustion.md)                        | the diagnosis image (fails)                                |
| ASIC result, encrypted submission, device ACK   | [Share-current-001](20261005-str005-current-candidate-share.md) (1 submitted, 1 accepted, 0 rejected) | [Share001](20261003-str005-accepted-share.md) (`654338d0`) |
| Renewal                                         | **Gap: share-current-001 confirmed 0 renewals**                                                       | Share001 (one renewal)                                     |
| Worker-requested normal Stop while mining       | Share-current-001 (`restoration_requested`, `fan_paused`)                                             | startup003 (`361425b9`), Share001                          |
| Restoration, settings and identity preservation | Heartbeat010 recovery; psram-default-install attempt-002                                              | every complete result                                      |
| Install/update continuity                       | psram-default-install attempt-002                                                                     | usb-bbpll-install, Channel006, Noise-serial                |
| Host, serial, signer and fixture cleanup        | Heartbeat010; psram-default-install attempt-002                                                       | every complete result                                      |
| Privacy                                         | Per-result redaction; `just verify-redaction` on every publication                                    | —                                                          |
| Fixture correctness                             | Share001's independent header, target, nonce and ACK joins                                            | —                                                          |
| Negative tests                                  | Software only, per probe (share, heartbeat, normal stop, recovery)                                    | not hardware evidence                                      |

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
  not. Submission, acknowledgement, Renew and a Worker-requested Stop while
  mining were never run on the candidate. The impact is **uncertain**, so
  the check is repeated.
- **startup003 (`361425b9`) → candidate:** superseded by Share001 and
  heartbeat010. Its one uncovered path, a Worker-requested Stop while
  mining, is part of the pending share check.
- **Channel006, Noise-serial and reflash preservation → candidate:**
  USB-adjacent startup clocking (BBPLL) and the allocation policy changed;
  ADR-0029 requires a continuity reassessment, which psram-default-install
  attempt-002 supplies on the exact candidate. USB controller, descriptors,
  partitions and packaging did not change.

## Resolved: internal-heap exhaustion on `60e344e2`

Restart006 panic-rebooted on a Worker connection after heartbeat008. The
[diagnosis](20261005-str005-internal-heap-exhaustion.md) found persistent
internal-RAM exhaustion, not a leak. The PSRAM-first candidate keeps about
52 KB free with a 31 KB largest block, idle and for an hour after a
heartbeat-loss session.

## Open items

1. **Renewal on the candidate.** Share-current-001 acknowledged its share
   before the first 20-second renewal. Covering Renew on this image needs a
   probe that holds the Start until one renewal is confirmed; that is a new
   bounded contract, not a re-run.
2. **Share002 retained resource proof.** It cannot be supplied: the device
   reports idle status with no retained record. This stays an explicit
   blocker on `task-str005-failure-recovery-accounting`. It does not block
   other claims (ADR-0029, ADR-0031).
3. **Channel006 publication.** Its result stays private. The candidate does
   not depend on it.
4. **Panic attribution.** The original Share001 and Share002 panics are
   "likely the same compiler bug, not proven", as accepted by the owner.
   The restart006 panic site is inferred from heap evidence, not captured.

## Naming

"Share001" names two results. The original failed attempt on `f000872f`
panicked. The accepted result on `654338d0` (seal `abde26a6…`) is the one
cited here.

## Non-claims

No pool, sustained or unbounded mining. Software negative tests are not
hardware evidence. Raw evidence stays in protected private roots.
