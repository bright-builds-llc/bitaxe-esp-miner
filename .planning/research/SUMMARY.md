# Project Research Summary

**Project:** Bitaxe Rust Firmware\
**Domain:** Rust ESP-IDF firmware rewrite for Bitaxe ESP-Miner\
**Researched:** 2026-06-20\
**Confidence:** MEDIUM-HIGH

## Executive Summary

This project is a hardware-bound firmware rewrite, not an application rewrite or a C-to-Rust transliteration. The correct success measure is device-user parity with upstream ESP-Miner: a Bitaxe owner, API client, mining pool, flashing tool, or admin UI should observe compatible behavior. Experts would build this by keeping the upstream implementation pinned and read-only, modelling behavior in typed Rust domains, and proving parity through unit, golden, API-compare, and hardware evidence.

Use a Rust ESP-IDF `std` firmware stack for the first production path. Pin ESP-IDF to `v5.5.4`, use the ESP Rust toolchain for `xtensa-esp32s3-espidf`, keep Bazel as the canonical automation graph, and keep `just` as the human command surface. Structure the code as a functional core plus imperative ESP-IDF shell: pure crates own config, ASIC codecs, Stratum, API models, statistics, state machines, and test fixtures; `firmware/bitaxe` owns Wi-Fi, FreeRTOS tasks, NVS, HTTP/WebSocket serving, SPIFFS, OTA, UART, GPIO, I2C, ADC, fan, voltage, display, and orchestration.

The first roadmap milestone must stay deliberately small: foundation plus safe Gamma 601 BM1370 boot/log, build/package/flash/monitor, reference guardrails, and parity evidence scaffolding. The main risks are false parity claims, unsafe hardware bring-up, GPL provenance leakage, and Bazel/Cargo/ESP-IDF workflow drift. Mitigate them with a read-only reference submodule, evidence-backed checklist status, staged hardware enablement, range-checked domain types, package manifests, SPDX/provenance review, and hardware evidence before safety-critical parity is marked verified.

## Key Findings

### Recommended Stack

Use ESP-IDF Rust `std` rather than bare-metal Rust for the first firmware because upstream ESP-Miner depends on ESP-IDF services: Wi-Fi, FreeRTOS, HTTP/WebSocket, NVS, partitions, SPIFFS/static assets, OTA, logging, and ESP image conventions. Bazel should own repeatable automation, but the firmware target should initially wrap Cargo plus `esp-idf-sys` instead of attempting a custom Bazel-native ESP-IDF toolchain.

**Core technologies:**

- ESP-IDF `v5.5.4` - stable baseline supported by released `esp-idf-*` crates.
- `esp-idf-svc 0.52.1` with `esp-idf-hal 0.46.2` and `esp-idf-sys 0.37.2` - Rust access to Wi-Fi, HTTP, WebSocket, NVS, OTA, logging, event loop, timers, and lower HAL/sys layers.
- ESP Rust toolchain via `espup v0.17.1` - required for Xtensa ESP32-S3 firmware builds with `std`.
- Target `xtensa-esp32s3-espidf`, `MCU=esp32s3` - expected Gamma 601 target, pending confirmation from initialized reference tree.
- Bazelisk with Bazel `9.1.1` and Bzlmod - canonical automation and CI/local graph.
- `rules_rust 0.70.0` with `crate_universe` - Bazel coverage for pure Rust crates, host tools, and tests using Cargo metadata.
- `espflash` / `cargo-espflash` - USB flashing, monitoring, port listing, board info, and image save workflows.
- `just` - ergonomic command surface only; recipes delegate to Bazel targets or repo-owned scripts represented in Bazel.

**Critical version requirements:**

- Do not use ESP-IDF `v6.0.1` as the first baseline; released Rust ESP-IDF crates have safer compatibility with ESP-IDF `v5.5.x`.
- Pin `esp_idf_version = "tag:v5.5.4"` explicitly in firmware metadata; do not rely on `esp-idf-sys` defaults.
- Use Rust 2021 initially for firmware and shared crates; consider Rust 2024 only after build/flash is stable.
- Add `espidf_time64` and `ldproxy` through the ESP-IDF Rust template-compatible configuration path.

