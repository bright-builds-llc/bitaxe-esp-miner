# Internal-heap exhaustion: diagnosed and corrected

Restart006 panic-rebooted on a Worker connection about 44 minutes after
heartbeat008's heartbeat-loss shutdown on `60e344e2`. The
[diagnosis](../../hardware/str005-heap-loss-diagnosis-amendment.md) found no
leak: internal RAM was persistently exhausted. The
[PSRAM-first candidate](../../hardware/str005-psram-default-allocation-amendment.md)
corrects it.

## Method

The firmware writes `internal_heap_sample` (internal 8-bit heap: free,
allocated, largest block, lifetime minimum, block counts) every 60 s, only
while no Worker session owns the serial link. Passive `just monitor`
captures read it without opening the connection under test.
`just internal-heap-series` judges captures against fixed minimums: every
sample must keep 16,384 free bytes and an 8,192-byte largest free block.

## Results

| Image and state                                  | Samples | Least free | Least largest block | Allocated blocks | Verdict |
| ------------------------------------------------ | ------- | ---------- | ------------------- | ---------------- | ------- |
| Diagnostic `31fa7238`, idle, no Start            | 20      | 2,631      | 1,728               | 1,039–1,236      | fails   |
| Diagnostic `31fa7238`, 80 min after heartbeat009 | 81      | 2,103      | 1,536               | 1,065–1,302      | fails   |
| Candidate `2bff1004`, idle, no Start             | 20      | 53,271     | 31,744              | 251, constant    | passes  |

- The diagnostic image idled at about 11.5 KB free with dips to 2.6 KB.
  After one heartbeat-loss session the baseline settled about 3.3 KB lower
  and stayed flat for 80 minutes: no leak.
- The dips came from periodic small allocations (settings snapshots rebuilt
  on 100 ms to 1 s loops) that a 2 KiB always-internal cutoff kept in
  internal RAM, beside about 230 KiB of internal thread stacks.
- Recovery019, a connection outside a dip, passed. The restart006 panic left
  no core dump, because the partition still held the archived boot-loop dump
  and dumps are never overwritten, so its exact site is not claimed.

## Correction

`CONFIG_SPIRAM_MALLOC_ALWAYSINTERNAL=0`: ordinary allocations prefer PSRAM at
every size. Explicit internal or DMA requests and the 96 KiB internal
reserve are unchanged. The build's resolved-config contract requires the
value.

The candidate (`2bff1004`, ELF `9783dc74…`) passed:

- its five-install, four-cycle install (`psram-default-install`
  attempt-002);
- the idle-headroom capture (table above);
- heartbeat010: `heartbeat_timeout`, gate closed 2,808 ms and shutdown
  started 2,813 ms after the last heartbeat, safe stop complete, ledger
  30/29/3,720,000 ms. Internal heap during its recovery was 52,743 bytes
  free.

Attempt-001 of that install also exposed a host fixture defect, now fixed: a
random attempt id beginning with `-` was read as a flag.

## Non-claims

One board, idle and after one heartbeat-loss session. Not sustained-mining,
pool, accepted-share or parity-promotion evidence. Raw captures stay in
protected private roots under `scratch/str005-heap-observation` and
`scratch/str005-psram-headroom`.
