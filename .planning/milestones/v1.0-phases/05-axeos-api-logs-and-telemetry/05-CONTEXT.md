---
generated_by: gsd-discuss-phase
lifecycle_mode: yolo
phase_lifecycle_id: 5-2026-06-27T18-02-49
generated_at: 2026-06-27T18:10:00.003Z
---

# Phase 5: AxeOS API, Logs, And Telemetry - Context

**Gathered:** 2026-06-27
**Status:** Ready for planning
**Mode:** Yolo

<domain>
## Phase Boundary

Phase 5 delivers the AxeOS-compatible administration and observation surface for the Rust firmware. This includes pure API wire models, HTTP handler behavior, system info/settings/ASIC/statistics/scoreboard/mining-state responses, settings PATCH semantics, log download, `/api/ws` log streaming, `/api/ws/live` telemetry streaming, non-OTA command routes, static AxeOS asset compatibility evidence, and API comparison fixtures.

This phase does not rewrite the Angular AxeOS UI, does not merge Phase 7 OTA/filesystem/release packaging, and does not unlock voltage, fan, thermal, power, or unsafe mining behavior. It may expose safe blocked or unavailable values for later-phase surfaces, but verified hardware-control parity still requires later Ultra 205 hardware evidence.

</domain>

<decisions>
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

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Phase Scope And Prior Decisions

- `.planning/ROADMAP.md` - Phase 5 goal, success criteria, verification expectations, UI hint, and boundaries.
- `.planning/REQUIREMENTS.md` - API-01 through API-10 plus evidence, safety, and deferred release requirements.
- `.planning/PROJECT.md` - Ultra 205 first target, architecture constraints, AxeOS API/asset compatibility boundary, and current state.
- `.planning/STATE.md` - Completed Phase 4 decisions, current Phase 5 focus, hardware evidence blockers, and quick display evidence.
- `.planning/phases/01-foundation-and-gamma-601-boot-log/01-CONTEXT.md` - Safe boot/log boundary, package/flash/monitor contract, parity policy, and disabled hardware-control status.
- `.planning/phases/02-ultra-205-config-and-nvs-model/02-CONTEXT.md` - Settings/NVS schema, validation, persistence boundary, and API settings reuse requirement.
- `.planning/phases/03-bm1366-asic-protocol-and-safe-initialization/03-CONTEXT.md` - BM1366 status/gate boundaries and hardware evidence rules.
- `.planning/phases/04-stratum-v1-and-first-mining-loop/04-CONTEXT.md` - Mining runtime state, accepted/rejected share counters, pool lifecycle status, and API deferral boundary.

### Existing Rust Integration Points

- `crates/bitaxe-api/src/lib.rs` - Current Phase 1 API deferral placeholder to replace.
- `crates/bitaxe-api/BUILD.bazel` - Bazel target that must expose new API sources, fixtures, and tests.
- `crates/bitaxe-config/src/settings.rs` - Pure settings patch validation and write decisions for `PATCH /api/system`.
- `crates/bitaxe-config/src/nvs.rs` - REST names, NVS keys, stored types, defaults, ranges, and compatibility writes.
- `crates/bitaxe-config/src/persistence.rs` - Host-testable persistence snapshot/reload semantics.
- `crates/bitaxe-config/src/catalog.rs` - Ultra 205 catalog, ASIC count, options, capabilities, and verification scope.
- `crates/bitaxe-config/src/defaults.rs` - Ultra 205 defaults for system/settings responses.
- `crates/bitaxe-stratum/src/v1/state.rs` - Mining runtime state for API-visible pool, share, hashrate, fallback, and mining activity fields.
- `crates/bitaxe-stratum/src/v1/mining_loop.rs` - Mining-loop gate and safe blocked status that API responses must not bypass.
- `crates/bitaxe-asic/src/bm1366/observation.rs` - ASIC init status and observations for API-visible ASIC state.
- `crates/bitaxe-asic/src/bm1366/init_plan.rs` - Initialization evidence gate and status boundary.
- `firmware/bitaxe/src/main.rs` - Current firmware boot/log shell and platform facts.
- `firmware/bitaxe/src/asic_adapter/status.rs` - Existing visible ASIC and mining-loop blocked status logs.
- `firmware/bitaxe/src/display_adapter.rs` - Existing Ultra 205 startup/display adapter surface relevant to identify behavior.
- `docs/parity/checklist.md` - API, log, telemetry, stats, command, static asset, and evidence rows to update.

