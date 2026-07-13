# Work-Result Upstream Diff v6 (H4 Task Orchestration)

Reference pin: `reference/esp-miner` @ `c1915b0`

Prior: [`work-result-upstream-diff-v5.md`](work-result-upstream-diff-v5.md)

## Upstream task model (post-`asic_initialize`)

| Task | Priority | Start condition | Core loop |
| --- | --- | --- | --- |
| `create_jobs_task` | 20 | `ASIC_initalized == true` | Dequeue stratum work (or hold `current_work`); on interval timeout re-send job with new extranonce_2 (V1); log `ASIC Ready!` |
| `ASIC_result_task` | 15 | `ASIC_initalized == true` | Forever: `ASIC_process_work` → `receive_work(10s)`; timeout returns NULL and **continues** (no fail-closed) |

Task creation order in `main.c`: `create_jobs_task` first, then `ASIC_result_task` — both start only after successful `asic_initialize()`.

## Rust Phase 27 bridge model (pre-H4)

| Aspect | Upstream | Rust (B4) | Gap |
| --- | --- | --- | --- |
| Job source | `create_jobs_task` queue + periodic re-feed | Single `WorkQueued` notify → one dispatch | **H4 job pump** |
| Result receive | Dedicated task, always blocking 10s | Poll slices only after dispatch; **fail-closed on timeout** | **H4 continuous listener** |
| Timeout handling | NULL → continue loop | `production_result_timeout` → mining disabled | **H4 non-fatal timeout** |
| Register reads | Handled in result task before job correlation | Same parser path but only after dispatch window | **H4 pre-dispatch listen** |

## Timing notes

- `receive_work` uses `SERIAL_rx(..., 10000)` — 10s blocking read per upstream iteration.
- `create_jobs_task` initial `timeout_ms` comes from `ASIC_get_asic_job_frequency_ms`; on empty queue with held work, V1 re-generates extranonce and re-sends at that interval (~2s class on Ultra boards per prior captures).
- Rust bridge pump: 32 base iterations + extended window while `awaiting_result_read`; 100ms-class socket interleave slices.

## H4 investigation flags (Wave B5)

| Flag | Upstream behavior emulated |
| --- | --- |
| `continuous_result_task` | Arm listener when production UART ready; poll UART before/after dispatch; **timeout continues** instead of fail-closed |
| `job_redispatch_pump` | After first pool dispatch, re-queue dispatch every 2000ms while registry valid (create_jobs re-feed) |

Recommended matrix combos (build via comma-separated `BITAXE_WORK_RESULT_INVESTIGATION`):

| Run | Modes | Hypothesis |
| --- | --- | --- |
| J1 | `continuous_result_task,initialized_no_mining_gate,frequency_ramp` | Upstream-style always-on result task |
| J2 | `continuous_result_task,job_redispatch_pump,initialized_no_mining_gate,frequency_ramp` | Result task + job re-feed |
| J3 | `continuous_result_task,job_redispatch_pump,skip_boot_diagnostic_work,initialized_no_mining_gate,frequency_ramp` | J2 + H3 boot path |

Success markers: `h4_continuous_result=timeout_continue` (non-fatal), `asic_uart_trace=rx_chunk`, `register_read_parsed`, or `result_correlated`.

Reference files:

- [`reference/esp-miner/main/tasks/asic_result_task.c`](../../../reference/esp-miner/main/tasks/asic_result_task.c)
- [`reference/esp-miner/main/tasks/create_jobs_task.c`](../../../reference/esp-miner/main/tasks/create_jobs_task.c)
- [`reference/esp-miner/components/asic/asic_common.c`](../../../reference/esp-miner/components/asic/asic_common.c) — `receive_work`
- [`reference/esp-miner/components/asic/bm1366.c`](../../../reference/esp-miner/components/asic/bm1366.c) — `BM1366_process_work`

Rust files:

- [`firmware/bitaxe/src/live_stratum_runtime.rs`](../../../firmware/bitaxe/src/live_stratum_runtime.rs)
- [`firmware/bitaxe/src/asic_adapter/work_result_investigation.rs`](../../../firmware/bitaxe/src/asic_adapter/work_result_investigation.rs)
