# ADR-0031: Prospective admission for staged panic diagnostics

Accepted 2026-09-27 by the owner's explicit approval to perform the staged
hardware diagnostic recommendation, including hardware flashing and use.

## Decision

Preserve Share002's missing retained resource record as an unresolved historical
gap. Its absence does not permanently prevent independent future diagnostics
when fresh authenticated observations prove current safe state, inactive work
authority, current resource quiescence, durable accounting and actual serial
release. This prospective admission does not complete the historical recovery
task, revise any sealed result, publish Channel006 or promote parity.

For new diagnostics, require fresh same-device identity, both ledgers with no
pending reservation, confirmed restoration and disabled mining, matched settings
and Device Identity, authenticated current idle V2 status, and actual ownership
release before another owner opens USB. A retained terminal record may support
these observations; an idle current record is not historical resource proof.
Old accounting and expired admission cannot authorize another effect.

This supersedes only the interpretation of ADR-0029 and ADR-0030 that made an
unrecoverable historical record a prerequisite for every future diagnostic.
Their safety, signed authority, accounting, immutable evidence, private capture,
continuity assessment, cleanup and progress-gated retry rules remain.

## Ordered diagnostic stages

1. Publish and software-verify the task's exact command/evidence/effect contract.
   Obtain fresh current-state recovery proof and preserve the existing dump
   partition privately before any write that could affect it.
1. Install the clean published diagnostic image using the existing state-
   preserving flash path. Verify exact identity, settings/accounting preservation
   and current safe baseline. No work grant is needed for installation.
1. Run one authenticated capture self-test only with ASIC power disabled and
   reset held. It must be one-use, acknowledge before the effect, reject stale
   sessions and record a deliberate native abort without starting mining.
   Reconnect, verify recovery, collect the dump, match its exact ELF and inspect it.
1. Admit one fresh bounded V2 Start only after capture and native panic-safe
   shutdown checks pass. Preserve the self-test dump before separately clearing
   its on-device copy. Maintain normal heartbeats, issue no renewals, and Stop
   immediately after startup observation or the first failure. Recover and collect
   evidence independently of the Start result.

V2 Start currently requires a normal qualification allowance: 180,000 ms is
reserved durably before preparation; the initial grant is 60,000 ms with a
20,000-ms renewal point. A shorter host observation or immediate Stop does not
refund that reservation or turn it into a 30,000-ms diagnostic allowance. Select
the ordinal from fresh authenticated accounting at issuance. Even if its number
is 18, issue a new context/nonce/signature; never replay Share002's old grant.

## Fatal-handler safety and evidence

ESP-IDF stalls ordinary execution during fatal handling. Browser Stop and normal
heartbeat tasks are not sufficient protection while a flash dump is written.
Require a reviewed native, IRAM-resident, allocation-free and lock-free cutoff
before the normal panic handler, with exact board pin polarity, generation
revocation, native code-placement/routing checks and capture self-test evidence.
If any required safety discriminator is absent, stop before Start. Do not relax
the device's existing heartbeat/shutdown deadlines or infer electrical behavior
from unrelated startup heap readings.

The active task and published command contract own exact bounds, proof freshness,
package/Gate identities, permitted reads/partition clearing, recovery and stop
conditions. No factory reset, NVS erasure, external pool, direct pins, repeated
blind attempt or unbounded fault loop is authorized. Standing authorization
covers execution within those bounds without further per-attempt confirmation.
