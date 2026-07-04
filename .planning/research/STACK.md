# Stack Research

**Domain:** Ultra 205 Rust ESP-IDF trusted Stratum v1 production mining
**Researched:** 2026-07-04
**Confidence:** HIGH for stack continuity, MEDIUM-HIGH for production-mining adapter boundaries

## Executive Recommendation

Do not change the foundational stack for v1.1. Keep ESP-IDF Rust `std`, ESP-IDF `v5.5.4`, `esp-idf-svc 0.52.1`, `esp-idf-sys 0.37.2`, Bazel, `just`, `espflash`, the pinned ESP-Miner reference, and exact-claim parity evidence rules.

The new work should add thin firmware adapters and evidence tooling around the existing pure Rust cores:

- A real Stratum v1 TCP adapter using ESP-IDF/lwIP BSD sockets through Rust `std::net::TcpStream`.
- A production mining runtime shell that connects Wi-Fi, socket I/O, BM1366 init/work/result handling, safety gates, runtime snapshots, API/WebSocket updates, watchdog checkpoints, and safe stop.
- Fresh read-only power/thermal prerequisite adapters that can mint existing safety evidence tokens from real INA260/EMC2101 observations.
- Narrow evidence wrappers and `tools/parity` allow-list extensions for accepted/rejected share claims with redaction as a first-class gate.

Avoid a dependency expansion. The repo already has the needed protocol, ASIC, safety, API, evidence, and packaging primitives. The risk is not missing crates; it is unsafe sequencing, over-broad production claims, and secret leakage.

## Recommended Stack

### Core Technologies

| Technology | Version | Purpose | Why Recommended |
|------------|---------|---------|-----------------|
| ESP-IDF | `v5.5.4` pinned by `esp-idf-sys` metadata | Production firmware platform for Wi-Fi, TCP/IP, HTTP/WebSocket, NVS, SPIFFS, OTA, FreeRTOS, logging, and image conventions | Current repo and released esp-rs crates are aligned on ESP-IDF 5.5.x; changing the IDF baseline during hardware mining bring-up would add risk without helping the milestone. |
| `esp-idf-svc` | `0.52.1` | Rust ESP-IDF service wrapper and re-export surface for `hal` and `sys` | Latest observed released crate supports ESP-IDF 5.4.x/5.5.x and already backs Wi-Fi, HTTP, logging, and platform calls in firmware. |
| `esp-idf-sys` | `0.37.2` | Raw ESP-IDF bindings and build integration | Current workspace pin is current on crates.io and already pins `esp_idf_version = "tag:v5.5.4"`. |
| Rust `std::net::TcpStream` over ESP-IDF/lwIP | Rust std on `xtensa-esp32s3-espidf` | Real Stratum v1 TCP socket connection, newline-framed JSON request/response I/O, timeouts, and reconnects | ESP-IDF v5.5.4 supports BSD sockets through lwIP; `EspWifi` connects the ESP-IDF netif/lwIP layer so Rust std TCP/UDP APIs can be used. |
| Rust `std::thread` / FreeRTOS-backed threads | Existing Rust std on ESP-IDF | Mining supervisor, socket reader/writer loop, live telemetry cadence, watchdog-friendly sleeps | The firmware already uses `std::thread::Builder` with explicit stack sizes; a blocking thread model is simpler and safer than adding async runtime complexity for one Stratum socket. |
| `esp_idf_svc::hal` UART/GPIO/I2C adapters | Through `esp-idf-svc 0.52.1` re-exports | BM1366 UART/reset, INA260/EMC2101/DS4432U I2C boundaries, and bounded hardware observation | Keeps hardware effects in the firmware shell and avoids direct raw `esp-idf-sys` calls unless HAL coverage is missing. |
| Bazel + Bzlmod + `rules_rust` | Bazel `9.1.1`, `rules_rust 0.70.0` | Canonical build/test/package/evidence graph | Existing v1.0 workflows are validated; v1.1 should extend targets instead of introducing a parallel build system. |
| `just` | Existing repo command surface | Human entrypoints for detect, build, package, flash, monitor, trusted mining evidence, parity | Keeps operator UX stable while routing real work through Bazel/repo-owned scripts. |
| `espflash` | Latest observed `4.4.0`; repo uses installed tool through wrappers | Detect, board-info, flash, monitor, package/image workflows | Current official tool supports ESP32-S3, `list-ports`, `board-info`, `flash`, `monitor`, and `save-image`; keep it as the hardware workflow backend. |

