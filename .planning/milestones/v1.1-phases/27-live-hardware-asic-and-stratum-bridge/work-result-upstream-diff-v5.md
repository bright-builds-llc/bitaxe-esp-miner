# Work-Result Upstream Diff v5 (Init State and Mining Enable)

Reference pin: `reference/esp-miner` @ `c1915b0`

Prior diffs: [`work-result-upstream-diff-v4.md`](work-result-upstream-diff-v4.md)

Rust path after B3: safety bring-up → chip detect → `mining_ready_init` → **boot diagnostic work** → W5 bootstrap → bridge pool work @ 1M → 10s polled result read

Upstream path: power prelude → `asic_initialize()` → `create_jobs_task` + `ASIC_result_task` → pool work only (no synthetic diagnostic)

## Init sequence mapping

| Step | Upstream (`asic_init.c` → `BM1366_init`) | Rust Phase 27 | Match |
| --- | --- | --- | --- |
| 1 | `asic_reset()` — 100ms pulse | `phase27_bring_up` + chip-detect reset | Partial (Rust may double-reset) |
| 2 | `SERIAL_init` @ 115200 | `AsicUart::new` @ 115200 | Match |
| 3 | `ASIC_init` → chip detect | `run_chip_detect_actions` | Match |
| 4 | Post-detect register sequence (0xA8, 0x18, 0x3C, 0x54, 0x58, 0x2C, per-chip) | `mining_ready_commands()` | **Match** (golden fixtures) |
| 5 | `do_frequency_transition` 50→485 MHz, 100ms steps | Default: single `SetFrequency(485)`; flag: `frequency_ramp` | **Diverge (W8)** |
| 6 | `BM1366_set_nonce_space` + reg 0xA4 | `SetNonceSpace` + `WriteRegister(0xA4)` | Match |
| 7 | `BM1366_set_max_baud` reg28 @ 115200 | `SetAsicMaxBaud` in `max_baud_prelude_actions` | **Match** (post-W7) |
| 8 | Host `SERIAL_set_baud(1M)` + clear buffer | `UseMaxBaud` + `ClearRx` | Match |
| 9 | Optional 2000ms stabilization (recovery init) | `post_max_baud_delay_2000` flag | **Diverge (W9, low)** |
| 10 | `GLOBAL_STATE->ASIC_initalized = true` | `AsicInitStatus::InitializedNoMining` after mining_ready | Partial |
| 11 | **No synthetic work** — start `create_jobs_task` | **Synthetic diagnostic job** `job_id=0x28` + 10s read | **Diverge (H3)** |
| 12 | W5 bootstrap on diagnostic timeout | `initialized_no_mining_gate` retains UART without proof | **Diverge (W13)** |
| 13 | `create_jobs_task: ASIC Ready!` → pool jobs ~2000ms | Bridge dispatch on Stratum notify | Partial (W10–W11) |
| 14 | `ASIC_result_task` continuous `receive_work(10s)` | Bridge polled read loop (B3 fixed) | **Match** (timing); still silent |

Reference files:

- [`reference/esp-miner/main/power/asic_init.c`](../../../reference/esp-miner/main/power/asic_init.c) — lines 12–69
- [`reference/esp-miner/components/asic/bm1366.c`](../../../reference/esp-miner/components/asic/bm1366.c) — `BM1366_init` 191–277, `BM1366_set_max_baud` 297–304
- [`reference/esp-miner/components/asic/frequency_transition_bmXX.c`](../../../reference/esp-miner/components/asic/frequency_transition_bmXX.c)
- [`reference/esp-miner/main/tasks/create_jobs_task.c`](../../../reference/esp-miner/main/tasks/create_jobs_task.c) — `ASIC Ready!`

Rust files:

- [`firmware/bitaxe/src/safety_adapter/phase27_bring_up.rs`](../../../firmware/bitaxe/src/safety_adapter/phase27_bring_up.rs)
- [`crates/bitaxe-asic/src/bm1366/mining_ready.rs`](../../../crates/bitaxe-asic/src/bm1366/mining_ready.rs)
- [`firmware/bitaxe/src/asic_adapter.rs`](../../../firmware/bitaxe/src/asic_adapter.rs) — `run_work_result_uart_bootstrap_after_reset`
- [`firmware/bitaxe/src/asic_adapter/work_result_investigation.rs`](../../../firmware/bitaxe/src/asic_adapter/work_result_investigation.rs)

## Register/frame gaps (beyond W8)

| Register / frame | Upstream | Rust | Gap |
| --- | --- | --- | --- |
| 0x54, 0x58, 0x2C global | init138–171 in `BM1366_init` | `mining_ready_commands` | **None** |
| Per-chip 0xA8/0x18/0x3C×3 | Loop in `BM1366_init` | Loop in `mining_ready_commands` | **None** |
| Version mask ×3 | Before chip detect in `BM1366_init` | Chip detect path / separate | Documented deferral |
| Synthetic diagnostic job | **Absent** | Boot gate always sends | **H3 gap** |
| Explicit mining enable register | Not found as separate step; implied by task start | `mining=disabled` until bridge | **W13 semantic gap** |

No missing register frames identified beyond frequency ramp default and boot-path semantics.

## Remaining divergences (W8, W13, H3, H4)

| ID | Upstream | Rust gap | Experiment |
| --- | --- | --- | --- |
| W8 | Stepped 50→485 MHz ramp | Default single frequency set | `frequency_ramp` (+ bootstrap combo G1) |
| W13 | `ASIC_initalized` after full init; always mines | Bootstrap without UART proof | `initialized_no_mining_gate` vs `require_uart_proof_for_production` |
| H3 | Pool work only after init | Synthetic diagnostic before bridge | `skip_boot_diagnostic_work` |
| H4 | `create_jobs_task` + `ASIC_result_task` parallel | Notify-driven bridge pump | Observe after H3/W8 fixes |
| W9 | 2000ms post max baud (recovery) | Optional flag | G3 combo |
| W10/W11 | — | **Fixed / ruled out** (B3) | — |

## B4 investigation matrix (2026-07-06)

| Run | Config | Post-work UART | Notes |
| --- | --- | --- | --- |
| G1 | `frequency_ramp,initialized_no_mining_gate` | **FAIL** — ~10s poll, silent | Canonical B4 evidence |
| G2 | `require_diagnostic_nonce,initialized_no_mining_gate` | N/A | **PASS** — fail-closed pre-bridge |
| G3 | ramp + W9 delay | N/A | **BLOCKED** — USB connection |
| H1 | `skip_boot_diagnostic_work` | N/A | **BLOCKED** — pending retry |
| H2 | `require_uart_proof_for_production` | N/A | **BLOCKED** — code matches G2 |
| H3 | ramp + skip diagnostic | N/A | **BLOCKED** — pending retry |

## Wave 3 structural changes (code)

- Comma-separated `BITAXE_WORK_RESULT_INVESTIGATION` modes
- Bridge-default frequency ramp (`skip_frequency_ramp` to disable)
- `skip_boot_diagnostic_work` — pool-only path after mining_ready_init
- `require_uart_proof_for_production` — tight bootstrap control

**Superseded by:** B4 hardware outcomes in `b4-init-state-20260706-run-*.md`.

