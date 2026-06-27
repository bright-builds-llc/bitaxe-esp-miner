# Phase 05: axeos-api-logs-and-telemetry - Research

**Researched:** 2026-06-27
**Domain:** ESP-IDF Rust HTTP API, AxeOS wire compatibility, log buffering, WebSocket telemetry, settings persistence
**Confidence:** HIGH

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions
## Implementation Decisions

### API Response And Schema Compatibility

- **D-01:** Replace the Phase 1 `ApiRuntimeStatus::DeferredUntilPhase5` placeholder with handwritten Serde wire DTOs and pure mapper functions in `crates/bitaxe-api`. Do not serialize internal config, Stratum, ASIC, or core domain structs directly as the public AxeOS contract.
- **D-02:** Treat upstream OpenAPI and captured upstream JSON responses as compatibility oracles, not as the primary firmware implementation source. OpenAPI-generated models may be used only for host-side checking if useful.
- **D-03:** Preserve upstream field names, casing, units, and compatibility quirks in the wire layer, including names such as `ASICModel`, `hashRate_1m`, `fanspeed`, `fanrpm`, numeric bool-like fields where upstream emits numbers, and bool values where upstream emits JSON booleans.
- **D-04:** Build API responses from existing typed Rust sources where possible: `bitaxe-config` catalog/default/settings data, Phase 4 `MiningRuntimeState`, BM1366 status and gate outputs, firmware platform facts, and future thin adapter snapshots.
- **D-05:** Dynamic or not-yet-implemented hardware values must be represented with upstream-compatible safe defaults, safe blocked status, or explicit unavailable/deferred values. Do not invent live voltage/fan/thermal/power claims without Phase 6 evidence.

### Settings PATCH Semantics

- **D-06:** Implement `PATCH /api/system` with upstream-compatible external behavior: parse JSON, validate all known REST-named settings before writing, ignore unknown fields, reject the whole patch when any known field is invalid, and keep user-visible errors generic enough to match upstream expectations.
- **D-07:** Reuse the existing pure settings boundary in `crates/bitaxe-config` (`SettingsPatch`, `apply_settings_patch`, compatibility writes, and persistence snapshot/reload semantics) instead of duplicating validation in HTTP handlers.
- **D-08:** Use a stronger Rust adapter contract internally: accepted settings should be persisted and reloaded before returning success when the adapter can prove that result. Preserve the upstream success/error response shape, and define firmware-visible failures for NVS write or commit errors without exposing a new rich public error schema.
- **D-09:** Preserve upstream side behaviors that are visible to clients: no writes on rejection, best-effort live hostname apply after accepted hostname changes, generic `400 Wrong API input` style rejection, empty success body, and typed internal logs for diagnostics.
- **D-10:** Settings updates may validate frequency, voltage, fan, thermal, pool, and display fields, but Phase 5 must not enable unsafe hardware effects. Later firmware adapters and Phase 6 safety controllers own those effects.

### System, ASIC, Statistics, Scoreboard, And Mining State

- **D-11:** Implement `/api/system/info` as the broad upstream-compatible full-state response built from pure wire DTOs and adapter snapshots. Include system/config/mining/pool/hashrate/fault/block fields where Phase 5 has source data, and use safe unavailable or blocked values for later safety surfaces.
- **D-12:** Implement `/api/system/asic` from typed Ultra 205 catalog and BM1366 status data, including ASIC model, device model, swarm color where known, ASIC count, hash domains, default frequency, frequency options, default voltage, and voltage options.
- **D-13:** Implement `/api/system/statistics` with upstream-compatible labels, optional `columns` selection, timestamp handling, and statistics row shape. If historical samples are not available yet, expose an empty or minimal compatible series with documented evidence rather than fake history.
- **D-14:** Implement `/api/system/scoreboard` with the upstream array shape (`difficulty`, `job_id`, `extranonce2`, `ntime`, `nonce`, `version_bits`) and preserve the AxeOS TypeScript model expectation that clients may add or display rank/since separately.
- **D-15:** Map Phase 4 mining runtime fields into API-visible accepted shares, rejected shares, rejected reasons, pool difficulty, fallback-active status, response time where known, and safe blocked/paused/active mining state without bypassing mining-loop evidence gates.

### Logs And WebSocket Telemetry

- **D-16:** Implement log buffer and log download behavior as a firmware adapter around a small host-testable contract. `/api/system/logs` should return `text/plain` with `Content-Disposition: attachment; filename="bitaxe-logs.txt"` and retained ring-buffer contents where available.
- **D-17:** Preserve upstream `/api/ws` log stream semantics: WebSocket text frames contain raw log text chunks, new clients start at the current end of the buffer rather than dumping all retained logs, and chunks are emitted only when clients are connected and new log data is available.
- **D-18:** Preserve upstream `/api/ws/live` telemetry semantics: on connect send the full current state as `{"event":"update","data":...}`, then send diff updates at the upstream-compatible cadence of 500 ms when values change.
- **D-19:** Keep payload construction and diff decisions host-testable where practical in `crates/bitaxe-api`; keep ESP-IDF HTTP server, WebSocket client tracking, async sends, task cadence, mutexes, and log hook integration inside `firmware/bitaxe`.
- **D-20:** WebSocket and log tests should include no-client hibernation/baseline reset, unchanged-state no-send behavior, connect-time full update, raw log chunking, buffer clamp/line resync behavior where modeled, and retained-log download fixtures.

### Command Routes, Static Assets, And Comparison Evidence

- **D-21:** Implement non-OTA command routes in Phase 5: pause, resume, restart, identify, and block-found dismiss. Preserve upstream response messages and safe visible state transitions where possible.
- **D-22:** Pause and resume should update typed mining activity state and visible API output, but must not force work submission ready or bypass Phase 4 mining-loop gates.
- **D-23:** Restart should send the upstream-compatible JSON response before scheduling or invoking the firmware restart action. Tests should prove the response action is produced before the restart effect.
- **D-24:** Identify should preserve the upstream toggle behavior and 30 second visible identify duration using the existing startup/display adapter boundary where practical.
- **D-25:** Prove static AxeOS compatibility without rewriting Angular. Phase 5 should either serve a reference-built/static fixture or fixture-test the existing AxeOS assets against the implemented routes. Do not introduce a full Angular build pipeline unless planning proves it is necessary and bounded.
- **D-26:** Keep `/api/system/OTA`, `/api/system/OTAWWW`, partition layout, SPIFFS image production, recovery update behavior, and release packaging in Phase 7. Phase 5 may expose safe unsupported/deferred behavior only when required for existing assets to remain administrable.
- **D-27:** Add API comparison evidence that checks OpenAPI route coverage and captured upstream response fixtures for representative success and error cases. This evidence must distinguish schema compatibility, captured response compatibility, and real firmware API smoke.

### the agent's Discretion

The agent may choose exact module names, DTO struct names, fixture formats, route grouping, adapter traits, host-side comparison tool shape, and plan count. Those choices must preserve functional core plus imperative shell, keep upstream reference files read-only, avoid hiding large generated code in strings, keep public wire compatibility separate from internal domain models, and update parity evidence without overclaiming safety-critical verification.

### Deferred Ideas (OUT OF SCOPE)
## Deferred Ideas