### Expected Features

**Must have (table stakes):**

- Gamma 601 defaults - `devicemodel=gamma`, `boardversion=601`, `asicmodel=BM1370`, `asicfrequency=525`, `asicvoltage=1150`, pool defaults, fan defaults, and self-test defaults.
- Build/package/flash/monitor workflow - `just build`, `test`, `package`, `flash`, `monitor`, `flash-monitor`, `verify-reference`, and `parity`.
- Safe boot/log path - firmware identity, ESP-IDF/Rust versions, reset reason, partition/image identity, PSRAM/platform status, board/ASIC target, and safe no-op mining state.
- Config and NVS compatibility - upstream key names, defaults, validation, persistence, reboot reload behavior, and migrations where needed.
- BM1370 ASIC path - reset, safe staged init, packet layout, CRC, work encoding, result parsing, frequency/voltage transitions, and hardware evidence.
- Stratum v1 mining - subscribe, authorize, notify, difficulty, submit, fallback pool, reconnect, accepted/rejected shares, and fake pool coverage.
- AxeOS API compatibility - OpenAPI-compatible system, ASIC, statistics, scoreboard, settings, logs, pause/resume, restart, identify, OTA, and telemetry routes.
- WebSocket logs and live telemetry - `/api/ws` and `/api/ws/live` behavior expected by clients.
- Power, voltage, thermal, and fan safety - Gamma TPS546, EMC2101, fan RPM/control, PID behavior, overheat/fault handling, and fail-closed policy.
- Self-test lifecycle - factory/self-test flags, fan/power/vcore/temp/hashrate/domain checks, pass/fail/restart/cancel behavior.
- OTA/filesystem/release behavior - partition layout, SPIFFS/static assets, recovery, OTA firmware, OTAWWW, image naming, and release manifests.

**Should have (differentiators):**

- Explicit parity evidence with breadcrumbs, statuses, command summaries, and artifact paths.
- Pure Rust domain crates with strong newtypes/state machines for board IDs, ASIC models, frequencies, millivolts, temperatures, fan duty, jobs, nonces, and API updates.
- Safer staged hardware-control adapters that calculate decisions in pure code but gate effects in firmware.
- Deterministic fake pool, API compare, golden fixture, and hardware-smoke workflows.
- Clear USB ergonomics with port discovery, actionable failure messages, and printed underlying `espflash` commands.

**Defer (v2+ or post-Gamma V1):**

- Non-601 boards and additional ASIC families, including Bitaxe 205/BM1366, BM1397, BM1368, Gamma Duo/Turbo, Max, Ultra, Hex, and Supra.
- Stratum v2 completeness unless an explicit Gamma 601 acceptance target requires it.
- BAP accessory protocol completeness unless accessory parity becomes a named V1 goal.
- All-board factory image release automation and verification matrix.
- Custom board config flows and advanced tuning beyond upstream-compatible ranges.
- Angular AxeOS rewrite; scope is API and static asset compatibility.

### Architecture Approach

Use behavior-led modules rather than upstream C file boundaries. Pure crates should expose deterministic data-in/data-out APIs and remain free of ESP-IDF, FreeRTOS, sockets, NVS, UART, GPIO, file system, clocks, and logging. The firmware app should translate real-world observations into typed domain events, call pure decision logic, then apply resulting commands through explicit adapters with logging and error handling.

**Major components:**

