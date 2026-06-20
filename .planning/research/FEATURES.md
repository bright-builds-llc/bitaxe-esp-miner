# Feature Research: Bitaxe Rust ESP-IDF Firmware

**Domain:** Bitaxe ESP-Miner Rust firmware rewrite\
**Researched:** 2026-06-20\
**Overall confidence:** HIGH for observable surfaces, MEDIUM for v1/deferred sequencing

## Summary

Device-user parity is the right feature boundary: the Rust firmware should match the behavior a Bitaxe owner, administrator, mining pool, API client, accessory, or flashing tool can observe from upstream ESP-Miner. It should not mirror C file boundaries, FreeRTOS task layout, or line-level implementation details unless those internals leak into behavior.

The upstream baseline used by this research is the project-accepted ESP-Miner commit `c1915b0a63bfabebdb95a515cedfee05146c1d50`, which GitHub currently reports as `master`. Local `reference/esp-miner` is not present yet, so source verification used the accepted project docs plus upstream official GitHub files at that commit-equivalent branch. Key checked surfaces: `config-601.cvs`, `main/device_config.h`, `main/http_server/openapi.yaml`, `main/http_server/http_server.c`, `main/nvs_config.c`, `main/main.c`, `main/self_test/self_test.c`, `main/bap/bap_readme.md`, and the repository tree.

Recommendation: split roadmap language into three scopes:

1. **First milestone:** project foundation plus Gamma 601 BM1370 boot/log/flash path. No mining parity.
1. **V1 Gamma parity:** a usable Gamma 601 miner with upstream-compatible board defaults, BM1370 mining, Stratum v1, safety controls, AxeOS API/static assets, telemetry, self-test, and update/flash behavior.
1. **Deferred full parity:** additional board families, additional ASIC models, Stratum v2 completeness, BAP completeness, release automation for all factory images, and advanced/nonessential UI-adjacent behavior.

Differentiators for the Rust rewrite should be project features, not end-user behavioral divergence: explicit parity evidence, pure Rust domain crates, safer hardware-control state modeling, `just` flash/monitor ergonomics, and repeatable API/golden/hardware evidence.

Source basis:

- Local project definition: `.planning/PROJECT.md`, `docs/project/gsd-new-project-brief.md`, `docs/parity/checklist.md`, `docs/project/first-milestone.md`, `CONTEXT.md`
- Upstream repo tree: `https://api.github.com/repos/bitaxeorg/ESP-Miner/git/trees/c1915b0a63bfabebdb95a515cedfee05146c1d50?recursive=1`
- Gamma 601 defaults: `https://raw.githubusercontent.com/bitaxeorg/ESP-Miner/master/config-601.cvs`
- Board/ASIC matrix: `https://raw.githubusercontent.com/bitaxeorg/ESP-Miner/master/main/device_config.h`
- AxeOS API contract: `https://raw.githubusercontent.com/bitaxeorg/ESP-Miner/master/main/http_server/openapi.yaml`
- User/admin surfaces: `https://raw.githubusercontent.com/bitaxeorg/ESP-Miner/master/readme.md`
- Flashing workflow: `https://raw.githubusercontent.com/bitaxeorg/ESP-Miner/master/flashing.md`

## Table Stakes

Features in this section must be tracked for device-user parity. The **V1 recommendation** column distinguishes first Gamma release scope from later full-parity scope.