- `/api/system/OTA`, `/api/system/OTAWWW`, SPIFFS image production, OTAWWW/static-asset update behavior, partition layout, recovery update behavior, release packaging, and license inventory belong to Phase 7.
- Full voltage, fan, thermal, power, self-test, watchdog-load behavior, and safety-controller telemetry verification belong to Phase 6.
- Angular AxeOS replacement remains out of V1 scope.
- Stratum v2 completeness, BAP completeness, Gamma 601, non-205 boards, and additional ASIC families remain deferred until each has its own roadmap scope and evidence set.
</user_constraints>

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|------------------|
| API-01 | Rust API models are compatible with the upstream OpenAPI schema for V1 user-facing routes. | Use handwritten Serde DTOs plus route/required-field fixture checks against `reference/esp-miner/main/http_server/openapi.yaml`; do not make generated OpenAPI models the firmware contract. [VERIFIED: .planning/REQUIREMENTS.md] [VERIFIED: .planning/phases/05-axeos-api-logs-and-telemetry/05-CONTEXT.md] |
| API-02 | System info and settings responses expose upstream-compatible fields, names, units, defaults, and encoding. | Map `bitaxe-config` defaults/catalog/NVS metadata and runtime snapshots into explicit wire fields such as `ASICModel`, `hashRate_1m`, `fanspeed`, and numeric bool-like fields. [VERIFIED: crates/bitaxe-config/src/nvs.rs] [VERIFIED: reference/esp-miner/main/http_server/system_api_json.c] |
| API-03 | Settings PATCH behavior validates, persists, rejects, reloads, and reports errors with upstream-compatible observable semantics. | Reuse `SettingsPatch`, `apply_settings_patch`, and persistence snapshot/reload semantics; fixture invalid known fields rejecting the whole patch and unknown fields being ignored. [VERIFIED: crates/bitaxe-config/src/settings.rs] [VERIFIED: crates/bitaxe-config/src/persistence.rs] [VERIFIED: reference/esp-miner/main/http_server/http_server.c] |
| API-04 | ASIC, statistics, scoreboard, and mining-state endpoints report values derived from the Rust runtime state model. | Pull ASIC catalog from `bitaxe-config`, mining/share/fallback state from `MiningRuntimeState`, and BM1366 safe status from existing ASIC crates; preserve empty/minimal statistics when no history exists. [VERIFIED: crates/bitaxe-config/src/catalog.rs] [VERIFIED: crates/bitaxe-stratum/src/v1/state.rs] [VERIFIED: crates/bitaxe-asic/src/bm1366/observation.rs] |
| API-05 | Log buffer, log download, and log retention behavior support the user-facing API and WebSocket surfaces. | Model a bounded retained ring buffer contract and firmware log hook; upstream uses a 512 KiB retained buffer and download route with `text/plain` plus `Content-Disposition`. [VERIFIED: reference/esp-miner/main/http_server/websocket.h] [VERIFIED: reference/esp-miner/main/log_buffer.c] |
| API-06 | `/api/ws` streams log events in a client-compatible format. | Stream raw text chunks, start clients at the current buffer end, and do not dump retained history on connect. [VERIFIED: reference/esp-miner/main/http_server/websocket_log.c] [VERIFIED: reference/esp-miner/main/http_server/axe-os/src/app/services/websocket.service.ts] |
| API-07 | `/api/ws/live` streams live telemetry with upstream-compatible payload shape, cadence, and state transitions. | Send `{"event":"update","data":...}` full state on connect, then only diffs every 500 ms when values change. [VERIFIED: reference/esp-miner/main/http_server/websocket_api.c] [VERIFIED: reference/esp-miner/main/http_server/cjson_utils.c] |
| API-08 | Pause, resume, restart, identify, and related command routes preserve user-visible behavior and safe failure modes. | Implement command planners that produce response JSON before side effects, and keep pause/resume from bypassing mining gates. [VERIFIED: reference/esp-miner/main/http_server/http_server.c] [VERIFIED: crates/bitaxe-stratum/src/v1/mining_loop.rs] |
| API-09 | Static AxeOS assets and recovery page behavior remain compatible enough for device administration without requiring an Angular rewrite in V1. | Fixture-test existing AxeOS route usage and recovery/static fallback behavior; keep full SPIFFS/OTA packaging deferred. [VERIFIED: reference/esp-miner/main/http_server/http_server.c] [VERIFIED: reference/esp-miner/main/http_server/axe-os/src/app/services/system-api.service.ts] |
| API-10 | API compare fixtures prove Rust responses match the upstream schema or captured upstream responses for representative success and error cases. | Add host-side comparison fixtures under `crates/bitaxe-api` or `tools/parity` and separate schema, captured-response, and firmware-smoke evidence. [VERIFIED: .planning/phases/05-axeos-api-logs-and-telemetry/05-CONTEXT.md] [VERIFIED: docs/adr/0012-parity-verification-evidence.md] |
</phase_requirements>

## Summary

Phase 5 should be planned as a compatibility and boundary-building phase, not as a hardware-control phase. The safe architecture is to make `crates/bitaxe-api` own public AxeOS wire DTOs, pure mappers, log/WebSocket payload decisions, command response plans, and fixture comparisons, while `firmware/bitaxe` owns ESP-IDF request handling, WebSocket client tracking, task cadence, NVS writes, display/restart effects, and the log hook. [VERIFIED: .planning/phases/05-axeos-api-logs-and-telemetry/05-CONTEXT.md] [VERIFIED: AGENTS.md]

The key implementation risk is compatibility drift in small observable details: upstream mixes numeric bool-like fields and JSON booleans, has casing quirks such as `ASICModel` and `hashRate_1m`, rejects settings patches with a generic `400 Wrong API input`, streams `/api/ws` raw log text from the current buffer end, and sends `/api/ws/live` a full update envelope on connect followed by changed-field diffs every 500 ms. [VERIFIED: reference/esp-miner/main/http_server/system_api_json.c] [VERIFIED: reference/esp-miner/main/http_server/http_server.c] [VERIFIED: reference/esp-miner/main/http_server/websocket_log.c] [VERIFIED: reference/esp-miner/main/http_server/websocket_api.c]

Security planning must treat settings, logs, and command routes as high-impact LAN control surfaces. Upstream-compatible behavior includes a private-network/AP-origin gate, bounded request sizes, bounded WebSocket clients, and generic public errors; Phase 5 should preserve or strengthen those controls without adding public authentication semantics that would break AxeOS compatibility unless the user explicitly accepts that divergence. [VERIFIED: reference/esp-miner/main/http_server/http_server.c] [VERIFIED: reference/esp-miner/main/http_server/websocket.h]

**Primary recommendation:** Implement handwritten Serde wire DTOs and host-testable compatibility mappers in `crates/bitaxe-api`, thin ESP-IDF adapters in `firmware/bitaxe`, and a Bazel-backed API comparison fixture suite before firmware smoke. [VERIFIED: .planning/phases/05-axeos-api-logs-and-telemetry/05-CONTEXT.md] [VERIFIED: Justfile]

## Project Constraints (from AGENTS.md)

