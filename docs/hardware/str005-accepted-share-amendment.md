# STR-005 accepted-share probe amendment

## Status and precedence

Owner-authorized on 2026-10-03 under `task-str005-v2-accepted-share-probe`. The
owner also authorized autonomous, iterative fixes and fresh attempts. This
amendment defines one bounded accepted-share Start, preceded by one no-mining
qualification restart. It reuses the
[share-probe contract](../../scripts/str005-share-probe/CONTRACT.md) semantics.
It replaces only the admission chain, the page origin and the install. The
original Share001 seal, status001, recovery006 and every earlier result keep
their meaning. Parity stays 90/95 unless the separate integration review
promotes it.

## Basis

- **Firmware.** `654338d0`/`2641c24f`, installed and qualified by the step-5
  reinstall (result `04f2f8d1…`, seal `0751d602…`). Every phase-2 native audit
  passed on it, and two diagnostic Starts on it passed preparation step 5. The
  second, start004 (result `4ea90e4f…`, seal `d37808f8…`), sealed complete with
  the ledger at next 24, last 23, 2,640,000 ms.
- **V2 channel path.** The reinstall's accepted network-only Noise diagnostic
  and start004's running share-scope status exercised the encrypted V2 channel
  on this exact image. This applicability review does not relabel Channel006
  as evidence for this image.
- **Submission filter.** Firmware submits only results that meet the pool
  target and are not duplicates (`crates/bitaxe-stratum/src/v2/session.rs`),
  so the difficulty-1024 fixture never sees a sub-target share.

## Phase A: restart002 (`just str005-step5-restart`)

The retained terminal V2 record of start004 clears only on reboot. One no-mining
`qualificationRestart` follows the step-5 restart rules, with start004 as its
pinned parent:
- stage `recovery` reads the record freshly by ID;
- stage `restart` makes exactly one software restart, keeps the ledger and
  budget unchanged and proves boot N+1, served on `127.0.0.1:48765`.
The pre-reset admission marker may show start004's completed attempt
(`prior_attempt_completed`). The task line is
`Accepted share restart hardware: enabled.`

## Phase B: one accepted-share Start (`just str005-accepted-share`)

Admission requires:
- the compiled `ENABLED` flag, the exact active task line
  `Accepted share probe hardware: enabled.`, and clean pushed source;
- the pinned chain of reinstall, then start004, then the sealed restart002;
- the four native audits on the installed ELF, Gate `86fc62d7` compatibility,
  the canonical fixture, and a detector at most 60 s old with no holder.

Effects, from the share-probe contract:
- One fresh `normal` attempt at the measured next ordinal (24): a
  180,000-ms reservation, a 60,000-ms initial lease and at most 2 same-lease
  renewals. Renewals are signed at issuance and executed only by the Gate's
  timer.
- The conservative profile only: 400 MHz / 1,100 mV with the fan at 100%.
- One local Stratum V2 fixture at difficulty 1024 on a private IPv4.
- The Start reply within 30 s. While running, the page polls known-attempt
  status every 200 ms and requests Stop as soon as a native acknowledgement
  joins an atomically persisted fixture acceptance. An independent timer
  stops no later than 45 s after the reply.
- Then fresh-possession recovery on the same page.

The expected after ledger is next 25, last 24, 2,820,000 ms.

A pass requires every share-probe judge criterion:
- an independently recomputed header, target, nonce, submission and native
  acknowledgement join;
- a normal Stop with terminal resources released;
- the authorization recovery match for the Start's generation;
- the 180,000-ms charge with nothing pending;
- 0 to 2 confirmed renewals;
- fixture natural completion (exit 0 after one accepted share and peer EOF);
- host, serial and fixture release.

Any shortfall seals `unverified`. Running out of time without a share is not
retry authority without a regression-backed change.

## Prohibited

- an external pool or pool credentials, Wi-Fi provisioning, or any firmware
  write;
- an NVS or factory reset, erase or rollback;
- a voltage or frequency override, a third renewal, a second Start, an extra
  reservation or a budget reset;
- replay of a historical grant;
- direct UART or pin access, network discovery;
- synthesized permission gestures;
- publishing raw private evidence.

## Recovery, retry and stop

Recovery uses only the normal Stop/Close path, a fresh-possession recovery and
host cleanup. The device's safe stop removes core voltage and ASIC power on any
revocation.

Retries need a targeted, regression-backed fix, a fresh ordinal and, before any
further Start, a new restart.

Stop on:
- a detector result other than exactly one Ultra 205;
- identity, ledger or baseline drift;
- a lost or ambiguous Start (never resend);
- a new panic;
- a repeated boundary after its fix;
- unproven cleanup.

## Non-claims

A pass proves one device-acknowledged accepted share on a local fixture with
normal restoration. It is not pool-mining, heartbeat-loss, sustained-mining or
parity-promotion evidence.