| Feature surface | Observable behavior to match | V1 recommendation | Complexity | Dependencies |
| --- | --- | --- | --- | --- |
| Board and ASIC config model | Board version, family, ASIC model, frequency/voltage options, core counts, power targets, temperature sensor and regulator capabilities appear correctly in logs/API/settings. | V1: Gamma 601 only; track all boards as deferred parity items. | Medium | Reference submodule, `bitaxe-config`, NVS model, board fixtures. |
| Gamma 601 defaults | `devicemodel=gamma`, `boardversion=601`, `asicmodel=BM1370`, `asicfrequency=525`, `asicvoltage=1150`, public-pool defaults, fallback pool defaults, fan/self-test defaults. | First milestone seed; V1 verified. | Low | Config parser, NVS seed data, flash/package workflow. |
| Boot and platform status | Firmware boots safely, logs identity/basic platform status, reports PSRAM, version, reset reason, partition, heap, CPU, MAC/IP, and degraded-mode failures. | First milestone boot/log subset; V1 full system info fields. | Medium | ESP-IDF app skeleton, logging, global system model, API model. |
| USB build/package/flash/monitor workflow | Developer can build, package, flash, and monitor a connected Gamma 601 with clear serial-port behavior and actionable failures. | First milestone. | Medium | Bazel graph, `just`, image packaging, flash tool wrapper. |
| Wi-Fi/AP/connectivity behavior | Wi-Fi config, AP/captive-portal behavior, DNS redirect, private-network API authorization, mDNS/admin access expectations. | V1, before AxeOS/API acceptance. | High | NVS, ESP-IDF Wi-Fi, HTTP server, DNS server, filesystem/recovery. |
| NVS/settings behavior | Upstream key names, defaults, REST names, validation ranges, migrations, async save behavior, factory config behavior. | V1 for Gamma-visible keys; full key matrix tracked. | High | Config model, API PATCH, flash config image, migration fixtures. |
| BM1370 ASIC behavior | ASIC reset, power-up, serial transport, CRC/packet layout, init sequence, frequency transition, work send, nonce/result parsing. | V1 for Gamma 601. | High | Board config, power rails, serial adapter, `bitaxe-asic`, hardware evidence. |
| Mining loop and Stratum v1 | Connect to pool, subscribe/authorize, construct jobs, send ASIC work, submit shares, handle fallback pool, report accepted/rejected shares and rejection reasons. | V1. | High | Wi-Fi, config, BM1370 ASIC path, work queue, `bitaxe-stratum`, telemetry. |
| Stratum v2 | SV2 protocol settings, authority pubkey/channel type, coordinator behavior, response batching, share indicators. | Deferred after V1 unless a Gamma 601 acceptance test explicitly requires SV2. | High | Stable SV1 loop, protocol coordinator, API settings, pool test fixtures. |
| AxeOS HTTP API | OpenAPI-compatible routes: `/api/system/info`, `/api/system/asic`, `/api/system/statistics`, `/api/system/scoreboard`, `/api/system/wifi/scan`, `/api/system/logs`, `/api/system`, pause/resume/restart/identify/blockFound dismiss, OTA/OTAWWW. | V1 core routes: info, asic, settings, logs, pause/resume, restart, identify, statistics/scoreboard if mining exists; OTA can be late V1. | High | API models, NVS, system state, log buffer, statistics, OTA partitions. |
| WebSocket logs and live telemetry | `/api/ws` streams logs and `/api/ws/live` streams partial system info updates expected by AxeOS/API clients. | V1. | Medium | HTTP server, log buffer, system info serializer, task scheduling. |
| Static AxeOS assets and recovery page | Serve packaged AxeOS assets, gzip/content-type behavior, cache headers, `/recovery`, fallback to recovery when filesystem unavailable, redirect behavior. | V1 for asset compatibility; no Angular rewrite. | Medium | SPIFFS/filesystem, partition layout, asset packaging, HTTP server. |
| Theme API | `/api/theme` GET/POST persists scheme/colors used by existing AxeOS assets. | V1 if existing AxeOS assets are shipped unchanged; otherwise explicitly defer with UI compatibility evidence. | Low | NVS theme keys, HTTP server. |
| Power/voltage/thermal/fan safety | ASIC reset/init, TPS546 support for Gamma, vcore control, EMC2101 thermal, fan RPM/control, PID behavior, fault reporting, overheat behavior. | V1 safety gate for mining. | High | I2C/ADC, board config, power adapters, thermal model, hardware evidence. |
| Self-test lifecycle | Factory/self-test flag behavior, display messages, fan/power/vcore/temp/hashrate/domain checks, PASS/FAIL/restart/cancel behavior. | V1 for Gamma 601 if factory-user parity is required; can follow first mining loop. | High | Display/input, power/thermal, BM1370, hashrate monitor, NVS. |
| Display and input | Screen rendering, rotation/invert/timeout settings, identify behavior, self-test messages, input/BOOT/RESET interaction where observable. | V1 minimal status/self-test behavior; full rendering polish after mining/API. | Medium | NVS, display driver, input GPIO, system state. |
| Logging/statistics/scoreboard | Log buffer/download, live logs, hashrate windows, error percentage, best/session difficulty, top shares, block found state, statistics labels/history. | V1 for fields visible in API/AxeOS after mining. | Medium | Mining loop, result parser, time source, NVS best diff/scoreboard persistence. |
| OTA/filesystem/release artifacts | Firmware OTA, AxeOS OTAWWW, partition layout, SPIFFS behavior, factory/update image packaging, per-board image naming. | V1 for 601 update path if release-worthy; all-board release matrix deferred. | High | Partition CSV equivalent, package tool, HTTP upload, flash artifacts. |
| BAP accessory port | UART protocol with checksum, request/response, subscriptions, settings, AP-mode limitations, errors, 115200 8N1 behavior. | Deferred after core Gamma parity unless accessory use is an explicit V1 acceptance requirement. | Medium | System info/settings model, UART, scheduler, API-equivalent serializers. |
| Additional boards and ASICs | BM1397, BM1366, BM1368, BM1370XP/Gamma Duo/Turbo, Max/Ultra/Hex/Supra families, board-specific regulators/sensors. | Deferred after Gamma 601 V1; track from day one in checklist. | High | Board fixtures, ASIC variants, hardware access, per-board safety evidence. |