- Prefer root `AGENTS.md`, then `AGENTS.bright-builds.md`, `standards-overrides.md`, and relevant managed standards before planning or implementation. [VERIFIED: AGENTS.md]
- Keep `reference/esp-miner` pinned and read-only; use it as behavioral evidence, not a workspace for changes. [VERIFIED: AGENTS.md] [VERIFIED: docs/adr/0005-read-only-reference-implementation.md]
- Use ESP-IDF Rust bindings as the production firmware stack and keep ESP-IDF, FreeRTOS, Wi-Fi, NVS, SPIFFS, OTA, HTTP serving, logging, and hardware orchestration in firmware adapters. [VERIFIED: AGENTS.md] [VERIFIED: docs/adr/0003-esp-idf-rust-production-stack.md]
- Use Bazel as the canonical automation graph and `just` as the human command surface. [VERIFIED: AGENTS.md] [VERIFIED: Justfile]
- Prefer functional core plus imperative shell: pure logic in crates, ESP-IDF effects in `firmware/bitaxe`. [VERIFIED: AGENTS.md] [VERIFIED: standards/core/architecture.md]
- Do not overclaim hardware-control parity; voltage, fan, thermal, power, ASIC initialization, and mining behavior need hardware evidence before verified parity. [VERIFIED: AGENTS.md] [VERIFIED: docs/adr/0012-parity-verification-evidence.md]
- Preserve GPL provenance guardrails: original work stays MIT-first where possible, while intentionally ported GPL-covered expression must be isolated and marked. [VERIFIED: AGENTS.md] [VERIFIED: docs/adr/0013-mit-first-with-gpl-guardrails.md]
- Rust commits require `cargo fmt --all`, `cargo clippy --all-targets --all-features -- -D warnings`, `cargo build --all-targets --all-features`, and `cargo test --all-features` before commit. [VERIFIED: AGENTS.md]
- Unit tests should test one concern, follow Arrange/Act/Assert, and verify behavior instead of implementation details. [VERIFIED: AGENTS.md] [VERIFIED: standards/core/testing.md]
- No project skill directories were found under `.claude/skills/` or `.agents/skills/`. [VERIFIED: shell find .claude/skills .agents/skills]

## Standard Stack

### Core

| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| `serde` | `1.0.228`, published 2025-09-27 | Derive handwritten AxeOS wire DTO serialization/deserialization. | Public JSON contract needs explicit field renames and type choices without exposing internal structs. [VERIFIED: crates.io API 2026-06-27] [VERIFIED: .planning/phases/05-axeos-api-logs-and-telemetry/05-CONTEXT.md] |
| `serde_json` | `1.0.150`, published 2026-05-21 | Fixture parsing, JSON comparison, WebSocket update envelopes, and pure diff tests. | Upstream compatibility evidence is JSON-shaped and should be checked with structured values, not string matching. [VERIFIED: crates.io API 2026-06-27] [VERIFIED: reference/esp-miner/main/http_server/cjson_utils.c] |
| `thiserror` | `2.0.18`, published 2026-01-18 | Library error enums for API mapping, fixture validation, and settings adapter failures. | Repo Rust guidance uses `thiserror` for library errors. [VERIFIED: crates.io API 2026-06-27] [VERIFIED: AGENTS.md] |
| `esp-idf-svc` | `0.52.1`, published 2026-03-10 | Firmware-side ESP-IDF service integration. | Firmware already depends on it and project stack standardizes on ESP-IDF Rust services. [VERIFIED: crates.io API 2026-06-27] [VERIFIED: firmware/bitaxe/Cargo.toml] [CITED: https://docs.esp-rs.org/esp-idf-svc/] |
| `esp-idf-sys` | `0.37.2`, published 2026-03-10 | Firmware-side raw ESP-IDF bindings when HTTP/WebSocket features need lower-level access. | Firmware already depends on it and upstream behavior is based on ESP-IDF `httpd_*` APIs. [VERIFIED: crates.io API 2026-06-27] [VERIFIED: firmware/bitaxe/Cargo.toml] [VERIFIED: reference/esp-miner/main/http_server/websocket.c] |
| `bitaxe-config` | workspace crate | Ultra 205 catalog, defaults, REST setting names, validation, compatibility writes, and persistence snapshots. | Settings/API values must reuse existing typed project boundaries. [VERIFIED: crates/bitaxe-config/src/settings.rs] [VERIFIED: crates/bitaxe-config/src/nvs.rs] |
| `bitaxe-stratum` | workspace crate | Mining runtime state, shares, fallback, pool difficulty, and mining activity mapping. | API output must reflect Phase 4 runtime state without bypassing mining-loop gates. [VERIFIED: crates/bitaxe-stratum/src/v1/state.rs] [VERIFIED: crates/bitaxe-stratum/src/v1/mining_loop.rs] |
| `bitaxe-asic` | workspace crate | BM1366 observations and safe initialization status for API-visible ASIC state. | API must report safe ASIC status from typed existing boundaries, not raw firmware guesses. [VERIFIED: crates/bitaxe-asic/src/bm1366/observation.rs] [VERIFIED: crates/bitaxe-asic/src/bm1366/init_plan.rs] |

### Supporting

| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| `anyhow` | workspace `1.0.102`; latest `1.0.103`, published 2026-06-25 | Host tool and firmware adapter error context. | Use in `tools/parity` or a new API-compare binary; avoid public rich error schema changes. [VERIFIED: Cargo.toml] [VERIFIED: crates.io API 2026-06-27] |
| `clap` | `4.6.1`, published 2026-04-15 | Host API comparison CLI flags and subcommands. | Use only if comparison grows beyond existing `tools/parity report`. [VERIFIED: crates.io API 2026-06-27] [VERIFIED: tools/parity/BUILD.bazel] |
| `camino` | `1.2.3`, published 2026-06-18 | UTF-8 paths for host fixtures and parity evidence. | Use in host tooling where checked-in fixture paths are passed around. [VERIFIED: crates.io API 2026-06-27] [VERIFIED: tools/parity/BUILD.bazel] |
| `jsonschema` | `0.46.6` | Optional host-only JSON Schema validation if OpenAPI-derived schemas are extracted. | Use only after a Wave 0 spike proves the extraction is simpler than explicit route/property checks. [VERIFIED: cargo info jsonschema 2026-06-27] |

### Alternatives Considered

| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| Handwritten Serde DTOs | OpenAPI-generated firmware models | Rejected for firmware contract: locked decision says generated models are host-side only if useful. [VERIFIED: .planning/phases/05-axeos-api-logs-and-telemetry/05-CONTEXT.md] |
| `serde_json::Value` fixture comparisons | Raw JSON string comparisons | Raw strings are brittle to ordering and formatting; structured comparisons can still assert exact field names and encodings. [VERIFIED: reference/esp-miner/main/http_server/system_api_json.c] [VERIFIED: cargo info serde_json 2026-06-27] |
| `openapiv3` as schema parser | Explicit required route/property extraction | `openapiv3 2.2.0` documents OpenAPI v3.0.x structures, while upstream schema declares OpenAPI 3.1.0, so it is not the default standard. [VERIFIED: cargo info openapiv3 2026-06-27] [VERIFIED: reference/esp-miner/main/http_server/openapi.yaml] |
| `serde_yaml` for OpenAPI parsing | Avoid YAML parsing or validate an actively maintained YAML crate first | `serde_yaml 0.9.34+deprecated` is deprecated, so it should not be the default new dependency. [VERIFIED: cargo info serde_yaml 2026-06-27] |
| Angular rewrite | Fixture-test existing AxeOS assets against implemented routes | Rejected by phase scope; V1 must preserve static asset compatibility without a rewrite. [VERIFIED: .planning/phases/05-axeos-api-logs-and-telemetry/05-CONTEXT.md] |
| OTA/static filesystem implementation | Safe unsupported/deferred responses when needed | Full OTA/SPIFFS/release packaging is Phase 7 scope. [VERIFIED: .planning/phases/05-axeos-api-logs-and-telemetry/05-CONTEXT.md] |

**Installation:**
```bash
cargo add serde --features derive -p bitaxe-api
cargo add serde_json -p bitaxe-api
cargo add thiserror -p bitaxe-api
```

**Version verification:** Package versions above were checked with `cargo info`, `cargo search`, and crates.io API queries on 2026-06-27. [VERIFIED: cargo info/cargo search/crates.io API 2026-06-27]

## Architecture Patterns

### Recommended Project Structure

```text
crates/bitaxe-api/src/
|-- lib.rs             # public exports and crate-level breadcrumbs
|-- snapshot.rs        # adapter input structs; no ESP-IDF dependencies
|-- wire.rs            # handwritten Serde DTOs for AxeOS JSON
|-- mappers.rs         # pure snapshot -> wire mappings
|-- settings.rs        # PATCH request parse/mapping wrappers around bitaxe-config
|-- telemetry.rs       # update envelope and diff decisions
|-- logs.rs            # host-testable log buffer/read contract
|-- statistics.rs      # labels, columns selection, row shape
|-- scoreboard.rs      # upstream scoreboard wire shape
`-- commands.rs        # command responses and planned side effects

firmware/bitaxe/src/
|-- http_api.rs        # ESP-IDF route registration and request/response shell
|-- runtime_snapshot.rs# collects platform/mining/config/ASIC facts
|-- websocket_api.rs   # client tracking, 500 ms cadence, async sends
|-- log_buffer.rs      # ESP log hook and retained ring buffer adapter
`-- settings_adapter.rs# NVS write/commit/reload shell

tools/parity/src/
`-- api_compare.rs     # optional host-side route/fixture comparison subcommand
```

This structure keeps public JSON compatibility in a pure crate and ESP-IDF effects in firmware adapters. [VERIFIED: standards/core/architecture.md] [VERIFIED: .planning/phases/05-axeos-api-logs-and-telemetry/05-CONTEXT.md]

### Pattern 1: Wire DTOs Are Not Domain Structs

**What:** Define public AxeOS JSON structs with exact Serde field names and explicit numeric/string/bool encodings. [VERIFIED: reference/esp-miner/main/http_server/system_api_json.c]

**When to use:** Use for every route response and WebSocket payload exposed to AxeOS clients. [VERIFIED: reference/esp-miner/main/http_server/openapi.yaml]

**Example:**
```rust
// Source: reference/esp-miner/main/http_server/system_api_json.c
// Source: .planning/phases/05-axeos-api-logs-and-telemetry/05-CONTEXT.md
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq)]
pub struct SystemInfoWire {
    #[serde(rename = "ASICModel")]
    pub asic_model: String,
    #[serde(rename = "hashRate_1m")]
    pub hash_rate_1m: f64,
    pub fanspeed: u16,
    pub fanrpm: u16,
    pub miningPaused: bool,
    pub apEnabled: u8,
}
```

### Pattern 2: Snapshot Input Boundary

**What:** Firmware collects an `ApiSnapshot` with platform/config/mining/ASIC/log facts, then pure mappers produce DTOs. [VERIFIED: .planning/phases/05-axeos-api-logs-and-telemetry/05-CONTEXT.md]

**When to use:** Use when route responses need ESP-IDF facts such as reset reason or uptime but the public mapping should stay host-testable. [VERIFIED: firmware/bitaxe/src/main.rs]

**Example:**
```rust
// Source: crates/bitaxe-stratum/src/v1/state.rs
pub fn map_mining_state(snapshot: &ApiSnapshot) -> MiningStateWire {
    MiningStateWire {
        sharesAccepted: snapshot.mining.shares.accepted,
        sharesRejected: snapshot.mining.shares.rejected,
        poolDifficulty: snapshot.mining.maybe_pool_difficulty.unwrap_or_default(),
        miningPaused: snapshot.mining.activity.is_paused(),
    }
}
```

### Pattern 3: Settings PATCH Is Atomic At The Known-Field Boundary

**What:** Parse JSON, convert known REST names into `SettingsPatch`, validate all known fields, ignore unknown fields, and reject without writes when any known field is invalid. [VERIFIED: crates/bitaxe-config/src/settings.rs] [VERIFIED: reference/esp-miner/main/http_server/http_server.c]

**When to use:** Use for `PATCH /api/system`; firmware adapters should write, commit, and reload only after pure validation accepts. [VERIFIED: crates/bitaxe-config/src/persistence.rs]

**Example:**
```rust
// Source: crates/bitaxe-config/src/settings.rs
pub fn plan_settings_update(
    body: &serde_json::Map<String, serde_json::Value>,
) -> Result<AcceptedSettingsPlan, PublicPatchError> {
    let patch = SettingsPatch::from_rest_json(body)?;
    let accepted = bitaxe_config::settings::apply_settings_patch(&patch)
        .map_err(|_| PublicPatchError::wrong_api_input())?;
    Ok(AcceptedSettingsPlan::from(accepted))
}
```

### Pattern 4: Telemetry Diff Is Payload-Level

**What:** Store the previous full telemetry JSON value per live API task, send a full `update` envelope on connect, and later send only changed fields when the current value differs. [VERIFIED: reference/esp-miner/main/http_server/websocket_api.c] [VERIFIED: reference/esp-miner/main/http_server/cjson_utils.c]

**When to use:** Use for `/api/ws/live`; no-send is the expected unchanged-state behavior. [VERIFIED: reference/esp-miner/main/http_server/websocket_api.c]

**Example:**
```rust
// Source: reference/esp-miner/main/http_server/cjson_utils.c
pub fn update_envelope(data: serde_json::Value) -> serde_json::Value {
    serde_json::json!({ "event": "update", "data": data })
}
```

### Pattern 5: Commands Produce Responses Before Effects

**What:** Pure command planners return the public JSON response plus a side-effect enum for firmware to execute after response send where upstream does so. [VERIFIED: reference/esp-miner/main/http_server/http_server.c]

**When to use:** Use for restart, identify, pause/resume, and block-found dismiss. [VERIFIED: reference/esp-miner/main/http_server/http_server.c]

**Example:**
```rust
// Source: reference/esp-miner/main/http_server/http_server.c
pub struct CommandPlan {
    pub response: serde_json::Value,
    pub effect: CommandEffect,
}

