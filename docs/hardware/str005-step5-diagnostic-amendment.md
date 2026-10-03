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

## Phase 1b: reinstall (noise-serial profile `step5-diagnostic-reinstall`)

Phase 1 attempt-003 passed, but its firmware failed the phase-2 native panic
cutoff audit (`native_generation_revoke`). The new revocation field had let
the compiler reorder `GenerationGate` fields, which moved the cutoff's `state`
word off the gate symbol's address. Firmware now declares `GenerationGate`
`#[repr(C)]` with a regression for offset 0. The Start owner admits only an
install that passes every phase-2 audit, so the corrected firmware is
reinstalled under successor profile `step5-diagnostic-reinstall`.

| Requirement                     | Meaning                                                                                                                                                                                                                                                                            |
| ------------------------------- | ---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Predecessor                     | The sealed passing `step5-diagnostic-install` attempt-003: result SHA-256 `ed497f6142a4a6574fe4b80e7ed2944ac6a9781e25778a22b41e483c956c1ee5`, seal `b5ccca60868dfee957bfef87f63a51da4f116a36f0bd95cf7793d5139b16259b`.                                                          |
| Before identity and ledgers     | Source `96cf5e081344446a241b3f9f9882e2eccae659cd`, ELF `d74d863a8a724fe73b591d45aebe1aa7014e4af8e24248075a256a2de3be4991`. Expected idle ledger next 22, last 21, charged 2,280,000 ms, with the original budget exhausted.                                                     |
| Attempt namespace and task gate | Private roots live under `scratch/str005-step5-reinstall/`. Preflight requires the exact active task line `Step-5 diagnostic reinstall hardware: enabled.` A pass publishes to `docs/parity/evidence/str005-step5-reinstall/`.                                                     |

Everything else is as phase 1 requires.

## Phase 2: one diagnostic Start (`scripts/str005-step5-diagnostic`)

Admission: the compiled `ENABLED` flag, the exact active task line
`Step-5 diagnostic Start hardware: enabled.`, and clean pushed source. The
pinned install seal, result and profile (phase 1 or 1b), and that install's
identity, Gate and physical identity, must match. Preflight also requires:
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

## Phase 2 rerun (owner `task-str005-step5-diagnostic-rerun`)

Start002 passed step 5 but sealed `complete=false` for three judge blockers.
None of them was a device-safety failure. The owner fixes them as follows:

- **Authorization.** A signed Start advances the authorization high-water
  mark. The device's authenticated `authorizationRecovery` match for the
  Start's own generation now counts as restored, the same rule the shared
  recovery check uses.
- **Fixture completion.** The share-scope fixture waits for a share, so a
  status-only Start can never satisfy its natural exit code 0. This owner
  requires no share (`required: false`). It counts completion as a clean
  natural close: the exact peer only, no rejected or duplicate share, closed
  peer, socket and listener, a natural exit, and either an accepted outcome or
  `peer_eof`. The fixture's own outcome and share counts stay in the result,
  and no share is claimed.
- **Page release.** Release no longer waits on that share-only exit.

The rerun preflight pins start002 (result `0a66996a…`, seal `1b2fbfcb…`) as the
previous Start. Its fresh recovery supplies the ledger the new baseline must
observe unchanged (next 23, last 22, 2,460,000 ms) and the boot ordinal. One
new `normal` attempt charges another 180,000 ms (expected after: next 24, last
23, 2,640,000 ms). Before collecting fresh recovery, the operator waits at least
8 s after Connect, so the replayed preparation receipt (every 6 s) and any
revocation detail (every 2 s) reach the page. All other phase-2 terms are
unchanged.

Start003 was refused at baseline before any effect. After a completed Start,
the Worker keeps that attempt's terminal V2 record until reboot, so the
baseline's null-attempt status query fails with `invalid_transition`. Firmware
has no clear path other than reboot. Each further Start is therefore preceded
by one no-mining qualification restart (`scripts/str005-step5-diagnostic/restart-main.mjs`,
`just str005-step5-restart`). It reuses the startup-preparation recovery and
restart servers and the restart evidence model, served on the granted origin:
- stage `recovery` freshly reads the retained record by its attempt ID;
- stage `restart` performs exactly one `qualificationRestart` on a separate
  restart-only page, with the ledger and budget unchanged and boot N+1 proven.
The next Start then binds the sealed restart: boot N+1, with the ledger
unchanged.

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
