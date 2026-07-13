# Work-Result Upstream Diff v3 (Post Blocker Fix + W9 Hardware)

Reference pin: `reference/esp-miner` @ `c1915b0`

Prior diffs: [`work-result-upstream-diff-v2.md`](work-result-upstream-diff-v2.md) (W7–W12 pre-hardware)

Rust path after blocker fix: safety bring-up (category thermal logs) → chip detect → `mining_ready_init` (W7 reg28 + optional W9 delay) → diagnostic or bridge production work @ 1M → read loop

## Blocker fix outcomes (2026-07-06)

| Blocker | Fix | Hardware result |
| --- | --- | --- |
| B1 Thermal gate | Fresh status → `Fresh` prerequisite; fault/unavailable → bounded `phase27_thermal`; bring-up logs `category=` not raw temps | Bridge reaches `connecting` (no `thermal_reading_invalid`) |
| B2 Pool consumed marker | Emit on settings patch commit + `maybe_refresh_phase27_from_settings` before bridge gate | `pool_settings_consumed_by_runtime=true` |
| B3 UART post-work | Bridge unblocked; production work dispatches | **Still silent** — `production_result_timeout` after `work_dispatched` |

Evidence: [`bridge-blocker-fix-20260706/`](bridge-blocker-fix-20260706/)

## Remaining divergences (updated W7–W13)

| ID | Upstream | Rust gap | Hardware (2026-07-06) | Experiment |
| --- | --- | --- | --- | --- |
| W7 ASIC max baud reg | reg28 @ 115200 → host 1M | **match** (Wave 1 deep dive) | REG28 on wire; chip detect RX OK | default |
| W8 frequency ramp | 50→485 MHz stepped ramp | Optional `frequency_ramp` flag | No RX improvement (run B 2026-07-05) | `frequency_ramp` |
| W9 stabilization delay | 2000ms after max baud (`asic_init.c:63-66`) | `post_max_baud_delay_2000` flag adds 2s after `clear_rx` | E2: ~2s gap clear_rx→next action; **no post-work RX** | `post_max_baud_delay_2000` |
| W10 result loop | Continuous `receive_work` 10s | Single boot diagnostic read; bridge pump exists | Bridge dispatches pool work; **zero post-dispatch `rx_chunk`** | bridge E1 |
| W11 work source | Pool-derived `BM1366_send_work` | Boot diagnostic synthetic; bridge uses pool notify | Production TX logged (88 B); no nonce/result parse | bridge after B1/B2 |
| W12 pool bridge | Stratum + NVS pool keys | **match after B2** | `connecting` → `subscribed` → `authorized` → `active` | bridge-blocker-fix |
| W13 bootstrap gate | Upstream always mines after init | Default fail-closed on diagnostic timeout | E3: `fail_closed reason=work_result_diagnostic_timeout` | default package |

## Hardware matrix (blocker fix wave)

| Run | Config | Post-work UART | Bridge / init |
| --- | --- | --- | --- |
| E1 (bridge) | W7 + `initialized_no_mining_gate` + pool bridge | **FAIL** — chip-detect RX only; `production_result_timeout` | **PASS** — Stratum active, `work_dispatched` |
| E2 | W7 + `post_max_baud_delay_2000` | **FAIL** — diagnostic timeout @ 1M | 2s delay confirmed between `clear_rx` and next init step |
| E3 | Default (no W5 bootstrap) | N/A (fail-closed before bridge) | **PASS baseline** — `work_result_diagnostic_timeout` |

See `work-result-blocker-fix-20260706-run-{E2,E3}.md` and `bridge-blocker-fix-20260706/`.

## Primary suspect (post W7/W9)

Missing or mis-timed **continuous production result read** (W10) and/or pool-work field encoding vs upstream `BM1366_send_work` (W11). Baud and stabilization delays alone do not restore post-work RX.

**Superseded by:** [`work-result-upstream-diff-v4.md`](work-result-upstream-diff-v4.md) (B3 production-read hardware matrix).