## First Milestone Features

The first milestone is not a mining release. It should prove the repo, build, flash, and safest possible Gamma 601 boot path.

| Feature | Observable acceptance target | Complexity | Dependencies | Notes |
| --- | --- | --- | --- | --- |
| Bazel/Cargo/ESP-IDF Rust skeleton | `just build` builds firmware skeleton through Bazel. | Medium | Toolchain pinning, ESP-IDF Rust setup. | Foundation only; avoid premature mining abstractions. |
| Read-only reference submodule guard | `just verify-reference` fails on modified upstream reference files. | Low | Submodule addition, guard script. | Required before using reference breadcrumbs seriously. |
| Initial crate layout | Pure crates exist for core, ASIC, Stratum, config, API, test support. | Low | Workspace setup. | Empty or stubbed crates are acceptable if boundaries are clear. |
| Human command surface | `just build`, `test`, `package`, `flash`, `monitor`, `flash-monitor`, `verify-reference`, `parity`. | Medium | Bazel targets, flash tool wrapper. | Commands should fail clearly before all internals exist. |
| Firmware package artifact | `just package` creates a flashable Gamma 601 image artifact or a clearly named placeholder path if packaging is staged. | Medium | ESP-IDF build outputs, partition/image merge plan. | Do not claim release compatibility before proof. |
| Gamma 601 USB flash workflow | `just flash board=601 port=<port>` flashes or fails with actionable serial/port guidance. | Medium | Flash wrapper, port handling, package artifact. | Print underlying command for debugging. |
| Monitor and flash-monitor | Serial monitor shows Rust firmware boot logs; combined command flashes then monitors. | Low | Flash wrapper, serial monitor tool. | This is the first hardware smoke loop. |
| Minimal boot/log firmware | Gamma 601 boots and logs Rust firmware identity plus basic platform status. | Medium | ESP-IDF app, logging, PSRAM/platform probes. | No BM1370 init beyond safe no-op/reset handling. |
| Seed parity checklist integration | Checklist remains the audit source of truth with statuses and evidence placeholders. | Low | Existing `docs/parity/checklist.md`, parity helper tooling. | Keep implementation status separate from verification status. |
| Provenance/licensing guardrails | GPL-derived behavior breadcrumbs and MIT-first original-code posture are documented. | Low | `PROVENANCE.md`, reference breadcrumbs. | Needed before porting ASIC/protocol logic. |

