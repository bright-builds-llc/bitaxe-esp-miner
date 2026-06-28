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

Safety-critical and hardware-control surfaces require hardware evidence before `verified`: voltage, fan, thermal, power, ASIC initialization, self-test hardware, runtime input, and runtime display.

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
| ASIC-002 | BM1366 initialization | `reference/esp-miner/components/asic/bm1366.c:BM1366_init`, `reference/esp-miner/main/power/asic_reset.c` | `crates/bitaxe-asic/src/bm1366/init_plan.rs`, `crates/bitaxe-safety`, `firmware/bitaxe/src/asic_adapter.rs`, `firmware/bitaxe/src/asic_adapter/reset.rs` | implemented | unit,workflow | Pure staged init and default fail-closed firmware gate are implemented; Phase 6 wires power, thermal, and safety evidence tokens into full-init gating. Live Ultra 205 chip-detect and safety hardware evidence are [not run - hardware verification pending](evidence/phase-06-ultra-205-safety-hardware-smoke.md), so initialization is not verified. |
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
| STR-001 | Stratum v1 socket behavior | `reference/esp-miner/components/stratum/stratum_socket.c` | `crates/bitaxe-stratum/src/v1/fake_pool.rs`, `crates/bitaxe-stratum/src/v1/state.rs`, `firmware/bitaxe/src/main.rs` | implemented | unit,workflow | Deterministic fake-pool reconnect/fallback lifecycle and firmware blocked mining-loop status are implemented; live socket adapter remains a later hardware-gated shell. See [Phase 4 evidence](evidence/phase-04-stratum-v1-mining-loop.md). |
| STR-002 | Stratum v1 API messages | `reference/esp-miner/components/stratum/stratum_api.c` | `crates/bitaxe-stratum/src/v1/messages.rs`, `crates/bitaxe-stratum/fixtures/v1/protocol-cases.json` | implemented | unit,golden | Typed Stratum v1 subscribe, authorize, configure, difficulty, extranonce, notify, submit, response, error, ping, and reconnect-relevant messages are parsed or serialized. See [Phase 4 evidence](evidence/phase-04-stratum-v1-mining-loop.md). |
| STR-003 | Mining job construction | `reference/esp-miner/components/stratum/mining.c` | `crates/bitaxe-stratum/src/v1/coinbase.rs`, `crates/bitaxe-stratum/src/v1/mining.rs`, `crates/bitaxe-stratum/src/v1/mining_loop.rs` | implemented | unit,golden | Stratum notify/extranonce/difficulty state produces typed BM1366 work fields and guarded dispatch plans without raw ASIC frame construction in Stratum. See [Phase 4 evidence](evidence/phase-04-stratum-v1-mining-loop.md). |
| STR-004 | Coinbase decoding | `reference/esp-miner/components/stratum/coinbase_decoder.c` | `crates/bitaxe-stratum/src/v1/coinbase.rs`, `crates/bitaxe-stratum/fixtures/v1/mining-job-cases.json` | implemented | unit,golden | Coinbase/extranonce hex handling, double SHA-256, merkle folding, and malformed input rejection are covered by host tests. See [Phase 4 evidence](evidence/phase-04-stratum-v1-mining-loop.md). |
| STR-005 | Stratum v2 protocol | `reference/esp-miner/components/stratum_v2/*.c` | `crates/bitaxe-stratum/src/v1.rs` | deferred | deferred | Phase 4 intentionally covers Stratum v1 first-loop behavior only; Stratum v2 remains deferred by scope decision. See [Phase 4 evidence](evidence/phase-04-stratum-v1-mining-loop.md). |
| STR-006 | Protocol coordinator | `reference/esp-miner/main/tasks/protocol_coordinator.c` | `crates/bitaxe-stratum/src/v1/mining_loop.rs`, `crates/bitaxe-safety`, `firmware/bitaxe/src/main.rs`, `firmware/bitaxe/src/asic_adapter/status.rs` | implemented | unit,workflow | First-loop coordination is fail-closed unless ASIC initialization, Phase 6 safety evidence, and hardware-evidence acknowledgment are all present; firmware publishes `mining_loop_status=blocked`. See [Phase 4 evidence](evidence/phase-04-stratum-v1-mining-loop.md) and [Phase 6 evidence](evidence/phase-06-safety-controllers-and-self-test.md). |
| STR-007 | Mining smoke and soak criteria | `reference/esp-miner/main/tasks/protocol_coordinator.c`, `reference/esp-miner/main/system.c` | `docs/parity/evidence/phase-04-stratum-v1-mining-loop.md` | implemented | workflow | Smoke and soak criteria are recorded without pool credentials or secret-bearing logs. This is criteria documentation only, not live hardware evidence. |
| STR-008 | Live mining smoke and soak evidence | `reference/esp-miner/main/tasks/protocol_coordinator.c`, `reference/esp-miner/main/system.c` | `docs/parity/evidence/phase-04-stratum-v1-mining-loop.md` | not-started | pending | Live Ultra 205 mining smoke and soak have not run and remain pending until hardware evidence is recorded. |