1. `reference/esp-miner` - pinned, read-only upstream behavior and provenance reference.
1. `firmware/bitaxe` - ESP-IDF app, boot order, task orchestration, hardware adapters, HTTP/WebSocket server, storage, OTA, logging, and board bring-up.
1. `crates/bitaxe-core` - shared device state, work queues, statistics, scoreboard, PID decisions, events, and state machines.
1. `crates/bitaxe-config` - board/device models, Gamma 601 defaults, NVS key schema, typed settings, and validation.
1. `crates/bitaxe-asic` - ASIC model dispatch, packet layouts, CRC, register commands, BM1370 work/result encoding.
1. `crates/bitaxe-stratum` - Stratum message parsing/serialization, job construction, coinbase decode, reconnect/fallback decisions.
1. `crates/bitaxe-api` - AxeOS OpenAPI-compatible request/response models, serializers, settings validation, telemetry/log payloads.
1. `crates/bitaxe-test-support` - reference-derived fixtures, fake adapters, golden data, and hardware-test helpers.
1. `tools/flash` and `tools/parity` - USB workflow support and evidence/checklist reporting.

**Key patterns to follow:**

- Dependency direction flows from firmware to pure crates; pure crates do not depend on firmware or ESP-IDF.
- Parse raw inputs at boundaries into domain types; do not pass raw strings or integers through hardware-control logic.
- Make illegal states unrepresentable where practical, especially mining lifecycle, configured/unconfigured state, power state, and API PATCH results.
- Put reference breadcrumbs at module and behavior boundaries, not line-by-line.
- Treat checklist `verified` as an evidence state, not an implementation state.

### Critical Pitfalls

1. **False parity confidence** - prevent by requiring evidence in `docs/parity/checklist.md`; safety-critical rows need `hardware-smoke` or `hardware-regression`, not only unit/golden tests.
1. **Unsafe hardware bring-up** - prevent by booting/logging first, then staged power/thermal/fan/ASIC enablement with preflight gates, fail-closed behavior, and hardware logs.
1. **Reference/provenance drift** - prevent by adding `reference/esp-miner` as a pinned read-only submodule, failing on dirty/missing reference state, and reviewing any GPL-derived expression or fixture.
1. **Bazel/Cargo/ESP-IDF divergence** - prevent by making Bazel the visible graph, Cargo.lock authoritative for Rust dependencies, `just` a thin wrapper, and package artifacts declared with versions/manifests.
1. **NVS/API/settings mismatch** - prevent by comparing upstream OpenAPI, captured responses, NVS keys, defaults, persistence, and PATCH semantics before broad handler implementation.
1. **Mining happy-path bias** - prevent by adding deterministic Stratum fixtures and fake pool scenarios before relying on public-pool smoke tests.
1. **Broad board claims too early** - prevent by keeping Gamma 601 as the only verified hardware path until each additional board has its own evidence set.

## Implications for Roadmap

Based on research, suggested phase structure:

### Phase 1: Foundation And Gamma 601 Boot/Log

**Rationale:** This must come first because every later parity claim depends on a reproducible graph, reference guardrails, flash/monitor workflow, and safe hardware smoke loop. It should not attempt mining.\
**Delivers:** Repo/workspace skeleton, `reference/esp-miner` guard, Bazel/Bzlmod/Cargo setup, pure crate stubs, `just` command surface, ESP-IDF Rust firmware skeleton, firmware package target, `tools/flash`, `tools/parity`, provenance policy, and minimal Gamma 601 boot/log firmware with ASIC work disabled.\
**Addresses:** First milestone foundation, USB flash workflow, boot/platform status, parity checklist integration, provenance guardrails.\
**Avoids:** P-01, P-02, P-07, P-08, P-11, P-13.\
**Research flag:** Mostly standard patterns, but the Bazel wrapper around Cargo/ESP-IDF and flashable image manifest may need an implementation spike.

### Phase 2: Gamma 601 Config And NVS Model

**Rationale:** Full typed config should follow the safe boot/log path so hardware identity, defaults, API settings, and future ASIC work are not scattered as constants.\
**Delivers:** `bitaxe-config`, Gamma 601 golden defaults, board/device identifiers, NVS key schema, validation ranges, settings update model, reboot reload tests, and explicit deferred/non-verified status for non-601 boards.\
**Addresses:** Gamma 601 defaults, board and ASIC config model, NVS/settings behavior, early API setting names.\
**Avoids:** P-02, P-04, P-09, P-12.\
**Research flag:** Standard patterns; use the reference tree and golden fixtures rather than a separate research phase unless upstream config extraction is ambiguous.