### Existing Pure Cores To Reuse

| Crate / Module | Current Role | v1.1 Use |
|----------------|--------------|----------|
| `crates/bitaxe-stratum` | Stratum v1 messages, state, work queue, controlled runtime, guarded mining loop | Keep message parsing/serialization, notify-to-work construction, share submission, share counters, hashrate inputs, and pool lifecycle in the pure core. Add pure state-machine pieces only if the real socket runtime needs additional deterministic transitions. |
| `crates/bitaxe-asic` | BM1366 init plan, command encoding, work payloads, result parsing | Reuse typed commands and result parsing; add production dispatch/result planning in the pure crate before wiring firmware UART effects. |
| `crates/bitaxe-safety` | Power, thermal, fan, status, evidence tokens, watchdog decisions | Reuse `PowerObservation`, `ThermalObservation`, `PowerEvidenceToken`, `ThermalEvidenceToken`, and fail-closed effects. v1.1 should mint tokens only from fresh observed values or explicitly bounded evidence. |
| `crates/bitaxe-api` | AxeOS API, statistics, scoreboard, WebSocket DTOs | Extend snapshots/statistics/scoreboard mapping for live accepted/rejected shares and hashrate. Do not bypass the DTO layer from firmware logs. |
| `tools/parity` | Checklist validation, API compare, mining allow-list | Extend mining allow manifests for v1.1 trusted production share surfaces rather than creating ad hoc shell gates. |

### Supporting Libraries

| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| `serde` / `serde_json` | `1.0.228` / `1.0.150` | Stratum JSON-RPC line protocol, API bodies, evidence manifests | Keep using these for protocol and evidence data; do not add another JSON stack. |
| `anyhow` | `1.0.102` | Firmware adapter and CLI/script helper errors | Use at imperative shells where context-rich operational errors matter. |
| `thiserror` | `2.0.18` | Pure crate domain errors | Use for Stratum, ASIC, safety, and API domain errors. |
| `camino` | `1.2.3` | UTF-8 paths in host tools | Continue using in parity/evidence tools. |
| Node helper scripts | Existing repo scripts | Credential JSON parsing, WebSocket capture, redacted evidence post-processing | Acceptable for host-side evidence only. Keep firmware Rust-only. |
| `curl` | System tool through scripts | API probes and settings bridge during hardware evidence runs | Keep behind repo-owned scripts and redaction filters. |

No new firmware runtime dependency is recommended for the first trusted production mining implementation. Add a crate only when a concrete implementation gap appears after trying the current stack.

### Development Tools

| Tool | Purpose | Notes |
|------|---------|-------|
| `just detect-ultra205` | Mandatory hardware gate | Continue to require exactly one likely ESP32-S3 serial port and successful `espflash board-info --chip esp32s3 --port <port> --non-interactive`. |
| `just package` / `just flash-monitor` | Package and detector-gated serial evidence | Keep package manifest/source commit/reference commit in every mining evidence run. |
| New v1.1 trusted mining evidence wrapper | Accepted/rejected share evidence capture | Prefer a new wrapper name such as `scripts/v11-trusted-mining-evidence.sh` over reusing Phase 21 controlled no-share wrappers, so claims cannot blur. |
| `tools/parity mining-allow` | Procedure allow-list and claim gate | Add v1.1 claim tiers/surfaces for trusted production mining, accepted share, rejected share, and redacted evidence. Keep unsafe command token rejection. |
| Existing redaction filters | Secret-safe evidence output | Promote reusable redaction helpers if duplication between Phase 21 scripts grows, but keep the filter owned by repo scripts rather than user shell history. |

## Installation

No new required packages are recommended for v1.1. Keep using the existing repo bootstrap:

```bash
just doctor
just bootstrap-esp
just build
just test
just package
```

If the local `espflash` binary is older than the current supported workflow requires, update through the documented esp-rs path:

```bash
cargo install espflash --locked
```

Do not add `tokio`, `async-std`, `smoltcp`, MQTT, TLS, or a new mining daemon framework for this milestone unless phase-specific implementation proves the blocking std socket path cannot meet the requirement.

## Required Stack Additions

### 1. Real Stratum Socket Adapter

Add a firmware-only module such as `firmware/bitaxe/src/stratum_socket_adapter.rs`.

Recommended shape:

- Use `std::net::TcpStream` plus `ToSocketAddrs`.
- Configure bounded connect/read/write timeouts.
- Write newline-terminated `StratumV1ClientMessage::to_json_line()` values.
- Read line-framed JSON from a buffered reader and parse through `parse_server_message`.
- Return typed events to a production runtime loop; do not mutate mining state inside socket parsing.
- Redact all endpoint/user/password values before logs or retained log buffer output.

Why: ESP-IDF v5.5.4 supports BSD sockets over lwIP, and `EspWifi` exposes Rust std TCP/UDP APIs. This is the smallest production I/O addition that matches the existing pure Stratum protocol model.

### 2. Production Mining Runtime Shell

Add a firmware runtime shell distinct from `controlled_mining_runtime.rs`, for example `production_mining_runtime.rs`.

Responsibilities:

- Start only after Wi-Fi is connected, safety supervisor is running, prerequisite safety observations are fresh, BM1366 init is complete, and the production-mining compile/runtime acknowledgment is present.
- Own the loop over subscribe, authorize, difficulty/extranonce/notify, BM1366 work dispatch, result polling, share submission, share response, stats update, watchdog checkpoint, and safe-stop.
- Keep state transitions in `crates/bitaxe-stratum` where practical.
- Publish only redacted lifecycle markers and aggregate counters.
- Replace the API-visible mining state through `runtime_snapshot`, not by constructing API JSON in the runtime loop.

This likely requires reordering `main.rs`: current controlled evidence starts before Wi-Fi. Real socket mining must start after STA networking is up and after prerequisite safety has been collected.

### 3. BM1366 Production Adapter Mode

Extend `firmware/bitaxe/src/asic_adapter.rs` behind a new explicit mode, not the existing diagnostic-only modes.

Recommended shape:

- Add a production mode separate from `ChipDetectOnly` and `WorkResultDiagnostic`.
- Use `Bm1366InitPlan::full_init` only when board/config/power/thermal/safety preflight evidence is present.
- Add a typed adapter interpreter for production work dispatch and result polling that reuses the existing UART/reset wrappers.
- Keep raw frame bytes out of logs and evidence.
- Keep fail-closed behavior: setup failure, read timeout, parse failure, stale safety token, or watchdog blocker should hold/disable the ASIC path and mark work submission blocked.

Do not jump directly to voltage/frequency/fan active parity. v1.1 needs enough safe prerequisite evidence to run trusted mining, not full active safety closure.

### 4. Prerequisite Safety Sensor Adapters

Extend `firmware/bitaxe/src/safety_adapter/power.rs` and `thermal.rs` from observe-only unavailable reports to fresh observation adapters.

Recommended shape:

- Read INA260 bus voltage/current/power over ESP-IDF I2C and convert into `PowerObservation::from_ina260_sample`.
- Read EMC2101 thermal/fan data where the Ultra 205 path requires it and convert into `ThermalObservation::from_reading`.
- Mint `PowerEvidenceToken` and `ThermalEvidenceToken` only from fresh safe observations.
- Preserve `SafetyStatus::Normal` as an explicit gate; stale, missing, invalid, over-limit, or unsafe observations should block work submission.
- Keep DS4432U voltage writes and fan duty writes suppressed unless a later phase owns bounded active safety evidence.

This provides mining prerequisite safety without claiming full voltage, fan, thermal fault-stimulus, or recovery parity.

### 5. Live Stats, API, WebSocket, And Scoreboard

Use existing `MiningRuntimeState`, `HashrateInputs`, `ShareCounters`, `runtime_snapshot`, `http_api`, and `websocket_api`.

Add only the missing mapping/runtime pieces:

- Accepted/rejected share counters from real `mining.submit` responses.
- Rejected reason aggregation with secret-safe reason text.
- Hashrate inputs based on actual work/result window measurements, with units clearly documented.
- `/api/system/info`, `/api/system/statistics`, `/api/system/scoreboard`, `/api/ws`, and `/api/ws/live` correlation evidence from the same trusted session.
- Watchdog checkpoint markers from the production runtime loop.
- Safe-stop markers after the run.

Avoid duplicating statistics state in firmware globals if `MiningRuntimeState` can be the source of truth.

### 6. Evidence And Redaction Tooling

Create v1.1-owned evidence scripts and parity allow-list rows instead of expanding ad hoc shell commands.

Required evidence stack additions:

- A mining allow manifest schema extension for `trusted-production-mining`, `accepted-share-observed`, `rejected-share-observed`, and `production-share-pending` claim tiers.
- A v1.1 trusted mining wrapper that requires detector output, package manifest, board-info output, source commit, reference commit, prerequisites, exact command, abort conditions, recovery steps, logs, API/WebSocket artifacts, and redaction review.
- Redacted evidence outputs that may record `pool_config: local-owner-supplied` but never raw pool URL, port, user, worker, password, owner address, token, `DEVICE_URL`, Wi-Fi secrets, IP, MAC, or NVS secret values in committed artifacts.
- A final redaction scan over the evidence directory before parity checklist promotion.

Keep using `pool-credentials.json.example` for shape only. Real `pool-credentials*.json` files remain local runtime inputs and must not be read into research/planning artifacts or committed evidence.

## Alternatives Considered

| Recommended | Alternative | When to Use Alternative |
|-------------|-------------|-------------------------|
| Blocking `std::net::TcpStream` | `tokio`, `async-std`, `async-io`, `smol` | Only if one blocking Stratum socket plus existing HTTP/WebSocket tasks cannot meet watchdog and responsiveness requirements. |
| ESP-IDF/lwIP BSD sockets | `smoltcp` or custom network stack | Only for a future non-ESP-IDF/no_std architecture, not this ESP-IDF parity milestone. |
| Existing `serde_json` Stratum JSON | New custom parser or `simd-json` | Avoid unless profiling proves JSON parse overhead matters; correctness and testability matter more now. |
| Extend existing pure crates | New `bitaxe-mining-runtime` crate | Use a new crate only if production runtime pure state grows beyond Stratum scope and starts coupling ASIC, safety, stats, and evidence logic awkwardly. |
| v1.1 evidence wrapper + `tools/parity` allow-list | Freeform manual command transcript | Manual transcripts are acceptable as raw evidence but should not be the claim gate. |
| Read-only prerequisite safety sensors | Active voltage/fan/fault stimulation | Active safety closure belongs to a later phase unless it becomes a hard prerequisite and has a documented recovery path. |

## What NOT To Use

| Avoid | Why | Use Instead |
|-------|-----|-------------|
| ESP-IDF `v6.x` migration in v1.1 | Adds SDK and binding churn while the milestone is hardware/evidence sensitive. | Keep `v5.5.4`; schedule an IDF 6 reassessment after trusted mining closure. |
| Async runtime by default | Adds executor and wakeup complexity on FreeRTOS before proving blocking socket I/O is insufficient. | One bounded blocking socket thread with explicit stack, timeouts, watchdog yields, and safe stop. |
| Raw `esp-idf-sys` socket code | Harder to test and easier to get ownership/error handling wrong. | Rust `std::net` over ESP-IDF BSD sockets, dropping to raw sys only for missing socket options. |
| TLS for Stratum v1 in this milestone | Most mining pools still support plaintext Stratum v1; TLS adds certificate/time/heap failure modes unrelated to accepted/rejected share proof. | Plain Stratum TCP first; document TLS as future hardening. |
| MQTT, Stratum v2, BAP, display/input, OTAWWW work | Not required to prove trusted Ultra 205 Stratum v1 production mining. | Defer to later milestones. |
| Direct pool credentials in logs, summaries, or artifacts | Leaks owner addresses, workers, endpoints, and passwords. | Redacted category labels and local runtime inputs only. |
| Treating controlled no-share evidence as production share evidence | v1.0 explicitly made accepted/rejected shares an exact non-claim. | Require at least one real accepted or rejected pool response, or record the milestone as blocked/pending. |
| Voltage/fan actuation shortcuts | Can damage hardware and would overclaim full active safety. | Fresh prerequisite telemetry plus fail-closed guards; active controls need separate hardware-regression evidence. |
| Network scans or stale device URLs | Violates current target-lock guidance and can hit the wrong device. | Use only detector-gated fresh monitor output or explicit same-session `DEVICE_URL` when allowed. |

## Stack Patterns By Variant

**If the first v1.1 goal is accepted/rejected share proof:**

- Use blocking `TcpStream` with real pool credentials supplied only as local runtime input.
- Use production BM1366 init only after fresh power/thermal/safety gates pass.
- Run until one share response is accepted or rejected, or until a bounded timeout records `production-share-pending`.
- Promote only the exact observed claim.

**If hardware safety prerequisites cannot be freshly observed:**

- Keep production mining blocked.
- Record prerequisite blocker evidence with board, port, source commit, reference commit, and redaction status.
- Do not use synthetic fixture tokens to bypass production gates.

