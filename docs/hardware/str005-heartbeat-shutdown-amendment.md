# STR-005 heartbeat-loss shutdown probe amendment

## Status and precedence

Owner-authorized on 2026-10-03 under `task-str005-heartbeat-shutdown-probe`. The
owner also authorized autonomous, iterative fixes and fresh attempts. This
amendment defines one bounded heartbeat-loss Start, preceded by one no-mining
qualification restart. It reuses the
[heartbeat-probe contract](../../scripts/str005-heartbeat-probe/README.md):
its coordinator, page, suppression and checkpoint seam, passive cadence
observer, judge and inspector. It replaces only the admission chain, the page
origin and the install. Earlier results keep their meaning. Parity stays 90/95
unless the separate integration review promotes it.

## Basis

- **Firmware.** `654338d0`/`2641c24f`, installed and audited by the step-5
  reinstall (result `04f2f8d1…`, seal `0751d602…`).
- **Previous Start.** Share001 (result `9e17a8b2…`, seal `abde26a6…`) proved a
  device-acknowledged accepted share and normal restoration on this image. It
  left the ledger at next 25, last 24, 2,820,000 ms, on boot 13.
- **Native constants.** Heartbeat timeout 2,800 ms
  (`HEARTBEAT_TIMEOUT_MILLISECONDS`, `HEARTBEAT_CUTOFF_MS`); pre-reset shutdown
  reserve 15,550 ms.
- **Lease rule.** V2 lease grants must be `normal`
  (`crates/bitaxe-worker-control/src/lease.rs`), so the fault rides a normal
  attempt with zero renewals.

## Phase A: restart003 (`just str005-step5-restart`)

`restart-config.mjs` names share001 as the parent and this task's line
`Heartbeat restart hardware: enabled.` Everything else follows the
accepted-share amendment's restart rules:
- one fresh recovery stage that reads the retained record by ID;
- exactly one software `qualificationRestart`, with boot N+1 proven, the ledger
  and budget unchanged, the pre-reset completed-attempt marker admitted, and
  the pages on `127.0.0.1:48765`.

## Phase B: one heartbeat-loss Start (`just str005-heartbeat-shutdown`)

Admission requires:
- the compiled `ENABLED` flag, the exact active task line
  `Heartbeat shutdown probe hardware: enabled.`, and clean pushed source;
- the pinned chain of reinstall, then share001, then the sealed restart003;
- the four native audits on the installed ELF, Gate `86fc62d7` (whose bundle
  must include `suppressHeartbeats`) with zero-renewal compatibility, the
  canonical fixture, and the passive `cadence_observer` built from this clean
  source with its build receipt;
- a detector at most 60 s old with no holder.

Effects follow the heartbeat-probe contract:
- One fresh `normal` attempt at the measured next ordinal (25): a
  180,000-ms reservation, a 60,000-ms grant and zero renewals, on the
  conservative profile (400 MHz / 1,100 mV, fan 100%), against one local V2
  fixture.
- The prearmed passive observer starts before Start. The Start reply is
  bounded at 30 s.
- After same-generation `work_ready` and `asic_dispatch`, the page calls the
  Gate's `suppressHeartbeats` once. This must happen at most 5 s after the
  reply and 35 s after invocation, with at least 5,000 ms of lease and
  work-gate headroom.
- The page then observes passively for 8,000 ms. Stop and Close follow, then
  fresh-possession recovery.
- No share is required, so the fixture is closed and reaped, not judged on
  share completion.

The expected after ledger is next 26, last 25, 3,000,000 ms.

A pass requires every heartbeat judge and inspector criterion. The core ones:
- **Revocation.** Native `revocation_reason=heartbeat_timeout`, with the gate
  closing between 2,800 and 3,000 ms after the last valid heartbeat.
- **Shutdown.** Initiated after closure and within 3,000 ms of the last
  heartbeat, inside the 15,550-ms reserve, with no work after revocation.
- **Stop and cooling.** An ordered normal-stop event join, then qualified
  cooling: fan paused, at most 45 °C, nonzero fan, bus 4.5–5.5 V, at most
  15 W, and `mine_on_boot=false`.
- **Recovery and accounting.** An authorization checkpoint, fresh recovery on
  the same boot with a new serial epoch, and exactly 180,000 ms charged.
- **Observer joins.** Matched device, state, suppression and tail records.
- **Release.** Host, serial, signer and fixture owners all released.

Any shortfall seals `unverified`.

## Prohibited

- an external pool or pool credentials, Wi-Fi provisioning, or any firmware
  write;
- an NVS or factory reset, erase or rollback;
- a voltage or frequency override, any renewal, a second Start, an extra
  reservation, a budget reset or refund;
- any deadline relaxation;
- direct UART or pin access, network discovery;
- synthesized permission gestures;
- publishing raw private evidence.

## Recovery, retry and stop

Recovery uses only the normal Stop/Close path, a fresh-possession recovery and
host cleanup. The device's own heartbeat revocation and safe stop remove core
voltage and ASIC power.

Retries need a targeted, regression-backed fix, a fresh ordinal and a new
restart before any further Start.

Stop on:
- a detector result other than exactly one Ultra 205;
- identity, ledger or baseline drift;
- a lost or ambiguous Start (never resend);
- a missed native deadline (record it; do not relax it);
- a new panic;
- unproven cleanup.

## Non-claims

A pass proves device-local heartbeat-loss revocation and bounded shutdown for
one generation on a local fixture. It is not sustained-mining, pool or parity
promotion evidence.