pub fn restart_plan() -> CommandPlan {
    CommandPlan {
        response: serde_json::json!({ "message": "System will restart shortly." }),
        effect: CommandEffect::RestartAfterResponse { delay_ms: 1000 },
    }
}
```

### Anti-Patterns to Avoid

- **Serializing internal structs directly:** Internal model evolution would become a breaking AxeOS API change. [VERIFIED: .planning/phases/05-axeos-api-logs-and-telemetry/05-CONTEXT.md]
- **Patching NVS field-by-field before full validation:** Upstream-compatible rejection requires no writes when any known field is invalid. [VERIFIED: reference/esp-miner/main/http_server/http_server.c] [VERIFIED: crates/bitaxe-config/src/settings.rs]
- **Sending full `/api/ws/live` snapshots every 500 ms:** Upstream sends diffs after connect and skips unchanged state. [VERIFIED: reference/esp-miner/main/http_server/websocket_api.c]
- **Dumping retained logs to `/api/ws` clients on connect:** Upstream starts log WebSocket clients at the current end of the buffer. [VERIFIED: reference/esp-miner/main/http_server/websocket_log.c]
- **Adding scoreboard `rank`/`since` server-side as authoritative fields:** Upstream server emits six raw fields and the AxeOS client derives display fields. [VERIFIED: reference/esp-miner/main/http_server/http_server.c] [VERIFIED: reference/esp-miner/main/http_server/axe-os/src/models/ISystemScoreboard.ts]

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| JSON parsing and serialization | Custom string concatenation or ad hoc parsing | `serde` and `serde_json` | Exact field names can be modeled with Serde while preserving structured tests. [VERIFIED: crates.io API 2026-06-27] [VERIFIED: reference/esp-miner/main/http_server/system_api_json.c] |
| Settings validation | A second HTTP-only validation table | `bitaxe-config::settings` and `bitaxe-config::persistence` | Existing code already owns REST names, types, defaults, ranges, compatibility writes, and reload semantics. [VERIFIED: crates/bitaxe-config/src/settings.rs] [VERIFIED: crates/bitaxe-config/src/nvs.rs] |
| Mining/share/fallback state | New API-only mining counters | `bitaxe-stratum::v1::MiningRuntimeState` | Phase 4 runtime state is the source of API-visible mining values. [VERIFIED: crates/bitaxe-stratum/src/v1/state.rs] |
| BM1366 status | Raw API guesses from firmware logs | `bitaxe-asic` observations and init gate status | Existing ASIC types encode safe blocked and preflight states. [VERIFIED: crates/bitaxe-asic/src/bm1366/observation.rs] |
| ESP HTTP/WebSocket server core | A custom TCP/HTTP stack | ESP-IDF HTTP server through `esp-idf-svc` or raw `esp-idf-sys` bindings | Upstream behavior is ESP-IDF `httpd_*` based and project stack is ESP-IDF Rust. [VERIFIED: reference/esp-miner/main/http_server/http_server.c] [VERIFIED: AGENTS.md] |
| Public OpenAPI model generation | Generated DTOs as firmware contract | Handwritten wire DTOs plus host-side schema checks | Locked decision requires handcrafted DTOs and only host-side generated use if useful. [VERIFIED: .planning/phases/05-axeos-api-logs-and-telemetry/05-CONTEXT.md] |
| Static UI replacement | Angular rewrite | Route/static fixture compatibility against existing AxeOS assets | V1 scope prohibits Angular replacement. [VERIFIED: .planning/phases/05-axeos-api-logs-and-telemetry/05-CONTEXT.md] |

**Key insight:** This phase is risky because compatibility is in observable quirks, not hard algorithms; the planner should spend tasks on fixture coverage and adapter boundaries before broad route plumbing. [VERIFIED: reference/esp-miner/main/http_server/system_api_json.c] [VERIFIED: reference/esp-miner/main/http_server/websocket_api.c]

## Common Pitfalls

### Pitfall 1: Boolean Encoding Drift

**What goes wrong:** Fields that upstream emits as numbers are emitted as JSON booleans, or vice versa. [VERIFIED: reference/esp-miner/main/http_server/system_api_json.c]

**Why it happens:** Rust domain models naturally prefer `bool`, but AxeOS wire compatibility sometimes expects numeric bool-like fields. [VERIFIED: .planning/phases/05-axeos-api-logs-and-telemetry/05-CONTEXT.md]

**How to avoid:** Keep wire DTO field types explicit and fixture exact encodings for representative fields. [VERIFIED: reference/esp-miner/main/http_server/system_api_json.c]

**Warning signs:** Tests only check that fields exist, not their JSON value type. [ASSUMED]

### Pitfall 2: Treating OpenAPI As Perfect Truth

**What goes wrong:** Implementation follows schema quirks instead of captured/reference behavior. [VERIFIED: reference/esp-miner/main/http_server/openapi.yaml] [VERIFIED: reference/esp-miner/main/nvs_config.c]

**Why it happens:** Upstream OpenAPI is useful but not complete enough to replace captured route behavior; rotation validation differs between schema-style enum documentation and C validation behavior. [VERIFIED: reference/esp-miner/main/http_server/openapi.yaml] [VERIFIED: reference/esp-miner/main/nvs_config.c]

**How to avoid:** Use OpenAPI for route and required-field coverage, then captured JSON and C reference behavior for exact semantics. [VERIFIED: .planning/phases/05-axeos-api-logs-and-telemetry/05-CONTEXT.md]

**Warning signs:** A generated OpenAPI model becomes the only compatibility oracle. [VERIFIED: .planning/phases/05-axeos-api-logs-and-telemetry/05-CONTEXT.md]

### Pitfall 3: PATCH Partial Writes

**What goes wrong:** One valid setting is persisted before another known invalid setting causes rejection. [VERIFIED: reference/esp-miner/main/http_server/http_server.c]

**Why it happens:** HTTP handlers validate and write incrementally instead of using the pure settings boundary. [VERIFIED: crates/bitaxe-config/src/settings.rs]

**How to avoid:** Plan pure validation first, then firmware write/commit/reload as a separate accepted path. [VERIFIED: crates/bitaxe-config/src/persistence.rs]

**Warning signs:** Tests assert only error status and do not assert snapshot unchanged. [ASSUMED]

### Pitfall 4: WebSocket Baseline Semantics Wrong

**What goes wrong:** Log clients receive retained history on connect or live telemetry clients miss the initial full state. [VERIFIED: reference/esp-miner/main/http_server/websocket_log.c] [VERIFIED: reference/esp-miner/main/http_server/websocket_api.c]

**Why it happens:** `/api/ws` and `/api/ws/live` are both WebSockets but have different baseline behavior. [VERIFIED: reference/esp-miner/main/http_server/websocket_log.c] [VERIFIED: reference/esp-miner/main/http_server/websocket_api.c]

**How to avoid:** Separate log-stream and live-telemetry planners/tests. [VERIFIED: .planning/phases/05-axeos-api-logs-and-telemetry/05-CONTEXT.md]

**Warning signs:** One generic WebSocket test suite covers both endpoints. [ASSUMED]

### Pitfall 5: Invented Hardware Values

**What goes wrong:** API returns plausible voltage, fan, temperature, or power values without Phase 6 evidence. [VERIFIED: .planning/phases/05-axeos-api-logs-and-telemetry/05-CONTEXT.md]

**Why it happens:** Full-state response requires many fields and upstream always has hardware-backed globals. [VERIFIED: reference/esp-miner/main/http_server/system_api_json.c]

**How to avoid:** Return safe unavailable/blocked/default values with evidence labels, and do not mark safety-critical parity verified. [VERIFIED: docs/adr/0012-parity-verification-evidence.md]

**Warning signs:** Parity checklist changes hardware-control rows to verified from API-only tests. [VERIFIED: docs/parity/checklist.md]

### Pitfall 6: Static Asset Scope Creep

**What goes wrong:** Phase 5 becomes an Angular build or SPIFFS/OTA packaging phase. [VERIFIED: .planning/phases/05-axeos-api-logs-and-telemetry/05-CONTEXT.md]

**Why it happens:** Existing AxeOS assets include update/static routes that belong to later filesystem/release work. [VERIFIED: reference/esp-miner/main/http_server/axe-os/src/app/services/system-api.service.ts]

**How to avoid:** Fixture-test route compatibility needed for administration and return safe unsupported/deferred behavior where needed. [VERIFIED: .planning/phases/05-axeos-api-logs-and-telemetry/05-CONTEXT.md]

**Warning signs:** A plan adds Node/Angular production builds as a core dependency. [VERIFIED: .planning/phases/05-axeos-api-logs-and-telemetry/05-CONTEXT.md]

## Code Examples

Verified patterns from official or local sources:

### Exact Wire Field Names

```rust
// Source: reference/esp-miner/main/http_server/system_api_json.c
#[derive(Debug, Clone, serde::Serialize, PartialEq)]
pub struct AsicSettingsWire {
    #[serde(rename = "ASICModel")]
    pub asic_model: String,
    pub deviceModel: String,
    pub swarmColor: String,
    pub asicCount: u8,
    pub hashDomains: u8,
    pub defaultFrequency: u16,
    pub frequencyOptions: Vec<u16>,
    pub defaultVoltage: u16,
    pub voltageOptions: Vec<u16>,
}
```

### Scoreboard Shape

```rust
// Source: reference/esp-miner/main/http_server/http_server.c
#[derive(Debug, Clone, serde::Serialize, PartialEq)]
pub struct ScoreboardEntryWire {
    pub difficulty: f64,
    pub job_id: String,
    pub extranonce2: String,
    pub ntime: u32,
    pub nonce: String,
    pub version_bits: String,
}
```

### Settings Error Shape

```rust
// Source: reference/esp-miner/main/http_server/http_server.c
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PublicPatchError {
    WrongApiInput,
}