### Phase 3: BM1370 ASIC Protocol And Safe Initialization

**Rationale:** ASIC packet formats and work/result parsing can be tested in pure Rust before live hardware control, while live init must be narrow and evidence-heavy.\
**Delivers:** `bitaxe-asic` BM1370 codecs, CRC, register/work/result fixtures, UART adapter boundary, reset/preflight flow, staged hardware init logs, and no successful-mining claims yet.\
**Addresses:** BM1370 ASIC behavior, ASIC reset/init, frequency transitions, work packet construction, nonce/result parsing.\
**Avoids:** P-03, P-04, P-05, P-06, P-11.\
**Research flag:** Needs phase research or spike for upstream BM1370 sequencing, timing, reset, voltage dependencies, and hardware evidence plan.

### Phase 4: Stratum V1 And First Mining Loop

**Rationale:** Mining depends on boot, config, safe ASIC initialization, Wi-Fi/socket lifecycle, and tested protocol construction.\
**Delivers:** `bitaxe-stratum`, deterministic protocol fixtures, fake pool harness, pool subscribe/authorize/notify/set_difficulty/submit, work queue integration, first accepted-share smoke, fallback/reconnect behavior, and mining task/watchdog plan.\
**Addresses:** Stratum v1 mining, public/fallback pool behavior, job construction, share submission, accepted/rejected share reporting.\
**Avoids:** P-03, P-06, P-10, P-13.\
**Research flag:** Needs deeper research for Stratum edge cases, fake pool design, reconnect behavior, watchdog/yielding, and long-running soak criteria.

### Phase 5: AxeOS API, Logs, And Telemetry

**Rationale:** API and telemetry are user-visible parity surfaces that should be backed by real config, mining, ASIC, and system state rather than early stubs.\
**Delivers:** `bitaxe-api`, OpenAPI models, captured response/API compare fixtures, HTTP handlers, settings PATCH, log buffer/download, `/api/ws`, `/api/ws/live`, statistics, scoreboard, pause/resume, restart, identify, and telemetry cadence checks.\
**Addresses:** AxeOS HTTP API, WebSocket logs/live telemetry, logging/statistics/scoreboard, theme API if unchanged static assets require it.\
**Avoids:** P-02, P-09, P-13.\
**Research flag:** Standard HTTP/model patterns, but route-by-route compare fixtures and upstream response capture should be planned carefully.

### Phase 6: Safety Controllers And Self-Test

**Rationale:** Power, voltage, thermal, fan, and self-test parity must be validated after the device can mine and report telemetry, because meaningful checks depend on real runtime behavior.\
**Delivers:** TPS546/vcore control, EMC2101 thermal path, fan RPM/control, PID behavior, overheat/fault handling, fail-closed policy, display/input minimums, self-test lifecycle, hardware regression logs, and soak evidence.\
**Addresses:** Power/voltage/thermal/fan safety, self-test lifecycle, display/input surfaces needed for status and factory flow.\
**Avoids:** P-05, P-06, P-12.\
**Research flag:** Needs deeper research and hardware planning; safety-critical surfaces cannot be verified without real Gamma 601 evidence.

### Phase 7: OTA, Filesystem, Static Assets, And Release Packaging

**Rationale:** OTA and release behavior depends on stable partition/image layout, NVS/API choices, static assets, recovery behavior, and license provenance.\
**Delivers:** Partition table/image manifest with offsets and SHA256s, SPIFFS/static AxeOS assets, recovery page behavior, firmware OTA, OTAWWW, image size checks, release naming, dependency license inventory, source/reference commit manifest, and installation/flashing instructions.\
**Addresses:** Static assets/recovery, OTA firmware, OTAWWW, filesystem behavior, factory/update image packaging, release artifacts.\
**Avoids:** P-08, P-09, P-11, P-14.\
**Research flag:** Needs phase research for ESP-IDF OTA/partition details, rollback/recovery testing, large erase watchdog behavior, and release compliance.

