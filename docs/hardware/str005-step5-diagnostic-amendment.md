# STR-005 step-5 diagnostic Start amendment

## Status and precedence

Owner-authorized on 2026-10-02 under `task-str005-step5-revocation-detail`.
Contract ID: `str005-noise-serial-v2`, successor profile `step5-diagnostic-install`,
followed by one diagnostic Start owned by `scripts/str005-step5-diagnostic`.

This amendment defines two bounded phases:
- an install phase that reuses the noise-serial harness;
- a single-Start phase that reuses the status-repro Start, status and Stop
  machinery.

It overrides only the requirements named below. The archived status001 and
recovery006 results and every earlier seal keep their meaning. Parity stays
90/95, and the accepted-share task remains active.

## Why

status001 was revoked as `unsafe_observation` during preparation step 5
(500 ms core-voltage stabilization). The trigger and the Gate's rejection
category were not preserved. Firmware `99081666`/`43c687c3` now records the
first failing safety fact, its state, value and age, and which check revoked:
an unsafe sample, a zero fan, or no safe sample for over one second. It emits
this as `worker_revocation_detail`. Gate `86fc62d7` (ADR-0102) parses the
line. The harness keeps that row, the preparation receipt, `control_failure`
rows and the Worker's rejection category in private evidence.

## Phase 1: install (noise-serial profile `step5-diagnostic-install`)

| Requirement                     | Meaning                                                                                                                                                                                                                                                                                                      |
| ------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Predecessor                     | The sealed passing `control-stack-port-reuse` attempt-001: result SHA-256 `973ee99497104eb9f9b74a9c947a09f8f2983b5373d496621b427a0ce16babc7`, seal `e84ad6b0132774a512491e4882c13c9f4ac3d27378a286130cf5ee0b3f9c902c`.                                                                                      |
| Before identity and ledgers     | Source `2bd65aa033172a041cb139062b9c87f00686bd36`, ELF `bbd4500d60b4e666b95cc92b0a049114bd3860a8c955774c40e3f023c8290b71`. Expected idle ledger next 22, last 21, charged 2,280,000 ms, with the original budget exhausted. Both must be observed unchanged before and after.                                  |
| Attempt namespace and task gate | Private roots live under `scratch/str005-step5-install/`. Preflight requires `task-str005-step5-revocation-detail` under `## Active` with the exact line `Step-5 diagnostic install hardware: enabled.` A pass publishes to `docs/parity/evidence/str005-step5-install/`.                                       |

Everything else is as the helper and control-stack amendments require:
- the state-preserving install and four continuity cycles (at most five writes)
  and the network-only Noise diagnostic;
- no mining, Work Lease, grant, signer, pool credentials, Wi-Fi provisioning,
  factory reset, erase or rollback;
- Connects that reuse the single granted port;
- mandatory `finalize` and `review`.

## Phase 2: one diagnostic Start (`scripts/str005-step5-diagnostic`)

Admission: the compiled `ENABLED` flag, the exact active task line
`Step-5 diagnostic Start hardware: enabled.`, and clean pushed source. The
pinned phase-1 seal and result, and its installed identity, Gate and physical
identity, must match. Preflight also requires:
- the fixture identity, Gate timer compatibility and native USB symbols;
- the cutoff, store, signed Start and fault-provenance audits on the sealed
  installed ELF;
- a fresh detector with no serial holder.

Effects, all bounded by the existing status-repro limits:
- One fresh `normal` attempt at the measured next ordinal (22), with an
  initial 60,000-ms window, renewal after 20,000 ms and zero renewal
  signatures. The ledger charge is 180,000 ms, so the expected after ledger
  is next 23, last 22, charged 2,460,000 ms.
- Conservative profile 400 MHz / 1,100 mV with the fan at 100%, against one
  local Stratum V2 fixture on a private IPv4.
- The Start reply within 30 s and one known-attempt status within 10 s. An
  independent Stop no later than 15 s after the reply and 45 s after
  invocation. Stop and Close stay unconditional.
- Then fresh authenticated recovery: ledger, original budget, status,
  diagnostics, restoration and closure.

The page is served on `127.0.0.1:48765`, the origin that holds the Ultra 205
grant, so Connect reuses the granted port.

Evidence: the private root keeps the status-repro records plus:
- the diagnostics projection with boot, `worker_revocation_detail`,
  `worker_preparation_receipt`, `control_failure` and `worker_admission` rows;
- `str005-client-failure-v2` with the Worker's closed rejection.

Raw device, signer and fixture inputs stay in ignored 0700 roots with 0600
files. Committed findings are redacted categories only.

Outcomes:
- **Revocation diagnosed**: the Start is rejected or revoked, and a
  revocation detail for the attempt's generation is retained. This resolves
  the diagnostic gap. A fix needs a new task.
- **Step 5 passed**: the Start was observed running and normally stopped. The
  boundary did not recur, and this is still not share evidence.
- **Unverified**: neither of the above, or missing cleanup.

## Prohibited

- pool credentials, Wi-Fi provisioning, factory reset, erase or rollback;
- any flash in phase 2;
- renewals, a second Start, or a replayed grant;
- direct UART or pin access, network discovery;
- synthesized permission gestures;
- publishing raw private evidence.

## Recovery, retry and stop

Recovery: only normal Stop/Close, fresh possession and recovery collection. The
device's own safe stop removes core voltage and ASIC power on any revocation.
If restoration fails, collect the bounded safe observations, release host owners
and stop.

Retry: ordinal 1 of each phase only, with no unchanged retry.

Stop on:
- a detector result other than exactly one admitted Ultra 205;
- board-info, identity, ledger or baseline drift;
- an install failure;
- a lost or ambiguous Start (never resend);
- a new panic;
- unproven cleanup.

## Non-claims

No phase claims an accepted share, renewed mining, heartbeat-loss behavior or
parity promotion. The revocation detail is a local observation; it does not by
itself prove an electrical cause.