impl PublicPatchError {
    pub fn status_and_body(self) -> (u16, &'static str) {
        match self {
            Self::WrongApiInput => (400, "Wrong API input"),
        }
    }
}
```

### Log Stream Baseline

```rust
// Source: reference/esp-miner/main/http_server/websocket_log.c
pub fn log_ws_initial_position(total_written: u64) -> u64 {
    total_written
}
```

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| Generated or internal structs as public API | Handwritten compatibility DTOs plus fixtures | Locked for Phase 5 on 2026-06-27 | Planner should schedule DTO/fixture work before firmware route plumbing. [VERIFIED: .planning/phases/05-axeos-api-logs-and-telemetry/05-CONTEXT.md] |
| Full polling as the only live UI path | WebSocket full-on-connect plus changed-field diffs at 500 ms | Upstream reference behavior in pinned source | Planner must include cadence and no-change tests. [VERIFIED: reference/esp-miner/main/http_server/websocket_api.c] |
| `cargo-espmonitor` style separate monitor dependency | `espflash`/project tools for flash and monitor | Existing repo command surface | Phase 5 firmware smoke can use existing `just flash-monitor` path when hardware is available. [VERIFIED: Justfile] [VERIFIED: espflash --version 2026-06-27] |

**Deprecated/outdated:**
- `serde_yaml 0.9.34+deprecated`: Do not make it the standard parser for new OpenAPI tooling. [VERIFIED: cargo info serde_yaml 2026-06-27]
- `openapiv3 2.2.0` as a full parser for this source schema: It represents OpenAPI v3.0.x while upstream declares OpenAPI 3.1.0. [VERIFIED: cargo info openapiv3 2026-06-27] [VERIFIED: reference/esp-miner/main/http_server/openapi.yaml]

## Assumptions Log

| # | Claim | Section | Risk if Wrong |
|---|-------|---------|---------------|
| A1 | Warning signs in the pitfalls section are planning heuristics rather than verified bugs in this repo. [ASSUMED] | Common Pitfalls | Planner may need to refine those warnings after implementation details are chosen. |
| A2 | Whether high-level `esp-idf-svc` route APIs cover exact async WebSocket send/client tracking behavior without raw `esp_idf_sys` calls was not fully verified. [ASSUMED] | Open Questions | Planner may choose the wrong binding layer unless it starts with a compile spike. |
| A3 | Stricter-than-upstream controls for V1 settings/command routes are a product/security decision. [ASSUMED] | Open Questions / Security Domain | Planner may need user confirmation before diverging from upstream-compatible behavior. |
| A4 | Whether a small static fixture is enough or firmware must serve a recovery page in Phase 5 is unresolved. [ASSUMED] | Open Questions | Planner may overbuild static serving or under-cover AxeOS administration smoke. |
| A5 | Hardware smoke still depends on a connected Ultra 205 board and serial port, which were not probed by this research. [ASSUMED] | Environment Availability | Planner must not require hardware smoke without checking board/port availability. |
| A6 | A Wave 0 ESP-IDF HTTP/WebSocket compile spike is needed before broad firmware handler work. [ASSUMED] | Validation Architecture | Planner may skip a low-cost integration risk reducer. |
| A7 | Pool certificate/key settings should be treated as persisted settings only, and Phase 5 code should avoid logging secrets. [ASSUMED] | Security Domain | Planner may miss a log disclosure regression test. |
| A8 | New Phase 5 code should add tests that settings secrets are not logged. [ASSUMED] | Security Domain | Planner may omit a useful information-disclosure guard. |

## Open Questions

1. **Which ESP-IDF Rust binding layer should own WebSocket route plumbing?**
   - What we know: Firmware already uses `esp-idf-svc` and `esp-idf-sys`, and upstream uses ESP-IDF HTTP server WebSocket APIs. [VERIFIED: firmware/bitaxe/Cargo.toml] [VERIFIED: reference/esp-miner/main/http_server/websocket.c]
   - What's unclear: Whether high-level `esp-idf-svc` route APIs cover the exact async send/client tracking behavior without raw `esp_idf_sys` calls was not fully verified in this session. [ASSUMED]
   - Recommendation: Plan a Wave 0 compile spike that registers `/api/system/info`, `/api/ws`, and `/api/ws/live` with the chosen binding layer before broad implementation. [VERIFIED: .planning/phases/05-axeos-api-logs-and-telemetry/05-CONTEXT.md]

2. **Where should captured upstream JSON fixtures come from?**
   - What we know: Phase 5 requires captured upstream responses and distinguishes captured response compatibility from schema and firmware smoke. [VERIFIED: .planning/phases/05-axeos-api-logs-and-telemetry/05-CONTEXT.md]
   - What's unclear: The repo does not yet contain captured Phase 5 response fixtures. [VERIFIED: rg captured fixtures 2026-06-27]
   - Recommendation: Plan an explicit fixture capture/curation task with provenance comments and avoid marking routes verified from schema-only tests. [VERIFIED: docs/adr/0012-parity-verification-evidence.md] [VERIFIED: PROVENANCE.md]

3. **How strict should LAN access control be relative to upstream CORS behavior?**
   - What we know: Upstream applies a private-network/AP-origin allow check and sets permissive CORS headers. [VERIFIED: reference/esp-miner/main/http_server/http_server.c]
   - What's unclear: Whether the project wants stricter-than-upstream controls for V1 settings/command routes is a product/security decision. [ASSUMED]
   - Recommendation: Preserve upstream-compatible local-network gating as the minimum and block planning if any route skips the gate. [VERIFIED: reference/esp-miner/main/http_server/http_server.c]

4. **How much static serving belongs in Phase 5?**
   - What we know: Static AxeOS compatibility is required, while Angular rewrite and full OTA/SPIFFS/release packaging are deferred. [VERIFIED: .planning/phases/05-axeos-api-logs-and-telemetry/05-CONTEXT.md]
   - What's unclear: Whether serving a small static fixture is enough or whether firmware must serve a recovery page in this phase. [ASSUMED]
   - Recommendation: Plan fixture tests against existing AxeOS API usage first, then add the smallest firmware static/recovery behavior needed for administration smoke. [VERIFIED: reference/esp-miner/main/http_server/http_server.c]

## Environment Availability

| Dependency | Required By | Available | Version | Fallback |
|------------|-------------|-----------|---------|----------|
| Rust compiler | Crate and host tests | Yes | `rustc 1.88.0-nightly (2ab28d2e7 2025-06-24)` | None needed. [VERIFIED: rustc --version 2026-06-27] |
| Cargo | Dependency checks and Rust test fallback | Yes | `cargo 1.88.0-nightly (873a06493 2025-05-10)` | Bazel remains canonical command surface. [VERIFIED: cargo --version 2026-06-27] |
| Bazel | Canonical build/test graph | Yes | `9.1.1` | None needed. [VERIFIED: bazel --version 2026-06-27] |
| just | Human command surface | Yes | `1.48.0` | Direct Bazel commands if just is unavailable elsewhere. [VERIFIED: just --version 2026-06-27] |
| espflash | Firmware flash/monitor smoke | Yes | `4.0.1` | Hardware smoke can be skipped with clear reason if no board/port is present. [VERIFIED: espflash --version 2026-06-27] |
| espup | ESP Rust toolchain setup | Yes | `0.15.1` | Upgrade path may be needed if ESP toolchain drift appears. [VERIFIED: espup --version 2026-06-27] |
| ldproxy | ESP-IDF Rust linker proxy | Present | Binary at `/Users/peterryszkiewicz/.cargo/bin/ldproxy`; `--version` panics for this tool | Use through Cargo ESP build config, not as a standalone version probe. [VERIFIED: command -v ldproxy 2026-06-27] [VERIFIED: ldproxy --version 2026-06-27] |
| curl | HTTP smoke tests | Yes | `8.7.1` | Rust host smoke helper if richer checks are needed. [VERIFIED: curl --version 2026-06-27] |
| jq | JSON smoke assertions | Yes | `1.7.1-apple` | `serde_json` host tool/tests. [VERIFIED: jq --version 2026-06-27] |
| websocat | Manual WebSocket smoke | No | Not installed | Add repo-owned WebSocket smoke helper or use pure WebSocket unit tests; do not assume system `websocat`. [VERIFIED: websocat --version 2026-06-27] |

**Missing dependencies with no fallback:**
- None identified for planning; hardware smoke still depends on a connected Ultra 205 board and serial port, which were not probed by this research. [VERIFIED: environment audit 2026-06-27] [ASSUMED]

**Missing dependencies with fallback:**
- `websocat` is missing; use host-testable WebSocket payload/cadence tests and, if needed, a repo-owned smoke tool instead of a global CLI dependency. [VERIFIED: websocat --version 2026-06-27]

## Validation Architecture

### Test Framework

| Property | Value |
|----------|-------|
| Framework | Bazel `rust_test` plus Rust unit tests in workspace crates. [VERIFIED: crates/bitaxe-api/BUILD.bazel] |
| Config file | `BUILD.bazel` targets and `Justfile`; no separate Rust test framework config found. [VERIFIED: rg --files -g BUILD.bazel 2026-06-27] [VERIFIED: Justfile] |
| Quick run command | `bazel test //crates/bitaxe-api:tests //tools/parity:tests` after Wave 0 targets are updated. [VERIFIED: crates/bitaxe-api/BUILD.bazel] [VERIFIED: tools/parity/BUILD.bazel] |
| Full suite command | `just test` or `bazel test //...`. [VERIFIED: Justfile] |