## Deferred Features

Deferred does not mean optional. It means the feature should not block the first milestone or, where noted, should not block Gamma 601 V1.

| Feature | Defer until | Why defer | Complexity | Dependencies |
| --- | --- | --- | --- | --- |
| BM1370 cold init/work/result parity | V1 Gamma parity, after boot/log foundation. | Hardware-control surface; unsafe to rush before config, power, and serial adapters are in place. | High | Gamma config, TPS546/vcore, ASIC serial, hardware smoke evidence. |
| Stratum v1 mining loop | V1 Gamma parity. | Needs ASIC work path and stable network/config model. | High | Wi-Fi, NVS, BM1370, work queue, protocol fixtures. |
| AxeOS API compatibility | V1 Gamma parity after system state model exists. | API fields depend on mining, power, telemetry, and NVS; early stubs risk false parity. | High | `bitaxe-api`, serializers, API compare fixtures. |
| Power/thermal/fan closed-loop control | V1 Gamma parity before mining is considered usable. | Safety-critical; requires hardware evidence. | High | I2C/ADC, TPS546, EMC2101, fan driver, PID tests. |
| Self-test lifecycle | Late V1 Gamma parity. | Meaningful self-test needs real ASIC/power/fan/hashrate behavior. | High | BM1370 mining, display/input, thermal, NVS. |
| OTA firmware and OTAWWW | Late V1 or first post-V1 maintenance release. | Requires stable partition/package strategy and recovery testing. | High | Partition layout, HTTP upload, flash artifacts, recovery page. |
| Full static AxeOS asset compatibility | V1 if API clients need existing UI; otherwise post-V1. | Asset packaging is valuable only after underlying API surfaces are credible. | Medium | SPIFFS, content-type/gzip behavior, API route coverage. |
| Stratum v2 | Post-V1 unless required by an explicit pool/user acceptance target. | Separate protocol stack with more complex coordinator and security settings. | High | Stable SV1, SV2 fixtures, pool interoperability tests. |
| BAP protocol completeness | Post-V1 unless accessory parity is a named release goal. | Important external surface, but not required for a first standalone Gamma miner. | Medium | UART, system model, settings model, subscription scheduler. |
| Bitaxe 205 / BM1366 secondary path | Post-Gamma V1. | User has a 205, but project has explicitly prioritized 601 first. | High | BM1366 ASIC support, DS4432U/INA260 path, hardware evidence. |
| All upstream board factory images | Post-Gamma V1. | Requires per-board configs, packaging, and hardware or explicit unverified status. | High | Config matrix, CI/release graph, per-board acceptance records. |
| Custom board config flow | Post-Gamma V1. | Useful for power users, but dangerous before supported configs are strongly typed. | Medium | Board model parser, validation, provenance/evidence policy. |
| Advanced AxeOS UI changes | Out of v1; likely never for firmware parity unless upstream asset compatibility demands it. | Project scope is API/static asset compatibility, not Angular product development. | Medium | Existing static assets only. |

## Anti-Features

