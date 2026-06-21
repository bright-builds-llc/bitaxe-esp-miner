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
- `hardware-smoke`: Behavior observed on Gamma 601 hardware, with command/log captured.
- `hardware-regression`: Repeatable hardware test or scripted probe passes.
- `workflow`: Repo-owned command, build, package, flash-shaped, or report workflow passes with captured plan evidence.
- `deferred`: Accepted gap with reason and owner.

Safety-critical and hardware-control surfaces require hardware evidence before `verified`: voltage, fan, thermal, power, and ASIC initialization.

## Foundation And Workflow

| ID | Surface | Reference Breadcrumb | Rust-Owned Target | Status | Evidence | Notes |
| --- | --- | --- | --- | --- | --- | --- |
| WF-001 | Read-only reference submodule | `reference/esp-miner` | `scripts/verify-reference-clean.sh`, `tools/parity` | verified | workflow | `just verify-reference` passed in Phase 01 Plan 09 and printed `reference clean: c1915b0a63bfabebdb95a515cedfee05146c1d50`; package and parity targets run the guard before trusted output. |
| WF-002 | Bazel build graph | `reference/esp-miner/CMakeLists.txt` | `MODULE.bazel`, `tools/parity/BUILD.bazel` | verified | workflow | `just build`, `just test`, `just package`, `just parity`, and guard dependency queries passed in Phase 01 Plan 09. |
| WF-003 | Human command surface | `reference/esp-miner/README.md` | `Justfile` | verified | workflow | `Justfile` exposes `build`, `test`, `package`, `flash`, `monitor`, `flash-monitor`, `verify-reference`, and `parity` as thin Bazel wrappers. |
| WF-004 | Firmware image packaging | `reference/esp-miner/merge_bin.sh` | `//firmware/bitaxe:firmware_image` | verified | workflow | `just package` produced `bitaxe-gamma601.elf`, `bitaxe-gamma601-factory.bin`, and `bitaxe-gamma601-package.json`; the manifest default remains `bitaxe-gamma601.elf`. |
| WF-005 | USB flash workflow | `reference/esp-miner/flashing.md`, `reference/esp-miner/tools/upload2device.py` | `tools/flash`, `Justfile` | implemented | workflow | `just flash`, `just monitor`, and `just flash-monitor` route through `//tools/flash:flash`; live Gamma 601 flash-monitor evidence is missing because no serial port was visible. |

## Boot And System Runtime

| ID | Surface | Reference Breadcrumb | Rust-Owned Target | Status | Evidence | Notes |
| --- | --- | --- | --- | --- | --- | --- |
| SYS-001 | App entrypoint boot order | `reference/esp-miner/main/main.c` | `firmware/bitaxe` | implemented | pending | Safe boot/log entrypoint exists; missing Gamma 601 hardware-smoke evidence is recorded in `docs/parity/evidence/phase-01-gamma-601-boot-log.md`. |
| SYS-002 | PSRAM availability handling | `reference/esp-miner/main/main.c` | `firmware/bitaxe` | implemented | pending | Firmware logs PSRAM/platform status; hardware evidence remains pending in `docs/parity/evidence/phase-01-gamma-601-boot-log.md`. |
| SYS-003 | Global system status model | `reference/esp-miner/main/global_state.h` | `crates/bitaxe-core` | implemented | pending | Phase 1 safe-state model makes mining, ASIC work submission, and hardware control disabled by default. |
| SYS-004 | Version reporting | `reference/esp-miner/main/system.c` | `firmware/bitaxe`, `crates/bitaxe-core`, `crates/bitaxe-api`, `tools/parity` | in-progress | pending | Firmware logs and parity reports include source/reference identifiers; API version surface remains later-phase work. |
| SYS-005 | Task orchestration behavior | `reference/esp-miner/main/tasks/*.c` | `firmware/bitaxe` | not-started | pending | Internal task layout may differ if observable behavior matches. |

## Board Config And NVS

