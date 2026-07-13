# Chip-Detect Upstream Diff Trace Matrix

Reference pin: `reference/esp-miner` @ `c1915b0a63bfabebdb95a515cedfee05146c1d50`

Rust path: Phase 27 `Phase27ProductionBridge` → safety bring-up → `run_chip_detect_actions` → `Bm1366InitPlan::chip_detect_only`

Upstream path: `main.c` boot → `VCORE_init` / thermal → `asic_initialize` → `ASIC_init` → `BM1366_init`

| Step | Upstream @ c1915b0 | Rust Phase 27 | Verdict | Evidence |
| --- | --- | --- | --- | --- |
| Reset pulse timing | `asic_reset`: 100 ms low, 100 ms high (`main/power/asic_reset.c:9-15`) | `AsicReset::reset_pulse(100, 100)` (`firmware/bitaxe/src/asic_adapter/reset.rs:25-33`) | **match** | Same GPIO timing constants |
| Reset count before chip-ID | Single pulse inside `asic_initialize` after VCORE/thermal (`main/power/asic_init.c:17-48`) | Safety bring-up pulse + chip-detect plan pulse (`phase27_bring_up.rs`, `init_plan.rs:43`) | **diverge** | Double reset on Phase 27 path (H5) |
| UART TX/RX pins | GPIO17 TX, GPIO18 RX (`components/asic/serial.c:15-16,35-36`) | GPIO17/18 (`firmware/bitaxe/src/asic_adapter/uart.rs:10-11`) | **match** | Pin constants aligned |
| Initial baud | 115200 `UART_FREQ` (`components/asic/include/serial.h:12`) | 115200 `UART_INITIAL_BAUD` (`uart.rs:9`) | **match** | |
| UART RX buffer | `BUF_SIZE * 2` = 2048 B (`serial.c:17,41`) | `UART_BUF_SIZE * 2` = 2048 B (`uart.rs:16-17,39`) | **match** | |
| Pre-detect version mask | 3× `BM1366_set_version_mask(0x1fffe000)` before chip-ID (`bm1366.c:195-198`, `utils.h`) | None in `chip_detect_only`; once after detect in `full_init` (`init_plan.rs:97-98`) | **diverge** | Order and count differ (H2) |
| Chip-ID TX frame | `{55 AA 52 05 00 00 0A}` via `_send_simple` (`bm1366.c:201-202`) | `Bm1366Command::ReadChipId` golden frame (`crates/bitaxe-asic/src/lib.rs`) | **match** | 7-byte command frame identical |
| Wait TX done before read | Not explicit before chip-ID; only on baud change (`serial.c:53-54`) | Not emitted in chip-detect plan | **match** | Both omit for chip-ID; investigation adds for H3 |
| Clear RX before read | Not before chip-ID loop | Not before read | **match** | |
| RX read strategy | `count_asic_chips`: loop `SERIAL_rx(11, 1000)` until timeout 0; retry on bad preamble/CRC (`asic_common.c:89-120`) | Single `driver.read()` in `read_exact`; fail on first partial (`uart.rs:69-81`) | **diverge** | Rust fails closed on 9/11; upstream also breaks on wrong length but loops for next frame attempt (H1) |
| Partial read handling | `received != 11` → log hex, **break** (fail) (`asic_common.c:98-101`) | `read != len` → clear_rx, bail partial error | **diverge** | Same fail on partial single read; Rust lacks multi-read accumulate within deadline (H1) |
| Soft error retry | `continue` on preamble/CRC mismatch (`asic_common.c:105-120`) | Fail-closed on first invalid frame (`chip_detect.rs:68-83`) | **diverge** | Upstream tolerates garbage between frames |
| Init scope after detect | Full register init, frequency, max baud 1M (`bm1366.c:211+`) | Stops at chip detect for Phase 27 bridge prelude | **diverge** | H4: full_init prefix not run before production bridge |
| Power/thermal before ASIC | `VCORE_init`, fan/thermal before `asic_initialize` | Phase 27 safety bring-up (INA260, DS4432U, EMC2101, enable) | **match** | Both enable power/thermal before UART chip detect |

## Ranked hypotheses (from divergences)

1. **H1 (HIGH):** Single-shot UART read returns 9 bytes before remaining 2 arrive; need `read_accumulate` within timeout.
2. **H2 (MEDIUM-HIGH):** Missing 3× version mask before chip-ID may leave ASIC chain unprepared.
3. **H3 (MEDIUM):** Missing `WaitTxDone` after TX before RX (investigation-only unless trace shows TX incomplete).
4. **H4 (MEDIUM):** Chip-detect-only scope vs upstream full `BM1366_init` prefix.
5. **H5 (LOW-MEDIUM):** Double reset pulse (safety + plan) vs upstream single pulse.

## Investigation compile flags

| Env | Purpose |
| --- | --- |
| `BITAXE_ASIC_UART_TRACE=1` | Hex/timing UART trace logs |
| `BITAXE_CHIP_DETECT_INVESTIGATION=version_mask_prelude` | H2 only |
| `BITAXE_CHIP_DETECT_INVESTIGATION=wait_tx_clear_rx` | H3 only |
| `BITAXE_CHIP_DETECT_INVESTIGATION=skip_second_reset` | H5 only |
| `BITAXE_CHIP_DETECT_INVESTIGATION=full_init_prefix` | H4 only |

Phase 27 production fix applies upstream-aligned prelude (version mask ×3, skip second reset, wait TX done) plus `read_accumulate` by default when investigation env is unset.