**If socket I/O works but no ASIC result appears:**

- Record subscribe/authorize/notify/work-dispatch evidence separately from share evidence.
- Keep accepted/rejected share rows below `verified`.
- Investigate BM1366 init/result path before changing network stack.

**If socket I/O starves HTTP/WebSocket/watchdog:**

- First tune blocking timeouts, thread stack, sleep/yield cadence, and message pump boundaries.
- Only then consider `async-io`/`smol` style async sockets as a scoped implementation experiment.

## Version Compatibility

| Package A | Compatible With | Notes |
|-----------|-----------------|-------|
| `esp-idf-svc 0.52.1` | ESP-IDF `v5.4.x` / `v5.5.x` | Current released service crate; keep current workspace pin. |
| `esp-idf-sys 0.37.2` | ESP-IDF `v5.5.4` through explicit metadata | Current raw binding/build crate; do not rely on defaults. |
| ESP-IDF `v5.5.4` | ESP32-S3 / `xtensa-esp32s3-espidf` | Official bugfix release for the 5.5 branch; repo firmware target remains Ultra 205 ESP32-S3. |
| `espflash 4.4.0` | ESP32-S3 board-info/flash/monitor/save-image workflows | Current observed release supports the commands this repo already wraps. |
| `rules_rust 0.70.0` | Bazel 7/8/9; repo uses Bazel 9.1.1 | Keep Cargo.lock authoritative and mirror through `crate_universe`. |

## Sources

- `.planning/PROJECT.md`, `.planning/MILESTONES.md`, `.planning/milestones/v1.0-MILESTONE-AUDIT.md`, `docs/parity/checklist.md` - current milestone scope, exact non-claims, and v1.0 shipped state. HIGH confidence.
- `firmware/bitaxe/src/mining_evidence_mode.rs`, `controlled_mining_runtime.rs`, `asic_adapter.rs`, `safety_adapter.rs`, `main.rs`, `runtime_snapshot.rs`, `http_api.rs`, `websocket_api.rs` - current firmware gates, controlled no-share runtime, ASIC shell, safety shell, and API/WebSocket integration points. HIGH confidence.
- `crates/bitaxe-stratum/src/v1/*`, `crates/bitaxe-asic/src/bm1366/init_plan.rs`, `crates/bitaxe-safety/src/power.rs`, `thermal.rs` - pure protocol, ASIC, and safety decision surfaces to preserve. HIGH confidence.
- `scripts/phase21-live-mining-evidence.sh`, `scripts/phase21-live-mining-package.sh`, `scripts/phase21-pool-input-bridge.sh`, `tools/parity/src/mining_allow.rs` - existing evidence/allow-list/redaction patterns to extend. HIGH confidence.
- ESP-IDF v5.5.4 release: https://github.com/espressif/esp-idf/releases/tag/v5.5.4 - official release/date and v5.5.4 status. HIGH confidence.
- ESP-IDF v5.5.4 ESP32-S3 lwIP docs: https://docs.espressif.com/projects/esp-idf/en/v5.5.4/esp32s3/api-guides/lwip.html - BSD socket support, thread-safety caveats, supported functions. HIGH confidence.
- `esp-idf-svc` `EspWifi` docs: https://esp-rs.github.io/esp-idf-svc/esp_idf_svc/wifi/struct.EspWifi.html - confirms `EspWifi` binds ESP-IDF netif/lwIP to allow Rust std TCP/UDP APIs. HIGH confidence.
- crates.io `esp-idf-sys`: https://crates.io/crates/esp-idf-sys - current `0.37.2` version and build model. HIGH confidence.
- crates.io `esp-idf-svc`: https://crates.io/crates/esp-idf-svc and docs.rs changelog search result - current `0.52.1` and ESP-IDF 5.4/5.5 compatibility. MEDIUM-HIGH confidence because direct fetch timed out once, but search and docs.rs snippets agreed.
- crates.io `espflash`: https://crates.io/crates/espflash - current `4.4.0` and supported commands. HIGH confidence.
- `AGENTS.md`, `AGENTS.bright-builds.md`, `standards/core/architecture.md`, `standards/core/verification.md`, `standards/core/testing.md`, `standards/core/code-shape.md`, `standards/languages/rust.md` - local workflow, functional-core/imperative-shell, verification, and Rust guidance. HIGH confidence.

*Stack research for: Ultra 205 trusted production mining*
*Researched: 2026-07-04*
