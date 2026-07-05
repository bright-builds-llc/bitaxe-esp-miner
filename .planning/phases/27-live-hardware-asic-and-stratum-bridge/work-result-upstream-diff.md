# Work-Result Upstream Diff Trace Matrix

Reference pin: `reference/esp-miner` @ `c1915b0a63bfabebdb95a515cedfee05146c1d50`

Rust path (Phase 27): safety bring-up → chip detect (upstream-aligned) → diagnostic work @ 115200 → single 10s read

Upstream path: `BM1366_init` post chip-detect → `asic_init.c` max baud → mining task `BM1366_send_work` → `receive_work`

| Step | Upstream @ c1915b0 | Rust Phase 27 (pre-fix) | Verdict |
| --- | --- | --- | --- |
| init4 register 0xA8 | `_send_simple` write `[00 07 00 00]` (`bm1366.c:211-212`) | Not emitted | **diverge** (W1) |
| init5 register 0x18 | `_send_simple` misc control (`bm1366.c:214-215`) | Not emitted | **diverge** (W1) |
| Chain inactive | `_send_chain_inactive` CMD_INACTIVE (`bm1366.c:217-218`) | Not emitted | **diverge** (W1) |
| Chip address | `address_interval = 256/chip_count`; `_set_chip_address(0)` for 1 chip (`bm1366.c:221-225`) | Not emitted | **diverge** (W1) |
| init135/136 register 0x3C | `_send_simple` (`bm1366.c:227-231`) | Not emitted | **diverge** (W1) |
| Difficulty mask | `get_difficulty_mask` + `_send_BM1366` (`bm1366.c:233-238`) | Not emitted | **diverge** (W1) |
| init138/139/171 | `_send_simple` (`bm1366.c:240-247`) | Not emitted | **diverge** (W1) |
| Per-chip A8/18/3C writes | Loop per chip (`bm1366.c:253-264`) | Not emitted | **diverge** (W1) |
| Frequency | `do_frequency_transition` + `BM1366_send_hash_frequency` (`bm1366.c:266-267`) | Not emitted | **diverge** (W1) |
| Nonce space | `BM1366_set_nonce_space(1.0, freq, asic_count, cores)` (`bm1366.c:271`) | `full_init` uses `hash_counting_number: 0` only | **diverge** (W1) |
| Final init795 ticket mask | `_send_simple` A4 90 00 FF FF (`bm1366.c:273-274`) | Not emitted | **diverge** (W1) |
| Max baud + clear RX | `SERIAL_set_baud` + `SERIAL_clear_buffer` in `asic_init.c:57-59` | Diagnostic work at 115200 | **diverge** (W2) |
| Work dispatch | After full init at max baud in mining loop | Immediately after chip detect @ 115200 | **diverge** |
| Result read | `receive_work` 10s timeout, mining loop polls (`asic_common.c:148-188`) | Single `read_accumulate` 10s after one job | **diverge** (W4) |
| `address_interval` parse | `256 / chip_count` = **256** for Ultra 205 (`bm1366.c:357`) | Hardcoded **16** in `asic_adapter.rs` | **diverge** (W3) |
| Chip detect prelude | Version mask ×3 before chip-ID | Upstream-aligned (fixed) | **match** |
| Diagnostic job frame | 88-byte `_send_BM1366` job (`bm1366.c:338`) | Golden `diagnostic_job_frame` | **match** |

## Ranked hypotheses

1. **W1 (HIGH):** Missing mining-ready register/frequency/nonce init — ASIC ignores work (zero RX in trace).
2. **W2 (MEDIUM-HIGH):** Work dispatched at 115200 before max baud switch.
3. **W3 (MEDIUM):** Wrong `address_interval` (16 vs 256) breaks parse if RX appears.
4. **W4 (MEDIUM):** Single 10s read vs continuous mining-loop receive.
5. **W5 (MEDIUM):** No nonce in 10s even after valid init — bootstrap gate may need `InitializedNoMining` path.

## Fix bundle (this investigation)

- `mining_ready_init` actions mirroring upstream post-detect sequence
- Phase 27 boot: chip detect → mining ready init → max baud → diagnostic work
- `address_interval` from catalog (`256 / asic_count`)
- UART trace on init actions and work-result read