| Anti-feature | Why avoid | What to do instead |
| --- | --- | --- |
| C source mirroring | Preserves incidental complexity and conflicts with device-user parity definition. | Recreate observable behavior with Rust-owned boundaries and breadcrumbs. |
| Initial Angular AxeOS rewrite | Expands project scope away from firmware parity and duplicates upstream UI work. | Serve compatible static assets and match API contracts. |
| Mining stubs that look successful | Misleads users and hides safety/API gaps. | Report unavailable/degraded states clearly until verified. |
| Hardware-control code without hardware evidence | Voltage, thermal, fan, power, and ASIC init mistakes can damage devices. | Require hardware-smoke or hardware-regression evidence before `verified`. |
| Non-601 verification claims | The project only has Gamma 601 prioritized for first evidence. | Track all boards, but mark non-601 surfaces deferred/unverified until tested. |
| Byte-for-byte image parity as a release goal | Users need flash/update behavior, not identical binary layout. | Match partition/update/flashing behavior and document any harmless differences. |
| Hidden patches in `reference/esp-miner` | Corrupts the comparison baseline. | Keep reference read-only and refresh by explicit pinned commit update. |
| New cloud services or remote dependencies | Upstream device behavior is local firmware/admin/pool interaction. | Keep device operation local except configured mining pools/update workflows. |
| Unbounded overclock/tuning features | Upstream already gates voltage/frequency controls; extra tuning raises safety risk. | Match upstream setting ranges and overclock visibility before considering extensions. |
| Treating implementation as verification | A port can compile while still diverging from device behavior. | Require checklist evidence: unit, golden, API compare, hardware smoke/regression, or explicit deferred gap. |

## Dependencies

Recommended roadmap dependency flow:

```text
Reference guard + parity checklist
  -> Bazel/Cargo/ESP-IDF skeleton
  -> Gamma 601 config/NVS seed
  -> USB package/flash/monitor
  -> Minimal boot/log firmware
  -> Board/system state model
  -> Power/thermal/fan safe adapters
  -> BM1370 serial/CRC/init/work/result
  -> Stratum v1 + mining loop
  -> Logs/statistics/scoreboard
  -> AxeOS API + WebSocket telemetry
  -> Static assets/recovery + OTA
  -> Self-test completion evidence
  -> Stratum v2, BAP, non-601 boards, all-board releases
```

Important cross-feature dependencies:

- API parity depends on state model, NVS, telemetry, logs, and mining counters; do not build API fields as isolated string maps.
- Mining depends on Wi-Fi, Stratum, BM1370 ASIC path, power/thermal/fan safety, and work queue behavior.
- Gamma 601 safety depends on TPS546, EMC2101, fan control, ADC/I2C, and hardware smoke evidence.
- Self-test depends on display/input plus real fan, vcore, temperature, domain hashrate, and restart/cancel behavior.
- OTA depends on partition layout, flash artifact layout, HTTP upload behavior, filesystem availability, and recovery.
- Additional boards depend on the same feature stack plus board-specific regulator/sensor/ASIC hardware evidence.

## Complexity Notes

- **High complexity / high risk:** BM1370 initialization, ASIC work/result parsing, voltage/power/thermal/fan control, Stratum v2, OTA/partition behavior, and all-board release parity. These need phase-specific research and hardware evidence gates.
- **Medium complexity:** AxeOS API serialization, WebSocket telemetry, NVS migration/default behavior, static asset serving, self-test orchestration, display/input parity, statistics/scoreboard persistence.
- **Lower complexity:** Gamma 601 default extraction, workflow commands, reference cleanliness checks, source breadcrumbs, parity-report scaffolding, and pure model fixtures.
- **Best early pure-test targets:** config parsing, NVS key/REST-name mapping, API schema models, ASIC CRC, PLL/frequency option logic, Stratum JSON fixtures, coinbase decoding, scoreboard ordering, PID math.
- **Best early hardware-smoke targets:** boot log, PSRAM status, I2C init, reset pin held low, flash/monitor loop, safe TPS546/EMC2101 detection without enabling mining.
- **Evidence standard:** Safety-critical and hardware-control surfaces should remain below `verified` until real Gamma 601 command/log evidence exists. API surfaces should use API comparison fixtures against captured upstream responses. Protocol/config logic should use unit and golden tests.