| ID | Surface | Reference Breadcrumb | Rust-Owned Target | Status | Evidence | Notes |
| --- | --- | --- | --- | --- | --- | --- |
| CFG-001 | Gamma 601 defaults | `reference/esp-miner/config-601.cvs` | `crates/bitaxe-config` | not-started | pending | First hardware target: gamma, 601, BM1370, 525 MHz, 1150 mV. |
| CFG-002 | Secondary 205 defaults | `reference/esp-miner/config-205.cvs` | `crates/bitaxe-config` | not-started | pending | Available secondary device, not first priority. |
| CFG-003 | Board/device model table | `reference/esp-miner/main/device_config.h` | `crates/bitaxe-config` | not-started | pending | Include all upstream board configs in parity scope. |
| CFG-004 | NVS key model | `reference/esp-miner/main/nvs_config.c` | `crates/bitaxe-config`, `firmware/bitaxe` | not-started | pending | Preserve settings names and defaults. |
| CFG-005 | Runtime settings update behavior | `reference/esp-miner/main/http_server/system_api_json.c` | `crates/bitaxe-api`, `crates/bitaxe-config` | not-started | pending | PATCH behavior must match API clients. |

## ASIC And Mining Hardware

| ID | Surface | Reference Breadcrumb | Rust-Owned Target | Status | Evidence | Notes |
| --- | --- | --- | --- | --- | --- | --- |
| ASIC-001 | ASIC dispatch by model | `reference/esp-miner/components/asic/asic.c` | `crates/bitaxe-asic` | not-started | pending | BM1366, BM1368, BM1370, BM1397 in scope. |
| ASIC-002 | BM1370 initialization | `reference/esp-miner/components/asic/bm1370.c:BM1370_init` | `crates/bitaxe-asic`, `firmware/bitaxe` | not-started | pending | Requires hardware evidence before verified. |
| ASIC-003 | BM1370 work send | `reference/esp-miner/components/asic/bm1370.c:BM1370_send_work` | `crates/bitaxe-asic` | not-started | pending | Use breadcrumbs for packet layout and constants. |
| ASIC-004 | BM1370 result parsing | `reference/esp-miner/components/asic/bm1370.c:BM1370_process_work` | `crates/bitaxe-asic` | not-started | pending | Unit fixtures plus hardware evidence. |
| ASIC-005 | ASIC serial transport | `reference/esp-miner/components/asic/serial.c` | `firmware/bitaxe` | not-started | pending | Hardware-bound adapter. |
| ASIC-006 | ASIC CRC behavior | `reference/esp-miner/components/asic/crc.c` | `crates/bitaxe-asic` | not-started | pending | Good early unit/golden target. |
| ASIC-007 | Frequency transition behavior | `reference/esp-miner/components/asic/frequency_transition_bmXX.c` | `crates/bitaxe-asic`, `firmware/bitaxe` | not-started | pending | Hardware-control surface. |
| ASIC-008 | BM1366 parity | `reference/esp-miner/components/asic/bm1366.c` | `crates/bitaxe-asic` | not-started | pending | Needed for 205 and other boards, but after 601 path. |
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
| PWR-004 | TPS546 support | `reference/esp-miner/main/power/TPS546.c` | `firmware/bitaxe` | not-started | pending | Relevant to Gamma 601. |
| PWR-005 | DS4432U support | `reference/esp-miner/main/power/DS4432U.c` | `firmware/bitaxe` | not-started | pending | Relevant to 205 and other boards. |
| PWR-006 | INA260 support | `reference/esp-miner/main/power/INA260.c` | `firmware/bitaxe` | not-started | pending | Current/power telemetry. |
| THR-001 | Thermal model | `reference/esp-miner/main/thermal/thermal.c` | `firmware/bitaxe`, `crates/bitaxe-core` | not-started | pending | Safety-critical. |
| THR-002 | Fan controller task | `reference/esp-miner/main/tasks/fan_controller_task.c` | `firmware/bitaxe`, `crates/bitaxe-core` | not-started | pending | Safety-critical. |
| THR-003 | PID behavior | `reference/esp-miner/main/thermal/PID.c` | `crates/bitaxe-core` | not-started | pending | Good pure unit target before hardware. |
| IO-001 | I2C initialization | `reference/esp-miner/main/i2c_bitaxe.c` | `firmware/bitaxe` | not-started | pending | Hardware adapter. |
| IO-002 | ADC behavior | `reference/esp-miner/main/adc.c` | `firmware/bitaxe` | not-started | pending | Hardware adapter. |

## Display, Input, Self-Test, And BAP

| ID | Surface | Reference Breadcrumb | Rust-Owned Target | Status | Evidence | Notes |
| --- | --- | --- | --- | --- | --- | --- |
| UI-001 | Display behavior | `reference/esp-miner/main/display.c` | `firmware/bitaxe` | not-started | pending | User-visible surface. |
| UI-002 | Screen rendering flow | `reference/esp-miner/main/screen.c` | `firmware/bitaxe` | not-started | pending | User-visible surface. |
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
- Do not mark non-601 boards `verified` until those boards have evidence.
