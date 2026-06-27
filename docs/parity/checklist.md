# Parity Checklist

This checklist is the audit source of truth for device-user parity against `reference/esp-miner`. It is not a generic task list.

## Status Values

- `not-started`: No Rust-owned implementation exists yet.
- `in-progress`: Rust-owned implementation exists but is incomplete.
- `implemented`: Code exists, but parity evidence is incomplete.
- `verified`: Evidence proves behavior matches the reference or accepted project behavior.
- `deferred`: Gap is accepted with reason and owner.

## Evidence Types

- `unit`: Pure Rust unit tests compare behavior to reference-derived fixtures.
- `golden`: Generated output matches checked-in golden data derived from upstream behavior.
- `api-compare`: Rust firmware response matches upstream OpenAPI/schema or captured upstream response.
- `hardware-smoke`: Behavior observed on named physical Bitaxe hardware, with board, command, firmware commit, reference commit, and log captured.
- `hardware-regression`: Repeatable hardware test or scripted probe passes.
- `workflow`: Repo-owned command, build, package, flash-shaped, or report workflow passes with captured plan evidence.
- `deferred`: Accepted gap with reason and owner.

Safety-critical and hardware-control surfaces require hardware evidence before `verified`: voltage, fan, thermal, power, and ASIC initialization.

## Foundation And Workflow

| ID | Surface | Reference Breadcrumb | Rust-Owned Target | Status | Evidence | Notes |
| --- | --- | --- | --- | --- | --- | --- |
| WF-001 | Read-only reference submodule | `reference/esp-miner` | `scripts/verify-reference-clean.sh`, `tools/parity` | verified | workflow | `just verify-reference` passed in Phase 01 Plan 09 and printed `reference clean: c1915b0a63bfabebdb95a515cedfee05146c1d50`; package and parity targets run the guard before trusted output. |
| WF-002 | Bazel build graph | `reference/esp-miner/CMakeLists.txt` | `MODULE.bazel`, `tools/parity/BUILD.bazel` | verified | workflow | `just build`, `just test`, `just package`, and `just parity` passed after the Ultra 205 pivot; see `docs/parity/evidence/ultra-205-pivot-safe-state-smoke-2026-06-26.md`. |
| WF-003 | Human command surface | `reference/esp-miner/README.md` | `Justfile` | verified | workflow | `Justfile` exposes `build`, `test`, `package`, `flash`, `monitor`, `flash-monitor`, `verify-reference`, and `parity` as thin Bazel wrappers. |
| WF-004 | Firmware image packaging | `reference/esp-miner/merge_bin.sh` | `//firmware/bitaxe:firmware_image` | verified | workflow | `just package` produced `bitaxe-ultra205.elf`, `bitaxe-ultra205-factory.bin`, and `bitaxe-ultra205-package.json` after the pivot; see `docs/parity/evidence/ultra-205-pivot-safe-state-smoke-2026-06-26.md`. |
| WF-005 | USB flash workflow | `reference/esp-miner/flashing.md`, `reference/esp-miner/tools/upload2device.py` | `tools/flash`, `Justfile` | verified | hardware-smoke | `tools/flash` defaults to `board=205`, rejects deferred `board=601`, and `just flash-monitor board=205 port=/dev/cu.usbmodem1101` flashed and monitored the connected Ultra 205; see `docs/parity/evidence/ultra-205-pivot-safe-state-smoke-2026-06-26.md`. |

## Boot And System Runtime

| ID | Surface | Reference Breadcrumb | Rust-Owned Target | Status | Evidence | Notes |
| --- | --- | --- | --- | --- | --- | --- |
| SYS-001 | App entrypoint boot order | `reference/esp-miner/main/main.c` | `firmware/bitaxe` | verified | hardware-smoke | Connected Ultra 205 boot log contains `bitaxe-rust boot: board=Ultra 205 asic=BM1366`; see `docs/parity/evidence/ultra-205-pivot-safe-state-smoke-2026-06-26.md`. |
| SYS-002 | PSRAM availability handling | `reference/esp-miner/main/main.c` | `firmware/bitaxe` | verified | hardware-smoke | Connected Ultra 205 boot log contains `psram_status=unavailable` and platform boot details; see `docs/parity/evidence/ultra-205-pivot-safe-state-smoke-2026-06-26.md`. |
| SYS-003 | Global system status model | `reference/esp-miner/main/global_state.h` | `crates/bitaxe-core` | verified | hardware-smoke | Connected Ultra 205 boot log contains `safe_state: mining=disabled asic_work_submission=disabled hardware_control=disabled`. This remains the only allowed Ultra 205 firmware behavior until hardware-control phases add evidence. |
| SYS-004 | Version reporting | `reference/esp-miner/main/system.c` | `firmware/bitaxe`, `crates/bitaxe-core`, `crates/bitaxe-api`, `tools/parity` | in-progress | pending | Firmware logs and parity reports include source/reference identifiers; API version surface remains later-phase work. |
| SYS-005 | Task orchestration behavior | `reference/esp-miner/main/tasks/*.c` | `firmware/bitaxe` | not-started | pending | Internal task layout may differ if observable behavior matches. |