### Phase 8: Expansion Beyond Gamma 601

**Rationale:** Additional boards, ASICs, Stratum v2, BAP, and all-board release automation should wait until the Gamma 601 path has stable evidence and release discipline.\
**Delivers:** One-board/one-ASIC-at-a-time expansion, Bitaxe 205/BM1366 path, additional ASIC families, Stratum v2, BAP, all-board factory image matrix, and per-board evidence gates.\
**Addresses:** Non-601 board parity, additional ASICs, Stratum v2, BAP accessory port, all-board release artifacts.\
**Avoids:** P-04, P-05, P-10, P-11, P-12.\
**Research flag:** Needs research per board/ASIC/protocol; do not inherit verification from Gamma 601.

### Phase Ordering Rationale

- Foundation and safe boot/log come first because no later evidence is credible without a repeatable build/package/flash/monitor loop and a known reference baseline.
- Config comes before ASIC and Stratum because board defaults, NVS names, ranges, and identity drive hardware control, API output, and mining behavior.
- ASIC comes before Stratum because work packets, reset/init, and result parsing are prerequisites for meaningful mining.
- Stratum comes before API/telemetry completion because API statistics and WebSocket live state should reflect real mining lifecycle and counters.
- Safety/self-test follows initial mining but must gate release-worthy mining parity because voltage, fan, thermal, and fail-closed behavior are safety-critical.
- OTA/release comes after stable runtime surfaces because update behavior and release compliance depend on partition, asset, source, and license decisions.
- Expansion is last to avoid diluting Gamma 601 evidence and creating broad unverified board claims.

### Research Flags

Phases likely needing `/gsd-research-phase` during planning:

- **Phase 3:** BM1370 init sequence, timing, hardware safety gates, UART behavior, and power dependencies.
- **Phase 4:** Stratum edge cases, fake pool harness, reconnect/fallback behavior, watchdog/yielding, and soak evidence.
- **Phase 6:** TPS546, EMC2101, fan/PID behavior, self-test sequencing, and hardware regression protocol.
- **Phase 7:** OTA partition layout, rollback/recovery behavior, static asset packaging, image manifests, and release license obligations.
- **Phase 8:** Each new board, ASIC family, Stratum v2, and BAP path needs targeted research and evidence planning.

Phases with standard patterns that can usually skip a standalone research phase:

- **Phase 1:** Repo skeleton, Bazel/Bzlmod setup, `just` wrappers, and pure crate scaffolding are documented patterns; only the ESP-IDF image wrapper may need a spike.
- **Phase 2:** Typed config, NVS schema modelling, and golden defaults are standard Rust modelling work once the reference files are available.
- **Phase 5:** HTTP model/serializer work is standard, but API compare fixture design should be explicit in the phase plan.

## Confidence Assessment

| Area | Confidence | Notes |
| --- | --- | --- |
| Stack | MEDIUM-HIGH | Strong official source support for ESP-IDF Rust, Bazel 9, `rules_rust`, `espup`, and `espflash`; ESP-IDF/Cargo/Bazel wrapper details still need implementation proof. |
| Features | HIGH | Observable parity surfaces are well established from project docs and upstream ESP-Miner contracts; exact V1/deferred sequencing is more judgment-based. |
| Architecture | HIGH | Functional core plus imperative shell is directly aligned with local standards and project decisions; ESP-IDF service adapters need spikes as they are implemented. |
| Pitfalls | MEDIUM-HIGH | Roadmap, parity, safety, and provenance risks are well supported; exact ASIC/power sequencing confidence is lower until the reference tree and hardware traces exist. |

**Overall confidence:** MEDIUM-HIGH

## Gaps to Address