### Phase Requirements -> Test Map

| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|--------------|
| API-01 | OpenAPI route/required-field coverage for V1 API routes | host fixture | `bazel test //tools/parity:tests` or new `//tools/api-compare:tests` | Partial: `tools/parity` exists; API compare logic missing. [VERIFIED: tools/parity/BUILD.bazel] |
| API-02 | System info/settings field names, units, defaults, and encodings | unit/golden | `bazel test //crates/bitaxe-api:tests` | Target exists; DTO fixtures missing. [VERIFIED: crates/bitaxe-api/BUILD.bazel] |
| API-03 | PATCH validation, persistence plan, reload, and public errors | unit/integration | `bazel test //crates/bitaxe-api:tests //crates/bitaxe-config:tests` | Config tests exist; API wrapper tests missing. [VERIFIED: crates/bitaxe-config/src/settings.rs] |
| API-04 | ASIC/statistics/scoreboard/mining state mapping | unit/golden | `bazel test //crates/bitaxe-api:tests` | Target exists; mapping modules missing. [VERIFIED: crates/bitaxe-api/src/lib.rs] |
| API-05 | Ring buffer retained download and clamp/line resync model | unit | `bazel test //crates/bitaxe-api:tests` | Missing. [VERIFIED: crates/bitaxe-api/src/lib.rs] |
| API-06 | `/api/ws` raw log chunks and current-end baseline | unit/smoke | `bazel test //crates/bitaxe-api:tests`; firmware smoke later | Missing. [VERIFIED: crates/bitaxe-api/src/lib.rs] |
| API-07 | `/api/ws/live` initial full update, changed-field diff, 500 ms cadence | unit/smoke | `bazel test //crates/bitaxe-api:tests`; firmware smoke later | Missing. [VERIFIED: crates/bitaxe-api/src/lib.rs] |
| API-08 | Command routes response/effect ordering and safe state changes | unit/smoke | `bazel test //crates/bitaxe-api:tests` | Missing. [VERIFIED: crates/bitaxe-api/src/lib.rs] |
| API-09 | Existing AxeOS asset route usage remains administrable | fixture/static scan | `bazel test //tools/parity:tests` or new static asset fixture target | Missing. [VERIFIED: reference/esp-miner/main/http_server/axe-os/src/app/services/system-api.service.ts] |
| API-10 | Representative success/error captured response comparisons | golden/host fixture | `bazel test //tools/parity:tests` or new `//tools/api-compare:tests` | Missing. [VERIFIED: tools/parity/BUILD.bazel] |