## AxeOS API And Web Compatibility

| ID | Surface | Reference Breadcrumb | Rust-Owned Target | Status | Evidence | Notes |
| --- | --- | --- | --- | --- | --- | --- |
| API-001 | OpenAPI schema compatibility | `reference/esp-miner/main/http_server/openapi.yaml` | `crates/bitaxe-api`, `tools/parity` | verified | api-compare | `bazel run //tools/parity:report -- api-compare` in [Phase 05 evidence](evidence/phase-05-axeos-api-logs-and-telemetry.md) checked Phase 05 route/property coverage against upstream OpenAPI and the Rust route manifest with `validation_errors: none`. |
| API-002 | System info response | `reference/esp-miner/main/http_server/system_api_json.c` | `crates/bitaxe-api`, `crates/bitaxe-safety`, `firmware/bitaxe` | implemented | unit,api-compare | Pure DTO/mappers, safe Ultra 205 fixture, firmware route shell, API compare coverage, and Phase 6 explicit safety telemetry projection exist; live firmware HTTP/safety telemetry smoke was not run. |
| API-003 | System settings PATCH | `reference/esp-miner/main/http_server/system_api_json.c` | `crates/bitaxe-api`, `crates/bitaxe-config`, `firmware/bitaxe` | implemented | unit,api-compare | Pure PATCH validation/persistence planning, firmware NVS adapter, body-cap route-shell tests, and API compare coverage exist; live firmware PATCH smoke was not run in Phase 05 evidence. |
| API-004 | HTTP server routes | `reference/esp-miner/main/http_server/http_server.c` | `firmware/bitaxe`, `crates/bitaxe-api`, `tools/parity` | implemented | unit,workflow,api-compare | Phase 05 route shell and firmware build cover API/log/command routes; OTA and OTAWWW remain fail-closed Phase 7-owned unsupported routes, and live firmware HTTP smoke was not run. |
| API-005 | WebSocket logs | `reference/esp-miner/main/http_server/websocket_log.c` | `firmware/bitaxe`, `crates/bitaxe-api`, `tools/parity` | implemented | unit,api-compare | Retained log/raw `/api/ws` semantics and static AxeOS route usage are covered by unit fixtures and API compare; live WebSocket smoke was not run. |
| API-006 | Live WebSocket telemetry | `reference/esp-miner/main/http_server/websocket_api.c` | `firmware/bitaxe`, `crates/bitaxe-api`, `crates/bitaxe-safety`, `tools/parity` | implemented | unit,api-compare | Full-on-connect, diff, 500 ms cadence, and Phase 6 safety telemetry fixture cases are covered; live firmware WebSocket safety telemetry smoke was not run. |
| API-007 | Recovery page | `reference/esp-miner/main/http_server/recovery_page.html` | `firmware/bitaxe`, `tools/parity` | deferred | deferred | Phase 05 API compare records `/recovery` separately from ordinary API/static success and marks recovery/static packaging evidence Phase 7 pending; no recovery smoke was run. |
| API-008 | Static AxeOS asset packaging | `reference/esp-miner/main/http_server/axe-os` | `firmware/bitaxe`, `tools/parity` | implemented | api-compare | Static route usage fixture proves existing AxeOS API/log/WebSocket service calls remain administrable without an Angular rewrite; SPIFFS/static release packaging remains Phase 7 pending and is not counted as Phase 05 success. |

