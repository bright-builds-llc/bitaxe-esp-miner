# Diagnostic Work vs Upstream BM1366_send_work (W11)

Date: 2026-07-05

## Frame structure: **MATCH**

Rust `Bm1366WorkPayload` mirrors `BM1366_job` in `reference/esp-miner/components/asic/include/bm1366.h`:

| Offset | Field | Rust | Upstream |
| --- | --- | --- | --- |
| 0 | job_id | `Bm1366JobId` | `job.job_id` (incremented by 8 mod 128) |
| 1 | num_midstates | `0x01` | `0x01` |
| 2–5 | starting_nonce | 4 bytes | `memcpy` from pool job |
| 6–9 | nbits | 4 bytes | target from pool |
| 10–13 | ntime | 4 bytes | ntime from pool |
| 14–45 | merkle_root | 32 bytes | from pool |
| 46–77 | prev_block_hash | 32 bytes | from pool |
| 78–81 | version | 4 bytes | from pool |

Job frame header: `TYPE_JOB | GROUP_SINGLE | CMD_WRITE` — matches `_send_BM1366` in `bm1366.c:338`.

## Field values: **DIVERGE (expected for boot diagnostic)**

Boot diagnostic in `asic_adapter.rs` uses deterministic synthetic fields:

- `starting_nonce`: `[1,2,3,4]`
- `merkle_root`: `[0x11; 32]`
- `prev_block_hash`: `[0x22; 32]`
- Fixed `job_id=0x28`

Upstream `BM1366_send_work` copies live `bm_job` from Stratum notify — valid block header, target, and merkle path required for ASIC to hash and return nonces.

## Hardware implication

Runs A–C confirmed:

- Chip detect RX @ 115200 works
- Full mining-ready init + reg28 + host 1M completes
- **Zero post-work RX** even at 115200 (Run C)

So W11 alone does not explain all silence (115200 control still silent), but even with a working UART path the synthetic diagnostic job should not be treated as mining proof. Production bridge must dispatch pool-derived work via `Bm1366ProductionCommand::SendProductionWork`.

## Recommended follow-up

1. Add a host golden fixture from a captured upstream `bm_job` bytes (no secrets in repo — use redacted or synthetic-but-valid header from checklist fixture).
2. Boot diagnostic tier remains `bounded_no_result`; promote only on register-read parse or production result correlation.
