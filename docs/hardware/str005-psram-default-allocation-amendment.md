# STR-005 PSRAM-first allocation candidate amendment

## Status and precedence

Owner-authorized on 2026-10-05 as the fix step of the
[heap-loss diagnosis](str005-heap-loss-diagnosis-amendment.md) ("write the
contract and start the investigation"; the plan named the fix, a new
candidate install, a heartbeat-loss re-run, then the restart and accepted
share). It runs under `task-str005-heap-loss-diagnosis` and reuses existing
owners unchanged:

- the [noise-serial install](str005-noise-serial-qualification.md)
  operator flow, journal, judges and cleanup;
- the [heartbeat-loss Start](str005-heartbeat-shutdown-amendment.md),
  Phase B;
- the receive-only `just monitor` and `just internal-heap-series`.

Parity stays 90/95.

## Finding and fix

The diagnosis found no leak. Internal RAM is persistently exhausted:

- thread stacks take about 230 KiB;
- `CONFIG_SPIRAM_MALLOC_ALWAYSINTERNAL=2048` kept about 1,000 ordinary
  small allocations, and their periodic churn, in internal RAM.

At idle the largest free internal block never exceeded 1,792 bytes, and
free bytes dipped to 2.1 KB. A Worker connection that lands in a dip can
exhaust it.

The candidate sets `CONFIG_SPIRAM_MALLOC_ALWAYSINTERNAL=0`, so ordinary
allocations prefer PSRAM at every size. Explicit internal or DMA requests
(task stacks, FreeRTOS objects, DMA buffers, mbedTLS) are unchanged, and
so is the 96 KiB internal reserve. No Rust code runs from IRAM or an IRAM
interrupt except the panic cutoff, which does not allocate. The idle
`internal_heap_sample` line stays in the candidate.

## Candidate

The clean pushed HEAD's canonical `just package` output. The build's
resolved-config contract requires `CONFIG_SPIRAM_MALLOC_ALWAYSINTERNAL=0`.
`just audit-stack-realignment` and `just audit-startup-frames` must pass on
the exact ELF.

## Phases

### Phase A: install

This phase defines the successor profile `psram-default-install`,
namespace `scratch/psram-default-install`. Its predecessor is the sealed
recovery019: a current safe recovery on the diagnostic image that read
heartbeat009's retained record by its attempt, proving its resources
released. The ledger is unchanged at next 29, last 28, 3,540,000 ms.
Line: `PSRAM default install hardware: enabled.` The operator sequence,
the five-install limit, the four verified cycles and every prohibition are
the [queue-workaround reinstall's](str005-queue-workaround-reinstall-amendment.md).
After a pass, `just str005-lineage advance-install --root <attempt>`.

### Phase B: idle headroom

After the install's page closure and release, run the detector and one
1,200-second receive-only capture with no Worker connection. It passes
only when every sample keeps at least 16,384 free bytes and an 8,192-byte
largest free block:

```sh
just internal-heap-series --min-free-bytes 16384 --min-largest-block-bytes 8192 --min-samples 18 <capture>
```

### Phase C: heartbeat010

One heartbeat-loss Start under the heartbeat amendment's Phase B, on the
lineage head's new install, with no previous Start or current recovery.
Ordinal 29; expected after ledger next 30, last 29, 3,720,000 ms. Line:
`Heartbeat PSRAM candidate hardware: enabled.`

### Phase D: post-shutdown headroom

After heartbeat010's page closure and release, three consecutive
1,200-second receive-only captures with no Worker connection, judged
together with the Phase B thresholds and `--min-samples 54`.

### Phase E: handoff

On a pass the candidate replaces `60e344e2` in the STR-005 integration
review. `task-str005-share-current-image` then resumes its restart and
accepted-share check on the lineage head; that check has its own gates.

## Prohibited

- any Start other than heartbeat010, any renewal, mining beyond its single
  generation, a pool or pool credentials, Wi-Fi provisioning;
- an NVS or factory reset, erase, rollback or core-dump clearing;
- more than five installs, a firmware write outside Phase A;
- a Worker connection during a Phase B or Phase D capture;
- direct UART or pin access, network discovery, synthesized permission
  gestures;
- publishing raw private evidence.

## Stop conditions

- a detector result other than exactly one Ultra 205;
- an identity, ledger or baseline drift;
- a lost or ambiguous Start (never resend);
- any panic;
- a Phase B or Phase D headroom failure (recorded, not retried without a
  regression-backed change);
- unproven cleanup.

## Non-claims

Headroom on one board at idle and after one heartbeat-loss shutdown. It is
not sustained-mining, pool, accepted-share or parity-promotion evidence.