## Board Config And NVS

| ID | Surface | Reference Breadcrumb | Rust-Owned Target | Status | Evidence | Notes |
| --- | --- | --- | --- | --- | --- | --- |
| CFG-001 | Ultra 205 defaults | `reference/esp-miner/config-205.cvs` | `crates/bitaxe-config/src/defaults.rs`, `crates/bitaxe-config/fixtures/ultra-205-defaults.csv` | implemented | unit,golden | `Ultra205Defaults` and fixture-backed tests cover pure Ultra 205 hostname, pool, ASIC, fan, self-test, device, board, frequency, and voltage defaults; see [Phase 2 evidence](evidence/phase-02-ultra-205-config-nvs-model.md). Hardware use of frequency/voltage remains unverified until safety-critical evidence exists. |
| CFG-002 | Deferred Gamma 601 defaults | `reference/esp-miner/config-601.cvs` | `crates/bitaxe-config` | deferred | deferred | Gamma 601/BM1370 remains in project scope but is deferred after the Ultra 205 path; [Phase 2 evidence](evidence/phase-02-ultra-205-config-nvs-model.md) does not verify Gamma 601 or BM1370 behavior. |
| CFG-003 | Board/device model table | `reference/esp-miner/main/device_config.h` | `crates/bitaxe-config/src/catalog.rs` | verified | unit,golden | `BoardCatalogEntry`, `AsicProfile`, and `VerificationScope` model Ultra 205/BM1366 plus upstream non-205 boards; [Phase 2 evidence](evidence/phase-02-ultra-205-config-nvs-model.md) records that non-205 catalog entries are not hardware-verified. |
| CFG-004 | NVS key model | `reference/esp-miner/main/nvs_config.c` | `crates/bitaxe-config/src/nvs.rs`, `crates/bitaxe-config/src/persistence.rs` | verified | unit,golden | Exact key names, defaults, missing-key loading, legacy migrations, corrupt float fallback, and pure snapshot reload semantics are covered by [Phase 2 evidence](evidence/phase-02-ultra-205-config-nvs-model.md). |
| CFG-005 | Runtime settings update behavior | `reference/esp-miner/main/http_server/system_api_json.c` | `crates/bitaxe-config/src/settings.rs`, `crates/bitaxe-config/src/persistence.rs` | implemented | unit | Pure settings update/reload tests cover accepted writes, rejected invalid updates, and legacy mirrors; [Phase 2 evidence](evidence/phase-02-ultra-205-config-nvs-model.md) does not verify the API PATCH route or firmware NVS adapter. |

## ASIC And Mining Hardware

