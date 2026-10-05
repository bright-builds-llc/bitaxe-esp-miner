# STR-005 heap-loss diagnosis amendment

## Status and precedence

Owner-authorized on 2026-10-05 ("write the contract and start the
investigation") under `task-str005-heap-loss-diagnosis`, together with the
standing autonomous fix-and-retry authorization. It reuses existing owners
unchanged except where named:

- the control-diagnostic recovery
  ([reproduction amendment](str005-control-stack-reproduction-amendment.md));
- the [noise-serial install](str005-noise-serial-qualification.md) operator
  flow, journal, judges and cleanup;
- the [heartbeat-loss Start](str005-heartbeat-shutdown-amendment.md),
  Phase B;
- the receive-only `just monitor`.

Parity stays 90/95. This is diagnosis, not qualification.

## Question

Restart006 panic-rebooted on Connect about 44 minutes after heartbeat008's
heartbeat-loss shutdown on `60e344e2`. Free internal heap sampled during
heartbeat008's recovery connection was 9,687 bytes (largest block 1,920).
Sampled during the next connection it was 607 bytes (largest 168), and that
connection failed an 8,192-byte internal allocation. Was the heap lost while
the board sat idle, or inside the connection? If idle, what allocates?

## Diagnostic image

The clean pushed HEAD's canonical `just package` output. It adds one line,
`internal_heap_sample schema=v1 …`, to the idle serial link every 60 s. The
line carries numeric fields only (uptime, free, allocated, largest block,
lifetime minimum, allocated and free block counts, revoked). It is written
only while no Worker session owns the link, so sessions and their judges see
an unchanged stream. `just audit-stack-realignment` and
`just audit-startup-frames` must pass on the exact ELF.

## Phases

### Phase 0: recovery018

One control-diagnostic recovery on the current boot (302, after the panic).
The lineage head still names heartbeat008 as its latest Start, so the owner
admits restart006's sealed root as proof that the board rebooted afterwards.
It therefore reads status in discovery mode, not by heartbeat008's attempt.
Expected: idle V2, ledger next 28, last 27, 3,360,000 ms, not pending. Line:
`Control diagnostic recovery hardware: enabled.`

### Phase 1: internal-heap-diagnostic install

Profile `internal-heap-diagnostic-install`, namespace
`scratch/internal-heap-diagnostic-install`, predecessor the sealed
recovery018, ledger unchanged at 28/27/3,360,000 ms. Line:
`Internal heap diagnostic install hardware: enabled.` The operator sequence,
the five-install limit, the four verified cycles and every prohibition are
the [queue-workaround reinstall's](str005-queue-workaround-reinstall-amendment.md).
After a pass, `just str005-lineage advance-install --root <attempt>`.

### Phase 2: idle baseline

After the install's page closure (`about:blank`) and full release, run the
detector, then one receive-only capture:

```sh
just monitor --board 205 --port <port> --expected-physical-sha256 <physical> --capture-timeout-seconds 1200
```

Stdout goes to a mode-0600 file below a mode-0700 ignored root. No Worker
connection is made. This shows the drift with no Start.

### Phase 3: heartbeat009

One heartbeat-loss Start under the heartbeat amendment's Phase B, on the
lineage head's new install, with no previous Start or current recovery.
Ordinal 28; expected after ledger next 29, last 28, 3,540,000 ms. Line:
`Heartbeat heap diagnosis hardware: enabled.`

### Phase 4: post-shutdown idle

After heartbeat009's page closure and release, run the detector, then
consecutive 1,200-second receive-only captures, as in Phase 2, for at least
60 minutes. No Worker connection is made during this window.

### Phase 5: next connection

One fresh control-diagnostic recovery (recovery019). If the last Phase 4
sample shows a largest free block below 8,192 bytes, a panic reboot during
this connection is the expected reproduction. It is recorded as
`reproduced_connect_allocation_failure`, not as a new panic, and
recovery020 then reads the new boot. Any other panic is a stop.

## Analysis and evidence

A private script reads the captured lines into a numeric series. Rising
`allocated_blocks` with falling `free_bytes` means a leak; rising
`free_blocks` at stable `free_bytes` means fragmentation. Committed evidence
holds only the numeric series, categories and digests.

## Prohibited

- any Start other than heartbeat009, any renewal, mining beyond its single
  generation, a pool or pool credentials, Wi-Fi provisioning;
- an NVS or factory reset, erase, rollback or core-dump clearing;
- more than five installs, a firmware write outside Phase 1;
- a Worker connection during a Phase 2 or Phase 4 capture;
- direct UART or pin access, network discovery, synthesized permission
  gestures;
- publishing raw private evidence.

## Stop conditions

- a detector result other than exactly one Ultra 205;
- an identity, ledger or baseline drift;
- a lost or ambiguous Start (never resend);
- a panic other than the Phase 5 reproduction;
- unproven cleanup.

Retries need a regression-backed fix and a fresh ordinal.

## Non-claims

The diagnostic image is not a parity candidate. Its results explain the heap
loss; they do not qualify any image.
