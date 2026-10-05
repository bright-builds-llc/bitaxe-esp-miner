# STR-005 integration review

Status: in progress. The obligation map and change-impact review are
complete. One bounded accepted-share check on the final candidate is pending
(`task-str005-share-current-image`). Parity stays 90/95; any transition
belongs to `task-str005-evidence-promotion`.

Policy: [ADR-0029](../../adr/0029-piecewise-str005-qualification.md), with
prospective diagnostics under
[ADR-0031](../../adr/0031-prospective-panic-diagnostics.md). Checklist row:
STR-005 in [the parity checklist](../checklist.md), currently `implemented`.

## Final candidate

The candidate is the verified lineage head (`scripts/str005-lineage/head.json`):

| Item     | Identity                                                              |
| -------- | --------------------------------------------------------------------- |
| Firmware | `60e344e21a89acab13c108311d315417c0704eba`                            |
| App ELF  | `3f01a5f4dea1676b505ea686a58d16f1bd2122138556b7ac1a1bbb4c43f4ac3d`    |
| Gate     | `86fc62d7a9d75da1affa2d51bc3b9eab41d86031`                            |
| Fixture  | `66b6659b…` (the canonical V2 fixture, unchanged since Share001)      |
| Install  | `usb-bbpll-install` attempt-001 (five installs, four verified cycles) |

The candidate keeps the diagnostic end-of-stack watchpoint and the
per-command heap check. Every candidate result was measured with them, so
removing them would be a new candidate and would reopen this review.

## Obligations and covering results

| Obligation                                      | Covering result on the candidate                                            | Older results (kept on their own identities)               |
| ----------------------------------------------- | --------------------------------------------------------------------------- | ---------------------------------------------------------- |
| Noise authentication, encrypted proof           | usb-bbpll-install attempt-001 (all 12 criteria true)                        | Noise-serial attempt-001 (`ad629679`)                      |
| Channel and job dispatch over V2                | Heartbeat008 (`work_ready`, `asic_dispatch`, same generation)               | Share001 (`654338d0`), Channel006 (private)                |
| Fresh admission, signed Start, charged ledger   | Heartbeat008 (ordinal 27, exactly +180,000 ms, not pending)                 | startup003, Share001, heartbeat007                         |
| Heartbeat revocation, bounded shutdown, cooling | [Heartbeat008](20261004-str005-heartbeat-loss-shutdown.md) (2,803/2,814 ms) | heartbeat007 (`7ca3e29c`)                                  |
| ASIC result, encrypted submission, device ACK   | **Pending: share-current-001**                                              | [Share001](20261003-str005-accepted-share.md) (`654338d0`) |
| Renewal                                         | **Pending: share-current-001, only if a renewal is confirmed**              | Share001 (one renewal)                                     |
| Worker-requested normal Stop while mining       | **Pending: share-current-001**                                              | startup003 (`361425b9`), Share001                          |
| Restoration, settings and identity preservation | Heartbeat008 recovery; usb-bbpll-install attempt-001                        | installation005, every complete result                     |
| Install/update continuity                       | usb-bbpll-install attempt-001; queue-workaround reinstall (`6f268518`)      | Channel006, Noise-serial                                   |
| Host, serial, signer and fixture cleanup        | Heartbeat008; usb-bbpll-install attempt-001                                 | every complete result                                      |
| Privacy                                         | Per-result redaction; `just verify-redaction` on every publication          | —                                                          |
| Fixture correctness                             | Share001's independent header, target, nonce and ACK joins; same fixture    | —                                                          |
| Negative tests                                  | Software only, per probe (share, heartbeat, normal stop, recovery)          | not hardware evidence                                      |

## Change impact since each result

- **Heartbeat007 → candidate:** the queue workaround (`00f84eae`),
  `#[inline(never)]` on the mining-session start (`9386999d`) and the BBPLL
  setting (`2b784c49`). Heartbeat008 re-ran on the exact candidate, so
  nothing is reused.
- **Share001 (`654338d0`) → candidate:** the three changes above, plus the
  stack-realignment fix (`24af10be`: one-shot replies for Start, Renew,
  SafeStop, cooling, USB writes and safety actuation) and the diagnostic
  watchpoint and heap check (`c634cc20`). The ASIC result, V2 transport and
  session queues on the submit and ACK path all changed. The fixture,
  `bitaxe-stratum`, `bitaxe-asic`, `bitaxe-worker-control`, `Cargo.lock` and
  the toolchain did not. The changes are mechanical, but submission,
  acknowledgement, Renew and a Worker-requested Stop while mining were never
  run on the candidate. The impact is **uncertain**, so the check is
  repeated.
- **startup003 (`361425b9`) → candidate:** 21 commits, including Renew
  isolation, the Noise helper stack, revocation refactors and the changes
  above. Superseded by Share001 and heartbeat008. Its one uncovered path, a
  Worker-requested Stop while mining, is part of the pending share check.
- **Channel006, Noise-serial and reflash preservation → candidate:** about
  27 to 29 commits, including USB-adjacent startup clocking (BBPLL). ADR-0029
  requires a continuity reassessment for that; usb-bbpll-install attempt-001
  is a full four-cycle continuity result on the exact candidate. USB
  controller, descriptors, partitions and packaging did not change.

## Open items

1. **Accepted share, renewal and normal Stop on the candidate.** One
   bounded check: restart006 after heartbeat008, then share-current-001
   ([contract](../../hardware/str005-accepted-share-amendment.md),
   "Current-image re-run").
2. **Share002 retained resource proof.** It cannot be supplied: the device
   reports idle status with no retained record. This stays an explicit
   blocker on `task-str005-failure-recovery-accounting`. It does not block
   other claims (ADR-0029, ADR-0031).
3. **Channel006 publication.** Its result stays private. The candidate does
   not depend on it.
4. **Panic attribution.** The original Share001 and Share002 panics are
   "likely the same compiler bug, not proven", as accepted by the owner.

## Naming

"Share001" names two results. The original failed attempt on `f000872f`
panicked. The accepted result on `654338d0` (seal `abde26a6…`) is the one
cited here.

## Non-claims

No pool, sustained or unbounded mining. Software negative tests are not
hardware evidence. Raw evidence stays in protected private roots.