### Sampling Rate

- **Per task commit:** Run the focused target for changed ownership, usually `bazel test //crates/bitaxe-api:tests` or `bazel test //tools/parity:tests`. [VERIFIED: Justfile]
- **Per wave merge:** Run `bazel test //...`. [VERIFIED: Justfile]
- **Phase gate:** Run `just test`, API comparison fixtures, and firmware API smoke for representative success/error cases when hardware/network is available. [VERIFIED: .planning/phases/05-axeos-api-logs-and-telemetry/05-CONTEXT.md] [VERIFIED: Justfile]

### Wave 0 Gaps

- [ ] Add `serde`, `serde_json`, and likely `thiserror` dependencies to `crates/bitaxe-api` Cargo/Bazel metadata. [VERIFIED: crates/bitaxe-api/Cargo.toml] [VERIFIED: crates/bitaxe-api/BUILD.bazel]
- [ ] Replace `crates/bitaxe-api/src/lib.rs` placeholder with module skeleton and first DTO/fixture tests. [VERIFIED: crates/bitaxe-api/src/lib.rs]
- [ ] Add captured upstream fixture directory and provenance notes for system info, ASIC, statistics, scoreboard, PATCH success/failure, live WebSocket update, and command responses. [VERIFIED: .planning/phases/05-axeos-api-logs-and-telemetry/05-CONTEXT.md]
- [ ] Add API compare route/property checks under `tools/parity` or a new `tools/api-compare` target. [VERIFIED: tools/parity/src/main.rs]
- [ ] Add a compile spike for ESP-IDF HTTP/WebSocket route registration before planning full firmware handlers. [ASSUMED]
- [ ] Add static AxeOS route usage fixture from existing `SystemApiService`, `LiveDataService`, `WebsocketService`, and logs component. [VERIFIED: reference/esp-miner/main/http_server/axe-os/src/app/services/system-api.service.ts] [VERIFIED: reference/esp-miner/main/http_server/axe-os/src/app/services/live-data.service.ts] [VERIFIED: reference/esp-miner/main/http_server/axe-os/src/app/services/websocket.service.ts]

## Security Domain

### Applicable ASVS Categories