## Power, Thermal, Fan, And Peripherals

| ID | Surface | Reference Breadcrumb | Rust-Owned Target | Status | Evidence | Notes |
| --- | --- | --- | --- | --- | --- | --- |
| PWR-001 | ASIC reset behavior | `reference/esp-miner/main/power/asic_reset.c` | `crates/bitaxe-safety/src/effects.rs`, `crates/bitaxe-asic/src/bm1366/init_plan.rs`, `firmware/bitaxe/src/asic_adapter/reset.rs`, `firmware/bitaxe/src/safety_adapter.rs` | implemented | unit,workflow | Fail-closed plans include `HoldResetLow`, ASIC init remains gated by safety evidence, and firmware logs unavailable reset effects when peripherals are absent. Live reset hardware smoke is [not run - hardware verification pending](evidence/phase-06-ultra-205-safety-hardware-smoke.md). |
| PWR-002 | ASIC power initialization | `reference/esp-miner/main/power/asic_init.c` | `crates/bitaxe-asic/src/bm1366/init_plan.rs`, `crates/bitaxe-safety`, `firmware/bitaxe/src/asic_adapter.rs`, `firmware/bitaxe/src/safety_adapter.rs` | implemented | unit,workflow | Full init and mining gates require power, thermal, safety, and hardware evidence tokens before hardware work can proceed. Live Ultra 205 power initialization evidence is [not run - hardware verification pending](evidence/phase-06-ultra-205-safety-hardware-smoke.md). |
| PWR-003 | Core voltage control | `reference/esp-miner/main/power/vcore.c` | `crates/bitaxe-safety/src/power.rs`, `firmware/bitaxe/src/safety_adapter/power.rs` | implemented | unit,workflow | Pure voltage planning validates Ultra 205 setpoints and suppresses DS4432U writes without hardware evidence; firmware adapter remains observe-only. Live VCORE control evidence is [not run - hardware verification pending](evidence/phase-06-ultra-205-safety-hardware-smoke.md). |
| PWR-004 | TPS546 support | `reference/esp-miner/main/power/TPS546.c` | `firmware/bitaxe` | deferred | deferred | Relevant to deferred Gamma 601 and related boards. |
| PWR-005 | DS4432U support | `reference/esp-miner/main/power/DS4432U.c` | `crates/bitaxe-safety/src/power.rs`, `firmware/bitaxe/src/safety_adapter/power.rs` | implemented | unit,workflow | DS4432U address/register constants and observe-only voltage effect handling exist, but firmware does not write DS4432U hardware without evidence. Live DS4432U smoke is [not run - hardware verification pending](evidence/phase-06-ultra-205-safety-hardware-smoke.md). |
| PWR-006 | INA260 support | `reference/esp-miner/main/power/INA260.c` | `crates/bitaxe-safety/src/power.rs`, `firmware/bitaxe/src/safety_adapter/power.rs`, `crates/bitaxe-api` | implemented | unit,workflow | INA260 telemetry classification and firmware constants exist; default firmware telemetry is explicit unavailable until live sensor reads are evidenced. Hardware telemetry smoke is [not run - hardware verification pending](evidence/phase-06-ultra-205-safety-hardware-smoke.md). |
| THR-001 | Thermal model | `reference/esp-miner/main/thermal/thermal.c` | `crates/bitaxe-safety/src/thermal.rs`, `crates/bitaxe-safety/src/fault.rs`, `firmware/bitaxe/src/safety_adapter/thermal.rs`, `crates/bitaxe-api` | implemented | unit,workflow | Pure thermal observation, fault, and API status projection exist; firmware reports unavailable thermal telemetry until live sensor evidence exists. Thermal hardware smoke is [not run - hardware verification pending](evidence/phase-06-ultra-205-safety-hardware-smoke.md). |
| THR-002 | Fan controller task | `reference/esp-miner/main/tasks/fan_controller_task.c` | `crates/bitaxe-safety/src/thermal.rs`, `firmware/bitaxe/src/safety_adapter/thermal.rs` | implemented | unit,workflow | Fan/PID decisions and fan fault classification exist, while firmware suppresses fan writes without hardware evidence. Fan duty/RPM smoke is [not run - hardware verification pending](evidence/phase-06-ultra-205-safety-hardware-smoke.md). |
| THR-003 | PID behavior | `reference/esp-miner/main/thermal/PID.c` | `crates/bitaxe-safety/src/thermal.rs`, `crates/bitaxe-safety/fixtures/safety/fan-pid-cases.json` | implemented | unit | Pure PID constants, duty clamps, and fixture-backed fan decisions are covered by unit tests; live fan hardware verification remains separate and pending. |
| IO-001 | I2C initialization | `reference/esp-miner/main/i2c_bitaxe.c` | `firmware/bitaxe/src/display_adapter.rs`, `firmware/bitaxe/src/safety_adapter/power.rs`, `firmware/bitaxe/src/safety_adapter/thermal.rs` | in-progress | unit,workflow,hardware-smoke | Startup debug display initializes I2C0 on SDA GPIO47/SCL GPIO48 at 400 kHz for SSD1306 address `0x3c`; Phase 6 records DS4432U, INA260, and EMC2101 adapter constants but live shared I2C safety peripherals remain [not run - hardware verification pending](evidence/phase-06-ultra-205-safety-hardware-smoke.md). |
| IO-002 | ADC behavior | `reference/esp-miner/main/adc.c` | `firmware/bitaxe` | not-started | pending | Hardware adapter. |