- `reference/esp-miner` is currently absent in this checkout; Phase 1 must initialize and pin it before serious parity work.
- Confirm Gamma 601 ESP32-S3 target details, board pins, power path, and BM1370 assumptions from the pinned reference tree.
- Capture upstream API responses, config defaults, logs, and hardware behavior before claiming API or hardware parity.
- Define the hardware evidence format early: command, board, port, firmware commit, reference commit, timestamp, log path, observed result, and conclusion.
- Design the HIL/hardware-smoke strategy for voltage, fan, thermal, ASIC init, mining, and long-running soak before safety-critical phases.
- Prove the Bazel wrapper around Cargo/ESP-IDF firmware builds can produce declared ELF/bin/image artifacts and package manifests.
- Decide how reference-derived fixtures and any intentionally ported GPL-covered expression are labeled and isolated before release artifacts.
- Reassess ESP-IDF 6 only after released `esp-idf-svc`/`esp-idf-hal`/`esp-idf-sys` versions support it fully and the Gamma 601 baseline is stable.

## Sources

### Primary (HIGH confidence)

- `.planning/PROJECT.md` - project scope, requirements, decisions, constraints, and first milestone target.
- `.planning/research/STACK.md` - recommended toolchain, versions, build strategy, flashing workflow, and risks.
- `.planning/research/FEATURES.md` - parity features, first milestone, deferred features, anti-features, and dependency flow.
- `.planning/research/ARCHITECTURE.md` - components, boundaries, data flow, build order, and testing strategy.
- `.planning/research/PITFALLS.md` - critical pitfalls, warning signs, prevention strategies, phase mapping, and guardrails.
- `docs/project/gsd-new-project-brief.md`, `docs/project/seed-layout.md`, `docs/project/project-decisions.md`, `docs/project/first-milestone.md` - accepted project decisions.
- `docs/parity/checklist.md` and `PROVENANCE.md` - parity evidence and licensing guardrails.
- Upstream ESP-Miner commit `c1915b0a63bfabebdb95a515cedfee05146c1d50` - accepted reference baseline for parity research.

### Official / Vendor (HIGH confidence)

- ESP-IDF versions and releases - `https://docs.espressif.com/projects/esp-idf/en/stable/esp32/versions.html`, `https://github.com/espressif/esp-idf/releases`
- ESP-IDF build, partition, OTA, NVS, FreeRTOS, watchdog, and power-management docs - used for platform and release risk assessment.
- ESP Rust and crates - `https://docs.espressif.com/projects/rust/`, `https://github.com/esp-rs/esp-idf-svc`, `https://docs.rs/crate/esp-idf-sys/latest`
- `espup` and `espflash` - `https://github.com/esp-rs/espup`, `https://github.com/esp-rs/espflash`
- Bazel, Bazelisk, and `rules_rust` - `https://bazel.build/release`, `https://bazel.build/install/bazelisk`, `https://bazelbuild.github.io/rules_rust/`, `https://registry.bazel.build/modules/rules_rust`
- SPDX and GPL-3.0 guidance - `https://spdx.org/licenses/GPL-3.0-only`, `https://spdx.dev/learn/handling-license-info/`

### Upstream ESP-Miner Surfaces (HIGH for observed contracts, MEDIUM until local reference is initialized)

- `config-601.cvs` - Gamma 601 defaults.
- `main/device_config.h` - board and ASIC matrix.
- `main/http_server/openapi.yaml` - AxeOS API contract.
- `main/http_server/http_server.c`, `main/nvs_config.c`, `main/main.c`, `main/self_test/self_test.c`, `main/bap/bap_readme.md` - observable behavior and parity targets.
- `readme.md` and `flashing.md` - user/admin and flashing workflow surfaces.

### Local Observation (MEDIUM confidence)

- Current checkout had no `reference/` directory and `git submodule status --recursive` returned no submodules during pitfalls research on 2026-06-20.

______________________________________________________________________

*Research completed: 2026-06-20*\
*Ready for roadmap: yes*