| ID | Surface | Reference Breadcrumb | Rust-Owned Target | Status | Evidence | Notes |
| --- | --- | --- | --- | --- | --- | --- |
| ASIC-001 | ASIC dispatch by model | `reference/esp-miner/components/asic/asic.c` | `crates/bitaxe-asic/src/dispatch.rs` | verified | unit | `cargo test -p bitaxe-asic dispatch --all-features` from Phase 03 Plan 03 proves Ultra 205 BM1366 is the only active V1 dispatch path and non-205 ASIC families remain deferred/not hardware-verified. |
| ASIC-002 | BM1366 initialization | `reference/esp-miner/components/asic/bm1366.c:BM1366_init`, `reference/esp-miner/main/power/asic_reset.c` | `crates/bitaxe-asic/src/bm1366/init_plan.rs`, `firmware/bitaxe/src/asic_adapter.rs`, `firmware/bitaxe/src/asic_adapter/reset.rs` | implemented | unit,workflow | Pure staged init and default fail-closed firmware gate are implemented by Phase 03 Plans 04-05. Live Ultra 205 chip-detect evidence is [not run - hardware verification pending](evidence/phase-03-ultra-205-bm1366-chip-detect.md), so initialization is not verified. |
| ASIC-003 | BM1366 work send | `reference/esp-miner/components/asic/bm1366.c:BM1366_send_work` | `crates/bitaxe-asic/src/bm1366/work.rs`, `crates/bitaxe-asic/src/bm1366/command.rs` | implemented | unit,golden | Phase 03 Plan 02 covers diagnostic work payload and job-frame encoding only. Production work submission and live work-send evidence remain pending in [Phase 03 chip-detect evidence](evidence/phase-03-ultra-205-bm1366-chip-detect.md). |
| ASIC-004 | BM1366 result parsing | `reference/esp-miner/components/asic/bm1366.c:BM1366_process_work`, `reference/esp-miner/components/asic/asic_common.c` | `crates/bitaxe-asic/src/bm1366/result.rs`, `crates/bitaxe-asic/src/bm1366/transcript.rs` | implemented | unit,golden | Phase 03 Plans 02-03 cover 11-byte result parsing, nonce/register observations, and fake UART fault transcripts. Live result-receive evidence remains pending in [Phase 03 chip-detect evidence](evidence/phase-03-ultra-205-bm1366-chip-detect.md). |
| ASIC-005 | ASIC serial transport | `reference/esp-miner/components/asic/serial.c` | `firmware/bitaxe/src/asic_adapter/uart.rs`, `firmware/bitaxe/src/asic_adapter.rs` | implemented | workflow | Firmware compiles with typed UART1 TX17/RX18 adapter actions and default fail-closed gate in Phase 03 Plan 05. Live serial chip-detect evidence is [not run - hardware verification pending](evidence/phase-03-ultra-205-bm1366-chip-detect.md). |
| ASIC-006 | ASIC CRC behavior | `reference/esp-miner/components/asic/crc.c` | `crates/bitaxe-asic/src/bm1366/crc.rs`, `crates/bitaxe-asic/src/bm1366/packet.rs` | implemented | unit,golden | Phase 03 Plan 01 unit and fixture coverage proves BM1366 command CRC and job CRC behavior in pure Rust; live ASIC communication evidence remains separate and pending. |
| ASIC-007 | Frequency transition behavior | `reference/esp-miner/components/asic/frequency_transition_bmXX.c` | `crates/bitaxe-asic/src/bm1366/frequency_voltage.rs`, `firmware/bitaxe/src/asic_adapter.rs` | implemented | unit | Phase 03 Plan 04 range-checks pure BM1366 frequency decisions and marks hardware effect status missing evidence. No live frequency transition has been run; see [Phase 03 chip-detect evidence](evidence/phase-03-ultra-205-bm1366-chip-detect.md). |
| ASIC-008 | BM1370 parity | `reference/esp-miner/components/asic/bm1370.c` | `crates/bitaxe-asic` | deferred | deferred | Needed for Gamma 601 and related boards after the Ultra 205/BM1366 path. |
| ASIC-009 | BM1368 parity | `reference/esp-miner/components/asic/bm1368.c` | `crates/bitaxe-asic` | not-started | pending | Later board expansion. |
| ASIC-010 | BM1397 parity | `reference/esp-miner/components/asic/bm1397.c` | `crates/bitaxe-asic` | not-started | pending | Later board expansion. |

## Stratum And Mining Logic

| ID | Surface | Reference Breadcrumb | Rust-Owned Target | Status | Evidence | Notes |
| --- | --- | --- | --- | --- | --- | --- |
| STR-001 | Stratum v1 socket behavior | `reference/esp-miner/components/stratum/stratum_socket.c` | `crates/bitaxe-stratum`, `firmware/bitaxe` | not-started | pending | Include reconnect/fallback behavior. |
| STR-002 | Stratum v1 API messages | `reference/esp-miner/components/stratum/stratum_api.c` | `crates/bitaxe-stratum` | not-started | pending | Unit/golden fixtures from upstream tests. |
| STR-003 | Mining job construction | `reference/esp-miner/components/stratum/mining.c` | `crates/bitaxe-stratum`, `crates/bitaxe-core` | not-started | pending | Pure logic should be heavily unit tested. |
| STR-004 | Coinbase decoding | `reference/esp-miner/components/stratum/coinbase_decoder.c` | `crates/bitaxe-stratum` | not-started | pending | Good unit/golden target. |
| STR-005 | Stratum v2 protocol | `reference/esp-miner/components/stratum_v2/*.c` | `crates/bitaxe-stratum` | not-started | pending | Later parity path if not needed for first mining loop. |
| STR-006 | Protocol coordinator | `reference/esp-miner/main/tasks/protocol_coordinator.c` | `firmware/bitaxe`, `crates/bitaxe-core` | not-started | pending | Observable protocol selection and fallback behavior. |