### Upstream API, Log, WebSocket, And Asset References

- `reference/esp-miner/main/http_server/openapi.yaml` - Upstream route/schema contract for system, logs, ASIC, statistics, scoreboard, command, settings, OTA, and static-related routes.
- `reference/esp-miner/main/http_server/system_api_json.c` - Full-state JSON fields, telemetry/config/hashrate/rejected-reason/block sections, field names, defaults, and reset reason strings.
- `reference/esp-miner/main/http_server/system_api_json.h` - Upstream full-state JSON boundary.
- `reference/esp-miner/main/http_server/http_server.c` - Route registration, settings PATCH behavior, command routes, log download headers, static/recovery routing, and WebSocket route mapping.
- `reference/esp-miner/main/http_server/cjson_utils.c` - JSON diff behavior used by live telemetry.
- `reference/esp-miner/main/http_server/websocket.c` - WebSocket client tracking, route type handling, and broadcast/send behavior.
- `reference/esp-miner/main/http_server/websocket_api.c` - `/api/ws/live` full-on-connect and 500 ms diff update semantics.
- `reference/esp-miner/main/http_server/websocket_log.c` - `/api/ws` raw log streaming semantics.
- `reference/esp-miner/main/log_buffer.c` - Retained ring-buffer log behavior, vprintf hook, soft reboot marker, absolute reads, clamp, and line resync.
- `reference/esp-miner/main/log_buffer.h` - Log buffer public API and configured buffer size.
- `reference/esp-miner/main/nvs_config.c` - Upstream settings table, validation, persistence queue, and getters used by API responses.
- `reference/esp-miner/main/nvs_config.h` - NVS setting enum and metadata types.
- `reference/esp-miner/main/global_state.h` - Global state fields consumed by API responses and WebSocket telemetry.
- `reference/esp-miner/main/system.c` - System initialization, AxeOS version, filesystem availability, and mining paused defaults.
- `reference/esp-miner/main/system.h` - System module fields and status boundaries.
- `reference/esp-miner/main/tasks/statistics_task.h` - Statistics source labels and row shape.
- `reference/esp-miner/main/tasks/hashrate_monitor_task.c` - Hashrate monitor values surfaced in API responses.
- `reference/esp-miner/main/http_server/axe-os/api/system/asic_settings.c` - `/api/system/asic` response shape.
- `reference/esp-miner/main/http_server/axe-os/src/models/ISystemScoreboard.ts` - AxeOS scoreboard client model.
- `reference/esp-miner/main/http_server/axe-os/src/models/enum/eChartLabel.ts` - AxeOS statistics label expectations.
- `reference/esp-miner/main/http_server/axe-os/src/environments/environment.ts` - AxeOS route/environment expectations.
- `reference/esp-miner/main/http_server/axe-os/src/assets/share-rejection-explanations.json` - Client-visible rejected share explanation asset.
- `reference/esp-miner/main/screen.c` - Upstream identify mode display behavior and mining paused display hints.
- `reference/esp-miner/main/filesystem.c` - Static/recovery serving behavior to fixture-test or defer without merging Phase 7.

### Architecture, Evidence, And Policy

- `docs/adr/0001-device-user-parity.md` - Observable behavior parity definition.
- `docs/adr/0003-esp-idf-rust-production-stack.md` - ESP-IDF Rust stack and adapter boundary.
- `docs/adr/0005-read-only-reference-implementation.md` - Reference implementation policy.
- `docs/adr/0006-parity-checklist-as-audit-evidence.md` - Checklist as evidence policy.
- `docs/adr/0008-reference-breadcrumb-comments.md` - Breadcrumb placement policy.
- `docs/adr/0009-monorepo-package-layout.md` - Crate and firmware path ownership.
- `docs/adr/0010-axeos-api-and-asset-compatibility.md` - API and static asset compatibility before UI rewrite.
- `docs/adr/0012-parity-verification-evidence.md` - Evidence requirements and hardware-control verification gate.
- `docs/adr/0013-mit-first-with-gpl-guardrails.md` - GPL guardrails for upstream-derived behavior and fixtures.
- `docs/adr/0014-pivot-to-ultra-205-bm1366-first-parity.md` - Ultra 205/BM1366 first parity target and deferred Gamma 601 scope.
- `PROVENANCE.md` - Provenance, SPDX, upstream reference, fixture/source attribution, and release review policy.

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets

- `crates/bitaxe-config` already provides Ultra 205 catalog/defaults, REST names, typed settings validation, compatibility writes, and host-testable persistence snapshots. Phase 5 should call these surfaces for settings and system response data.
- `crates/bitaxe-stratum/src/v1/state.rs` already provides `MiningRuntimeState`, share counters, pool lifecycle, fallback-active status, hashrate inputs, and mining activity status for API and telemetry mapping.
- `crates/bitaxe-stratum/src/v1/mining_loop.rs` already records safe blocked mining-loop reasons such as `hardware_evidence_ack_missing`; API responses should surface those safely rather than enabling work.
- `crates/bitaxe-asic` already exposes typed BM1366 status and gate concepts that can feed `/api/system/asic` and full-state JSON without leaking raw frame logic.
- `firmware/bitaxe/src/display_adapter.rs` and the quick display evidence provide a starting display boundary for identify behavior, but full display/input parity remains later scope.

### Established Patterns

- Pure domain and wire-shape decisions belong in crates; ESP-IDF HTTP server, WebSocket, NVS I/O, timers, tasks, mutexes, logging hooks, and restart effects belong in `firmware/bitaxe`.
- Tests use Arrange, Act, Assert and should prove one concern per unit test.
- Reference breadcrumbs should sit at module or behavior boundaries and point to pinned upstream files.
- Parity evidence must distinguish unit/golden/API-compare evidence from hardware smoke and must not mark safety-critical hardware effects verified without hardware evidence.
- Keep existing AxeOS as the compatibility target. Do not rewrite Angular in V1.

### Integration Points

- `crates/bitaxe-api` should become the owner of AxeOS wire DTOs, response mappers, settings request mapping, WebSocket update envelopes, log stream payload contracts, command response models, fixtures, and API comparison helpers.
- `firmware/bitaxe` should add thin HTTP/WebSocket adapters that translate ESP-IDF requests and runtime snapshots to/from `bitaxe-api` models, then perform NVS, restart, display identify, and log hook effects.
- `tools/parity` or a new repo-owned host tool may run OpenAPI/captured-response checks, but the canonical human command surface should remain `just` through Bazel-backed targets.
- `docs/parity/checklist.md` and `docs/parity/evidence/` must be updated as API surfaces move from deferred to implemented or verified.

</code_context>

<specifics>
## Specific Ideas

- Build captured upstream response fixtures from the pinned reference for `/api/system/info`, `/api/system/asic`, `/api/system/statistics`, `/api/system/scoreboard`, settings PATCH success/failure, `/api/ws/live` initial/full update, and representative command responses.
- Model `ApiSnapshot` or an equivalent adapter input separately from wire DTOs so firmware can supply platform facts without making ESP-IDF a dependency of `bitaxe-api`.
- Include a fixture proving unknown PATCH fields are ignored while invalid known fields reject the full patch.
- Include a fixture proving `/api/ws/live` sends an initial full `update` event and later diff-only updates at the 500 ms cadence.
- Include a fixture or smoke note proving static AxeOS assets can call the implemented V1 routes without requiring an Angular rewrite.

</specifics>

<deferred>
## Deferred Ideas

- `/api/system/OTA`, `/api/system/OTAWWW`, SPIFFS image production, OTAWWW/static-asset update behavior, partition layout, recovery update behavior, release packaging, and license inventory belong to Phase 7.
- Full voltage, fan, thermal, power, self-test, watchdog-load behavior, and safety-controller telemetry verification belong to Phase 6.
- Angular AxeOS replacement remains out of V1 scope.
- Stratum v2 completeness, BAP completeness, Gamma 601, non-205 boards, and additional ASIC families remain deferred until each has its own roadmap scope and evidence set.

</deferred>

---

*Phase: 05-axeos-api-logs-and-telemetry*
*Context gathered: 2026-06-27*
