# Phase 5: AxeOS API, Logs, And Telemetry - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md - this log preserves the alternatives considered.

**Date:** 2026-06-27T18:10:00.003Z
**Phase:** 5 - AxeOS API, Logs, And Telemetry
**Mode:** Yolo
**Areas discussed:** API response and schema compatibility, Settings PATCH semantics, Logs and WebSocket telemetry, Command routes/static assets/API comparison evidence

---

## API Response And Schema Compatibility

| Option | Description | Selected |
| --- | --- | --- |
| Handwritten Serde wire DTOs plus pure mappers | Exact upstream field names while keeping internal domain/config/mining state separate from public JSON. | yes |
| OpenAPI-generated Rust models | Mechanical schema coverage, useful mainly as a host-side compatibility oracle. | no |
| Capture-first `serde_json::Value` builders | Fast captured-response matching with weaker type safety. | no |
| Serialize existing domain structs directly | Minimal code but leaks internal names/units and cannot represent upstream quirks cleanly. | no |

**User's choice:** Yolo selected the recommended option: handwritten Serde wire DTOs plus pure mappers.
**Notes:** OpenAPI and captured responses remain verification inputs. Generated models are not the firmware implementation source.

---

## Settings PATCH Semantics

| Option | Description | Selected |
| --- | --- | --- |
| Upstream-exact async clone | Mirrors upstream timing and empty success/generic error shape, but persistence failures may only be logged later. | no |
| Upstream-compatible synchronous commit | Preserves external response compatibility while proving accepted settings persist and reload before success where possible. | yes |
| Strict typed REST errors | Better diagnostics for new clients but breaks existing AxeOS/upstream compatibility. | no |

**User's choice:** Yolo selected the recommended option: upstream-compatible synchronous commit.
**Notes:** External behavior stays upstream-compatible: known-field all-or-nothing validation, unknown fields ignored, generic rejection, empty success, and hostname live apply best effort.

---

## Logs And WebSocket Telemetry

| Option | Description | Selected |
| --- | --- | --- |
| Reference-first wire contracts in `crates/bitaxe-api` plus thin ESP-IDF adapters | Preserves upstream log download, raw log WebSocket chunks, and live telemetry diff behavior with host-testable contracts. | yes |
| Firmware-local reference clone | Fastest live demo path but buries protocol behavior in ESP-IDF code. | no |
| Typed internal telemetry/log event model with upstream renderers | Strong long-term architecture, but risks over-modeling before hardware data exists. | no |

**User's choice:** Yolo selected the recommended option: reference-first wire contracts in `crates/bitaxe-api` plus thin ESP-IDF adapters.
**Notes:** Upstream behavior to preserve includes raw text frames on `/api/ws`, full `update` event on `/api/ws/live` connect, 500 ms diff cadence, no-send on unchanged state, and retained log download.

---

## Command Routes, Static Assets, And API Comparison Evidence

| Option | Description | Selected |
| --- | --- | --- |
| API/evidence-only slice | Tight API route scope but leaves API-09 static asset compatibility under-proven. | no |
| Reference-built asset fixture plus non-OTA command compatibility | Covers non-OTA command routes, static asset compatibility evidence, and API compare fixtures while keeping Phase 7 deferred. | yes |
| Canonical Angular build integration in Phase 5 | Fresh asset build but adds Node/Angular build graph complexity. | no |
| Merge web filesystem/OTA scope into Phase 5 | End-to-end UI/filesystem/OTA but collapses Phase 5 with Phase 7. | no |

**User's choice:** Yolo selected the recommended option: reference-built asset fixture plus non-OTA command compatibility.
**Notes:** Implement pause/resume/restart/identify/blockFound dismiss. Keep OTA, OTAWWW, SPIFFS image production, partition layout, recovery update, and release packaging in Phase 7.

---

## the agent's Discretion

- Exact module names, DTO names, fixture formats, adapter traits, and plan count.
- Exact API comparison tool shape, as long as OpenAPI and captured responses are both represented.
- Whether static asset compatibility is proved through a served fixture, host fixture, or bounded firmware smoke, as long as Angular is not rewritten.

## Deferred Ideas

- Phase 7 owns OTA, OTAWWW, SPIFFS/recovery update behavior, release packaging, and license inventory.
- Phase 6 owns safety-controller telemetry verification and hardware-control behavior.
- Angular AxeOS replacement remains out of V1 scope.