## AxeOS API And Web Compatibility

| ID | Surface | Reference Breadcrumb | Rust-Owned Target | Status | Evidence | Notes |
| --- | --- | --- | --- | --- | --- | --- |
| API-001 | OpenAPI schema compatibility | `reference/esp-miner/main/http_server/openapi.yaml` | `crates/bitaxe-api` | not-started | pending | Schema is the API contract. |
| API-002 | System info response | `reference/esp-miner/main/http_server/system_api_json.c` | `crates/bitaxe-api` | not-started | pending | Compare fields and encoding. |
| API-003 | System settings PATCH | `reference/esp-miner/main/http_server/system_api_json.c` | `crates/bitaxe-api`, `crates/bitaxe-config` | not-started | pending | Settings update behavior. |
| API-004 | HTTP server routes | `reference/esp-miner/main/http_server/http_server.c` | `firmware/bitaxe` | not-started | pending | Include restart, identify, OTA, OTAWWW, logs. |
| API-005 | WebSocket logs | `reference/esp-miner/main/http_server/websocket_log.c` | `firmware/bitaxe`, `crates/bitaxe-api` | not-started | pending | Preserve client-facing stream behavior. |
| API-006 | Live WebSocket telemetry | `reference/esp-miner/main/http_server/websocket_api.c` | `firmware/bitaxe`, `crates/bitaxe-api` | not-started | pending | API compare plus hardware smoke. |
| API-007 | Recovery page | `reference/esp-miner/main/http_server/recovery_page.html` | `firmware/bitaxe` | not-started | pending | Asset compatibility, not UI rewrite. |
| API-008 | Static AxeOS asset packaging | `reference/esp-miner/main/http_server/axe-os` | `firmware/bitaxe` | not-started | pending | Serve compatible assets or packaged equivalent. |

## Power, Thermal, Fan, And Peripherals

| ID | Surface | Reference Breadcrumb | Rust-Owned Target | Status | Evidence | Notes |
| --- | --- | --- | --- | --- | --- | --- |
| PWR-001 | ASIC reset behavior | `reference/esp-miner/main/power/asic_reset.c` | `firmware/bitaxe` | not-started | pending | Safety-critical; requires hardware evidence. |
| PWR-002 | ASIC power initialization | `reference/esp-miner/main/power/asic_init.c` | `firmware/bitaxe` | not-started | pending | Safety-critical; requires hardware evidence. |
| PWR-003 | Core voltage control | `reference/esp-miner/main/power/vcore.c` | `firmware/bitaxe` | not-started | pending | Safety-critical; requires hardware evidence. |
| PWR-004 | TPS546 support | `reference/esp-miner/main/power/TPS546.c` | `firmware/bitaxe` | deferred | deferred | Relevant to deferred Gamma 601 and related boards. |
| PWR-005 | DS4432U support | `reference/esp-miner/main/power/DS4432U.c` | `firmware/bitaxe` | not-started | pending | Relevant to Ultra 205 first path; safety-critical. |
| PWR-006 | INA260 support | `reference/esp-miner/main/power/INA260.c` | `firmware/bitaxe` | not-started | pending | Current/power telemetry for Ultra 205 first path. |
| THR-001 | Thermal model | `reference/esp-miner/main/thermal/thermal.c` | `firmware/bitaxe`, `crates/bitaxe-core` | not-started | pending | Safety-critical. |
| THR-002 | Fan controller task | `reference/esp-miner/main/tasks/fan_controller_task.c` | `firmware/bitaxe`, `crates/bitaxe-core` | not-started | pending | Safety-critical. |
| THR-003 | PID behavior | `reference/esp-miner/main/thermal/PID.c` | `crates/bitaxe-core` | not-started | pending | Good pure unit target before hardware. |
| IO-001 | I2C initialization | `reference/esp-miner/main/i2c_bitaxe.c` | `firmware/bitaxe/src/display_adapter.rs` | in-progress | unit,workflow,hardware-smoke | Startup debug display initializes I2C0 on SDA GPIO47/SCL GPIO48 at 400 kHz for SSD1306 address `0x3c` only; shared I2C device map and other I2C peripherals remain pending. |
| IO-002 | ADC behavior | `reference/esp-miner/main/adc.c` | `firmware/bitaxe` | not-started | pending | Hardware adapter. |