## Display, Input, Self-Test, And BAP

| ID | Surface | Reference Breadcrumb | Rust-Owned Target | Status | Evidence | Notes |
| --- | --- | --- | --- | --- | --- | --- |
| UI-001 | Display behavior | `reference/esp-miner/main/display.c` | `firmware/bitaxe/src/display_adapter.rs`, `firmware/bitaxe/src/safety_adapter/watchdog.rs` | in-progress | unit,workflow,hardware-smoke | Startup debug text evidence remains startup-only; Phase 6 now logs the runtime display/input gap and preserves API/log/WebSocket safety status. Full runtime display parity is documented in [Phase 6 display/input gap evidence](evidence/phase-06-display-input-runtime-gap.md). |
| UI-002 | Screen rendering flow | `reference/esp-miner/main/screen.c` | `firmware/bitaxe/src/display_adapter.rs`, `crates/bitaxe-api` | in-progress | unit,workflow,hardware-smoke | Startup-only four-line screen is drawn once and left on; runtime safety status is available through AxeOS API/log/WebSocket surfaces, while screen task/page flow remains a V1 gap documented in [Phase 6 display/input gap evidence](evidence/phase-06-display-input-runtime-gap.md). |
| UI-003 | Input behavior | `reference/esp-miner/main/input.c` | `firmware/bitaxe/src/display_adapter.rs`, `firmware/bitaxe/src/main.rs` | in-progress | workflow | Firmware explicitly logs `display_input_status=runtime_gap reason=hardware_evidence_pending startup_only=true`; runtime input hardware-control behavior is not verified and needs hardware smoke. |
| SELF-001 | Self-test lifecycle | `reference/esp-miner/main/self_test/self_test.c` | `crates/bitaxe-safety/src/self_test.rs`, `crates/bitaxe-safety/src/watchdog.rs`, `firmware/bitaxe/src/safety_adapter/watchdog.rs` | implemented | unit,workflow | Pure self-test lifecycle, factory/manual/cancel/pass/fail effects, missing evidence gates, and watchdog-friendly steps are implemented. Self-test hardware submodes are [not run - hardware verification pending](evidence/phase-06-ultra-205-safety-hardware-smoke.md). |
| BAP-001 | BAP interface initialization | `reference/esp-miner/main/bap/bap.c` | `firmware/bitaxe` | not-started | pending | Preserve observable interface behavior. |
| BAP-002 | BAP protocol behavior | `reference/esp-miner/main/bap/bap_protocol.c` | `crates/bitaxe-core`, `firmware/bitaxe` | not-started | pending | Determine pure/hardware boundary during planning. |

