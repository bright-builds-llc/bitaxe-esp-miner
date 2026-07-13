# Work-Result Upstream Diff v2 (Post Mining-Ready Fix)

Reference pin: `reference/esp-miner` @ `c1915b0`

Rust path after first investigation: safety bring-up → chip detect → `mining_ready_init` → host `UseMaxBaud` → diagnostic work @ 1M → 10s read (W5 bootstrap on timeout)

Prior diff: [`work-result-upstream-diff.md`](work-result-upstream-diff.md) (W1–W6, mostly addressed)

## Remaining divergences (W7–W12)

| Step | Upstream @ c1915b0 | Rust Phase 27 (post W1–W5) | Verdict |
| --- | --- | --- | --- |
| W7 ASIC max baud reg | `BM1366_set_max_baud` sends reg28 frame @ 115200, then `SERIAL_set_baud(1M)` (`asic_init.c:57-59`, `bm1366.c:297-304`) | `UseMaxBaud` ESP-only; no reg 0x28 frame | **diverge (CRITICAL)** |
| W7 ordering | reg28 → wait TX → host baud → clear buffer | host baud → clear only | **diverge** |
| W8 frequency ramp | `do_frequency_transition`: 50→485 MHz in 6.25 MHz steps, 100ms delay (`frequency_transition_bmXX.c`) | Single `SetFrequency(485)` in `mining_ready_init` | **diverge** |
| W8 actual_frequency init | `POWER_MANAGEMENT_init_frequency` sets `actual_frequency = 50.0` before ramp | Not modeled | **diverge** |
| W9 voltage/UART prelude | `VCORE_set_voltage` + 500ms + UART flush before `asic_initialize` (`power_management_task.c:65-79`) | Phase 27 safety bring-up (partial match) | **partial match** |
| W9 stabilization delay | Recovery init uses 2000ms post max-baud (`asic_init.c:63-66`) | None after max baud | **diverge (low)** |
| W10 result loop | `ASIC_result_task` continuous `receive_work` 10s; register reads accepted | Single boot diagnostic read | **diverge** |
| W11 work source | `create_jobs_task` → pool notify → `BM1366_send_work` real jobs | Boot diagnostic `0x28` synthetic fields | **diverge (by design at boot)** |
| W12 pool bridge | Stratum connected; pool settings in NVS | `phase27_pool_wait_timeout` without pool NVS | **diverge (separate track)** |
| W1–W6 init frames | Post-detect register/frequency/nonce sequence | `mining_ready_init` golden-matched | **match** |
| W3 address_interval | 256 / chip_count | `ultra_205_result_address_interval()` | **match** |
| W5 bootstrap | N/A (upstream always initializes for mining) | `bounded_no_result bootstrap=initialized_no_mining` | **Rust-only gate** |

## Reference call chains

### Max baud (W7)

```
asic_initialize()
  ASIC_init() → BM1366_init()     @ 115200
  ASIC_set_max_baud()
    BM1366_set_max_baud()
      _send_simple(reg28, 11)      @ 115200  ← ASIC-side baud config
  SERIAL_set_baud(1000000)         ← host UART
  SERIAL_clear_buffer()
```

### Frequency (W8)

```
BM1366_init()
  do_frequency_transition(GLOBAL_STATE, BM1366_send_hash_frequency)
    actual_frequency starts at 50 MHz
    ramp in 6.25 MHz steps with vTaskDelay(100ms)
  BM1366_set_nonce_space(1.0, frequency, asic_count, cores)
```

### Mining loop (W10–W11)

```
create_jobs_task → ASIC_send_work(pool job) @ 1M
ASIC_result_task → ASIC_process_work → receive_work(10s)
```

## Hardware evidence (2026-07-05)

Post W1–W5 fix (`work-result-verify-20260705/flash-monitor.log`):

- Full mining-ready init + host 1M + diagnostic TX
- **Zero `asic_uart_trace=rx_chunk`** during 10s wait
- W5 bootstrap unblocks `asic_production_status=initialized`
- Bridge: `phase27_pool_wait_timeout` (W12)

**Primary suspect for zero RX:** W7 missing ASIC reg28 before host 1M switch.

## Fix bundle (this deep dive)

1. W7: `SetAsicMaxBaud` → `WaitTxDone` → `UseMaxBaud` → `ClearRx`
2. W8 (if needed): frequency ramp behind investigation flag
3. Hardware matrix runs A–D
4. Bootstrap gate refinement when RX appears
5. Bridge E2E with pool credentials
