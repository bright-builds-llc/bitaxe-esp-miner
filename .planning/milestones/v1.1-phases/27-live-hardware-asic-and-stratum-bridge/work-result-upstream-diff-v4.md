# Work-Result Upstream Diff v4 (Post B3 Production Read Fix)

Reference pin: `reference/esp-miner` @ `c1915b0`

Prior diffs: [`work-result-upstream-diff-v3.md`](work-result-upstream-diff-v3.md)

Rust path after B3 wave: safety bring-up → chip detect → `mining_ready_init` → diagnostic or bridge production work @ 1M → **10s polled result read loop** (100ms pump slices)

## B3 production-read outcomes (2026-07-06)

| Fix | Hardware result | Evidence |
| --- | --- | --- |
| P0: 1s `READ_RESULT_FRAME` prelude removed; executor owns 10s read | **FIXED** — ~9960ms poll window after `work_dispatched` (was ~1110ms timeout) | [`b3-production-read-20260706-retry/`](b3-production-read-20260706-retry/) |
| W10: bridge result read loop across pump iterations | **IMPLEMENTED** — 47× `result_read_attempt` before capture end | same |
| W10: accept `RegisterRead` in production path | **NOT OBSERVED** on hardware | same |
| W11: pool work frame encoding (host golden) | **PASS** (host); 88-byte TX on wire | [`production-work-redacted.json`](../../../crates/bitaxe-asic/fixtures/bm1366/production-work-redacted.json) |
| Post-dispatch UART nonce/register proof | **FAIL** — zero post-work `rx_chunk` / `register_read_parsed` | F1 retry |
| F3 default fail-closed regression | **PASS** | [`b3-production-read-20260706-run-F3/`](b3-production-read-20260706-run-F3/) |

## Remaining divergences (updated W7–W13)

| ID | Upstream | Rust gap | Hardware (2026-07-06 B3) | Experiment |
| --- | --- | --- | --- | --- |
| W7 ASIC max baud reg | reg28 @ 115200 → host 1M | **match** | REG28 on wire; chip detect RX OK | default |
| W8 frequency ramp | 50→485 MHz stepped ramp | Optional `frequency_ramp` flag | No RX improvement (2026-07-05) | `frequency_ramp` |
| W9 stabilization delay | 2000ms after max baud | `post_max_baud_delay_2000` flag | No post-work RX gain | `post_max_baud_delay_2000` |
| W10 result loop | Continuous `receive_work` 10s | **Polled loop added**; boot diagnostic still single read | 47 poll attempts / ~10s; **still silent** | F1 retry |
| W11 work source | Pool-derived `BM1366_send_work` | Golden matches upstream layout; bridge uses pool notify | TX 88 B; **no nonce** | F1 retry |
| W12 pool bridge | Stratum + NVS pool keys | **match after B2** | `active`, `work_dispatched` | bridge |
| W13 bootstrap gate | Upstream always mines after init | Default fail-closed on diagnostic timeout | F3: `work_result_diagnostic_timeout` | default package |

## Hardware matrix (B3 wave)

| Run | Config | Read window | Post-work UART | Share |
| --- | --- | --- | --- | --- |
| F1 (first) | W1+W2, bridge | ~220ms (malformed fail-closed) | none | blocked |
| F1 (retry) | W1+W2 + poll fix | **~9960ms** | none | blocked |
| F2 | `clear_rx_before_production_work` | N/A (no bridge) | boot diagnostic silent | N/A |
| F3 | Default package | N/A (fail-closed pre-bridge) | N/A | N/A |

## Primary suspect (post B3)

ASIC still does not emit post-production-work results within 10s despite correct read timing and pool-framed TX. Next investigation should target upstream init/state gaps (W13 `GLOBAL_STATE->ASIC_initialized`, frequency/mining enable timing) rather than baud, delay, or read-timeout tuning.

See [`b3-production-read-20260706-run-F1.md`](b3-production-read-20260706-run-F1.md), F2, F3 under this phase directory.

**Superseded by:** [`work-result-upstream-diff-v5.md`](work-result-upstream-diff-v5.md) (B4 init-state hardware matrix).