## Logging, Statistics, And Scoreboard

| ID | Surface | Reference Breadcrumb | Rust-Owned Target | Status | Evidence | Notes |
| --- | --- | --- | --- | --- | --- | --- |
| LOG-001 | Log buffer behavior | `reference/esp-miner/main/log_buffer.c` | `firmware/bitaxe`, `crates/bitaxe-api` | implemented | unit,api-compare | Phase 05 implements retained log contracts, download headers, raw WebSocket baseline semantics, firmware retained log shell, and API compare log fixture coverage; live firmware log smoke was not run. |
| STAT-001 | Hashrate monitor | `reference/esp-miner/main/tasks/hashrate_monitor_task.c` | `crates/bitaxe-core`, `firmware/bitaxe` | not-started | pending | API fixtures expose safe zero/unavailable hashrate fields only; the live hashrate monitor task remains future work. |
| STAT-002 | Statistics task | `reference/esp-miner/main/tasks/statistics_task.c` | `crates/bitaxe-api`, `crates/bitaxe-safety`, `firmware/bitaxe/src/safety_adapter.rs` | in-progress | unit,api-compare | Phase 05 implements compatible statistics response shape and Phase 6 projects explicit safety telemetry into statistics samples; a live firmware statistics history producer and live sensor values remain pending. |
| STAT-003 | Scoreboard | `reference/esp-miner/main/tasks/scoreboard.c` | `crates/bitaxe-api` | in-progress | unit,api-compare | Phase 05 implements scoreboard response shape and empty-array fixture; live scoreboard population remains pending. |
| STAT-004 | Work queue behavior | `reference/esp-miner/main/work_queue.c` | `crates/bitaxe-stratum/src/v1/queue.rs`, `crates/bitaxe-stratum/src/v1/mining_loop.rs` | implemented | unit | Bounded queue capacity, FIFO dequeue, clean-jobs clearing, valid-job invalidation, and guarded dispatch planning are covered by host tests. See [Phase 4 evidence](evidence/phase-04-stratum-v1-mining-loop.md). |

## OTA, Filesystem, And Release Artifacts

| ID | Surface | Reference Breadcrumb | Rust-Owned Target | Status | Evidence | Notes |
| --- | --- | --- | --- | --- | --- | --- |
| FS-001 | SPIFFS/filesystem behavior | `reference/esp-miner/main/filesystem.c` | `firmware/bitaxe`, `tools/parity` | not-started | pending | Phase 05 static fixture records `/recovery` and static fallback as Phase 7 packaging pending; no SPIFFS/static packaging success is claimed. |
| OTA-001 | Firmware OTA route | `reference/esp-miner/main/http_server/http_server.c` | `firmware/bitaxe`, `tools/parity` | deferred | deferred | Phase 05 records `/api/system/OTA` only as a Phase 7-owned unsafe-success-blocked route; firmware OTA upload/apply behavior remains Phase 7. |
| OTA-002 | AxeOS OTAWWW route | `reference/esp-miner/main/http_server/http_server.c` | `firmware/bitaxe`, `tools/parity` | deferred | deferred | Phase 05 records `/api/system/OTAWWW` only as a Phase 7-owned unsafe-success-blocked route; AxeOS asset update behavior remains Phase 7. |
| REL-001 | Partition layout | `reference/esp-miner/partitions.csv` | `firmware/bitaxe` | not-started | pending | Must support flash/OTA behavior. |
| REL-002 | SDK config parity | `reference/esp-miner/sdkconfig.defaults` | `firmware/bitaxe` | not-started | pending | ESP-IDF Rust equivalent must be explicit. |
| REL-003 | Release image behavior | `reference/esp-miner/.github/workflows/release.yml` | `MODULE.bazel`, `tools/flash` | not-started | pending | Later release parity. |

## Notes

- Add Rust-owned implementation pointers as crates and modules are created.
- Add evidence links or command summaries when statuses advance.
- Do not mark safety-critical hardware surfaces `verified` without hardware evidence.
- Do not mark boards other than the current Ultra 205 target `verified` until those boards have their own evidence.