| ASVS Category | Applies | Standard Control |
|---------------|---------|------------------|
| V2 Authentication | No formal user authentication in upstream V1; route access is network-gated. | Preserve private-network/AP gate on every API/WebSocket/static control path; do not add incompatible auth without an explicit product decision. [VERIFIED: reference/esp-miner/main/http_server/http_server.c] |
| V3 Session Management | No browser session or token session is introduced by Phase 5. | Keep route handlers stateless and do not add cookies/tokens. [VERIFIED: reference/esp-miner/main/http_server/openapi.yaml] |
| V4 Access Control | Yes, for settings, logs, and commands. | Enforce the same gate consistently on HTTP and WebSocket routes; block high severity if any command/settings/log route is reachable without the gate. [VERIFIED: reference/esp-miner/main/http_server/http_server.c] [VERIFIED: reference/esp-miner/main/http_server/websocket.c] |
| V5 Input Validation | Yes. | Use request size caps, Serde JSON parsing, `bitaxe-config` known-field validation, generic public errors, and route-specific query parsing. [VERIFIED: reference/esp-miner/main/http_server/http_server.c] [VERIFIED: crates/bitaxe-config/src/settings.rs] |
| V6 Cryptography | Not newly introduced for Phase 5. | Do not hand-roll crypto; treat pool certificate/key settings as persisted settings only and avoid logging secrets. [VERIFIED: crates/bitaxe-config/src/nvs.rs] [ASSUMED] |
| V7 Error Handling and Logging | Yes. | Keep public errors generic for settings validation and log typed internal diagnostics without leaking secrets. [VERIFIED: reference/esp-miner/main/http_server/http_server.c] [ASSUMED] |
| V10 Malicious Code | Yes for static assets if served. | Serve pinned/reference-built assets or fixture-test existing assets; do not introduce unmanaged Angular build output in Phase 5. [VERIFIED: .planning/phases/05-axeos-api-logs-and-telemetry/05-CONTEXT.md] |

### Known Threat Patterns for API/Logs/WebSockets/Settings

| Pattern | STRIDE | Standard Mitigation |
|---------|--------|---------------------|
| Remote settings or command execution from an untrusted network | Spoofing / Elevation of privilege | Private-network/AP-origin gate on every route and WebSocket handshake; tests for denied external-origin cases. [VERIFIED: reference/esp-miner/main/http_server/http_server.c] |
| CSRF-style browser requests to LAN device | Spoofing / Tampering | Preserve local-network gate, avoid credentialed sessions, and consider explicit Origin/IP tests because upstream CORS is permissive. [VERIFIED: reference/esp-miner/main/http_server/http_server.c] |
| Oversized PATCH body or malformed JSON | Denial of service / Tampering | Request body cap and structured JSON parse errors; upstream uses a scratch buffer cap and generic errors. [VERIFIED: reference/esp-miner/main/http_server/http_server.c] |
| Invalid known setting mixed with valid settings | Tampering | Validate all known fields before writes and reject the whole patch. [VERIFIED: reference/esp-miner/main/http_server/http_server.c] [VERIFIED: crates/bitaxe-config/src/settings.rs] |
| WebSocket client exhaustion | Denial of service | Bounded client count; upstream maximum is 10 clients and rejects over-capacity connections. [VERIFIED: reference/esp-miner/main/http_server/websocket.h] [VERIFIED: reference/esp-miner/main/http_server/websocket.c] |
| Log disclosure of sensitive data | Information disclosure | Gate log download/WebSocket routes and add tests that settings secrets are not logged by new Phase 5 code. [VERIFIED: reference/esp-miner/main/http_server/http_server.c] [ASSUMED] |
| Unsafe hardware behavior from API settings or commands | Tampering / Safety impact | Validate settings but do not enable Phase 6 voltage/fan/thermal/power effects or bypass mining gates. [VERIFIED: .planning/phases/05-axeos-api-logs-and-telemetry/05-CONTEXT.md] [VERIFIED: crates/bitaxe-stratum/src/v1/mining_loop.rs] |

## Sources

### Primary (HIGH confidence)

- `.planning/phases/05-axeos-api-logs-and-telemetry/05-CONTEXT.md` - Locked decisions, scope boundaries, discretion areas, and deferred ideas. [VERIFIED: local file]
- `.planning/REQUIREMENTS.md` - API-01 through API-10 requirements. [VERIFIED: local file]
- `.planning/ROADMAP.md` - Phase 5 goal, success criteria, and verification expectations. [VERIFIED: local file]
- `.planning/STATE.md` - Completed prior phase decisions and current safety/evidence state. [VERIFIED: local file]
- `AGENTS.md`, `AGENTS.bright-builds.md`, `standards/*` - Project workflow, architecture, testing, Rust, verification, and provenance constraints. [VERIFIED: local files]
- `reference/esp-miner/main/http_server/openapi.yaml` - Upstream OpenAPI route/schema contract. [VERIFIED: local file]
- `reference/esp-miner/main/http_server/system_api_json.c` - Full-state JSON fields and upstream encoding quirks. [VERIFIED: local file]
- `reference/esp-miner/main/http_server/http_server.c` - Route registration, PATCH semantics, commands, logs, access gate, static/recovery behavior. [VERIFIED: local file]
- `reference/esp-miner/main/http_server/websocket_api.c`, `websocket_log.c`, `websocket.c`, `websocket.h`, `cjson_utils.c` - WebSocket live/log behavior and diff semantics. [VERIFIED: local files]
- `reference/esp-miner/main/log_buffer.c` - Retained log buffer and read semantics. [VERIFIED: local file]
- `crates/bitaxe-config`, `crates/bitaxe-stratum`, `crates/bitaxe-asic`, `firmware/bitaxe`, `tools/parity`, `Justfile` - Existing Rust integration points and validation commands. [VERIFIED: local files]
- crates.io registry via `cargo info`, `cargo search`, and crates.io API - Package versions and publish dates. [VERIFIED: crates.io API 2026-06-27]

### Secondary (MEDIUM confidence)

- `https://docs.esp-rs.org/esp-idf-svc/` - Official `esp-idf-svc` documentation URL reported by the crate metadata. [CITED: https://docs.esp-rs.org/esp-idf-svc/]
- `https://docs.esp-rs.org/esp-idf-sys/` - Official `esp-idf-sys` documentation URL reported by the crate metadata. [CITED: https://docs.esp-rs.org/esp-idf-sys/]

### Tertiary (LOW confidence)

- Planning heuristics in warning-sign bullets and unresolved binding/static scope questions. [ASSUMED]

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH - core dependencies and versions were verified against local manifests and crates.io; optional schema tooling remains scoped as a Wave 0 decision. [VERIFIED: Cargo.toml] [VERIFIED: crates.io API 2026-06-27]
- Architecture: HIGH - follows locked phase decisions, local Bright Builds architecture rules, and existing crate/firmware ownership boundaries. [VERIFIED: .planning/phases/05-axeos-api-logs-and-telemetry/05-CONTEXT.md] [VERIFIED: standards/core/architecture.md]
- Pitfalls: HIGH for upstream compatibility quirks and settings/WebSocket/log semantics; MEDIUM for heuristic warning signs. [VERIFIED: reference/esp-miner/main/http_server/*.c] [ASSUMED]
- Security: MEDIUM-HIGH - upstream access-control/input constraints were verified, but stricter-than-upstream product policy remains an open decision. [VERIFIED: reference/esp-miner/main/http_server/http_server.c] [ASSUMED]

**Research date:** 2026-06-27
**Valid until:** 2026-07-27
