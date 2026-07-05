# Work-Result Hypothesis Results (Deep Dive v2)

Date: 2026-07-05  
Board: Ultra 205 (`port=/dev/cu.usbmodem1101`)

## Pre-W7 outcomes (first fix bundle)

| ID | Hypothesis | Result | Evidence |
| --- | --- | --- | --- |
| W1 | Missing mining-ready init after chip detect | **CONFIRMED (fixed)** | 18 init frames between chip detect and diagnostic work |
| W2 | Missing host max baud before work | **PARTIAL** | Host `use_max_baud baud=1000000` applied; still zero RX |
| W3 | Wrong `address_interval` (16 vs 256) | **FIXED** | Parser uses `address_interval=256` |
| W4 | Single 10s read vs mining-loop receive | **INCONCLUSIVE** | Zero RX at 1M after init |
| W5 | No nonce in 10s despite valid init | **CONFIRMED** | Timeout with no `asic_uart_trace=rx_chunk` |
| W6 | Job frame encoding mismatch | **REJECTED** | 88-byte TX matches golden diagnostic frame |

## Deep-dive hypotheses (W7–W12)

| ID | Hypothesis | Result | Evidence |
| --- | --- | --- | --- |
| W7 | Missing ASIC-side reg 0x28 before host 1M | **FIX APPLIED — RX still silent** | REG28 TX confirmed; post-work diagnostic still times out at 1M and 115200 |
| W8 | Single frequency step vs 50→485 MHz ramp | **FIX AVAILABLE** | `frequency_ramp` investigation flag; 6.25 MHz steps with 100 ms delay |
| W9 | Voltage/UART flush ordering before init | **NOT CHANGED** | Phase 27 safety bring-up covers voltage; no divergence evidence on silence |
| W10 | Single boot read vs continuous result loop | **NOT CHANGED** | Bridge uses pump loop; boot diagnostic remains single 10s read |
| W11 | Synthetic diagnostic job may not nonce | **DOCUMENTED** | Boot diagnostic uses fixed `job_id=0x28` and synthetic fields; register-read or production pool work required for nonce proof |
| W12 | Bridge blocked on pool settings | **SEPARATE** | `phase27_pool_wait_timeout` without local pool creds |

## Bootstrap gate (Wave 4)

Default Phase 27 behavior after deep dive:

- **Accept register-read or job-nonce parse** as UART proof (`asic_work_result_trace=register_read_parsed` or `bm1366_diagnostic_result=parsed`).
- **W5 timeout bootstrap disabled by default** — production UART is not retained on diagnostic timeout unless `BITAXE_WORK_RESULT_INVESTIGATION=initialized_no_mining_gate`.
- **`require_diagnostic_nonce`** disables bootstrap entirely (run D control).

## Investigation flags

| Env value | Effect |
| --- | --- |
| (default) | W7 reg28 prelude + host 1M |
| `skip_asic_max_baud` | Skip reg28; host still switches to 1M |
| `skip_max_baud` | Stay @ 115200 (run C control) |
| `frequency_ramp` | W8 ramp before nonce space |
| `require_diagnostic_nonce` | No W5 bootstrap on timeout/parse miss |
| `initialized_no_mining_gate` | Explicit W5 bootstrap on timeout |

## Hardware matrix evidence

See `work-result-deep-dive-20260705-run-*.md` under this phase directory.