## Display, Input, Self-Test, And BAP

| ID | Surface | Reference Breadcrumb | Rust-Owned Target | Status | Evidence | Notes |
| --- | --- | --- | --- | --- | --- | --- |
| UI-001 | Display behavior | `reference/esp-miner/main/display.c` | `firmware/bitaxe/src/display_adapter.rs` | in-progress | unit,workflow,hardware-smoke | Startup debug text display adapter renders four fixed SSD1306 128x32 lines; upstream display config, inversion, rotation options, timeout handling, and LVGL remain pending. |
| UI-002 | Screen rendering flow | `reference/esp-miner/main/screen.c` | `firmware/bitaxe/src/display_adapter.rs` | in-progress | unit,workflow,hardware-smoke | Startup-only four-line screen is drawn once and left on; upstream carousel, screen task, page flow, and runtime display updates remain pending. |
| UI-003 | Input behavior | `reference/esp-miner/main/input.c` | `firmware/bitaxe` | not-started | pending | Hardware smoke required for verified. |
| SELF-001 | Self-test lifecycle | `reference/esp-miner/main/self_test/self_test.c` | `firmware/bitaxe` | not-started | pending | First boot user experience. |
| BAP-001 | BAP interface initialization | `reference/esp-miner/main/bap/bap.c` | `firmware/bitaxe` | not-started | pending | Preserve observable interface behavior. |
| BAP-002 | BAP protocol behavior | `reference/esp-miner/main/bap/bap_protocol.c` | `crates/bitaxe-core`, `firmware/bitaxe` | not-started | pending | Determine pure/hardware boundary during planning. |

## Logging, Statistics, And Scoreboard

| ID | Surface | Reference Breadcrumb | Rust-Owned Target | Status | Evidence | Notes |
| --- | --- | --- | --- | --- | --- | --- |
| LOG-001 | Log buffer behavior | `reference/esp-miner/main/log_buffer.c` | `firmware/bitaxe`, `crates/bitaxe-core` | not-started | pending | Needed by API log download and WebSocket logs. |
| STAT-001 | Hashrate monitor | `reference/esp-miner/main/tasks/hashrate_monitor_task.c` | `crates/bitaxe-core`, `firmware/bitaxe` | not-started | pending | API-visible behavior. |
| STAT-002 | Statistics task | `reference/esp-miner/main/tasks/statistics_task.c` | `crates/bitaxe-core`, `firmware/bitaxe` | not-started | pending | API-visible behavior. |
| STAT-003 | Scoreboard | `reference/esp-miner/main/tasks/scoreboard.c` | `crates/bitaxe-core` | not-started | pending | Good pure unit target. |
| STAT-004 | Work queue behavior | `reference/esp-miner/main/work_queue.c` | `crates/bitaxe-core` | not-started | pending | Pure queue semantics where possible. |

## OTA, Filesystem, And Release Artifacts

| ID | Surface | Reference Breadcrumb | Rust-Owned Target | Status | Evidence | Notes |
| --- | --- | --- | --- | --- | --- | --- |
| FS-001 | SPIFFS/filesystem behavior | `reference/esp-miner/main/filesystem.c` | `firmware/bitaxe` | not-started | pending | Includes web asset partition behavior. |
| OTA-001 | Firmware OTA route | `reference/esp-miner/main/http_server/http_server.c` | `firmware/bitaxe` | not-started | pending | API and hardware evidence needed. |
| OTA-002 | AxeOS OTAWWW route | `reference/esp-miner/main/http_server/http_server.c` | `firmware/bitaxe` | not-started | pending | Asset update behavior. |
| REL-001 | Partition layout | `reference/esp-miner/partitions.csv` | `firmware/bitaxe` | not-started | pending | Must support flash/OTA behavior. |
| REL-002 | SDK config parity | `reference/esp-miner/sdkconfig.defaults` | `firmware/bitaxe` | not-started | pending | ESP-IDF Rust equivalent must be explicit. |
| REL-003 | Release image behavior | `reference/esp-miner/.github/workflows/release.yml` | `MODULE.bazel`, `tools/flash` | not-started | pending | Later release parity. |

## Notes

- Add Rust-owned implementation pointers as crates and modules are created.
- Add evidence links or command summaries when statuses advance.
- Do not mark safety-critical hardware surfaces `verified` without hardware evidence.
- Do not mark boards other than the current Ultra 205 target `verified` until those boards have their own evidence.
